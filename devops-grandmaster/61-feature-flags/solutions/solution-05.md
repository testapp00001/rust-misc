# Solution 05: Feature Flags in CI/CD Pipeline

## Part A: Pipeline Design

### 1. Pipeline Stages

| Stage | What Happens | Gates | Risks |
|-------|-------------|-------|-------|
| **Build** | Compile code, build container image, run linters | Code compiles, linting passes | Build failures, dependency issues |
| **Test** | Unit tests, integration tests, flag-specific tests (all combinations) | All tests pass, coverage threshold met | Test flakiness, missing coverage |
| **Deploy Staging** | Push to staging environment with all flags disabled | Build succeeded, tests passed | Staging environment issues |
| **Flag Configure** | Set new flag to enabled for test users in staging | Staging deployment healthy | Incorrect flag configuration |
| **Verify Staging** | Run integration tests against staging with flag enabled | Flag-gated tests pass | Feature works differently in staging |
| **Deploy Production** | Push to production with flag disabled | Staging verified | Production deployment failures |
| **Canary Release** | Enable flag for 1% of production traffic | Production deployment healthy | Edge cases in production |
| **Monitor** | Watch error rates, latency, business metrics for canary window | Canary deployed | Slow metric feedback |
| **Progressive Rollout** | Incrementally increase flag percentage through planned stages | Metrics within thresholds at each stage | Threshold too tight/loose |
| **Full Release** | Flag at 100%, mark for code cleanup | All stages passed | Stale flag risk if not cleaned up |

### 2. Flag-Aware Testing Strategy

```
With N flags, you have 2^N combinations. To reduce the matrix:

1. Test each flag independently (N tests):
   - Flag A on, all others off
   - Flag A off, all others default
   - ... for each flag

2. Test all new flags on (1 test):
   - Simulates what production will look like after full rollout

3. Test all flags off (1 test):
   - Baseline: current production behavior

4. Test known interactions (k tests):
   - If flag A and flag B both affect the checkout page, test them together

Total: ~2N + 2 + k test cases instead of 2^N

For critical flags, add:
- Transition test: flag off -> on mid-request (ensures no partial state)
- Rollback test: flag on -> off, verify cleanup
```

### 3. Pipeline Configuration (GitHub Actions)

