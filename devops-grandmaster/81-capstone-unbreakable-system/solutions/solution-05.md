# Solution 05: Full System Deployment

## Part A: GitOps Repository Structure

```
payment-platform/
|-- .github/
|   `-- workflows/
|       |-- ci.yaml                    # Build, test, push image
|       |-- cd-staging.yaml            # Deploy to staging
|       `-- cd-production.yaml         # Deploy to production (manual gate)
|
|-- base/
|   |-- kustomization.yaml             # Base resources shared by all envs
|   |-- namespace.yaml
|   |-- deployment.yaml
|   |-- service.yaml
|   |-- ingress.yaml
|   |-- hpa.yaml
|   |-- pdb.yaml
|   |-- network-policy.yaml
|   |-- service-monitor.yaml
|   `-- prometheus-rules.yaml
|
|-- infrastructure/
|   |-- prometheus/
|   |   |-- kustomization.yaml
|   |   |-- values.yaml                # Helm values for kube-prometheus-stack
|   |   `-- alertmanager-config.yaml
|   |-- grafana/
|   |   |-- kustomization.yaml
|   |   |-- values.yaml
|   |   |-- dashboards/
|   |   |   |-- payment-api.json
|   |   |   |-- database.json
|   |   |   `-- chaos-experiments.json
|   |   `-- datasources.yaml
|   |-- loki/
|   |   |-- kustomization.yaml
|   |   |-- values.yaml
|   |   `-- promtail-config.yaml
|   |-- jaeger/
|   |   |-- kustomization.yaml
|   |   `-- values.yaml
|   `-- otel-collector/
|       |-- kustomization.yaml
|       `-- config.yaml
|
|-- database/
|   |-- primary/
|   |   |-- kustomization.yaml
|   |   |-- statefulset.yaml
|   |   |-- service.yaml
|   |   `-- configmap.yaml             # postgresql.conf
|   |-- replica/
|   |   |-- kustomization.yaml
|   |   |-- statefulset.yaml
|   |   `-- service.yaml
|   `-- backup/
|       |-- kustomization.yaml
|       |-- backup-cronjob.yaml
|       |-- wal-archiver-cronjob.yaml
|       `-- recovery-rehearsal-cronjob.yaml
|
|-- chaos/
|   |-- kustomization.yaml
|   |-- pod-kill.yaml
|   |-- network-latency.yaml
|   |-- db-failover.yaml
|   `-- node-drain.yaml
|
|-- overlays/
|   |-- staging/
|   |   |-- us-east-1/
|   |   |   |-- kustomization.yaml     # References ../../base + patches
|   |   |   |-- replica-count.yaml     # Patch: 1 replica for staging
|   |   |   `-- resource-limits.yaml   # Patch: lower limits for staging
|   |   `-- eu-west-1/
|   |       |-- kustomization.yaml
|   |       |-- replica-count.yaml
|   |       `-- resource-limits.yaml
|   `-- production/
|       |-- us-east-1/
|       |   |-- kustomization.yaml     # References ../../base + patches
|       |   |-- replica-count.yaml     # Patch: 3 replicas for production
|       |   |-- resource-limits.yaml   # Patch: production limits
|       |   `-- ingress-patch.yaml     # Patch: production domain
|       `-- eu-west-1/
|           |-- kustomization.yaml
|           |-- replica-count.yaml
|           |-- resource-limits.yaml
|           `-- ingress-patch.yaml
|
|-- tests/
|   |-- smoke-test.sh                  # Post-deployment smoke tests
|   |-- load-test.js                   # k6 load test script
|   `-- integration-test.yaml          # K8s Job for integration tests
|
|-- docs/
|   |-- README.md                      # System documentation
|   |-- RUNBOOK.md                     # Incident runbooks
|   |-- ARCHITECTURE.md                # Architecture decision records
|   `-- PRODUCTION_READINESS.md        # Checklist
|
|-- Makefile                           # Convenience commands
|-- .kubevalignore                     # Files to skip during validation
`-- renovate.json                      # Dependency update automation
```

**Why this structure works:**

- `base/` contains the canonical manifests. Every environment inherits
  from base and patches only what differs. This eliminates drift between
  environments.
- `overlays/` uses Kustomize for environment-specific overrides. Staging
  gets fewer replicas and lower resource limits. Production gets full
  replicas and production-grade settings.
- `infrastructure/` is separate from the application because it changes
  on a different cadence. Prometheus and Grafana are updated monthly;
  the application is updated daily.
- `chaos/` is a first-class citizen, not an afterthought. Chaos
  experiments are version-controlled and deployed like any other
  resource.
- `tests/` contains executable tests, not documentation. The smoke
  test runs after every deployment.

**Example kustomization.yaml for production/us-east-1:**

```yaml
# overlays/production/us-east-1/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: payment-system

