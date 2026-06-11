# Solution 04: Multi-Cloud Disaster Recovery Strategy

## Part 1: Architecture Document

### Component Inventory

| Component | Primary (AWS) | DR (GCP) | Replication Method |
|-----------|--------------|----------|-------------------|
| Web/API compute | ECS Fargate / EKS | GKE Autopilot | Container images in both registries |
| PostgreSQL | RDS Multi-AZ | Cloud SQL | Logical replication (PG native) |
| Object storage | S3 | GCS | Cloud Storage Transfer Service |
| DNS | Route 53 | Cloud DNS (failover) | Health-check-based routing |
| CDN | CloudFront | Cloud CDN | Origin failover config |
| Secrets | AWS SSM / Secrets Mgr | GCP Secret Manager | Sync script on schedule |
| Monitoring | CloudWatch | Cloud Monitoring + Grafana | Dual exporters |
| IAM | AWS IAM | GCP IAM | Mapped policies (documented) |
| Container registry | ECR | GCR/Artifact Registry | Cross-registry image push |

### Data Replication Strategy

**PostgreSQL (RDS to Cloud SQL)**

- Method: PostgreSQL logical replication. RDS is the publisher, Cloud SQL
  is the subscriber.
- Replication lag target: < 5 seconds under normal load.
- Conflict resolution: Cloud SQL is read-only during normal operations.
  No conflicts arise. During failover, the subscription is dropped and
  Cloud SQL becomes the primary.
- Cost: Cloud SQL instance running 24/7 at a smaller tier than production
  (~$150/month for db-custom-1-3840).

Limitations:
- DDL changes (ALTER TABLE) are not replicated. Must be applied manually
  to both sides.
- Large schema migrations require pausing replication.
- Sequences are not synchronized -- use application-level UUIDs or reset
  sequences after failover.

**Object Storage (S3 to GCS)**

- Method: Cloud Storage Transfer Service (GCP-native) running hourly.
- Replication lag target: < 1 hour (acceptable for RPO of 1 hour for
  non-critical assets).
- Conflict resolution: S3 is the source of truth. GCS is overwritten.
- Cost: Data transfer costs (~$0.01/GB for egress from AWS).

**Application State**

- Stateless by design. No replication needed. Sessions stored in the
  database or a distributed cache (Redis) which is also replicated.

### Failover Decision Tree

```
START: Health check failure detected
  |
  v
Is it a single-AZ failure?
  YES -> AWS handles this automatically (Multi-AZ RDS, multi-AZ ECS)
         -> No action needed. Monitor.
  NO  -> Continue
  |
  v
Is it an AWS regional outage?
  Check: Can you reach AWS us-east-1 control plane?
  Check: Are multiple services (EC2, RDS, S3) failing simultaneously?
  Check: Is https://health.aws.amazon.com showing an incident?
  |
  YES -> Regional outage confirmed
  |     |
  |     v
  |   Wait 5 minutes for auto-recovery?
  |     |
  |     YES, recovered -> No action. Monitor for 1 hour.
  |     NO, still down  -> Continue to failover
  |
  v
Is it an AWS-wide outage affecting multiple regions?
  Check: Are other AWS regions affected?
  Check: Is the AWS status page showing multi-region impact?
  |
  YES -> Cloud-wide outage (rare but possible)
  |     -> Initiate IMMEDIATE failover to GCP
  |
  NO  -> Regional-only outage
        -> Initiate failover to GCP (consider 10-min wait for recovery)
```

Authorization:
- Regional outage lasting > 15 minutes: Automated failover (scripted)
- Cloud-wide outage: Automated failover (immediate)
- Ambiguous situation: On-call engineer decides within 10 minutes
- CEO/CTO notification: Always, within 5 minutes of failover initiation

### Network Architecture

DNS failover using Route 53:

