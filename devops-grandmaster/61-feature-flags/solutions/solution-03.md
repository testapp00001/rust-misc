# Solution 03: Gradual Rollout with Feature Flags

## Part A: Rollout Configuration

### Data Model

```python
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime, timedelta

class RolloutStatus(Enum):
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    PAUSED = "paused"
    COMPLETED = "completed"
    ROLLED_BACK = "rolled_back"

@dataclass
class HealthThresholds:
    max_error_rate: float        # e.g., 0.01 = 1%
    max_latency_p99_ms: float    # e.g., 500.0
    min_conversion_rate: float   # e.g., 0.03 = 3%

@dataclass
class RolloutStage:
    name: str
    target_percentage: int
    duration: timedelta
    thresholds: HealthThresholds

@dataclass
class RolloutPlan:
    flag_key: str
    stages: list[RolloutStage]
    current_stage_index: int = 0
    status: RolloutStatus = RolloutStatus.PENDING
    started_at: datetime = None
    stage_started_at: datetime = None
```

### Sample Rollout Plan

```python
plan = RolloutPlan(
    flag_key="search-v2",
    stages=[
        RolloutStage(
            name="Canary",
            target_percentage=1,
            duration=timedelta(hours=2),
            thresholds=HealthThresholds(
                max_error_rate=0.01,
                max_latency_p99_ms=500,
                min_conversion_rate=0.028
            )
        ),
        RolloutStage(
            name="Early Adopters",
            target_percentage=5,
            duration=timedelta(hours=6),
            thresholds=HealthThresholds(
                max_error_rate=0.008,
                max_latency_p99_ms=400,
                min_conversion_rate=0.029
            )
        ),
        RolloutStage(
            name="Broad Rollout",
            target_percentage=25,
            duration=timedelta(hours=24),
            thresholds=HealthThresholds(
                max_error_rate=0.005,
                max_latency_p99_ms=350,
                min_conversion_rate=0.03
            )
        ),
        RolloutStage(
            name="Majority",
            target_percentage=50,
            duration=timedelta(days=3),
            thresholds=HealthThresholds(
                max_error_rate=0.005,
                max_latency_p99_ms=300,
                min_conversion_rate=0.03
            )
        ),
        RolloutStage(
            name="Full Release",
            target_percentage=100,
            duration=timedelta(days=7),
            thresholds=HealthThresholds(
                max_error_rate=0.005,
                max_latency_p99_ms=300,
                min_conversion_rate=0.03
            )
        ),
    ]
)
```

---

## Part B: Rollout Controller

### Health Checker

```python
from dataclasses import dataclass

@dataclass
class HealthVerdict:
    passed: bool
    error_rate: float
    latency_p99_ms: float
    conversion_rate: float
    failures: list[str]

class HealthChecker:
    def check(
        self,
        metrics: dict,
        thresholds: HealthThresholds
    ) -> HealthVerdict:
        failures = []

        error_rate = metrics.get("error_rate", 0)
        latency_p99 = metrics.get("latency_p99_ms", 0)
        conversion_rate = metrics.get("conversion_rate", 1.0)

        if error_rate > thresholds.max_error_rate:
            failures.append(
                f"Error rate {error_rate:.4f} exceeds threshold {thresholds.max_error_rate:.4f}"
            )

        if latency_p99 > thresholds.max_latency_p99_ms:
            failures.append(
                f"Latency p99 {latency_p99:.0f}ms exceeds threshold {thresholds.max_latency_p99_ms:.0f}ms"
            )

        if conversion_rate < thresholds.min_conversion_rate:
            failures.append(
                f"Conversion rate {conversion_rate:.4f} below threshold {thresholds.min_conversion_rate:.4f}"
            )

        return HealthVerdict(
            passed=len(failures) == 0,
            error_rate=error_rate,
            latency_p99_ms=latency_p99,
            conversion_rate=conversion_rate,
            failures=failures,
        )
```

### Rollout Controller