resources:
  - ../../base
  - ../../database/primary
  - ../../database/backup

patchesStrategicMerge:
  - replica-count.yaml
  - resource-limits.yaml
  - ingress-patch.yaml

commonLabels:
  environment: production
  region: us-east-1

images:
  - name: payment-api
    newName: 123456789.dkr.ecr.us-east-1.amazonaws.com/payment-api
    newTag: v1.2.3
```

## Part B: CI/CD Pipeline

```yaml
# .github/workflows/cd-production.yaml
name: Deploy to Production

on:
  push:
    branches: [main]
    paths:
      - 'base/**'
      - 'overlays/production/**'

env:
  IMAGE: 123456789.dkr.ecr.us-east-1.amazonaws.com/payment-api

jobs:
  validate:
    name: Validate Manifests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install kubeconform
        run: |
          curl -sSL https://github.com/yannh/kubeconform/releases/latest/download/kubeconform-linux-amd64.tar.gz | tar xzf -
          sudo mv kubeconform /usr/local/bin/

      - name: Validate all manifests
        run: |
          kubeconform \
            -strict \
            -summary \
            -output json \
            -kubernetes-version 1.29.0 \
            -schema-location default \
            -schema-location 'https://raw.githubusercontent.com/datreeio/CRDs-catalog/main/{{.Group}}/{{.ResourceKind}}_{{.ResourceAPIVersion}}.json' \
            overlays/production/ > validation-report.json

          cat validation-report.json | jq .

          # Fail if any errors
          if jq -e '.summary.errors > 0' validation-report.json; then
            echo "Validation failed!"
            exit 1
          fi

  build:
    name: Build and Push Image
    needs: validate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: |
          docker build -t ${{ env.IMAGE }}:${{ github.sha }} .
          docker tag ${{ env.IMAGE }}:${{ github.sha }} ${{ env.IMAGE }}:latest

      - name: Run Trivy vulnerability scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: '${{ env.IMAGE }}:${{ github.sha }}'
          format: 'table'
          exit-code: '1'
          severity: 'CRITICAL,HIGH'

      - name: Push to ECR
        run: |
          aws ecr get-login-password | docker login --username AWS --password-stdin 123456789.dkr.ecr.us-east-1.amazonaws.com
          docker push ${{ env.IMAGE }}:${{ github.sha }}
          docker push ${{ env.IMAGE }}:latest

  deploy-staging:
    name: Deploy to Staging
    needs: build
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v4

      - name: Update image tag
        run: |
          cd overlays/staging/us-east-1
          kustomize edit set image payment-api=${{ env.IMAGE }}:${{ github.sha }}

      - name: Deploy to staging
        run: |
          kubectl config use-context staging-us-east-1
          kubectl apply -k overlays/staging/us-east-1/
          kubectl rollout status deployment/payment-api -n payment-system --timeout=300s

      - name: Run smoke tests
        run: |
          chmod +x tests/smoke-test.sh
          STAGING_URL="https://staging-api.payments.example.com" \
          ./tests/smoke-test.sh

  deploy-production:
    name: Deploy to Production
    needs: deploy-staging
    runs-on: ubuntu-latest
    environment:
      name: production
      url: https://api.payments.example.com
    steps:
      - uses: actions/checkout@v4

      - name: Update image tag
        run: |
          cd overlays/production/us-east-1
          kustomize edit set image payment-api=${{ env.IMAGE }}:${{ github.sha }}
          cd ../eu-west-1
          kustomize edit set image payment-api=${{ env.IMAGE }}:${{ github.sha }}

      - name: Deploy canary to us-east-1 (10% traffic)
        run: |
          kubectl config use-context production-us-east-1

          # Apply Argo Rollouts canary strategy
          kubectl apply -k overlays/production/us-east-1/

          # Wait for canary to be ready (10% of traffic)
          kubectl argo rollouts status payment-api \
            -n payment-system \
            --timeout=300s

      - name: Run canary analysis
        run: |
          # Wait 5 minutes and check error rate
          sleep 300

          ERROR_RATE=$(curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~'5..'}[5m])/rate(http_requests_total[5m])" | jq -r '.data.result[0].value[1]')

          if (( $(echo "$ERROR_RATE > 0.01" | bc -l) )); then
            echo "Canary error rate ${ERROR_RATE} exceeds 1%. Rolling back."
            kubectl argo rollouts abort payment-api -n payment-system
            kubectl argo rollouts undo payment-api -n payment-system
            exit 1
          fi

          echo "Canary analysis passed. Error rate: ${ERROR_RATE}"

      - name: Promote canary to full rollout
        run: |
          kubectl argo rollouts promote payment-api -n payment-system
          kubectl argo rollouts status payment-api \
            -n payment-system \
            --timeout=600s

      - name: Deploy to eu-west-1
        run: |
          kubectl config use-context production-eu-west-1
          kubectl apply -k overlays/production/eu-west-1/
          kubectl rollout status deployment/payment-api \
            -n payment-system \
            --timeout=300s

      - name: Run production smoke tests
        run: |
          chmod +x tests/smoke-test.sh
          PRODUCTION_URL="https://api.payments.example.com" \
          ./tests/smoke-test.sh

      - name: Rollback on failure
        if: failure()
        run: |
          kubectl config use-context production-us-east-1
          kubectl argo rollouts abort payment-api -n payment-system
          kubectl argo rollouts undo payment-api -n payment-system

          kubectl config use-context production-eu-west-1
          kubectl rollout undo deployment/payment-api -n payment-system