```
app.datavault.com
  |
  +-- Primary:   A record -> AWS ALB (us-east-1)
  |   Health check: HTTPS /health every 10s, failure threshold 3
  |
  +-- Secondary: A record -> GCP Global LB (us-central1)
      Health check: Same endpoint, different origin
      TTL: 60 seconds (low for fast failover)
```

Certificate management:
- AWS: ACM certificate for app.datavault.com
- GCP: Google-managed certificate for app.datavault.com
- Both are auto-renewed. No shared certificate needed.

### Security Considerations

**Secrets synchronization**:
- Run a scheduled job (every 15 minutes) that reads from AWS SSM and
  writes to GCP Secret Manager.
- Use a dedicated IAM user (AWS) and service account (GCP) with minimal
  permissions.
- Log all sync operations for audit trail.

**IAM mapping**:

| AWS Role | GCP Equivalent | Permissions |
|----------|---------------|-------------|
| ecsTaskExecutionRole | roles/container.defaultServiceAccount | Pull images, read secrets |
| rds-monitoring-role | roles/cloudsql.viewer | Read-only database metrics |
| datavault-app-role | roles/custom.datavaultApp | Application-specific |

**Network security during failover**:
- GCP firewall rules mirror AWS security groups.
- No public access to database -- only from application subnet.
- VPN between AWS and GCP for replication traffic (already established).

## Part 2: Terraform for DR Environment

### terraform/main.tf

```hcl
terraform {
  required_version = ">= 1.5.0"
  required_providers {
    google = { source = "hashicorp/google", version = "~> 5.0" }
  }
}

provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

variable "gcp_project_id" {
  type = string
}

variable "gcp_region" {
  default = "us-central1"
}

variable "dr_environment" {
  default = "dr"
}
```

### terraform/compute.tf

```hcl
# GKE cluster for DR -- starts with minimal node pool
resource "google_container_cluster" "dr" {
  name     = "datavault-dr-cluster"
  location = var.gcp_region

  # Start with 1 node, autoscale up during failover
  remove_default_node_pool = true
  initial_node_count       = 1

  networking_mode = "VPC_NATIVE"
  ip_allocation_policy {}

  release_channel {
    channel = "REGULAR"
  }
}

resource "google_container_node_pool" "dr_nodes" {
  name     = "dr-node-pool"
  location = var.gcp_region
  cluster  = google_container_cluster.dr.name

  autoscaling {
    min_node_count = 1      # Warm standby
    max_node_count = 20     # Full capacity during failover
  }

  node_config {
    machine_type = "e2-standard-4"
    disk_size_gb = 100
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
  }
}

# Horizontal Pod Autoscaler for the application
# (configured via kubectl/Helm after cluster is ready)
```

### terraform/database.tf

```hcl
resource "google_sql_database_instance" "dr" {
  name             = "datavault-dr-postgres"
  database_version = "POSTGRES_15"
  region           = var.gcp_region

  settings {
    tier              = "db-custom-1-3840"  # Smaller than production
    availability_type = "ZONAL"             # Single zone for cost savings
    disk_size         = 50
    disk_type         = "PD_SSD"

    backup_configuration {
      enabled                        = true
      point_in_time_recovery_enabled = true
    }

    database_flags {
      name  = "max_connections"
      value = "100"
    }

    ip_configuration {
      ipv4_enabled = false
      private_network = google_compute_network.dr_vpc.id
    }
  }

  deletion_protection = true
}

resource "google_sql_database" "app" {
  name     = "datavault"
  instance = google_sql_database_instance.dr.name
}

resource "google_sql_user" "app" {
  name     = "datavault_app"
  instance = google_sql_database_instance.dr.name
  password = var.db_password  # Use Secret Manager in production
}
```

### terraform/storage.tf