```yaml
name: Feature Flag CI/CD Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  FLAG_KEY: new-search-algorithm
  REGISTRY: ghcr.io
  IMAGE: ghcr.io/myorg/myapp

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: |
          docker build -t ${{ env.IMAGE }}:${{ github.sha }} .
          docker tag ${{ env.IMAGE }}:${{ github.sha }} ${{ env.IMAGE }}:latest
      - name: Push image
        run: |
          echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u ${{ github.actor }} --password-stdin
          docker push ${{ env.IMAGE }}:${{ github.sha }}

  test:
    needs: build
    runs-on: ubuntu-latest
    strategy:
      matrix:
        flag_state: [all-off, target-flag-on, all-on]
    steps:
      - uses: actions/checkout@v4
      - name: Set flag state for testing
        run: |
          echo "FEATURE_FLAGS={\"${{ env.FLAG_KEY }}\": \"${{ matrix.flag_state }}\"}" > .env.test
      - name: Run tests
        run: |
          docker compose -f docker-compose.test.yml up --abort-on-container-exit
          docker compose -f docker-compose.test.yml down

  deploy-staging:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - name: Deploy to staging
        run: |
          kubectl set image deployment/app \
            app=${{ env.IMAGE }}:${{ github.sha }} \
            --namespace=staging
          kubectl rollout status deployment/app --namespace=staging

  verify-staging:
    needs: deploy-staging
    runs-on: ubuntu-latest
    steps:
      - name: Enable flag for staging test
        run: |
          curl -X POST "${{ secrets.FLAG_SERVICE_URL }}/flags/${{ env.FLAG_KEY }}/environments/staging/toggle" \
            -H "Authorization: Bearer ${{ secrets.FLAG_SERVICE_TOKEN }}" \
            -H "X-Actor: ci-pipeline"
      - name: Run staging integration tests
        run: |
          ./scripts/run-integration-tests.sh --env staging --flag ${{ env.FLAG_KEY }}
      - name: Disable flag in staging
        if: always()
        run: |
          curl -X POST "${{ secrets.FLAG_SERVICE_URL }}/flags/${{ env.FLAG_KEY }}/environments/staging/toggle" \
            -H "Authorization: Bearer ${{ secrets.FLAG_SERVICE_TOKEN }}" \
            -H "X-Actor: ci-pipeline"

  deploy-production:
    needs: verify-staging
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Deploy to production (flag disabled)
        run: |
          kubectl set image deployment/app \
            app=${{ env.IMAGE }}:${{ github.sha }} \
            --namespace=production
          kubectl rollout status deployment/app --namespace=production
      - name: Confirm flag is disabled in production
        run: |
          curl -s "${{ secrets.FLAG_SERVICE_URL }}/flags/${{ env.FLAG_KEY }}/environments/production" \
            -H "Authorization: Bearer ${{ secrets.FLAG_SERVICE_TOKEN }}" \
            | jq '.data.enabled' | grep -q "false"

  canary-release:
    needs: deploy-production
    runs-on: ubuntu-latest
    environment: production-canary
    steps:
      - name: Enable flag at 1%
        run: |
          curl -X POST "${{ secrets.FLAG_SERVICE_URL }}/flags/${{ env.FLAG_KEY }}/environments/production" \
            -H "Authorization: Bearer ${{ secrets.FLAG_SERVICE_TOKEN }}" \
            -H "Content-Type: application/json" \
            -H "X-Actor: ci-pipeline" \
            -d '{"rollout_percentage": 1}'

  monitor-canary:
    needs: canary-release
    runs-on: ubuntu-latest
    timeout-minutes: 120
    steps:
      - name: Monitor canary for 2 hours
        run: |
          ./scripts/monitor-rollout.sh \
            --flag "${{ env.FLAG_KEY }}" \
            --environment production \
            --duration 7200 \
            --max-error-rate 0.01 \
            --max-latency-p99 500

  output-rollout-instructions:
    needs: monitor-canary
    runs-on: ubuntu-latest
    steps:
      - name: Print rollout instructions
        run: |
          echo "## Canary Verified" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "Flag '${{ env.FLAG_KEY }}' is at 1% in production." >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "### Next Steps" >> $GITHUB_STEP_SUMMARY
          echo "1. Monitor dashboards for 24h" >> $GITHUB_STEP_SUMMARY
          echo "2. Increase to 5%: \`./scripts/rollout.sh ${{ env.FLAG_KEY }} 5\`" >> $GITHUB_STEP_SUMMARY
          echo "3. Continue progressive rollout per the plan" >> $GITHUB_STEP_SUMMARY
          echo "4. If issues: \`./scripts/rollback.sh ${{ env.FLAG_KEY }}\`" >> $GITHUB_STEP_SUMMARY
```

---

## Part B: Deployment Patterns

### 4. Branch by Abstraction

```python
from abc import ABC, abstractmethod

# Interface
class SearchProvider(ABC):
    @abstractmethod
    def search(self, query: str, limit: int) -> list[dict]:
        pass

# Legacy implementation
class LegacySearch(SearchProvider):
    def search(self, query: str, limit: int) -> list[dict]:
        # Simple keyword matching
        return database.query("SELECT * FROM products WHERE name LIKE ?", f"%{query}%", limit)

# New implementation
class MLSearch(SearchProvider):
    def search(self, query: str, limit: int) -> list[dict]:
        # ML-powered semantic search
        embedding = ml_model.encode(query)
        return vector_db.nearest_neighbors(embedding, limit)

# Flag-based factory
class SearchProviderFactory:
    def __init__(self, flag_service, flag_key="new-search-algorithm"):
        self.flag_service = flag_service
        self.flag_key = flag_key

    def get_provider(self, context: dict) -> SearchProvider:
        use_new = self.flag_service.evaluate(
            self.flag_key, "production", context
        )
        if use_new:
            return MLSearch()
        return LegacySearch()

# Application code -- no flag logic here
def search_handler(request):
    factory = SearchProviderFactory(flag_service)
    provider = factory.get_provider({"user_id": request.user_id})
    results = provider.search(request.query, limit=20)
    return {"results": results}
```

### 5. Database Migration with Feature Flags