```python
import logging
from datetime import datetime

logger = logging.getLogger("rollout")

class RolloutController:
    def __init__(
        self,
        plan: RolloutPlan,
        flag_store,       # FeatureFlagStore from Exercise 02
        health_checker: HealthChecker,
        metrics_collector, # provides get_metrics() -> dict
    ):
        self.plan = plan
        self.flag_store = flag_store
        self.health_checker = health_checker
        self.metrics = metrics_collector
        self._log = []

    def _log_event(self, event: str, details: dict = None):
        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "event": event,
            "stage": self.plan.stages[self.plan.current_stage_index].name
                if self.plan.current_stage_index < len(self.plan.stages) else "N/A",
            "details": details or {},
        }
        self._log.append(entry)
        logger.info(f"[{entry['timestamp']}] {event}: {details}")

    def start(self):
        self.plan.status = RolloutStatus.IN_PROGRESS
        self.plan.started_at = datetime.utcnow()
        self._log_event("rollout_started", {"flag": self.plan.flag_key})
        self._begin_stage()

    def _begin_stage(self):
        stage = self.plan.stages[self.plan.current_stage_index]
        self.plan.stage_started_at = datetime.utcnow()
        self.flag_store.update(
            self.plan.flag_key,
            rollout_percentage=stage.target_percentage
        )
        self._log_event("stage_started", {
            "target_percentage": stage.target_percentage,
            "duration": str(stage.duration),
        })

    def check_and_advance(self):
        if self.plan.status != RolloutStatus.IN_PROGRESS:
            return

        stage = self.plan.stages[self.plan.current_stage_index]
        elapsed = datetime.utcnow() - self.plan.stage_started_at

        if elapsed < stage.duration:
            return  # not yet time to advance

        # Collect metrics and check health
        current_metrics = self.metrics.get_metrics()
        verdict = self.health_checker.check(current_metrics, stage.thresholds)

        if not verdict.passed:
            self._rollback(verdict)
            return

        self._log_event("stage_health_passed", {
            "error_rate": verdict.error_rate,
            "latency_p99_ms": verdict.latency_p99_ms,
            "conversion_rate": verdict.conversion_rate,
        })

        # Advance to next stage
        if self.plan.current_stage_index + 1 >= len(self.plan.stages):
            self._complete()
        else:
            self.plan.current_stage_index += 1
            self._begin_stage()

    def pause(self, reason: str = "manual"):
        self.plan.status = RolloutStatus.PAUSED
        self._log_event("rollout_paused", {"reason": reason})

    def resume(self):
        if self.plan.status != RolloutStatus.PAUSED:
            raise ValueError("Can only resume a paused rollout")
        self.plan.status = RolloutStatus.IN_PROGRESS
        self.plan.stage_started_at = datetime.utcnow()
        self._log_event("rollout_resumed")

    def _rollback(self, verdict: HealthVerdict):
        self.plan.status = RolloutStatus.ROLLED_BACK
        self.flag_store.update(self.plan.flag_key, rollout_percentage=0)
        self._log_event("rollout_rolled_back", {
            "reason": "Health check failed",
            "failures": verdict.failures,
            "metrics": {
                "error_rate": verdict.error_rate,
                "latency_p99_ms": verdict.latency_p99_ms,
                "conversion_rate": verdict.conversion_rate,
            },
        })

    def _complete(self):
        self.plan.status = RolloutStatus.COMPLETED
        self._log_event("rollout_completed")
```

---

## Part C: Simulation

### Simulator

```python
import random
import hashlib

class SimulatedMetricsCollector:
    """Simulates metrics based on which version users are hitting."""
    def __init__(self, flag_store, flag_key, old_metrics, new_metrics):
        self.flag_store = flag_store
        self.flag_key = flag_key
        self.old = old_metrics
        self.new = new_metrics
        self._samples = []

    def record_request(self, user_id: str):
        flag = self.flag_store.get(self.flag_key)
        bucket = int(hashlib.md5(
            f"{user_id}:{self.flag_key}".encode()
        ).hexdigest()[:8], 16) % 100

        uses_new = bucket < flag.rollout_percentage
        version = self.new if uses_new else self.old
        is_error = random.random() < version["error_rate"]
        latency = random.gauss(version["latency_p99_ms"], 20)
        converted = random.random() < version["conversion_rate"]
        self._samples.append({
            "error": is_error,
            "latency": max(0, latency),
            "converted": converted,
        })

    def get_metrics(self) -> dict:
        if not self._samples:
            return {"error_rate": 0, "latency_p99_ms": 0, "conversion_rate": 0}
        errors = sum(1 for s in self._samples if s["error"])
        latencies = sorted(s["latency"] for s in self._samples)
        conversions = sum(1 for s in self._samples if s["converted"])
        p99_idx = int(len(latencies) * 0.99)
        return {
            "error_rate": errors / len(self._samples),
            "latency_p99_ms": latencies[min(p99_idx, len(latencies) - 1)],
            "conversion_rate": conversions / len(self._samples),
        }

    def clear(self):
        self._samples.clear()
```

### Successful Simulation