```

**Why this works:**

- `kubeconform` validates manifests before deployment. Invalid manifests
  are caught in CI, not in production.
- Trivy scans the image for known vulnerabilities. Critical and high
  vulnerabilities block the build.
- Staging deployment is automatic; production requires manual approval
  via GitHub Environments.
- Canary deployment starts with 10% of traffic. The analysis step waits
  5 minutes and checks the error rate. If it exceeds 1%, the canary is
  automatically aborted and rolled back.
- Both regions are deployed sequentially, not in parallel. If us-east-1
  fails, eu-west-1 is not touched.

## Part C: Production Readiness Checklist

### Reliability

- [ ] **Are all pods running and ready?** Verify: `kubectl get pods -n payment-system -o wide`
- [ ] **Is the HPA configured and active?** Verify: `kubectl get hpa -n payment-system`
- [ ] **Is the PDB configured?** Verify: `kubectl get pdb -n payment-system`
- [ ] **Are resource requests and limits set on all pods?** Verify: `kubectl get pods -n payment-system -o jsonpath='{.items[*].spec.containers[*].resources}'`
- [ ] **Are liveness and readiness probes configured?** Verify: Check deployment manifest
- [ ] **Is `maxUnavailable: 0` set for rolling updates?** Verify: Check deployment strategy
- [ ] **Are topology spread constraints configured?** Verify: Check deployment manifest
- [ ] **Is the system deployed across at least 2 availability zones?** Verify: `kubectl get nodes -L topology.kubernetes.io/zone`

### Observability

- [ ] **Is Prometheus scraping the application?** Verify: Check Prometheus targets page
- [ ] **Are alert rules configured for error rate, latency, and restarts?** Verify: `kubectl get prometheusrules -n payment-system`
- [ ] **Are logs being collected by Loki/Promtail?** Verify: Query Loki for recent logs
- [ ] **Are distributed traces being collected?** Verify: Check Jaeger for recent traces
- [ ] **Is the Grafana dashboard displaying live data?** Verify: Open the dashboard
- [ ] **Is Alertmanager routing to PagerDuty and Slack?** Verify: Check Alertmanager config
- [ ] **Are recording rules configured for expensive queries?** Verify: Check PrometheusRule manifest

### Security

- [ ] **Does the pod run as non-root?** Verify: `securityContext.runAsNonRoot: true`
- [ ] **Is the root filesystem read-only?** Verify: `securityContext.readOnlyRootFilesystem: true`
- [ ] **Are capabilities dropped?** Verify: `securityContext.capabilities.drop: [ALL]`
- [ ] **Is a Network Policy applied?** Verify: `kubectl get networkpolicies -n payment-system`
- [ ] **Are secrets stored in Kubernetes Secrets (not in manifests)?** Verify: Check deployment for `secretKeyRef`
- [ ] **Is TLS terminated at the ingress?** Verify: Check Ingress manifest for `tls` section
- [ ] **Has the container image been scanned for vulnerabilities?** Verify: Check Trivy scan results

### Recovery

- [ ] **Is the database backup CronJob running?** Verify: `kubectl get cronjobs -n database`
- [ ] **Has a backup been successfully restored in the last 7 days?** Verify: Check Slack for recovery rehearsal results
- [ ] **Is the failover runbook up to date?** Verify: Check `docs/RUNBOOK.md` last updated date
- [ ] **Have chaos experiments been run in the last 30 days?** Verify: Check chaos dashboard
- [ ] **Is the RTO/RPO documented and tested?** Verify: Check recovery rehearsal results

### Scaling

- [ ] **Can the HPA scale to 10x the current replica count?** Verify: Check HPA `maxReplicas`
- [ ] **Are there sufficient cluster resources for 10x scale?** Verify: `kubectl describe nodes` and compare to quota
- [ ] **Is the database connection pool sized for 10x connections?** Verify: Check `max_connections` in PostgreSQL config
- [ ] **Has a load test been run at 10x traffic?** Verify: Check load test results

### Operations

- [ ] **Is there an on-call rotation?** Verify: Check PagerDuty schedule
- [ ] **Does the README answer all 7 required questions?** Verify: Read `docs/README.md`
- [ ] **Can the on-call engineer deploy a fix in under 15 minutes?** Verify: Time a dry-run deployment
- [ ] **Is the escalation path documented?** Verify: Check `docs/RUNBOOK.md`

## Part D: Smoke Test Suite

```bash
#!/bin/bash
# tests/smoke-test.sh
set -euo pipefail