```python
# Step 1: Migration adds the new column (nullable, no default)
"""
ALTER TABLE users ADD COLUMN preference_data JSONB NULL;
"""

# Step 2: Application code handles both old and new columns
class UserRepository:
    def __init__(self, flag_service):
        self.flag_service = flag_service

    def get_preference(self, user_id: str, context: dict) -> dict:
        use_new = self.flag_service.evaluate(
            "preference-data-migration", "production", context
        )

        if use_new:
            # Read from new column
            row = db.query("SELECT preference_data FROM users WHERE id = ?", user_id)
            if row and row.preference_data:
                return json.loads(row.preference_data)

        # Fallback to old column
        row = db.query("SELECT preference_key, preference_value FROM users WHERE id = ?", user_id)
        return {row.preference_key: row.preference_value} if row else {}

    def set_preference(self, user_id: str, data: dict, context: dict):
        use_new = self.flag_service.evaluate(
            "preference-data-migration", "production", context
        )

        if use_new:
            # Write to new column
            db.execute(
                "UPDATE users SET preference_data = ? WHERE id = ?",
                json.dumps(data), user_id
            )
        else:
            # Write to old column(s)
            for key, value in data.items():
                db.execute(
                    "UPDATE users SET preference_key = ?, preference_value = ? WHERE id = ?",
                    key, value, user_id
                )

# Migration rollout steps:
# 1. Deploy code with flag disabled (reads/writes old column)
# 2. Run migration to add preference_data column
# 3. Backfill: UPDATE users SET preference_data = jsonb_build_object(preference_key, preference_value)
# 4. Enable flag at 1% -> 100% (reads new, writes new)
# 5. After 100% for 7 days, remove old column reads
# 6. Later: DROP COLUMN preference_key, preference_value
```

### 6. Microservices Architecture

**Where should the flag live?**

The flag should live in the **API Gateway** (or a dedicated BFF -- Backend for Frontend):

```
[API Gateway] --flag check--> [Product Service] --> [Rec Service v1 or v2]
```

The gateway decides which downstream service to call based on the flag. This is preferred because:
- It is a single point of control
- The Product Service and Recommendation Service do not need to know about the flag
- Rollout percentage is applied once, at the edge

**Consistency**: Use a single flag evaluation service. All services that need flag state pull from the same source. The gateway evaluates the flag and can also pass the flag state as a header to downstream services if they need to adapt behavior.

**Deployed but unused service**: The new Recommendation Service v2 is deployed alongside v1. The flag is off, so the gateway routes all traffic to v1. v2 is idle but healthy and ready. This is safe -- idle services consume minimal resources (scale-to-zero if possible).

---

## Part C: Rollback Scenarios

### 7. Automated Rollback

```python
import time
import logging
import requests

logger = logging.getLogger("rollback")

class AutomatedRollback:
    def __init__(self, flag_service_url, flag_key, environment, token):
        self.flag_service_url = flag_service_url
        self.flag_key = flag_key
        self.environment = environment
        self.token = token

    def monitor_and_rollback(
        self,
        monitoring_duration_seconds: int = 600,
        check_interval_seconds: int = 30,
        max_error_rate: float = 0.01,
    ):
        """Monitor after flag enable and auto-rollback if needed."""
        start = time.time()
        errors = 0
        total = 0

        logger.info(f"Monitoring {self.flag_key} for {monitoring_duration_seconds}s")

        while time.time() - start < monitoring_duration_seconds:
            metrics = self._fetch_metrics()
            error_rate = metrics.get("error_rate", 0)
            request_count = metrics.get("request_count", 0)

            total += request_count
            errors += int(error_rate * request_count)

            if request_count > 100 and error_rate > max_error_rate:
                logger.warning(
                    f"Error rate {error_rate:.4f} exceeds threshold {max_error_rate}. Rolling back."
                )
                self._rollback(reason=f"Error rate {error_rate:.4f} > {max_error_rate}")
                return {"rolled_back": True, "reason": "error_rate_exceeded"}

            time.sleep(check_interval_seconds)

        logger.info("Monitoring period completed. No issues detected.")
        return {"rolled_back": False}

    def _rollback(self, reason: str):
        requests.post(
            f"{self.flag_service_url}/flags/{self.flag_key}/environments/{self.environment}/toggle",
            headers={
                "Authorization": f"Bearer {self.token}",
                "X-Actor": "automated-rollback",
                "Content-Type": "application/json",
            },
            json={"reason": reason},
        )
        self._send_alert(reason)

    def _send_alert(self, reason: str):
        logger.critical(
            f"ROLLBACK: Feature flag '{self.flag_key}' rolled back in "
            f"{self.environment}. Reason: {reason}"
        )
        # In production: send to PagerDuty, Slack, etc.

    def _fetch_metrics(self) -> dict:
        # In production: query Prometheus, Datadog, etc.
        resp = requests.get(f"{self.flag_service_url}/metrics/{self.flag_key}")
        return resp.json()
```

