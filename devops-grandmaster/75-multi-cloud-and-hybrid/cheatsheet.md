# Cheatsheet: Multi-Cloud & Hybrid

## Why Multi-Cloud
- Avoid vendor lock-in
- Best-of-breed services
- Compliance requirements
- Geographic distribution
- Cost optimization

## Multi-Cloud Strategies

| Strategy | Description | Complexity |
|----------|-------------|------------|
| Cloud-agnostic | Use portable tools | Low |
| Active-active | Run in multiple clouds | High |
| Primary + DR | Main + disaster recovery | Medium |
| Best-of-breed | Use best service per cloud | Medium |

## Terraform Multi-Cloud
```hcl
# AWS Provider
provider "aws" {
  region = "us-east-1"
}

# GCP Provider
provider "google" {
  project = "my-project"
  region  = "us-central1"
}

# AWS Resources
resource "aws_instance" "web" {
  ami           = "ami-12345678"
  instance_type = "t3.micro"
}

# GCP Resources
resource "google_compute_instance" "web" {
  name         = "web-instance"
  machine_type = "e2-micro"
}
```

## Kubernetes Federation
```yaml
# kubefed configuration
apiVersion: core.kubefed.io/v1beta1
kind: FederatedDeployment
metadata:
  name: my-app
  namespace: default
spec:
  template:
    spec:
      replicas: 3
  placement:
    clusters:
      - name: cluster-aws
      - name: cluster-gcp
```

## Cross-Cloud Networking
```
AWS VPC ←→ VPN ←→ GCP VPC
         or
AWS VPC ←→ Cloud Interconnect ←→ GCP VPC
```
