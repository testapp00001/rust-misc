# Solution 04: Feature Flag Management System

## Part A: Data Model

```python
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
from typing import Any, Optional
import uuid

class FlagType(Enum):
    BOOLEAN = "boolean"
    MULTIVARIATE = "multivariate"
    PERCENTAGE_ROLLOUT = "percentage_rollout"

class FlagStatus(Enum):
    ACTIVE = "active"
    INACTIVE = "inactive"
    ARCHIVED = "archived"

class Operator(Enum):
    EQUALS = "equals"
    NOT_EQUALS = "not_equals"
    IN = "in"
    NOT_IN = "not_in"
    CONTAINS = "contains"
    GT = "gt"
    LT = "lt"

@dataclass
class TargetingRule:
    attribute: str
    operator: Operator
    values: list[str]
    result_value: Any = True

@dataclass
class EnvironmentConfig:
    enabled: bool = False
    default_value: Any = False
    rollout_percentage: int = 0
    targeting_rules: list[TargetingRule] = field(default_factory=list)
    overrides: dict[str, Any] = field(default_factory=dict)

@dataclass
class FeatureFlag:
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    key: str = ""
    name: str = ""
    description: str = ""
    flag_type: FlagType = FlagType.BOOLEAN
    created_at: datetime = field(default_factory=datetime.utcnow)
    updated_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""
    status: FlagStatus = FlagStatus.ACTIVE
    tags: list[str] = field(default_factory=list)
    environments: dict[str, EnvironmentConfig] = field(default_factory=dict)

@dataclass
class AuditEntry:
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    timestamp: datetime = field(default_factory=datetime.utcnow)
    actor: str = ""
    action: str = ""         # created, updated, toggled, archived, deleted
    flag_key: str = ""
    environment: str = ""
    previous_value: Any = None
    new_value: Any = None
    reason: str = ""
```

---

## Part B: REST API

### Core Service

