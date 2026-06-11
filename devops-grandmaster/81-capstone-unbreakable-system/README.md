# Module 81: Capstone — The Unbreakable System

> **Previous Module:** [80 - Cost Optimization](../80-cost-optimization/README.md)

## The Ultimate Challenge

You have learned 80 modules of DevOps mastery. Now prove it. Build a production system that demonstrates every concept from this curriculum — a system that is observable, scalable, secure, resilient, and self-healing. A system that does not break.

This is not a toy project. This is a reference architecture for production systems.

## Architecture Overview

```
                    +------------------+
                    |   CloudFlare CDN  |
                    |   (Edge Cache)    |
                    +--------+---------+
                             |
                    +--------v---------+
                    |   Global Load     |
                    |   Balancer        |
                    +--------+---------+
                             |
              +--------------+--------------+
              |                             |
     +--------v--------+          +--------v--------+
     |  K8s Cluster     |          |  K8s Cluster     |
     |  (Primary)       |          |  (DR/Secondary)  |
     |  AWS us-east-1   |          |  GCP us-central1  |
     +--------+---------+          +------------------+
              |
     +--------v---------+
     |  Ingress (Nginx)  |
     |  Rate Limiting    |
     |  TLS Termination  |
     +--------+---------+
              |
     +--------v---------+
     |  API Gateway      |
     |  Auth, Routing    |
     +--------+---------+
              |
     +--------+--------+--------+
     |        |        |        |
   +--v--+  +-v--+  +--v--+  +--v--+
   | Web |  | API |  | Auth |  | Worker |
   | Svc |  | Svc |  | Svc  |  | Svc   |
   +--+--+  +--+--+  +--+--+  +--+---+
      |        |        |        |
      +--------+--------+--------+
               |
     +--------v---------+
     |  Service Mesh     |
     |  (Istio/Linkerd)  |
     |  mTLS, Observability |
     +--------+---------+
              |
     +--------+--------+
     |                  |
   +-v---+          +---v--+
   | DB  |          | Redis |
   | HA  |          | Cluster|
   +-----+          +-------+
```

## Phase 1: Containerized Microservices

### Application Services

```yaml
# web-service.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: production
  labels:
    app: web
    version: v1
    team: platform
    cost-center: engineering
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    metadata:
      labels:
        app: web
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
    spec:
      serviceAccountName: web
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
        - name: web
          image: registry.example.com/web:v1.0.0@sha256:abc123
          ports:
            - containerPort: 8080
              name: http
          env:
            - name: API_URL
              value: "http://api.production.svc.cluster.local:8080"
            - name: LOG_LEVEL
              valueFrom:
                configMapKeyRef:
                  name: web-config
                  key: log-level
          resources:
            requests:
              cpu: 250m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 10
          lifecycle:
            preStop:
              exec:
                command: ["/bin/sh", "-c", "sleep 10"]
          volumeMounts:
            - name: tmp
              mountPath: /tmp
      volumes:
        - name: tmp
          emptyDir:
            medium: Memory
            sizeLimit: 100Mi
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: topology.kubernetes.io/zone
          whenUnsatisfiable: DoNotSchedule
          labelSelector:
            matchLabels:
              app: web

---
apiVersion: v1
kind: Service
metadata:
  name: web
  namespace: production
spec:
  selector:
    app: web
  ports:
    - port: 80
      targetPort: 8080
      name: http

---
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: web
  namespace: production
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: web
```

```yaml
# api-service.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
  namespace: production
  labels:
    app: api
    version: v1
    team: backend
    cost-center: engineering
spec:
  replicas: 5
  selector:
    matchLabels:
      app: api
  template:
    metadata:
      labels:
        app: api
        version: v1
    spec:
      serviceAccountName: api
      containers:
        - name: api
          image: registry.example.com/api:v1.0.0@sha256:def456
          ports:
            - containerPort: 8080
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: database-credentials
                  key: url
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: redis-credentials
                  key: url
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 1
              memory: 1Gi
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 5
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 10
```

### Dockerfile (Multi-Stage, Security-Hardened)

```dockerfile
# Dockerfile — Production-ready, security-hardened
# Stage 1: Build
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

# Stage 2: Runtime (minimal image)
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/api /api
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/api"]
```

## Phase 2: Production Logging and Monitoring

### Structured Logging

