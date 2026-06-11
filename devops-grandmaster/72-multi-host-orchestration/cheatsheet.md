# Cheatsheet: Multi-Host Orchestration

## Docker Swarm
```bash
# Initialize swarm
docker swarm init

# Join worker
docker swarm join --token TOKEN MANAGER_IP:2377

# Deploy stack
docker stack deploy -c docker-compose.yml myapp

# List services
docker service ls

# Scale service
docker service scale myapp_web=5

# List nodes
docker node ls
```

## Docker Compose for Swarm
```yaml
version: '3.8'

services:
  web:
    image: my-app:1.0
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
    ports:
      - "80:80"
    networks:
      - webnet

networks:
  webnet:
    driver: overlay
```

## Swarm vs Kubernetes

| Feature | Swarm | Kubernetes |
|---------|-------|------------|
| Setup | Simple | Complex |
| Learning curve | Low | High |
| Features | Basic | Advanced |
| Ecosystem | Small | Large |
| Auto-scaling | No | Yes |
| Self-healing | Basic | Advanced |

## HashiCorp Nomad
```bash
# Install Nomad
curl -fsSL https://releases.hashicorp.com/nomad/1.6.0/nomad_1.6.0_linux_amd64.zip -o nomad.zip
unzip nomad.zip
sudo mv nomad /usr/local/bin/

# Run job
nomad job run my-job.hcl
```