```hcl
resource "google_storage_bucket" "dr_assets" {
  name          = "datavault-dr-assets"
  location      = var.gcp_region
  force_destroy = false

  versioning {
    enabled = true
  }

  lifecycle_rule {
    condition {
      age = 90
    }
    action {
      type = "Delete"
    }
  }
}

# Cloud Storage Transfer Service job (configured via gcloud CLI or API)
# This is configured out-of-band because the Terraform Google provider
# does not yet have a native transfer job resource for S3-to-GCS.
# Use: gcloud transfer jobs create ...
```

### terraform/networking.tf

```hcl
resource "google_compute_network" "dr_vpc" {
  name                    = "datavault-dr-vpc"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "app" {
  name          = "app-subnet"
  ip_cidr_range = "10.10.1.0/24"
  region        = var.gcp_region
  network       = google_compute_network.dr_vpc.id
}

resource "google_compute_firewall" "allow_lb" {
  name    = "allow-lb-health-checks"
  network = google_compute_network.dr_vpc.name

  allow {
    protocol = "tcp"
    ports    = ["8080"]  # Application port
  }
  source_ranges = ["130.211.0.0/22", "35.191.0.0/16"]  # GCP LB ranges
}

resource "google_compute_firewall" "allow_internal" {
  name    = "allow-internal"
  network = google_compute_network.dr_vpc.name

  allow {
    protocol = "tcp"
  }
  allow {
    protocol = "udp"
  }
  allow {
    protocol = "icmp"
  }
  source_ranges = ["10.10.0.0/16"]
}

# Global load balancer for DR
resource "google_compute_global_address" "dr" {
  name = "datavault-dr-ip"
}

# Backend service, URL map, HTTPS proxy configured for the GKE cluster
# (simplified -- full implementation requires GKE ingress or NEG)
```

### terraform/dns.tf

```hcl
# This assumes you use Route 53 for DNS and configure failover there.
# If using Cloud DNS instead:

resource "google_dns_managed_zone" "dr" {
  name     = "datavault-dr-zone"
  dns_name = "dr.datavault.com."
}

# Primary DNS (Route 53) handles the failover routing:
# - Primary record -> AWS ALB (health-checked)
# - Secondary record -> GCP LB IP (health-checked)
# Configured via AWS CLI or Terraform aws_route53_record resources.
```

### terraform/monitoring.tf

```hcl
resource "google_monitoring_uptime_check_config" "app_health" {
  display_name = "Datavault DR Health Check"
  timeout      = "10s"
  period       = "60s"

  http_check {
    port         = 443
    use_ssl      = true
    validate_ssl = true
    path         = "/health"
  }

  monitored_resource {
    type   = "uptime_url"
    labels = {
      project_id = var.gcp_project_id
      host       = "dr.datavault.com"
    }
  }
}

resource "google_monitoring_alert_policy" "dr_health" {
  display_name = "DR Health Check Failure"
  combiner     = "OR"

  conditions {
    display_name = "Health check failed"
    condition_threshold {
      filter          = "resource.type = \"uptime_url\" AND metric.type = \"monitoring.googleapis.com/uptime_check/check_passed\""
      comparison      = "COMPARISON_GT"
      threshold_value = 2
      duration        = "120s"
      aggregations {
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_NEXT_OLDER"
      }
    }
  }

  notification_channels = []  # Add your notification channel IDs
}
```

## Part 3: DR Runbook

### Failover Procedure