```yaml
# logging-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: logging-config
  namespace: production
data:
  log-format: json
  log-level: info
  # Fluentd config for log collection
  fluentd.conf: |
    <source>
      @type tail
      path /var/log/containers/*.log
      pos_file /var/log/fluentd-containers.log.pos
      tag kubernetes.*
      format json
      time_key time
      time_format %Y-%m-%dT%H:%M:%S.%NZ
    </source>

    <filter kubernetes.**>
      @type kubernetes_metadata
    </filter>

    <match **>
      @type elasticsearch
      host elasticsearch.logging.svc
      port 9200
      logstash_format true
      logstash_prefix k8s
      <buffer>
        flush_thread_count 8
        flush_interval 5s
        chunk_limit_size 2M
        queue_limit_length 32
        retry_max_interval 30
        retry_forever true
      </buffer>
    </match>
```

### Prometheus Monitoring

```yaml
# prometheus-rules.yaml — Comprehensive alerting
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: production-alerts
  namespace: monitoring
spec:
  groups:
    - name: application.rules
      rules:
        - alert: HighErrorRate
          expr: |
            sum(rate(http_requests_total{namespace="production",status=~"5.."}[5m])) by (service)
            / sum(rate(http_requests_total{namespace="production"}[5m])) by (service) > 0.05
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "High error rate on {{ $labels.service }}"
            runbook: "https://wiki/runbooks/high-error-rate"

        - alert: HighLatency
          expr: |
            histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{namespace="production"}[5m])) > 2
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "High p99 latency on {{ $labels.service }}"

    - name: infrastructure.rules
      rules:
        - alert: PodCrashLooping
          expr: rate(kube_pod_container_status_restarts_total{namespace="production"}[15m]) > 0
          for: 5m
          labels:
            severity: critical

        - alert: NodeNotReady
          expr: kube_node_status_condition{condition="Ready",status="true"} == 0
          for: 5m
          labels:
            severity: critical

        - alert: PersistentVolumeFillingUp
          expr: |
            (kubelet_volume_stats_available_bytes / kubelet_volume_stats_capacity_bytes) < 0.1
          for: 10m
          labels:
            severity: warning
```

## Phase 3: Load Balancing and Scaling

### Ingress Configuration

```yaml
# ingress.yaml — Production ingress with TLS and rate limiting
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: production-ingress
  namespace: production
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "10"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "30"
spec:
  tls:
    - hosts:
        - app.example.com
      secretName: app-tls
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web
                port:
                  number: 80
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api
                port:
                  number: 80
```

### HPA Configuration

```yaml
# hpa.yaml — Auto-scaling based on multiple metrics
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api
  minReplicas: 3
  maxReplicas: 50
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 65
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "1000"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

## Phase 4: Kubernetes Deployment

### Namespace and RBAC

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    istio-injection: enabled
    environment: production

---
# rbac.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: app-role
  namespace: production
rules:
  - apiGroups: [""]
    resources: ["configmaps", "secrets"]
    verbs: ["get", "list", "watch"]
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: app-rolebinding
  namespace: production
subjects:
  - kind: ServiceAccount
    name: api
    namespace: production
roleRef:
  kind: Role
  name: app-role
  apiGroup: rbac.authorization.k8s.io
```

### Network Policies

```yaml
# network-policies.yaml — Zero-trust networking
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-network-policy
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: api
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: web
      ports:
        - port: 8080
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: database
      ports:
        - port: 5432
    - to:
        - podSelector:
            matchLabels:
              app: redis
      ports:
        - port: 6379
    - to:    # Allow DNS
        - namespaceSelector: {}
      ports:
        - port: 53
          protocol: UDP
```

## Phase 5: Full Observability Stack

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Production Overview",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{namespace=\"production\"}[5m])) by (service)"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{namespace=\"production\",status=~\"5..\"}[5m])) / sum(rate(http_requests_total{namespace=\"production\"}[5m]))"
          }
        ],
        "thresholds": {
          "steps": [
            {"color": "green", "value": 0},
            {"color": "yellow", "value": 0.01},
            {"color": "red", "value": 0.05}
          ]
        }
      },
      {
        "title": "Latency (p99)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{namespace=\"production\"}[5m])) by (le, service))"
          }
        ]
      }
    ]
  }
}
```

## Phase 6: Database HA with Backup

```yaml
# postgres-ha.yaml — Highly available PostgreSQL
apiVersion: acid.zalan.do/v1
kind: postgresql
metadata:
  name: production-db
  namespace: database