```python
import re
from typing import Optional

class FeatureFlagService:
    def __init__(self):
        self._flags: dict[str, FeatureFlag] = {}
        self._audit: list[AuditEntry] = []

    # --- Validation ---

    @staticmethod
    def _validate_key(key: str):
        if not re.match(r'^[a-z0-9][a-z0-9\-]{1,62}[a-z0-9]$', key):
            raise ValueError(
                "Key must be 3-64 lowercase alphanumeric chars with hyphens"
            )

    # --- CRUD ---

    def create_flag(self, data: dict, actor: str) -> FeatureFlag:
        self._validate_key(data["key"])
        if data["key"] in self._flags:
            existing = self._flags[data["key"]]
            if existing.status != FlagStatus.ARCHIVED:
                raise ValueError(f"Flag '{data['key']}' already exists")

        flag = FeatureFlag(
            key=data["key"],
            name=data.get("name", data["key"]),
            description=data.get("description", ""),
            flag_type=FlagType(data.get("flag_type", "boolean")),
            created_by=actor,
            tags=data.get("tags", []),
        )

        # Initialize environments
        for env_name, env_config in data.get("environments", {}).items():
            flag.environments[env_name] = EnvironmentConfig(**env_config)

        self._flags[flag.key] = flag
        self._record_audit(actor, "created", flag.key, new_value=data)
        return flag

    def get_flag(self, key: str) -> Optional[FeatureFlag]:
        return self._flags.get(key)

    def list_flags(
        self,
        status: Optional[str] = None,
        tag: Optional[str] = None,
        environment: Optional[str] = None,
        search: Optional[str] = None,
        page: int = 1,
        per_page: int = 20,
    ) -> dict:
        results = list(self._flags.values())

        if status:
            results = [f for f in results if f.status.value == status]
        if tag:
            results = [f for f in results if tag in f.tags]
        if environment:
            results = [f for f in results if environment in f.environments]
        if search:
            q = search.lower()
            results = [
                f for f in results
                if q in f.name.lower() or q in f.description.lower()
            ]

        total = len(results)
        start = (page - 1) * per_page
        end = start + per_page

        return {
            "flags": results[start:end],
            "meta": {"page": page, "per_page": per_page, "total": total},
        }

    def update_flag(self, key: str, data: dict, actor: str) -> FeatureFlag:
        flag = self._flags.get(key)
        if not flag:
            raise KeyError(f"Flag '{key}' not found")
        if flag.status == FlagStatus.ARCHIVED:
            raise ValueError("Cannot update an archived flag")

        previous = {
            "name": flag.name,
            "description": flag.description,
            "tags": flag.tags,
        }

        if "name" in data:
            flag.name = data["name"]
        if "description" in data:
            flag.description = data["description"]
        if "tags" in data:
            flag.tags = data["tags"]

        flag.updated_at = datetime.utcnow()
        self._record_audit(
            actor, "updated", key,
            previous_value=previous,
            new_value=data,
        )
        return flag

    def delete_flag(self, key: str, actor: str, force: bool = False):
        flag = self._flags.get(key)
        if not flag:
            raise KeyError(f"Flag '{key}' not found")

        if flag.status == FlagStatus.ACTIVE and not force:
            raise ValueError(
                "Cannot delete an active flag. Deactivate first or use force=true"
            )

        if flag.status != FlagStatus.ARCHIVED:
            raise ValueError("Can only delete archived flags")

        self._record_audit(actor, "deleted", key, previous_value={"status": flag.status.value})
        del self._flags[key]

    # --- Toggle ---

    def toggle_flag(self, key: str, env: str, actor: str) -> FeatureFlag:
        flag = self._flags.get(key)
        if not flag:
            raise KeyError(f"Flag '{key}' not found")

        if env not in flag.environments:
            flag.environments[env] = EnvironmentConfig()

        old_enabled = flag.environments[env].enabled
        flag.environments[env].enabled = not old_enabled
        flag.updated_at = datetime.utcnow()

        self._record_audit(
            actor, "toggled", key,
            environment=env,
            previous_value=old_enabled,
            new_value=not old_enabled,
        )
        return flag

    # --- Evaluation ---

    def evaluate(self, key: str, env: str, context: dict) -> Any:
        flag = self._flags.get(key)
        if not flag or flag.status == FlagStatus.ARCHIVED:
            return None

        config = flag.environments.get(env) or flag.environments.get("default")
        if not config or not config.enabled:
            return config.default_value if config else None

        user_id = context.get("user_id", "")

        # Check overrides
        if user_id in config.overrides:
            return config.overrides[user_id]

        # Check targeting rules
        for rule in config.targeting_rules:
            if self._evaluate_rule(rule, context):
                return rule.result_value

        # Percentage rollout
        import hashlib
        bucket = int(hashlib.md5(
            f"{user_id}:{key}".encode()
        ).hexdigest()[:8], 16) % 100
        if bucket < config.rollout_percentage:
            return True

        return config.default_value

    @staticmethod
    def _evaluate_rule(rule: TargetingRule, context: dict) -> bool:
        attr_value = context.get(rule.attribute, "")
        if rule.operator == Operator.EQUALS:
            return str(attr_value) in rule.values
        if rule.operator == Operator.NOT_EQUALS:
            return str(attr_value) not in rule.values
        if rule.operator == Operator.IN:
            return str(attr_value) in rule.values
        if rule.operator == Operator.NOT_IN:
            return str(attr_value) not in rule.values
        if rule.operator == Operator.GT:
            return float(attr_value) > float(rule.values[0])
        if rule.operator == Operator.LT:
            return float(attr_value) < float(rule.values[0])
        return False

    # --- Audit ---

    def _record_audit(self, actor, action, flag_key, **kwargs):
        entry = AuditEntry(
            actor=actor,
            action=action,
            flag_key=flag_key,
            environment=kwargs.get("environment", ""),
            previous_value=kwargs.get("previous_value"),
            new_value=kwargs.get("new_value"),
            reason=kwargs.get("reason", ""),
        )
        self._audit.append(entry)

    def get_audit_log(
        self,
        flag_key: Optional[str] = None,
        actor: Optional[str] = None,
        since: Optional[datetime] = None,
    ) -> list[AuditEntry]:
        results = self._audit
        if flag_key:
            results = [e for e in results if e.flag_key == flag_key]
        if actor:
            results = [e for e in results if e.actor == actor]
        if since:
            results = [e for e in results if e.timestamp >= since]
        return results
```