```bash
#!/bin/bash
# failover.sh -- DataVault DR Failover Script
# Estimated time: 15-25 minutes

set -euo pipefail

echo "=== DATAVAULT FAILOVER PROCEDURE ==="
echo "Started at: $(date -u)"

# Step 1: Confirm failure (2 minutes)
echo "[1/8] Confirming failure..."
# Check AWS health
for i in $(seq 1 3); do
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    https://app.datavault.com/health --max-time 10 || echo "000")
  if [ "$HTTP_CODE" = "200" ]; then
    echo "ABORT: Health check returned 200. False alarm?"
    exit 1
  fi
  echo "  Attempt $i: HTTP $HTTP_CODE (expected failure)"
  sleep 10
done
echo "  Confirmed: Application is unreachable."

# Step 2: Scale up GKE cluster (5-10 minutes)
echo "[2/8] Scaling GKE cluster to full capacity..."
gcloud container clusters update datavault-dr-cluster \
  --region=us-central1 \
  --enable-autoscaling \
  --min-nodes=5 \
  --max-nodes=20
echo "  Cluster scaling initiated. Waiting for nodes..."
kubectl wait --for=condition=Ready nodes --all --timeout=600s

# Step 3: Promote Cloud SQL to primary (2 minutes)
echo "[3/8] Promoting Cloud SQL replica..."
gcloud sql instances promote-replica datavault-dr-postgres
echo "  Cloud SQL promoted to primary."

# Step 4: Resize Cloud SQL if needed (5-10 minutes)
echo "[4/8] Resizing Cloud SQL to production tier..."
gcloud sql instances patch datavault-dr-postgres \
  --tier=db-custom-4-16384
echo "  Cloud SQL resize initiated."

# Step 5: Deploy application (3-5 minutes)
echo "[5/8] Deploying application to GKE..."
helm upgrade --install datavault ./chart \
  --namespace production \
  --set cloudProvider=gcp \
  --set database.host=$(gcloud sql instances describe datavault-dr-postgres \
    --format="value(ipAddresses[0].ipAddress)") \
  --wait --timeout=300s

# Step 6: Switch DNS (1-2 minutes)
echo "[6/8] Switching DNS to GCP..."
# Update Route 53 to point to GCP LB
aws route53 change-resource-record-sets \
  --hosted-zone-id Z1234567890 \
  --change-batch '{
    "Changes": [{
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "app.datavault.com",
        "Type": "A",
        "SetIdentifier": "primary",
        "Failover": "PRIMARY",
        "TTL": 60,
        "ResourceRecords": [{"Value": "'$(gcloud compute global-addresses describe datavault-dr-ip --format="value(address)")'"}],
        "HealthCheckId": "gcp-health-check-id"
      }
    }]
  }'

# Step 7: Verify (3 minutes)
echo "[7/8] Verifying application health on GCP..."
for i in $(seq 1 6); do
  sleep 10
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    https://app.datavault.com/health --max-time 10 || echo "000")
  if [ "$HTTP_CODE" = "200" ]; then
    echo "  Application healthy on GCP."
    break
  fi
  echo "  Attempt $i: HTTP $HTTP_CODE"
done

# Step 8: Notify stakeholders
echo "[8/8] Sending notifications..."
# Slack, PagerDuty, email notifications
echo "=== FAILOVER COMPLETE at $(date -u) ==="
```

### Failback Procedure