spec:
  teamId: platform
  volume:
    size: 100Gi
    storageClass: gp3
  numberOfInstances: 3
  users:
    admin:
      - superuser
      - createdb
    app: []
  databases:
    app: app
  postgresql:
    version: "15"
    parameters:
      max_connections: "200"
      shared_buffers: "4GB"
      effective_cache_size: "12GB"
      work_mem: "64MB"
  resources:
    requests:
      cpu: 2
      memory: 8Gi
    limits:
      cpu: 4
      memory: 16Gi
  enableMasterLoadBalancer: true
  enableReplicaLoadBalancer: true

---
# Backup CronJob
apiVersion: batch/v1
kind: CronJob
metadata:
  name: database-backup
  namespace: database
spec:
  schedule: "0 2 * * *"  # 2 AM daily
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: postgres:15
              command:
                - /bin/bash
                - -c
                - |
                  pg_dump -h production-db -U admin -d app | gzip > /backup/app-$(date +%Y%m%d).sql.gz
                  aws s3 cp /backup/app-$(date +%Y%m%d).sql.gz s3://backups/database/
                  # Retain last 30 days
                  find /backup -mtime +30 -delete
              env:
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: database-credentials
                      key: password
          restartPolicy: OnFailure
```

## Phase 7: Security Hardening

```yaml
# security.yaml — Security policies
# Pod Security Standards
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted

---
# OPA/Gatekeeper policy: no privileged containers
apiVersion: templates.gatekeeper.sh/v1beta1
kind: ConstraintTemplate
metadata:
  name: k8spspprivilegedcontainer
spec:
  crd:
    spec:
      names:
        kind: K8sPSPPrivilegedContainer
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package k8spspprivilegedcontainer
        violation[{"msg": msg, "details": {}}] {
          c := input_containers[_]
          c.securityContext.privileged
          msg := sprintf("Privileged container is not allowed: %v, %v", [input.review.object.metadata.name, c.name])
        }
        input_containers[c] {
          c := input.review.object.spec.containers[_]
        }
        input_containers[c] {
          c := input.review.object.spec.initContainers[_]
        }

---
# External Secrets Operator
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: database-credentials
  namespace: production
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: ClusterSecretStore
  target:
    name: database-credentials
    creationPolicy: Owner
  data:
    - secretKey: url
      remoteRef:
        key: production/database
        property: url
```

## Phase 8: CI/CD Pipeline

```yaml
# .github/workflows/deploy.yaml
name: Deploy to Production
on:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build and push image
        run: |
          docker build -t registry.example.com/api:${{ github.sha }} .
          docker push registry.example.com/api:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v4
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/api \
            api=registry.example.com/api:${{ github.sha }} \
            -n production
          kubectl rollout status deployment/api -n production --timeout=300s

      - name: Verify deployment
        run: |
          # Wait for pods to be ready
          kubectl wait --for=condition=ready pod -l app=api -n production --timeout=120s
          # Check health endpoint
          curl -f https://app.example.com/api/health
```

## Phase 9: Network Architecture

```yaml
# service-mesh.yaml — Istio service mesh configuration
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: production
spec:
  profile: default
  meshConfig:
    enableTracing: true
    defaultConfig:
      tracing:
        sampling: 100
  components:
    ingressGateways:
      - name: istio-ingressgateway
        enabled: true
        k8s:
          service:
            type: LoadBalancer

---
# PeerAuthentication — Enforce mTLS
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT

---
# AuthorizationPolicy — Fine-grained access control
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: api-policy
  namespace: production