URL="${PRODUCTION_URL:-${STAGING_URL:-http://localhost:8080}}"
REPORT=""
PASS=true
TOTAL=0
PASSED=0

run_test() {
  local name="$1"
  local result="$2"
  TOTAL=$((TOTAL + 1))

  if [ "$result" = "PASS" ]; then
    PASSED=$((PASSED + 1))
    REPORT="${REPORT}  [PASS] ${name}\n"
  else
    PASS=false
    REPORT="${REPORT}  [FAIL] ${name}\n"
  fi
}

echo "========================================="
echo "  Smoke Test Suite"
echo "  Target: ${URL}"
echo "  Time:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "========================================="
echo ""

# Test 1: Health - All pods running and ready
echo "1. Checking pod health..."
NOT_READY=$(kubectl get pods -n payment-system -l app=payment-api \
  --field-selector=status.phase!=Running -o name 2>/dev/null | wc -l)
if [ "$NOT_READY" -eq 0 ]; then
  run_test "All pods running" "PASS"
else
  run_test "All pods running (${NOT_READY} not running)" "FAIL"
fi

READY_PODS=$(kubectl get pods -n payment-system -l app=payment-api \
  -o jsonpath='{.items[?(@.status.conditions[?(@.type=="Ready")].status=="True")].metadata.name}' | wc -w)
TOTAL_PODS=$(kubectl get pods -n payment-system -l app=payment-api --no-headers | wc -l)
if [ "$READY_PODS" -eq "$TOTAL_PODS" ]; then
  run_test "All pods ready (${READY_PODS}/${TOTAL_PODS})" "PASS"
else
  run_test "All pods ready (${READY_PODS}/${TOTAL_PODS})" "FAIL"
fi

# Test 2: Connectivity - API responds
echo "2. Checking connectivity..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "${URL}/healthz" || echo "000")
if [ "$HTTP_CODE" = "200" ]; then
  run_test "API responds (HTTP ${HTTP_CODE})" "PASS"
else
  run_test "API responds (HTTP ${HTTP_CODE})" "FAIL"
fi

# Test 3: Functionality - Test payment succeeds
echo "3. Testing payment flow..."
PAYMENT_RESPONSE=$(curl -s -X POST "${URL}/api/v1/payments" \
  -H "Authorization: Bearer ${TEST_TOKEN:-test}" \
  -H "Content-Type: application/json" \
  -d '{"amount": 1.00, "currency": "USD", "recipient": "smoke-test"}' \
  --max-time 30 || echo '{"error": "request failed"}')