### Sample API Endpoints (Flask-style)

```python
from flask import Flask, request, jsonify
app = Flask(__name__)
service = FeatureFlagService()

@app.route('/flags', methods=['POST'])
def create_flag():
    data = request.json
    actor = request.headers.get("X-Actor", "unknown")
    try:
        flag = service.create_flag(data, actor)
        return jsonify({"success": True, "data": flag.__dict__}), 201
    except ValueError as e:
        return jsonify({"success": False, "error": {"code": "VALIDATION_ERROR", "message": str(e)}}), 400

@app.route('/flags', methods=['GET'])
def list_flags():
    result = service.list_flags(
        status=request.args.get("status"),
        tag=request.args.get("tag"),
        environment=request.args.get("environment"),
        search=request.args.get("search"),
        page=int(request.args.get("page", 1)),
        per_page=int(request.args.get("per_page", 20)),
    )
    return jsonify({"success": True, "data": result["flags"], "meta": result["meta"]})

@app.route('/flags/<key>', methods=['GET'])
def get_flag(key):
    flag = service.get_flag(key)
    if not flag:
        return jsonify({"success": False, "error": {"code": "NOT_FOUND"}}), 404
    return jsonify({"success": True, "data": flag.__dict__})

@app.route('/flags/<key>/environments/<env>/toggle', methods=['POST'])
def toggle(key, env):
    actor = request.headers.get("X-Actor", "unknown")
    try:
        flag = service.toggle_flag(key, env, actor)
        return jsonify({"success": True, "data": flag.__dict__})
    except KeyError:
        return jsonify({"success": False, "error": {"code": "NOT_FOUND"}}), 404

@app.route('/flags/<key>/environments/<env>/evaluate', methods=['GET'])
def evaluate(key, env):
    context = request.args.to_dict()
    result = service.evaluate(key, env, context)
    return jsonify({"success": True, "data": {"result": result}})
```

---

## Part C: Lifecycle Management

### State Machine

```python
class FlagLifecycle:
    VALID_TRANSITIONS = {
        FlagStatus.ACTIVE: {FlagStatus.INACTIVE, FlagStatus.ARCHIVED},
        FlagStatus.INACTIVE: {FlagStatus.ACTIVE, FlagStatus.ARCHIVED},
        FlagStatus.ARCHIVED: {},  # terminal state (except deletion)
    }

    @staticmethod
    def can_transition(current: FlagStatus, target: FlagStatus) -> bool:
        return target in FlagLifecycle.VALID_TRANSITIONS.get(current, set())

    @staticmethod
    def archive(service: FeatureFlagService, key: str, actor: str, reason: str):
        flag = service._flags.get(key)
        if not flag:
            raise KeyError(f"Flag '{key}' not found")
        if flag.status != FlagStatus.INACTIVE:
            raise ValueError("Can only archive inactive flags")
        flag.status = FlagStatus.ARCHIVED
        flag.updated_at = datetime.utcnow()
        service._record_audit(
            actor, "archived", key,
            previous_value="inactive",
            new_value="archived",
            reason=reason,
        )
```

### Stale Flag Detection

