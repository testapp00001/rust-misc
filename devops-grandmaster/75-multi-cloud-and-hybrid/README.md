# 75 - Multi-Cloud and Hybrid

> **Previous:** [74 - Distributed Systems Patterns](../74-distributed-systems-patterns/README.md)
> **Next:** [76 - Incident Response](../76-incident-response/README.md)

## The Problem

Your organization has invested heavily in AWS, but the machine learning team insists on GCP's TPUs. The enterprise sales team requires Azure AD integration. A regulatory requirement mandates that certain data stays on-premises. Your company just acquired another company running entirely on a different cloud. You need your systems to work across all of these environments without being locked into any single provider.

Multi-cloud and hybrid architectures solve real business problems — avoiding vendor lock-in, meeting regulatory requirements, leveraging best-of-breed services, and enabling disaster recovery across providers. But they also introduce enormous complexity: inconsistent APIs, different networking models, data gravity, and the challenge of maintaining a coherent operational model across fundamentally different platforms.

---

## The Naive Way

Pick one cloud provider and build everything using their proprietary services.

```yaml
# The "all-in on AWS" approach
resources:
  - AWS Lambda for compute
  - DynamoDB for database
  - SQS for messaging
  - S3 for storage
  - CloudFront for CDN
  - Cognito for authentication
  - EventBridge for event routing
  - Step Functions for workflows
  - X-Ray for tracing
  - CloudWatch for monitoring
```

**Why this fails:**
- Deep vendor lock-in — migrating away requires rewriting the entire application.
- No leverage in pricing negotiations — you cannot credibly threaten to leave.
- Single region/provider outage takes down everything.
- Regulatory requirements may mandate data residency in specific locations or providers.
- Acquisitions bring infrastructure on other clouds that must integrate.
- Different teams have different expertise and preferences.

---

## The Right Way

Design for portability with abstraction layers, and use each cloud's strengths strategically.

### Why Multi-Cloud

**Legitimate reasons:**
- **Avoid vendor lock-in:** Maintain the ability to switch providers or negotiate pricing.
- **Best-of-breed services:** Use GCP for ML, AWS for compute, Azure for enterprise identity.
- **Regulatory compliance:** Some regulations require specific data residency or provider diversity.
- **Acquisition integration:** Merging companies that use different clouds.
- **Disaster recovery:** Geographic and provider-level redundancy.
- **Edge computing:** Some providers have better presence in specific regions.

**Anti-patterns (not real reasons):**
- "We might want to switch someday" — without a concrete plan, this is just added complexity.
- Running the same workload identically on two clouds simultaneously — doubles cost for marginal benefit.
- Political reasons — different teams choosing different clouds without technical justification.

### Multi-Cloud Strategies

**Strategy 1: Abstraction Layer**

Build on top of an abstraction that works across clouds.

```hcl
# Terraform — cloud-agnostic infrastructure definition
# Works on AWS, GCP, Azure, and more

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }
}

# Define infrastructure for both clouds
module "aws_compute" {
  source        = "./modules/compute"
  providers     = { cloud = aws }
  instance_type = "m6i.xlarge"
  count         = var.use_aws ? 3 : 0
}

module "gcp_compute" {
  source         = "./modules/compute"
  providers      = { cloud = google }
  machine_type   = "e2-standard-4"
  count          = var.use_gcp ? 3 : 0
}
```

**Strategy 2: Kubernetes as the Common Platform**

Kubernetes runs identically on every cloud. Applications deployed to Kubernetes are inherently portable.

```yaml
# This manifest works on EKS, GKE, AKS, or any Kubernetes cluster
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api
  template:
    metadata:
      labels:
        app: api
    spec:
      containers:
        - name: api
          image: myregistry.io/api:v2.1
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: "500m"
              memory: "512Mi"
            limits:
              cpu: "2"
              memory: "2Gi"
          env:
            - name: CLOUD_PROVIDER
              valueFrom:
                configMapKeyRef:
                  name: cloud-config
                  key: provider
---
apiVersion: v1
kind: Service
metadata:
  name: api
spec:
  selector:
    app: api
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP
```

**Strategy 3: Service Mesh for Cross-Cloud Networking**

```yaml
# Istio multi-cluster setup
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: istio-aws
spec:
  profile: default
  values:
    global:
      meshID: mesh1
      multiCluster:
        clusterName: aws-east
      network: aws-network
---
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: istio-gcp
spec:
  profile: default
  values:
    global:
      meshID: mesh1
      multiCluster:
        clusterName: gcp-central
      network: gcp-network
```

### Kubernetes Federation

Kubernetes federation manages multiple clusters as a single logical entity.

