# 72 - Multi-Host Orchestration

> **Previous:** [71 - Performance Tuning](../71-performance-tuning/README.md)
> **Next:** [73 - Service Discovery](../73-service-discovery/README.md)

## The Problem

Your application runs well on a single Docker host, but you have outgrown one machine. You need to spread containers across multiple servers for capacity, redundancy, or geographic distribution. Manually SSH-ing into each host to run `docker run` is not sustainable. You need a system that treats a pool of machines as a single deployable platform — scheduling containers, distributing load, handling failures, and managing networking between hosts automatically.

Multi-host orchestration is the layer between your application containers and your infrastructure. It answers: which host runs which container? How do containers on different hosts talk to each other? What happens when a host dies?

---

## The Naive Way

Manually manage containers on each host with scripts and SSH.

```bash
#!/bin/bash
# deploy.sh — the "works on my machines" approach
HOSTS=("server1.example.com" "server2.example.com" "server3.example.com")

for host in "${HOSTS[@]}"; do
    ssh "$host" "docker pull myapp/api:latest"
    ssh "$host" "docker stop api || true"
    ssh "$host" "docker run -d --name api -p 8080:8080 myapp/api:latest"
done

# Hope nothing goes wrong...
# No rolling updates, no health checks, no automatic failover.
```

**Why this fails:**
- No scheduling intelligence — you manually decide which host runs what.
- No automatic failover — if a host dies, its containers are gone.
- No overlay networking — containers on different hosts cannot communicate directly.
- No rolling updates — deploying means downtime.
- No health monitoring — crashed containers stay crashed.
- Scaling requires editing the script and re-running it.
- Split-brain scenarios when hosts disagree about cluster state.

---

## The Right Way

Use a purpose-built orchestrator. Docker Swarm and HashiCorp Nomad are simpler alternatives to Kubernetes that handle scheduling, networking, and failover.

### Docker Swarm Mode

Docker Swarm is built into Docker itself. No additional software to install. It turns a group of Docker hosts into a single virtual host.

**Initializing a Swarm:**

```bash
# On the manager node
docker swarm init --advertise-addr 192.168.1.10

# Output gives you a join token:
# docker swarm join --token SWMTKN-1-xxx 192.168.1.10:2377

# On worker nodes
docker swarm join --token SWMTKN-1-xxx 192.168.1.10:2377

# Verify cluster
docker node ls
```

**Deploying a Stack:**

```yaml
# docker-compose.yml (stack definition)
version: "3.8"

services:
  api:
    image: myapp/api:latest
    ports:
      - "8080:8080"
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      update_config:
        parallelism: 1
        delay: 30s
        failure_action: rollback
        monitor: 60s
      rollback_config:
        parallelism: 1
        delay: 10s
      placement:
        constraints:
          - node.role == worker
        preferences:
          - spread: node.labels.zone
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
    networks:
      - backend
      - frontend
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    secrets:
      - db_password
    configs:
      - source: api_config
        target: /app/config.yaml

  worker:
    image: myapp/worker:latest
    deploy:
      replicas: 5
      restart_policy:
        condition: on-failure
      placement:
        constraints:
          - node.labels.gpu == true
    networks:
      - backend

  redis:
    image: redis:7-alpine
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.ssd == true
    volumes:
      - redis_data:/data
    networks:
      - backend

networks:
  frontend:
    driver: overlay
    attachable: true
  backend:
    driver: overlay
    internal: true

volumes:
  redis_data:
    driver: local

secrets:
  db_password:
    file: ./secrets/db_password.txt

configs:
  api_config:
    file: ./config/api.yaml
```

```bash
# Deploy the stack
docker stack deploy -c docker-compose.yml myapp

# Manage the stack
docker stack services myapp
docker stack ps myapp

# Scale a service
docker service scale myapp_api=5

# Rolling update
docker service update --image myapp/api:v2.1 myapp_api

# Rollback
docker service rollback myapp_api

# View logs
docker service logs -f myapp_api
```

**Overlay Networking:**

Swarm creates an overlay network that spans all hosts. Containers on different hosts communicate as if they were on the same network.

```bash
# Create an overlay network
docker network create --driver overlay --attachable my_network

# Containers on different hosts can now reach each other by service name
# DNS resolution works automatically within the overlay
```

### HashiCorp Nomad