spec:
  selector:
    matchLabels:
      app: api
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/production/sa/web"]
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/*"]
```

## Phase 10: Auto-Scaling for Traffic Spikes

```yaml
# keda-scaling.yaml — Event-driven auto-scaling
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: order-processor
  namespace: production
spec:
  scaleTargetRef:
    name: order-processor
  pollingInterval: 15
  cooldownPeriod: 300
  minReplicaCount: 2
  maxReplicaCount: 100
  triggers:
    - type: rabbitmq
      metadata:
        host: amqp://rabbitmq.production.svc:5672
        queueName: orders
        queueLength: "5"
    - type: cron
      metadata:
        timezone: America/New_York
        start: "0 8 * * *"
        end: "0 20 * * *"
        desiredReplicas: "10"
```

## Phase 11: Multi-Host Deployment

```hcl
# multi-cluster.tf — Multi-cluster deployment
module "aws_cluster" {
  source = "./modules/eks"
  cluster_name = "production-aws"
  region = "us-east-1"
  node_count = 5
}

module "gcp_cluster" {
  source = "./modules/gke"
  cluster_name = "production-gcp"
  region = "us-central1"
  node_count = 3
}

# Global load balancing
resource "cloudflare_load_balancer" "global" {
  zone_id = var.cloudflare_zone_id
  name = "app.example.com"
  default_pool_ids = [
    cloudflare_load_balancer_pool.aws.id,
    cloudflare_load_balancer_pool.gcp.id,
  ]
  steering_policy = "dynamic_latency"
}
```

## Phase 12: Incident Response Procedures

```yaml
# incident-response.yaml — Automated incident response
# Alertmanager routes alerts to PagerDuty
# PagerDuty pages on-call engineer
# On-call engineer follows runbook
# Incident commander coordinates response
# Status page updated automatically

# Runbooks stored in Git
# Post-mortem template in repository
# Action items tracked in Jira
```

## Deployment Script

```bash
#!/bin/bash
# deploy-unbreakable.sh — Deploy the complete system
set -euo pipefail

echo "=== Deploying The Unbreakable System ==="

# 1. Create clusters
echo "[1/12] Creating Kubernetes clusters..."
terraform apply -auto-approve

# 2. Install infrastructure
echo "[2/12] Installing infrastructure components..."
helm install cert-manager jetstack/cert-manager -n cert-manager --create-namespace
helm install ingress-nginx ingress-nginx/ingress-nginx -n ingress-nginx
helm install prometheus prometheus-community/kube-prometheus-stack -n monitoring --create-namespace
helm install loki grafana/loki-stack -n logging --create-namespace

# 3. Deploy security policies
echo "[3/12] Applying security policies..."
kubectl apply -f security/

# 4. Deploy database
echo "[4/12] Deploying database..."
kubectl apply -f database/

# 5. Deploy application
echo "[5/12] Deploying application services..."
kubectl apply -f k8s/

# 6. Configure monitoring
echo "[6/12] Configuring monitoring..."
kubectl apply -f monitoring/

# 7. Configure logging
echo "[7/12] Configuring logging..."
kubectl apply -f logging/

# 8. Configure service mesh
echo "[8/12] Configuring service mesh..."
kubectl apply -f service-mesh/

# 9. Configure auto-scaling
echo "[9/12] Configuring auto-scaling..."
kubectl apply -f scaling/

# 10. Configure backups
echo "[10/12] Configuring backups..."
kubectl apply -f backups/

# 11. Configure incident response
echo "[11/12] Configuring incident response..."
kubectl apply -f incident-response/

# 12. Verify
echo "[12/12] Verifying deployment..."
kubectl get pods -n production
kubectl get services -n production
kubectl get ingress -n production

echo ""
echo "=== Deployment Complete ==="
echo "Application: https://app.example.com"
echo "Monitoring: https://grafana.example.com"
echo "Logging: https://kibana.example.com"
```

## Validation Checklist

- [ ] **Phase 1:** All services containerized with health checks
- [ ] **Phase 2:** Structured logging and Prometheus monitoring active
- [ ] **Phase 3:** Load balancing and HPA configured
- [ ] **Phase 4:** RBAC, NetworkPolicies, and PodSecurityPolicies applied
- [ ] **Phase 5:** Grafana dashboards showing all key metrics
- [ ] **Phase 6:** Database HA with automated backups
- [ ] **Phase 7:** Security hardening (non-root, read-only FS, secrets management)
- [ ] **Phase 8:** CI/CD pipeline with automated testing and deployment
- [ ] **Phase 9:** Service mesh with mTLS and authorization policies
- [ ] **Phase 10:** Auto-scaling handles 10x traffic spike
- [ ] **Phase 11:** Multi-cluster deployment with failover
- [ ] **Phase 12:** Incident response procedures documented and tested

## Congratulations

You have built a production system that is:
- **Observable:** You know what is happening at all times
- **Scalable:** It handles traffic spikes automatically
- **Secure:** Defense in depth at every layer
- **Resilient:** It recovers from failures automatically
- **Cost-optimized:** You pay only for what you need
- **Well-documented:** Runbooks, post-mortems, and change management

This is not the end. This is the beginning. Production systems are living things — they need constant care, continuous improvement, and relentless attention.

**Welcome to the ranks of DevOps mastery.**