```yaml
# KubeFed — Federated Deployment
apiVersion: types.kubefed.io/v1beta1
kind: FederatedDeployment
metadata:
  name: api-server
  namespace: production
spec:
  template:
    metadata:
      labels:
        app: api
    spec:
      replicas: 3
      selector:
        matchLabels:
          app: api
      template:
        spec:
          containers:
            - name: api
              image: myregistry.io/api:v2.1
  placement:
    clusters:
      - name: aws-east
      - name: gcp-central
      - name: on-prem
  overrides:
    - clusterName: aws-east
      clusterOverrides:
        - path: "/spec/replicas"
          value: 5
    - clusterName: on-prem
      clusterOverrides:
        - path: "/spec/template/spec/containers/0/resources/limits/memory"
          value: "4Gi"
```

### Hybrid Cloud Architecture

Hybrid cloud combines on-premises infrastructure with public cloud services.

```
On-Premises Data Center                    Public Cloud (AWS/GCP/Azure)
+----------------------------------+      +-------------------------------+
|  Legacy Applications             |      |  Cloud-Native Applications    |
|  +----------+  +----------+     |      |  +----------+ +----------+   |
|  | ERP      |  | Database |     |      |  | ML/AI    | | API GW   |   |
|  +----------+  +----------+     |      |  +----------+ +----------+   |
|                                  |      |                               |
|  Kubernetes Cluster              |      |  Kubernetes Cluster           |
|  +----------+ +----------+      |      |  +----------+ +----------+   |
|  | Services | | Services |      |      |  | Services | | Services |   |
|  +----------+ +----------+      |      |  +----------+ +----------+   |
+----------------|-----------------+      +----------|-------------------+
                 |                                   |
          +------+------+                    +------+------+
          | VPN / Direct |                    | VPN / Interconnect|
          | Connect     |                    |              |
          +------+------+                    +------+------+
                 |                                   |
                 +------ Encrypted Tunnel -----------+
```

**Cross-cloud networking:**

```hcl
# AWS side
resource "aws_vpn_gateway" "main" {
  vpc_id = aws_vpc.main.id
}

resource "aws_vpn_connection" "to_gcp" {
  vpn_gateway_id      = aws_vpn_gateway.main.id
  customer_gateway_id = aws_customer_gateway.gcp.id
  type                = "ipsec.1"
  static_routes_only  = true
}

# GCP side
resource "google_compute_ha_vpn_gateway" "main" {
  name    = "gcp-to-aws"
  network = google_compute_network.main.id
}

resource "google_compute_vpn_tunnel" "to_aws" {
  name          = "tunnel-to-aws"
  peer_ip       = aws_vpn_connection.to_gcp.tunnel1_address
  shared_secret = aws_vpn_connection.to_gcp.tunnel1_preshared_key
  vpn_gateway   = google_compute_ha_vpn_gateway.main.id
}
```

### Terraform for Multi-Cloud

```hcl
# Multi-cloud Terraform project structure
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }

  backend "s3" {
    bucket = "terraform-state"
    key    = "multi-cloud/terraform.tfstate"
    region = "us-east-1"
  }
}

provider "aws" {
  region = "us-east-1"
}

provider "google" {
  project = "my-gcp-project"
  region  = "us-central1"
}

provider "azurerm" {
  features {}
}

# Networking on each cloud
module "aws_network" {
  source = "./modules/aws-network"
  cidr   = "10.0.0.0/16"
}

module "gcp_network" {
  source = "./modules/gcp-network"
  cidr   = "10.1.0.0/16"
}

module "azure_network" {
  source = "./modules/azure-network"
  cidr   = "10.2.0.0/16"
}

# Cross-cloud VPN peering
module "aws_to_gcp_vpn" {
  source            = "./modules/cross-cloud-vpn"
  aws_vpc_id        = module.aws_network.vpc_id
  gcp_network_id    = module.gcp_network.network_id
  aws_cidr          = module.aws_network.cidr
  gcp_cidr          = module.gcp_network.cidr
}

# Compute on each cloud
module "aws_compute" {
  source       = "./modules/compute"
  cloud        = "aws"
  vpc_id       = module.aws_network.vpc_id
  subnet_ids   = module.aws_network.private_subnet_ids
  cluster_name = "aws-prod"
}

module "gcp_compute" {
  source       = "./modules/compute"
  cloud        = "gcp"
  network_id   = module.gcp_network.network_id
  subnet_ids   = module.gcp_network.subnet_ids
  cluster_name = "gcp-prod"
}
```

### Data Gravity

Data gravity is the tendency of data to attract applications and services. Large datasets are expensive and slow to move. Once data is in one cloud, applications naturally migrate to that cloud to be close to the data.

**Mitigation strategies:**