```python
import time

def run_successful_simulation():
    store = FeatureFlagStore()
    store._flags = {
        "search-v2": FeatureFlag(
            key="search-v2", name="Search V2", description="",
            enabled=True, default_value=False, rollout_percentage=0
        )
    }

    plan = RolloutPlan(
        flag_key="search-v2",
        stages=[
            RolloutStage("Canary", 1, timedelta(seconds=2),
                         HealthThresholds(0.01, 500, 0.028)),
            RolloutStage("Early", 5, timedelta(seconds=2),
                         HealthThresholds(0.008, 400, 0.029)),
            RolloutStage("Broad", 25, timedelta(seconds=2),
                         HealthThresholds(0.005, 350, 0.03)),
            RolloutStage("Full", 100, timedelta(seconds=2),
                         HealthThresholds(0.005, 300, 0.03)),
        ]
    )

    metrics = SimulatedMetricsCollector(
        store, "search-v2",
        old_metrics={"error_rate": 0.005, "latency_p99_ms": 200, "conversion_rate": 0.032},
        new_metrics={"error_rate": 0.006, "latency_p99_ms": 180, "conversion_rate": 0.035},
    )

    users = [f"user-{i:04d}" for i in range(1000)]
    checker = HealthChecker()
    controller = RolloutController(plan, store, checker, metrics)

    controller.start()

    while plan.status == RolloutStatus.IN_PROGRESS:
        for user in users:
            metrics.record_request(user)
        time.sleep(0.1)
        controller.check_and_advance()

    print(f"Final status: {plan.status.value}")
    for entry in controller._log:
        print(f"  {entry['timestamp']} | {entry['event']} | {entry['details']}")
```

### Failure Simulation

```python
def run_failure_simulation():
    store = FeatureFlagStore()
    store._flags = {
        "search-v2": FeatureFlag(
            key="search-v2", name="Search V2", description="",
            enabled=True, default_value=False, rollout_percentage=0
        )
    }

    plan = RolloutPlan(
        flag_key="search-v2",
        stages=[
            RolloutStage("Canary", 1, timedelta(seconds=2),
                         HealthThresholds(0.01, 500, 0.028)),
            RolloutStage("Failing", 25, timedelta(seconds=2),
                         HealthThresholds(0.01, 500, 0.028)),
        ]
    )

    # New version has 5% error rate -- will fail health check
    metrics = SimulatedMetricsCollector(
        store, "search-v2",
        old_metrics={"error_rate": 0.005, "latency_p99_ms": 200, "conversion_rate": 0.032},
        new_metrics={"error_rate": 0.05, "latency_p99_ms": 180, "conversion_rate": 0.035},
    )

    users = [f"user-{i:04d}" for i in range(1000)]
    checker = HealthChecker()
    controller = RolloutController(plan, store, checker, metrics)

    controller.start()

    # First stage passes (1% -- few users hit new version)
    for user in users:
        metrics.record_request(user)
    time.sleep(2.5)
    controller.check_and_advance()

    # Clear metrics for second stage
    metrics.clear()

    # Second stage: 25% of users now hit the broken new version
    for user in users:
        metrics.record_request(user)
    time.sleep(2.5)
    controller.check_and_advance()

    assert plan.status == RolloutStatus.ROLLED_BACK
    assert store.get("search-v2").rollout_percentage == 0
    print("Rollback detected and confirmed.")
```

### Pause and Resume

```python
def demonstrate_pause_resume():
    # ... setup same as above ...
    controller.start()

    # Simulate pausing mid-stage
    controller.pause(reason="Investigating user complaints about latency")
    assert plan.status == RolloutStatus.PAUSED

    # Simulate investigation
    print("Investigating... all clear.")

    # Resume
    controller.resume()
    assert plan.status == RolloutStatus.IN_PROGRESS
```

---

## Part D: Analysis

### 9. Stage Duration Trade-offs

**Short durations (10 minutes):**
- Pros: Faster rollout, quicker feedback loop, less time waiting
- Cons: May not capture enough traffic for statistical significance, might miss delayed effects, riskier if errors take time to manifest
- Use when: The change is low-risk, metrics are real-time, and the user base is large enough for quick signal

**Long durations (24 hours):**
- Pros: Captures daily traffic patterns, allows time for issues to surface, more statistically robust
- Cons: Slower rollout, delays feature availability, ties up engineering attention
- Use when: The change is high-risk, affects critical flows, or the user base is small

### 10. Metric Sensitivity

A 1.05% error rate vs a 1.0% threshold is within noise. Proceeding is likely fine, but:
- **Too tight**: Thresholds that trigger on noise cause false rollbacks, wasting time and eroding trust in the system
- **Too loose**: Thresholds that miss real degradation allow bad code to reach all users
- **Recommendation**: Use statistical significance testing. Compare the new version's metric against the old with a confidence interval. A 5% difference in a metric with 1% noise is not statistically significant. Only roll back if the difference exceeds the noise floor with 95% confidence.