```python
from datetime import datetime, timedelta

def detect_stale_flags(service: FeatureFlagService) -> list[dict]:
    now = datetime.utcnow()
    stale = []

    for flag in service._flags.values():
        if flag.status == FlagStatus.ARCHIVED:
            continue

        # Check: inactive for 30+ days
        if flag.status == FlagStatus.INACTIVE:
            days_inactive = (now - flag.updated_at).days
            if days_inactive > 30:
                stale.append({
                    "flag": flag.key,
                    "reason": f"Inactive for {days_inactive} days",
                    "recommendation": "Archive or delete",
                    "last_updated": flag.updated_at.isoformat(),
                })

        # Check: at 100% rollout for 90+ days
        if flag.status == FlagStatus.ACTIVE:
            for env_config in flag.environments.values():
                if env_config.rollout_percentage == 100:
                    days_full = (now - flag.updated_at).days
                    if days_full > 90:
                        stale.append({
                            "flag": flag.key,
                            "reason": f"At 100% rollout for {days_full} days",
                            "recommendation": "Remove flag, make permanent code",
                            "last_updated": flag.updated_at.isoformat(),
                        })
                        break

    return stale
```

---

## Part D: Environment Management

### Environment Fallback

```python
def evaluate_with_fallback(service, key, env, context):
    flag = service.get_flag(key)
    if not flag:
        return None

    config = flag.environments.get(env)
    if not config:
        config = flag.environments.get("default")
    if not config:
        return None

    if not config.enabled:
        return config.default_value

    # ... rest of evaluation logic ...
```

### Promotion

```python
def promote(service, key, source_env, target_env, actor, confirm=False):
    flag = service.get_flag(key)
    if not flag:
        raise KeyError(f"Flag '{key}' not found")

    source_config = flag.environments.get(source_env)
    if not source_config:
        raise ValueError(f"No configuration for environment '{source_env}'")

    target_config = flag.environments.get(target_env)

    # Generate diff
    diff = {
        "source_env": source_env,
        "target_env": target_env,
        "changes": {},
    }
    if target_config:
        if source_config.enabled != target_config.enabled:
            diff["changes"]["enabled"] = {
                "from": target_config.enabled,
                "to": source_config.enabled,
            }
        if source_config.rollout_percentage != target_config.rollout_percentage:
            diff["changes"]["rollout_percentage"] = {
                "from": target_config.rollout_percentage,
                "to": source_config.rollout_percentage,
            }
    else:
        diff["changes"]["new_environment"] = {
            "enabled": source_config.enabled,
            "rollout_percentage": source_config.rollout_percentage,
        }

    if not confirm:
        return {"diff": diff, "confirmed": False}

    # Apply promotion
    import copy
    flag.environments[target_env] = copy.deepcopy(source_config)
    flag.updated_at = datetime.utcnow()

    service._record_audit(
        actor, "promoted", key,
        environment=target_env,
        previous_value=target_config.__dict__ if target_config else None,
        new_value=source_config.__dict__,
    )

    return {"diff": diff, "confirmed": True}
```

---

## Part E: Safety Features

### Kill Switch

```python
def emergency_disable_all(service, env, actor):
    affected = []
    for flag in service._flags.values():
        if flag.status != FlagStatus.ACTIVE:
            continue
        config = flag.environments.get(env)
        if config and config.enabled:
            affected.append({
                "key": flag.key,
                "was_enabled": True,
                "was_rollout": config.rollout_percentage,
            })
            config.enabled = False
            config.rollout_percentage = 0

    service._record_audit(
        actor, "emergency_disable_all", "*",
        environment=env,
        new_value={"affected_count": len(affected)},
    )
    return affected

def emergency_restore(service, env, actor, affected_flags):
    for entry in affected_flags:
        flag = service.get_flag(entry["key"])
        if flag:
            config = flag.environments.get(env)
            if config:
                config.enabled = entry["was_enabled"]
                config.rollout_percentage = entry["was_rollout"]

    service._record_audit(
        actor, "emergency_restore", "*",
        environment=env,
        new_value={"restored_count": len(affected_flags)},
    )
```