```python
# Pattern 1: Event-driven replication
def write_with_replication(data, primary_cloud, secondary_cloud):
    # Write to primary
    result = primary_cloud.database.write(data)

    # Publish replication event
    primary_cloud.event_bus.publish("data.changed", {
        "operation": "write",
        "data": data,
        "timestamp": time.time(),
        "source": primary_cloud.name
    })

    # Secondary cloud consumes the event and applies the change
    # (asynchronous — eventual consistency)

# Pattern 2: Global database
# Use a globally distributed database
# CockroachDB, YugabyteDB, or Google Cloud Spanner
# Writes are replicated across regions automatically

# Pattern 3: Data mesh
# Each domain owns its data, exposes it via APIs
# Other domains consume via APIs, not direct database access
```

### Cloud-Agnostic Application Design

```python
# Abstract cloud-specific services behind interfaces
from abc import ABC, abstractmethod

class StorageProvider(ABC):
    @abstractmethod
    def put(self, key, data): pass

    @abstractmethod
    def get(self, key): pass

    @abstractmethod
    def delete(self, key): pass

class S3Storage(StorageProvider):
    def __init__(self, bucket):
        import boto3
        self.client = boto3.client('s3')
        self.bucket = bucket

    def put(self, key, data):
        self.client.put_object(Bucket=self.bucket, Key=key, Body=data)

    def get(self, key):
        response = self.client.get_object(Bucket=self.bucket, Key=key)
        return response['Body'].read()

    def delete(self, key):
        self.client.delete_object(Bucket=self.bucket, Key=key)

class GCSStorage(StorageProvider):
    def __init__(self, bucket):
        from google.cloud import storage
        self.client = storage.Client()
        self.bucket = self.client.bucket(bucket)

    def put(self, key, data):
        blob = self.bucket.blob(key)
        blob.upload_from_string(data)

    def get(self, key):
        blob = self.bucket.blob(key)
        return blob.download_as_bytes()

    def delete(self, key):
        blob = self.bucket.blob(key)
        blob.delete()

# Factory pattern for cloud selection
def get_storage_provider():
    cloud = os.environ.get("CLOUD_PROVIDER", "aws")
    bucket = os.environ.get("STORAGE_BUCKET")

    if cloud == "aws":
        return S3Storage(bucket)
    elif cloud == "gcp":
        return GCSStorage(bucket)
    else:
        raise ValueError(f"Unknown cloud provider: {cloud}")

# Application code is cloud-agnostic
storage = get_storage_provider()
storage.put("config/app.yaml", config_data)
data = storage.get("config/app.yaml")
```

---

## The Production Way

### Multi-Cloud GitOps

```yaml
# ArgoCD ApplicationSet for multi-cluster deployment
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: api-multi-cloud
  namespace: argocd
spec:
  generators:
    - clusters:
        selector:
          matchLabels:
            environment: production
  template:
    metadata:
      name: 'api-{{name}}'
    spec:
      project: default
      source:
        repoURL: https://github.com/myorg/k8s-manifests.git
        targetRevision: HEAD
        path: overlays/{{metadata.labels.cloud}}
      destination:
        server: '{{server}}'
        namespace: production
      syncPolicy:
        automated:
          prune: true
          selfHeal: true
```

### Global Load Balancing

```hcl
# Cloudflare for global load balancing across clouds
resource "cloudflare_load_balancer" "global_api" {
  zone_id          = var.cloudflare_zone_id
  name             = "api.example.com"
  fallback_pool_id = cloudflare_load_balancer_pool.aws_pool.id
  default_pool_ids = [
    cloudflare_load_balancer_pool.aws_pool.id,
    cloudflare_load_balancer_pool.gcp_pool.id,
  ]
  steering_policy = "dynamic_latency"

  region_pools {
    region   = "NAM"
    pool_ids = [cloudflare_load_balancer_pool.aws_pool.id]
  }

  region_pools {
    region   = "WEU"
    pool_ids = [cloudflare_load_balancer_pool.gcp_pool.id]
  }

  session_affinity = "cookie"
}

resource "cloudflare_load_balancer_pool" "aws_pool" {
  name = "aws-east-pool"

  origins {
    name    = "aws-api-1"
    address = "10.0.1.10"
    enabled = true
  }

  minimum_origins = 1
  check_regions    = ["NAM"]
}

resource "cloudflare_load_balancer_pool" "gcp_pool" {
  name = "gcp-central-pool"

  origins {
    name    = "gcp-api-1"
    address = "10.1.1.10"
    enabled = true
  }

  minimum_origins = 1
  check_regions    = ["WEU"]
}
```

### Cost Optimization Across Clouds