```bash
#!/bin/bash
# failback.sh -- DataVault Failback to AWS
# Estimated time: 2-4 hours (includes data verification)

set -euo pipefail

echo "=== DATAVAULT FAILBACK PROCEDURE ==="

# Step 1: Rebuild AWS infrastructure
echo "[1/7] Rebuilding AWS environment..."
terraform -chdir=./aws-primary apply -auto-approve

# Step 2: Set up reverse replication (GCP -> AWS)
echo "[2/7] Configuring reverse logical replication..."
# On Cloud SQL (now primary): create publication
gcloud sql connect datavault-dr-postgres --user=datavault_app
# SQL> ALTER SYSTEM SET wal_level = 'logical';
# SQL> CREATE PUBLICATION failback_pub FOR ALL TABLES;

# On RDS (now replica): create subscription
# SQL> CREATE SUBSCRIPTION failback_sub
#        CONNECTION 'host=<cloud-sql-ip> dbname=datavault user=replicator'
#        PUBLICATION failback_pub;

# Step 3: Wait for replication to catch up
echo "[3/7] Waiting for replication lag to reach zero..."
while true; do
  LAG=$(psql -h $RDS_HOST -U replicator -t -c \
    "SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp()))::int")
  echo "  Replication lag: ${LAG}s"
  if [ "$LAG" -lt 5 ]; then
    echo "  Replication caught up."
    break
  fi
  sleep 30
done

# Step 4: Data consistency check
echo "[4/7] Running data consistency checks..."
# Compare row counts for critical tables
TABLES=("users" "events" "analytics" "audit_logs")
for table in "${TABLES[@]}"; do
  GCP_COUNT=$(gcloud sql connect datavault-dr-postgres --user=datavault_app \
    -q -c "SELECT COUNT(*) FROM $table" | tail -1)
  AWS_COUNT=$(psql -h $RDS_HOST -U datavault_app -t -c \
    "SELECT COUNT(*) FROM $table" | tr -d ' ')
  echo "  $table: GCP=$GCP_COUNT, AWS=$AWS_COUNT"
  if [ "$GCP_COUNT" != "$AWS_COUNT" ]; then
    echo "  WARNING: Row count mismatch on $table!"
  fi
done

# Step 5: Switch DNS back to AWS
echo "[5/7] Switching DNS back to AWS..."
aws route53 change-resource-record-sets \
  --hosted-zone-id Z1234567890 \
  --change-batch '{
    "Changes": [{
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "app.datavault.com",
        "Type": "A",
        "SetIdentifier": "primary",
        "Failover": "PRIMARY",
        "TTL": 60,
        "ResourceRecords": [{"Value": "<aws-alb-ip>"}],
        "HealthCheckId": "<aws-health-check-id>"
      }
    }]
  }'

# Step 6: Verify on AWS
echo "[6/7] Verifying application health on AWS..."
# Same health check loop as failover

# Step 7: Scale down GCP to warm standby
echo "[7/7] Scaling down GCP to warm standby..."
gcloud container clusters update datavault-dr-cluster \
  --region=us-central1 \
  --enable-autoscaling \
  --min-nodes=1 \
  --max-nodes=2

# Re-establish forward replication (AWS -> GCP)
# ... reverse the publication/subscription setup

echo "=== FAILBACK COMPLETE at $(date -u) ==="
```

### Testing Schedule

| Test | Frequency | Scope | Duration |
|------|-----------|-------|----------|
| Health check validation | Daily (automated) | Verify all DR components are ready | 5 min |
| Database replication lag check | Daily (automated) | Confirm RPO is achievable | 2 min |
| Partial failover (DB only) | Quarterly | Promote Cloud SQL, run queries, demote | 2 hours |
| Full failover | Annually | Complete failover with synthetic traffic | 4 hours |
| Chaos engineering | Quarterly | Kill AWS instances, test auto-recovery | 1 hour |

## Part 4: RPO/RTO Validation

| Component | Replication Method | Actual RPO | Target RPO | Actual RTO | Target RTO | Gap? |
|-----------|-------------------|------------|------------|------------|------------|------|
| PostgreSQL | Logical replication | ~5 seconds | 15 min | 10 min (promote + resize) | 30 min | No |
| Object Storage | Transfer Service (hourly) | ~1 hour | 1 hour | 5 min (DNS switch) | 30 min | No |
| Application State | Stateless | N/A | N/A | 10 min (scale GKE) | 30 min | No |
| DNS | Route 53 failover | ~60 sec (TTL) | 5 min | 1-2 min | 5 min | No |
| Secrets | 15-min sync | ~15 min | 15 min | 0 (already synced) | 30 min | No |

**Identified gap**: Object storage RPO is tight (exactly 1 hour). If S3
data changes frequently, consider increasing sync frequency to every 15
minutes or using S3 Event Notifications + a custom replication service for
near-real-time sync.

**Cost estimate**: This DR environment costs approximately $400-600/month
during warm standby (1 GKE node, small Cloud SQL, minimal storage). During
failover, costs scale to $2,000-3,000/month for full capacity. This is
justified against the business impact of 30+ minutes of downtime for a
10,000 RPS SaaS platform.