PAYMENT_STATUS=$(echo "$PAYMENT_RESPONSE" | jq -r '.status // "error"')
if [ "$PAYMENT_STATUS" = "completed" ]; then
  run_test "Test payment succeeds" "PASS"
else
  run_test "Test payment succeeds (status: ${PAYMENT_STATUS})" "FAIL"
fi

# Test 4: Performance - p99 latency below 200ms
echo "4. Checking latency..."
P99=$(curl -s "${URL}/metrics" 2>/dev/null | \
  grep 'http_request_duration_seconds{quantile="0.99"}' | \
  awk '{print $2}' | head -1)

if [ -n "$P99" ]; then
  P99_MS=$(echo "$P99 * 1000" | bc | cut -d. -f1)
  if [ "$P99_MS" -lt 200 ]; then
    run_test "p99 latency (${P99_MS}ms < 200ms)" "PASS"
  else
    run_test "p99 latency (${P99_MS}ms >= 200ms)" "FAIL"
  fi
else
  run_test "p99 latency (metric not found)" "FAIL"
fi

# Test 5: Observability - Metrics flowing
echo "5. Checking observability..."
METRICS_COUNT=$(curl -s "http://prometheus:9090/api/v1/query?query=up{app='payment-api'}" | \
  jq -r '.data.result | length' 2>/dev/null || echo "0")
if [ "$METRICS_COUNT" -gt 0 ]; then
  run_test "Prometheus scraping (${METRICS_COUNT} targets)" "PASS"
else
  run_test "Prometheus scraping (0 targets)" "FAIL"
fi

LOGS_COUNT=$(curl -s "http://loki:3100/loki/api/v1/query?query={app='payment-api'}&limit=1" | \
  jq -r '.data.result | length' 2>/dev/null || echo "0")
if [ "$LOGS_COUNT" -gt 0 ]; then
  run_test "Loki collecting logs" "PASS"
else
  run_test "Loki collecting logs (no logs found)" "FAIL"
fi

# Test 6: Failover - Survive pod kill
echo "6. Testing pod failover..."
POD_TO_KILL=$(kubectl get pods -n payment-system -l app=payment-api \
  -o jsonpath='{.items[0].metadata.name}')

kubectl delete pod "$POD_TO_KILL" -n payment-system --grace-period=0 &
KILL_PID=$!

# Wait for new pod to be ready
sleep 15
REPLACED=$(kubectl get pods -n payment-system -l app=payment-api \
  --field-selector=status.phase=Running --no-headers | wc -l)

if [ "$REPLACED" -ge "$TOTAL_PODS" ]; then
  run_test "Pod failover (killed ${POD_TO_KILL}, ${REPLACED} pods running)" "PASS"
else
  run_test "Pod failover (killed ${POD_TO_KILL}, only ${REPLACED} pods running)" "FAIL"
fi

# Final API check after failover
POST_FAILOVER_CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "${URL}/healthz" || echo "000")
if [ "$POST_FAILOVER_CODE" = "200" ]; then
  run_test "API healthy after failover" "PASS"
else
  run_test "API healthy after failover (HTTP ${POST_FAILOVER_CODE})" "FAIL"
fi

# Print report
echo ""
echo "========================================="
echo "  Results: ${PASSED}/${TOTAL} passed"
echo "========================================="
echo ""
echo -e "$REPORT"

if [ "$PASS" = true ]; then
  echo "  Status: ALL TESTS PASSED"
  echo "========================================="
  exit 0
else
  echo "  Status: SOME TESTS FAILED"
  echo "========================================="
  exit 1
fi
```

## Part E: System Documentation

```markdown
# Payment Platform - Production System

## What does the system do?

The payment platform processes real-time financial transactions for
our customers. It accepts payment requests via a REST API, validates
them through a fraud detection service, records them in a ledger,
and sends confirmation notifications. The system handles 50,000
requests per second at peak with sub-200ms p99 latency and is
designed for 99.999% availability (five nines).

## How is it deployed?

```
                    Global DNS (Route53)
                    /        |        \
               us-east-1  eu-west-1  ap-south-1
               [K8s]      [K8s]      [K8s]
               [DB]       [DB]       [DB]
```

Deployment process:
1. Push code to `main` branch
2. CI validates manifests, builds image, runs tests
3. CD deploys to staging automatically
4. Smoke tests run against staging
5. Manual approval gates production deployment
6. Canary deployment (10% traffic) with automated analysis
7. Full rollout if error rate stays below 1%
8. Automatic rollback if smoke tests fail