```python
# Spot/Preemptible instance strategy across clouds
def get_spot_instances(cloud, workload_type):
    """Get the cheapest spot instances for a workload."""
    if cloud == "aws":
        return {
            "instance_types": ["m6i.xlarge", "m5.xlarge", "m5a.xlarge"],
            "spot_strategy": "capacity-optimized",
            "max_price": "0.10"
        }
    elif cloud == "gcp":
        return {
            "machine_type": "e2-standard-4",
            "preemptible": True,
            "automatic_restart": False
        }
    elif cloud == "azure":
        return {
            "vm_size": "Standard_D4s_v3",
            "priority": "Spot",
            "eviction_policy": "Deallocate"
        }
```

---

## Hands-On Lab

### Exercise 1: Terraform Multi-Cloud

```hcl
# main.tf — deploy a VM on both AWS and GCP
terraform {
  required_providers {
    aws    = { source = "hashicorp/aws",    version = "~> 5.0" }
    google = { source = "hashicorp/google", version = "~> 5.0" }
  }
}

provider "aws" { region = "us-east-1" }
provider "google" { project = "my-project", region = "us-central1" }

# AWS EC2
resource "aws_instance" "web" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t3.micro"
  tags = { Name = "multi-cloud-aws" }
}

# GCP Compute
resource "google_compute_instance" "web" {
  name         = "multi-cloud-gcp"
  machine_type = "e2-micro"
  zone         = "us-central1-a"
  boot_disk {
    initialize_params { image = "debian-cloud/debian-11" }
  }
  network_interface {
    network = "default"
    access_config {}
  }
}

output "aws_ip" { value = aws_instance.web.public_ip }
output "gcp_ip" { value = google_compute_instance.web.network_interface[0].access_config[0].nat_ip }
```

### Exercise 2: Cross-Cloud VPN

```bash
# Simulate cross-cloud networking with WireGuard
# Node on "Cloud A"
sudo apt install wireguard
wg genkey | tee /etc/wireguard/private.key | wg pubkey > /etc/wireguard/public.key

cat > /etc/wireguard/wg0.conf << EOF
[Interface]
PrivateKey = <cloud_a_private_key>
Address = 10.100.0.1/24
ListenPort = 51820

[Peer]
PublicKey = <cloud_b_public_key>
Endpoint = <cloud_b_public_ip>:51820
AllowedIPs = 10.100.0.2/32, 10.1.0.0/16
PersistentKeepalive = 25
EOF

sudo wg-quick up wg0

# Test connectivity
ping 10.100.0.2
```

### Exercise 3: Kubernetes Multi-Cluster with Kind

```bash
# Create two local Kubernetes clusters
kind create cluster --name cloud-a --config kind-cloud-a.yaml
kind create cluster --name cloud-b --config kind-cloud-b.yaml

# Deploy the same application to both
kubectl --context kind-cloud-a apply -f deployment.yaml
kubectl --context kind-cloud-b apply -f deployment.yaml

# Verify both clusters
kubectl --context kind-cloud-a get pods
kubectl --context kind-cloud-b get pods
```

### Exercise 4: Cloud-Agnostic Storage

```python
# cloud_storage_lab.py — test storage abstraction
import os
import tempfile

class LocalStorage:
    def __init__(self, base_path):
        self.base_path = base_path
        os.makedirs(base_path, exist_ok=True)

    def put(self, key, data):
        path = os.path.join(self.base_path, key)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, 'wb') as f:
            f.write(data if isinstance(data, bytes) else data.encode())

    def get(self, key):
        path = os.path.join(self.base_path, key)
        with open(path, 'rb') as f:
            return f.read()

    def delete(self, key):
        path = os.path.join(self.base_path, key)
        os.remove(path)

    def list(self, prefix=""):
        results = []
        for root, dirs, files in os.walk(os.path.join(self.base_path, prefix)):
            for f in files:
                full_path = os.path.join(root, f)
                rel_path = os.path.relpath(full_path, self.base_path)
                results.append(rel_path)
        return results

# Test
storage = LocalStorage("/tmp/cloud-storage-lab")
storage.put("config/app.yaml", "key: value")
storage.put("data/users/1.json", '{"name": "Alice"}')

print(storage.get("config/app.yaml"))
print(storage.list("data/"))

storage.delete("data/users/1.json")
print(storage.list("data/"))
```

---

## Limitation

Multi-cloud and hybrid architectures ensure your infrastructure is resilient and portable. But infrastructure resilience does not help when things go wrong at 3 AM — a production database is corrupted, a deployment introduces a critical bug, or a security breach is detected. You need a systematic approach to detecting, responding to, and recovering from incidents. Infrastructure is the foundation, but incident response is the practice of keeping it running when everything is on fire.

---

## Next Topic

[76 - Incident Response](../76-incident-response/README.md) — Build systematic incident response processes, from detection through resolution and post-mortem.
