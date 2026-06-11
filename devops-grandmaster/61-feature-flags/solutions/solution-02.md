# Solution 02: Implement Feature Flags in an Application

## Part A: Configuration Store

### Data Model

```python
from dataclasses import dataclass, field
from typing import Any, Optional

@dataclass
class FeatureFlag:
    key: str
    name: str
    description: str
    enabled: bool
    default_value: Any = False
    rollout_percentage: int = 100
    enabled_user_ids: list[str] = field(default_factory=list)
    disabled_user_ids: list[str] = field(default_factory=list)
```

### Store Implementation

```python
import json
import threading

class FeatureFlagStore:
    def __init__(self, config_path: str = None):
        self._flags: dict[str, FeatureFlag] = {}
        self._lock = threading.RLock()
        if config_path:
            self.load_from_file(config_path)

    def load_from_file(self, path: str):
        with open(path) as f:
            data = json.load(f)
        with self._lock:
            for flag_data in data["flags"]:
                flag = FeatureFlag(**flag_data)
                self._flags[flag.key] = flag

    def get(self, key: str) -> Optional[FeatureFlag]:
        with self._lock:
            return self._flags.get(key)

    def list_all(self) -> list[FeatureFlag]:
        with self._lock:
            return list(self._flags.values())

    def update(self, key: str, **kwargs) -> bool:
        with self._lock:
            flag = self._flags.get(key)
            if not flag:
                return False
            for attr, value in kwargs.items():
                if hasattr(flag, attr):
                    setattr(flag, attr, value)
            return True
```

### Sample Configuration

```json
{
  "flags": [
    {
      "key": "new-checkout-flow",
      "name": "New Checkout Flow",
      "description": "Redesigned checkout with fewer steps",
      "enabled": true,
      "default_value": false,
      "rollout_percentage": 25,
      "enabled_user_ids": ["beta-user-001"],
      "disabled_user_ids": ["excluded-user-001"]
    },
    {
      "key": "dark-mode",
      "name": "Dark Mode",
      "description": "Dark color theme for the application",
      "enabled": true,
      "default_value": true,
      "rollout_percentage": 100,
      "enabled_user_ids": [],
      "disabled_user_ids": []
    },
    {
      "key": "new-search-algorithm",
      "name": "New Search Algorithm",
      "description": "ML-powered search ranking",
      "enabled": false,
      "default_value": false,
      "rollout_percentage": 0,
      "enabled_user_ids": ["internal-tester-001"],
      "disabled_user_ids": []
    }
  ]
}
```

---

## Part B: Evaluation Engine

### Basic Evaluation

```python
def evaluate_basic(store: FeatureFlagStore, key: str) -> Any:
    flag = store.get(key)
    if flag is None:
        return False  # default for unknown flags
    if not flag.enabled:
        return flag.default_value
    return True
```

### Percentage Rollout (Consistent Hashing)

```python
import hashlib

def compute_rollout_bucket(user_id: str, flag_key: str) -> int:
    """Returns a consistent integer 0-99 for the user+flag combination."""
    raw = f"{user_id}:{flag_key}"
    hash_bytes = hashlib.md5(raw.encode()).hexdigest()
    return int(hash_bytes[:8], 16) % 100

def evaluate_with_rollout(store: FeatureFlagStore, key: str, user_id: str) -> Any:
    flag = store.get(key)
    if flag is None:
        return False
    if not flag.enabled:
        return flag.default_value
    bucket = compute_rollout_bucket(user_id, key)
    if bucket < flag.rollout_percentage:
        return True
    return flag.default_value
```

### Full Evaluation with User Targeting

```python
def evaluate(store: FeatureFlagStore, key: str, context: dict) -> Any:
    flag = store.get(key)
    if flag is None:
        return False

    if not flag.enabled:
        return flag.default_value

    user_id = context.get("user_id", "")

    # Priority 1: Disabled users always get false
    if user_id in flag.disabled_user_ids:
        return False

    # Priority 2: Enabled users always get true
    if user_id in flag.enabled_user_ids:
        return True

    # Priority 3: Percentage rollout
    bucket = compute_rollout_bucket(user_id, key)
    if bucket < flag.rollout_percentage:
        return True

    return flag.default_value
```

---

## Part C: Application Integration