Nomad is a simpler, more flexible orchestrator that supports containers, VMs, standalone binaries, Java applications, and more.

**Installation and Setup:**

```bash
# Install Nomad
wget https://releases.hashicorp.com/nomad/1.6.0/nomad_1.6.0_linux_amd64.zip
unzip nomad_1.6.0_linux_amd64.zip
sudo mv nomad /usr/local/bin/

# Server configuration (/etc/nomad.d/server.hcl)
datacenter = "dc1"
data_dir   = "/opt/nomad"

server {
  enabled          = true
  bootstrap_expect = 3
}

advertise {
  http = "192.168.1.10"
  rpc  = "192.168.1.10"
  serf = "192.168.1.10"
}
```

**Nomad Job Specification:**

```hcl
# api.nomad
job "api" {
  datacenters = ["dc1"]
  type        = "service"

  group "api" {
    count = 3

    spread {
      attribute = "${node.datacenter}"
      weight    = 100
    }

    update {
      max_parallel      = 1
      min_healthy_time  = "30s"
      healthy_deadline  = "5m"
      progress_deadline = "10m"
      auto_revert       = true
      canary            = 1
    }

    reschedule {
      attempts       = 3
      interval       = "30m"
      delay          = "5s"
      delay_function = "exponential"
      max_delay      = "1m"
    }

    network {
      port "http" {
        static = 8080
      }
    }

    service {
      name = "api"
      port = "http"

      check {
        type     = "http"
        path     = "/health"
        interval = "10s"
        timeout  = "3s"
      }

      tags = ["v1", "production"]
    }

    task "api" {
      driver = "docker"

      config {
        image = "myapp/api:latest"
        ports = ["http"]
      }

      template {
        data = <<EOF
API_PORT={{ env "NOMAD_PORT_http" }}
DB_HOST={{ range service "postgres" }}{{ .Address }}:{{ .Port }}{{ end }}
EOF
        destination = "local/config.yaml"
      }

      resources {
        cpu    = 2000  # MHz
        memory = 1024  # MB
      }

      logs {
        max_files     = 10
        max_file_size = 15
      }
    }
  }
}
```

```bash
# Deploy a job
nomad job run api.nomad

# Check status
nomad job status api
nomad alloc status <alloc_id>

# Scale
nomad job scale api api 5

# Rolling update (change image version and re-deploy)
nomad job run api.nomad

# Stop a job
nomad job stop api

# View logs
nomad alloc logs <alloc_id>
```

### Comparison Table

| Feature | Docker Swarm | Nomad | Kubernetes |
|---------|-------------|-------|------------|
| **Complexity** | Low | Low-Medium | High |
| **Installation** | Built into Docker | Single binary | Multiple components |
| **Learning curve** | Gentle | Gentle | Steep |
| **Container runtime** | Docker only | Docker, Podman, containerd, exec, Java, QEMU | CRI-compatible |
| **Orchestration** | Containers only | Containers, VMs, binaries, Java | Primarily containers |
| **Networking** | Built-in overlay | CNI plugins, Consul | CNI plugins, built-in Services |
| **Service discovery** | DNS-based (built-in) | Consul integration | DNS + Service objects |
| **Secrets management** | Docker secrets | Vault integration | K8s Secrets, Vault |
| **Storage** | Docker volumes | CSI plugins | CSI, PV/PVC |
| **Auto-scaling** | Manual | External autoscaler | HPA, VPA, Cluster Autoscaler |
| **Rolling updates** | Built-in | Built-in | Built-in |
| **Health checks** | HTTP/TCP/command | HTTP/TCP/gRPC/script | Liveness/readiness/startup probes |
| **Community** | Declining | Growing | Massive |
| **Ecosystem** | Limited | Moderate | Enormous |
| **Best for** | Simple deployments, small teams | Mixed workloads, simplicity | Large-scale, complex deployments |

**When to use Docker Swarm:**
- Small team, simple application.
- Already using Docker Compose and want to scale to multiple hosts.
- Don't want to learn Kubernetes.
- Need a quick production deployment without a steep learning curve.

**When to use Nomad:**
- Mixed workloads (containers + non-containerized applications).
- Already in the HashiCorp ecosystem (Consul, Vault, Terraform).
- Need simplicity with flexibility.
- Multi-region deployments with Nomad's built-in federation.