Deploy command (manual): `make deploy-production`

## How do you debug an incident?

Key dashboards:
- Payment API Overview: https://grafana.example.com/d/payment-api
- Database Health: https://grafana.example.com/d/database
- Chaos Experiments: https://grafana.example.com/d/chaos

Debug steps:
1. Check the alert that fired (PagerDuty or Slack)
2. Open the relevant Grafana dashboard
3. Check Loki for error logs: `{app="payment-api", level="error"}`
4. Check Jaeger for slow traces
5. Follow the runbook: `docs/RUNBOOK.md`

Runbooks:
- High Error Rate: `docs/RUNBOOK.md#high-error-rate`
- High Latency: `docs/RUNBOOK.md#high-latency`
- Database Failover: `docs/RUNBOOK.md#database-failover`
- Region Failover: `docs/RUNBOOK.md#region-failover`

## How do you scale it?

Automatic scaling:
- HPA scales pods between 3 and 20 based on CPU (70%) and memory (80%)
- Scale-up: 4 pods per minute (fast)
- Scale-down: 10% per minute (slow, prevents flapping)

Manual scaling:
```bash
kubectl scale deployment/payment-api -n payment-system --replicas=10
```

Cluster scaling:
```bash
# Add nodes to the cluster
aws eks update-nodegroup-config --cluster-name payment-cluster \
  --nodegroup-name primary --scaling-config minSize=3,maxSize=50
```

When to scale manually:
- Before known traffic events (sales, launches)
- When HPA is at max replicas for > 10 minutes
- When planning a large deployment

## How do you back it up?

Database backups:
- Full backup: every 6 hours to S3 (cross-region)
- WAL archive: every 5 minutes to S3
- Retention: 30 days
- Storage: s3://payments-backup-eu-west-1

Verify backup health:
```bash
kubectl get cronjobs -n database
kubectl logs -n database job/postgres-backup --tail=50
```

Restore procedure:
```bash
# See docs/RUNBOOK.md#database-restore
```

Recovery rehearsal runs daily at 4 AM UTC. Results are posted to
Slack #payments-ops.

## How do you update it?

Application update:
1. Merge PR to `main`
2. CI/CD pipeline runs automatically
3. Staging deployment is automatic
4. Production deployment requires approval
5. Canary deployment validates before full rollout

Rollback:
```bash
# Automatic (on smoke test failure):
# The CI/CD pipeline handles this automatically

# Manual rollback:
kubectl argo rollouts undo payment-api -n payment-system
```

Infrastructure update (Prometheus, Grafana, etc.):
1. Update Helm values in `infrastructure/`
2. PR review required
3. Apply to staging first
4. Apply to production after staging validation

## Who do you page?

On-call rotation: PagerDuty schedule "payment-platform-oncall"

Escalation path:
1. Primary on-call (PagerDuty, 5-minute acknowledgment)
2. Secondary on-call (PagerDuty, 10-minute escalation)
3. Engineering manager (phone call, 15-minute escalation)
4. VP Engineering (phone call, 30-minute escalation, P1 only)

Emergency contacts:
- Slack: #payments-incidents
- PagerDuty: payment-platform service
- Email: payments-oncall@example.com
```

## Common Mistakes

1. **Using `latest` tag in production.** The `latest` tag is mutable --
   the same tag can point to different images at different times. Always
   use an immutable tag (git SHA or semantic version) so you know exactly
   what is running.

2. **Deploying to all regions simultaneously.** If a bad deployment
   breaks us-east-1 and eu-west-1 at the same time, you have no
   fallback. Deploy sequentially: validate in the first region, then
   deploy to the second.

3. **Skipping the canary analysis step.** The canary deployment is
   useless without analysis. The 5-minute wait and error rate check
   catch issues that pass unit tests but fail under production traffic.
   Do not skip it to save time.

4. **Not automating the rollback.** If rollback requires a human to
   notice the failure, log in, and run a command, you have already
   lost 10+ minutes of downtime. The CI/CD pipeline must roll back
   automatically on smoke test failure.

5. **Writing documentation for yourself instead of the 3 AM on-call
   engineer.** The README must be written for someone who has never
   seen the system before and is panicking. Every link must work.
   Every command must be copy-pasteable. Every assumption must be
   stated. Test the documentation by having someone else follow it.