### Context

```python
from dataclasses import dataclass, field
from datetime import datetime

@dataclass
class Context:
    user_id: str
    attributes: dict = field(default_factory=dict)
    timestamp: datetime = field(default_factory=datetime.utcnow)
    environment: str = "production"
```

### Application Wiring

```python
import logging

logger = logging.getLogger("feature_flags")

def process_checkout(store: FeatureFlagStore, context: Context):
    use_new_checkout = evaluate(store, "new-checkout-flow", {
        "user_id": context.user_id
    })

    logger.info(
        "Flag evaluated",
        extra={
            "flag": "new-checkout-flow",
            "result": use_new_checkout,
            "user_id": context.user_id,
        }
    )

    if use_new_checkout:
        return new_checkout_flow(context)
    else:
        return legacy_checkout_flow(context)
```

### Tests

```python
import pytest

@pytest.fixture
def store():
    s = FeatureFlagStore()
    s._flags = {
        "always-on": FeatureFlag(
            key="always-on", name="Always On", description="",
            enabled=True, default_value=False, rollout_percentage=100
        ),
        "always-off": FeatureFlag(
            key="always-off", name="Always Off", description="",
            enabled=False, default_value=False, rollout_percentage=100
        ),
        "half-rollout": FeatureFlag(
            key="half-rollout", name="Half Rollout", description="",
            enabled=True, default_value=False, rollout_percentage=50
        ),
        "targeted": FeatureFlag(
            key="targeted", name="Targeted", description="",
            enabled=True, default_value=False, rollout_percentage=0,
            enabled_user_ids=["vip-user"],
            disabled_user_ids=["blocked-user"]
        ),
    }
    return s

def test_flag_on_returns_true(store):
    ctx = {"user_id": "any-user"}
    assert evaluate(store, "always-on", ctx) is True

def test_flag_off_returns_false(store):
    ctx = {"user_id": "any-user"}
    assert evaluate(store, "always-off", ctx) is False

def test_percentage_consistency(store):
    ctx = {"user_id": "consistent-user"}
    result1 = evaluate(store, "half-rollout", ctx)
    result2 = evaluate(store, "half-rollout", ctx)
    assert result1 == result2  # same user, same result every time

def test_enabled_user_overrides_percentage(store):
    ctx = {"user_id": "vip-user"}
    # rollout_percentage is 0, but user is in enabled list
    assert evaluate(store, "targeted", ctx) is True

def test_disabled_user_overrides_percentage(store):
    ctx = {"user_id": "blocked-user"}
    # even though flag is on, disabled user gets false
    assert evaluate(store, "targeted", ctx) is False

def test_missing_flag_returns_default(store):
    ctx = {"user_id": "any-user"}
    assert evaluate(store, "nonexistent-flag", ctx) is False
```

---

## Part D: Variation Support (Extension)

```python
@dataclass
class FeatureFlag:
    key: str
    name: str
    description: str
    enabled: bool
    flag_type: str = "boolean"  # "boolean" or "multivariate"
    default_value: Any = False
    variations: dict = field(default_factory=dict)
    # For multivariate: {"control": "old", "variant-a": "new-blue", "variant-b": "new-green"}
    variation_weights: dict = field(default_factory=dict)
    # {"control": 34, "variant-a": 33, "variant-b": 33}
    rollout_percentage: int = 100
    enabled_user_ids: list[str] = field(default_factory=list)
    disabled_user_ids: list[str] = field(default_factory=list)

def evaluate_multivariate(store, key, context):
    flag = store.get(key)
    if flag is None or not flag.enabled:
        return flag.default_value if flag else None

    user_id = context.get("user_id", "")

    if user_id in flag.disabled_user_ids:
        return flag.default_value

    if flag.flag_type == "boolean":
        if user_id in flag.enabled_user_ids:
            return True
        bucket = compute_rollout_bucket(user_id, key)
        return bucket < flag.rollout_percentage

    # Multivariate: hash to pick a variation
    bucket = compute_rollout_bucket(user_id, key)
    if bucket >= flag.rollout_percentage:
        return flag.default_value

    cumulative = 0
    for variation, weight in flag.variation_weights.items():
        cumulative += weight
        if bucket < cumulative:
            return variation

    return flag.default_value
```