**When to use Kubernetes:**
- Large organization with dedicated platform team.
- Need the rich ecosystem of operators, CRDs, and community tools.
- Multi-cloud portability is a hard requirement.
- Complex deployment strategies (canary, blue-green, A/B).

---

## The Production Way

### Multi-Host Networking Deep Dive

```bash
# Docker Swarm overlay network with encryption
docker network create \
  --driver overlay \
  --opt encrypted \
  --subnet 10.0.9.0/24 \
  my_secure_network

# VXLAN encapsulation is used for overlay traffic
# Default VXLAN port: UDP 4789
# Ensure firewall allows this between hosts
sudo iptables -A INPUT -p udp --dport 4789 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 2377 -j ACCEPT  # Swarm management
sudo iptables -A INPUT -p tcp --dport 7946 -j ACCEPT  # Node communication
sudo iptables -A INPUT -p udp --dport 7946 -j ACCEPT  # Node discovery
```

### High Availability Manager Nodes

```bash
# Docker Swarm — always run 3 or 5 managers for Raft consensus
docker swarm init --advertise-addr 192.168.1.10

# Get manager join token
docker swarm join-token manager

# Join additional managers (odd numbers: 3, 5, 7)
docker swarm join --token SWMTKN-1-manager-token 192.168.1.10:2377

# Verify Raft status
docker node ls
docker info | grep -A5 "Raft"
```

### Health Monitoring and Auto-Recovery

```yaml
# Production Docker Compose with comprehensive health checks
services:
  api:
    image: myapp/api:latest
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 5
        window: 120s
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8080/health"]
      interval: 15s
      timeout: 5s
      retries: 3
      start_period: 30s
```

---

## Hands-On Lab

### Exercise 1: Docker Swarm Cluster

```bash
# Create 3 VMs (or use Docker-in-Docker)
# manager1, worker1, worker2

# Initialize swarm on manager
docker swarm init --advertise-addr 192.168.1.10

# Join workers
docker swarm join --token <token> 192.168.1.10:2377

# Deploy a service
docker service create --name web --replicas 3 -p 8080:80 nginx:alpine

# Scale
docker service scale web=5

# Verify distribution
docker service ps web

# Simulate node failure
docker node update --availability drain <worker_node>

# Watch tasks migrate
docker service ps web
```

### Exercise 2: Rolling Update with Rollback

```bash
# Create a service with update configuration
docker service create \
  --name api \
  --replicas 6 \
  --update-parallelism 2 \
  --update-delay 10s \
  --update-failure-action rollback \
  -p 8080:8080 \
  myapp/api:v1

# Trigger rolling update
docker service update --image myapp/api:v2 api

# Watch progress
watch docker service ps api

# If v2 is bad, rollback
docker service rollback api
```

### Exercise 3: Nomad Cluster

```bash
# Start a Nomad dev server (single node, all roles)
nomad agent -dev

# In another terminal, run a job
cat > example.nomad << 'EOF'
job "example" {
  datacenters = ["dc1"]
  type = "service"

  group "web" {
    count = 3

    network {
      port "http" {
        to = 80
      }
    }

    service {
      name = "web"
      port = "http"
      check {
        type = "http"
        path = "/"
        interval = "10s"
      }
    }

    task "nginx" {
      driver = "docker"
      config {
        image = "nginx:alpine"
        ports = ["http"]
      }
      resources {
        cpu    = 100
        memory = 128
      }
    }
  }
}
EOF

nomad job run example.nomad
nomad job status example
```

### Exercise 4: Overlay Network Communication

```bash
# Create overlay network
docker network create --driver overlay testnet

# On host 1: start a container
docker run -d --name server1 --network testnet nginx:alpine

# On host 2: ping server1 by name
docker run --rm --network testnet alpine ping -c 3 server1

# Verify VXLAN traffic
sudo tcpdump -i eth0 udp port 4789
```

---

## Limitation

Docker Swarm and Nomad handle scheduling and networking, but they do not provide robust service discovery out of the box. Swarm has basic DNS, but for dynamic environments with hundreds of services, health-aware routing, and cross-datacenter discovery, you need a dedicated service discovery system. Nomad explicitly depends on Consul for this. Without proper service discovery, services cannot find each other as instances scale up, down, or move between hosts.

---

## Next Topic

[73 - Service Discovery](../73-service-discovery/README.md) — Build dynamic service registries with Consul, etcd, and DNS-based discovery so that services can find each other without hardcoded addresses.