### 8. Rollback Without Redeployment

**Feature flag rollback (steps):**
1. Call the flag service API: `POST /flags/{key}/environments/production/toggle`
2. All subsequent requests evaluate the flag as disabled
3. Old code path executes for all users immediately
4. Time to rollback: **< 1 second** (API call) + cache TTL (typically 30-60 seconds)

**Traditional rollback (steps):**
1. Identify the previous container image version
2. Update the deployment manifest with the old image tag
3. Apply the deployment: `kubectl rollout undo`
4. Wait for new pods to start, old pods to terminate
5. Verify the rollback is healthy
6. Time to rollback: **2-10 minutes** depending on deployment size

**Time comparison:**
- Feature flag: ~30-60 seconds (mostly cache propagation)
- Traditional: ~5-10 minutes (pod scheduling, startup, health checks, termination)
- Difference: **5-20x faster** with feature flags

The speed advantage is most critical during incidents where every minute of degraded service has business impact.

---

## Part D: Advanced Integration

### 9. GitOps and Feature Flags

**Repository structure:**
```
infrastructure/
  feature-flags/
    base/
      flags.yaml          # Default flag configurations
    overlays/
      dev/
        flags.yaml        # Dev-specific overrides
      staging/
        flags.yaml
      production/
        flags.yaml
```

**Flag change propagation:**
1. Developer creates a PR changing `production/flags.yaml` (e.g., rollout_percentage: 0 -> 5)
2. PR review includes flag change approval
3. PR merges, ArgoCD/Flux detects the change
4. GitOps controller applies the change to the flag service
5. Flag service updates its in-memory state

**Environment-specific states**: Use Kustomize overlays or Helm values per environment. Each overlay only overrides what differs from the base.

**Trade-offs: Git commits vs. UI**
- Git commits: Audit trail, review process, rollback via git revert. Slower for emergency changes.
- UI: Instant changes, better for operational toggles. Requires separate audit mechanism.

**Hybrid approach**: Git for planned rollouts (percentage increases), UI for emergency kill switches.

### 10. Observability

**Metrics to emit:**

```
# Counter: total evaluations
flag_evaluation_total{flag="new-search", result="true", env="production"} 15234
flag_evaluation_total{flag="new-search", result="false", env="production"} 84766

# Histogram: evaluation latency
flag_evaluation_duration_microseconds{flag="new-search", env="production"} 
  p50=12, p99=45, p999=120

# Counter: flag state changes
flag_change_total{flag="new-search", actor="ci-pipeline", env="production"} 1

# Gauge: current rollout percentage
flag_rollout_percentage{flag="new-search", env="production"} 25
```

**Log format:**
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "info",
  "event": "flag_evaluated",
  "flag_key": "new-search",
  "result": true,
  "user_id": "u-12345",
  "environment": "production",
  "evaluation_duration_us": 15,
  "rollout_percentage": 25,
  "matched_rule": "percentage_rollout",
  "trace_id": "abc123def456",
  "span_id": "span789"
}
```

**Correlating flag changes with metrics:**

Create a dashboard with three panels stacked vertically:
1. **Flag state timeline**: When was the flag changed? (annotation overlay)
2. **Deployment timeline**: When was new code deployed? (annotation overlay)
3. **Application metrics**: Error rate, latency, conversion rate

When you see a metric change, check if it correlates with a flag change annotation. Tools like Grafana support annotation overlays from external sources (your flag service's audit log).

**Tracing integration:**

```python
from opentelemetry import trace

tracer = trace.get_tracer("feature-flags")

def evaluate_with_tracing(flag_service, key, env, context):
    with tracer.start_as_current_span("flag_evaluation") as span:
        span.set_attribute("flag.key", key)
        span.set_attribute("flag.environment", env)
        
        result = flag_service.evaluate(key, env, context)
        
        span.set_attribute("flag.result", result)
        return result
```

This ensures every flag evaluation appears in your distributed trace, making it possible to see which flags were evaluated for a given request and how long each evaluation took.
