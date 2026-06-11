# Solution 04: Env Var Validation and Defaults

## Part A: Config struct

### Python implementation

```python
from dataclasses import dataclass, field
from typing import List


@dataclass
class Config:
    port: int = 8080
    log_level: str = "info"
    node_env: str = "development"
    database_url: str = ""
    redis_url: str = "redis://localhost:6379"
    max_retries: int = 3
    request_timeout_ms: int = 5000
    allowed_origins: List[str] = field(default_factory=lambda: ["*"])
    enable_metrics: bool = False
    worker_concurrency: int = 4
```

### Rust implementation

```rust
pub struct Config {
    pub port: u16,
    pub log_level: String,
    pub node_env: String,
    pub database_url: String,
    pub redis_url: String,
    pub max_retries: u8,
    pub request_timeout_ms: u32,
    pub allowed_origins: Vec<String>,
    pub enable_metrics: bool,
    pub worker_concurrency: u8,
}
```

### Why this works

- Each field has the correct type, not just `String`. This forces type conversion at the boundary (env var parsing) rather than throughout the application.
- `ALLOWED_ORIGINS` is a `List[str]` / `Vec<String>`, not a single string. The application should not have to split a comma-separated string on every request.
- `ENABLE_METRICS` is a `bool`, not a string. The parsing happens once in the config module.
- Defaults are specified in the struct definition, making them visible and documented.

### Common mistakes

- Using a dictionary/map instead of a typed struct. This loses type safety and makes it easy to misspell a key at runtime.
- Defining defaults in multiple places (once in the struct, once in the parser, once in the env file). Pick one source of truth for defaults.

---

## Part B: Parser and validator

### Python implementation

```python
import os
import sys
from typing import List, Tuple


def parse_bool(value: str) -> bool:
    """Parse a string as a boolean. Only 'true' (case-insensitive) is True."""
    return value.strip().lower() == "true"


def parse_int(value: str, name: str) -> int:
    """Parse a string as an integer, raising a clear error on failure."""
    try:
        return int(value)
    except ValueError:
        raise ValueError(f"{name} must be an integer, got: '{value}'")


def parse_string_list(value: str) -> List[str]:
    """Parse a comma-separated string into a list."""
    return [item.strip() for item in value.split(",") if item.strip()]


def parse_config() -> Config:
    """Read, parse, and return configuration from environment variables."""
    errors: List[str] = []

    # PORT
    port_raw = os.environ.get("PORT", "8080")
    try:
        port = parse_int(port_raw, "PORT")
    except ValueError as e:
        errors.append(str(e))
        port = 8080  # placeholder, will fail validation

    # LOG_LEVEL
    log_level = os.environ.get("LOG_LEVEL", "info").strip().lower()

    # NODE_ENV
    node_env = os.environ.get("NODE_ENV", "development").strip().lower()

    # DATABASE_URL (required)
    database_url = os.environ.get("DATABASE_URL", "")
    if not database_url:
        errors.append("DATABASE_URL is required but not set")

    # REDIS_URL
    redis_url = os.environ.get("REDIS_URL", "redis://localhost:6379")

    # MAX_RETRIES
    max_retries_raw = os.environ.get("MAX_RETRIES", "3")
    try:
        max_retries = parse_int(max_retries_raw, "MAX_RETRIES")
    except ValueError as e:
        errors.append(str(e))
        max_retries = 3

    # REQUEST_TIMEOUT_MS
    timeout_raw = os.environ.get("REQUEST_TIMEOUT_MS", "5000")
    try:
        request_timeout_ms = parse_int(timeout_raw, "REQUEST_TIMEOUT_MS")
    except ValueError as e:
        errors.append(str(e))
        request_timeout_ms = 5000

    # ALLOWED_ORIGINS
    origins_raw = os.environ.get("ALLOWED_ORIGINS", "*")
    allowed_origins = parse_string_list(origins_raw)

    # ENABLE_METRICS
    metrics_raw = os.environ.get("ENABLE_METRICS", "false")
    enable_metrics = parse_bool(metrics_raw)

    # WORKER_CONCURRENCY
    concurrency_raw = os.environ.get("WORKER_CONCURRENCY", "4")
    try:
        worker_concurrency = parse_int(concurrency_raw, "WORKER_CONCURRENCY")
    except ValueError as e:
        errors.append(str(e))
        worker_concurrency = 4

    if errors:
        return None  # validation step will handle errors

    return Config(
        port=port,
        log_level=log_level,
        node_env=node_env,
        database_url=database_url,
        redis_url=redis_url,
        max_retries=max_retries,
        request_timeout_ms=request_timeout_ms,
        allowed_origins=allowed_origins,
        enable_metrics=enable_metrics,
        worker_concurrency=worker_concurrency,
    )
```

### Why this works

- Each variable is parsed with its own error handling. A bad `PORT` does not prevent `LOG_LEVEL` from being checked.
- Type conversion happens in the parser, not downstream. The rest of the application receives typed values.
- `parse_bool` only accepts `"true"` as true -- everything else is false. This avoids ambiguity ("yes", "1", "on" are all debatable).
- The parser returns `None` if any parse error occurred. The validator (Part C) handles the error reporting.

### Common mistakes

- Raising an exception on the first parse error. This forces the developer to fix one error, restart, discover the next error, restart again. Parse all variables, collect all errors, report them together.
- Using `value.lower() in ("true", "1", "yes", "on")` for booleans. This is ambiguous -- different team members will use different conventions. Pick one: `true`/`false`.
- Not handling empty strings. `os.environ.get("PORT", "8080")` returns `""` if `PORT` is set to an empty string, not `"8080"`. Consider whether empty strings should be treated as "not set."

---

## Part C: Fail-fast validation

### Python implementation

```python
import sys


def validate_config(config: Config) -> None:
    """Validate all configuration constraints. Exit on any error."""
    errors: List[str] = []

    # PORT: 1024-65535
    if not (1024 <= config.port <= 65535):
        errors.append(
            f"PORT must be between 1024 and 65535, got: {config.port}"
        )

    # LOG_LEVEL: allowed values
    allowed_log_levels = {"trace", "debug", "info", "warn", "error"}
    if config.log_level not in allowed_log_levels:
        errors.append(
            f"LOG_LEVEL must be one of {sorted(allowed_log_levels)}, "
            f"got: '{config.log_level}'"
        )

    # NODE_ENV: allowed values
    allowed_envs = {"development", "staging", "production"}
    if config.node_env not in allowed_envs:
        errors.append(
            f"NODE_ENV must be one of {sorted(allowed_envs)}, "
            f"got: '{config.node_env}'"
        )

    # DATABASE_URL: must start with postgres://
    if config.database_url and not config.database_url.startswith("postgres://"):
        errors.append(
            f"DATABASE_URL must start with 'postgres://', "
            f"got: '{config.database_url[:20]}...'"
        )
    elif not config.database_url:
        errors.append("DATABASE_URL is required but not set")

    # REDIS_URL: must start with redis://
    if config.redis_url and not config.redis_url.startswith("redis://"):
        errors.append(
            f"REDIS_URL must start with 'redis://', "
            f"got: '{config.redis_url[:20]}...'"
        )

    # MAX_RETRIES: 1-10
    if not (1 <= config.max_retries <= 10):
        errors.append(
            f"MAX_RETRIES must be between 1 and 10, got: {config.max_retries}"
        )

    # REQUEST_TIMEOUT_MS: 100-60000
    if not (100 <= config.request_timeout_ms <= 60000):
        errors.append(
            f"REQUEST_TIMEOUT_MS must be between 100 and 60000, "
            f"got: {config.request_timeout_ms}"
        )

    # WORKER_CONCURRENCY: 1-32
    if not (1 <= config.worker_concurrency <= 32):
        errors.append(
            f"WORKER_CONCURRENCY must be between 1 and 32, "
            f"got: {config.worker_concurrency}"
        )

    if errors:
        print("FATAL: Configuration validation failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        sys.exit(1)
```

### Why this works

- Every constraint is checked in a single pass. All errors are collected before reporting.
- Error messages include the variable name, the expected range or values, and the actual value. This makes debugging fast.
- The function prints to stderr and exits with code 1. This is the standard convention for fatal errors -- stderr is not captured by pipes, and exit code 1 signals failure to the orchestrator.
- The format uses `FATAL:` as a prefix so log aggregation tools can filter for startup failures.

### Common mistakes

- Validating only at the point of use (e.g., checking `PORT` when binding the socket). By then, the application has already started, connected to databases, and initialized state. Validate at startup.
- Using `sys.exit(0)` on error. Exit code 0 means success. Always use a non-zero exit code for failures.
- Printing errors without the variable name. "Invalid value" is useless. "PORT must be between 1024 and 65535, got: 99999" is actionable.

---

## Part D: Config summary logger

### Python implementation

```python
def log_config_summary(config: Config) -> None:
    """Print resolved configuration with sensitive values masked."""
    # Mask the database URL password
    masked_db_url = config.database_url
    if "@" in masked_db_url:
        scheme_end = masked_db_url.index("://") + 3
        at_sign = masked_db_url.index("@")
        masked_db_url = (
            masked_db_url[:scheme_end] + "***" + masked_db_url[at_sign:]
        )

    print(f"[CONFIG] PORT={config.port}")
    print(f"[CONFIG] LOG_LEVEL={config.log_level}")
    print(f"[CONFIG] NODE_ENV={config.node_env}")
    print(f"[CONFIG] DATABASE_URL={masked_db_url}")
    print(f"[CONFIG] REDIS_URL={config.redis_url}")
    print(f"[CONFIG] MAX_RETRIES={config.max_retries}")
    print(f"[CONFIG] REQUEST_TIMEOUT_MS={config.request_timeout_ms}")
    print(f"[CONFIG] ALLOWED_ORIGINS={config.allowed_origins}")
    print(f"[CONFIG] ENABLE_METRICS={config.enable_metrics}")
    print(f"[CONFIG] WORKER_CONCURRENCY={config.worker_concurrency}")
```

### Why this works

- The password in `DATABASE_URL` is masked by replacing everything between `://` and `@` with `***`.
- Non-sensitive values are printed as-is so operators can verify the configuration.
- The `[CONFIG]` prefix makes it easy to filter config logs from application logs.

### Common mistakes

- Printing the raw `DATABASE_URL` with the password. The config endpoint and startup logs are often captured by monitoring systems.
- Masking too aggressively (e.g., hiding the entire URL). Operators need to see the host and database name to verify the correct environment.

---

## Part E: Test script

```bash
#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0

run_test() {
    local description="$1"
    local expect_exit="$2"
    shift 2

    echo "TEST: $description"

    # Run the app with the given env vars and capture exit code
    set +e
    output=$(env "$@" python app.py 2>&1)
    actual_exit=$?
    set -e

    if [ "$actual_exit" -eq "$expect_exit" ]; then
        echo "  PASS (exit code $actual_exit)"
        PASS=$((PASS + 1))
    else
        echo "  FAIL (expected exit $expect_exit, got $actual_exit)"
        echo "  Output: $output"
        FAIL=$((FAIL + 1))
    fi
    echo ""
}

# Test 1: Valid configuration
run_test "Valid config should succeed" 0 \
    DATABASE_URL="postgres://user:pass@localhost:5432/myapp"

# Test 2: PORT out of range
run_test "PORT=99999 should fail" 1 \
    DATABASE_URL="postgres://user:pass@localhost:5432/myapp" \
    PORT=99999

# Test 3: Invalid LOG_LEVEL
run_test "LOG_LEVEL=verbose should fail" 1 \
    DATABASE_URL="postgres://user:pass@localhost:5432/myapp" \
    LOG_LEVEL=verbose

# Test 4: Missing DATABASE_URL
run_test "Missing DATABASE_URL should fail" 1 \
    DATABASE_URL=""

# Test 5: Wrong DATABASE_URL format
run_test "DATABASE_URL=mysql://... should fail" 1 \
    DATABASE_URL="mysql://user:pass@localhost:3306/myapp"

# Test 6: Multiple invalid values
run_test "Multiple errors should all be reported" 1 \
    DATABASE_URL="" \
    PORT=99999 \
    LOG_LEVEL=verbose

echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
```

### Why this works

- Each test sets specific environment variables and checks the exit code.
- `set +e` / `set -e` around the app run prevents the test script from exiting on the first app failure.
- `env "$@"` applies the test-specific variables without polluting the shell environment.
- The script counts passes and failures, then exits with the appropriate code.

### Common mistakes

- Not isolating environment variables between tests. If test 2 sets `PORT=99999` and test 3 does not explicitly set `PORT`, test 3 might inherit the bad value. Use `env` to start a clean environment for each test.
- Only testing the happy path. The validation module exists to catch errors -- test the error paths thoroughly.
- Not capturing stderr. Validation errors are printed to stderr. Use `2>&1` to capture both stdout and stderr in the output variable.

---

## Full working example (Python)

Putting it all together in a single `app.py`:

```python
import os
import sys
from dataclasses import dataclass, field
from typing import List


@dataclass
class Config:
    port: int = 8080
    log_level: str = "info"
    node_env: str = "development"
    database_url: str = ""
    redis_url: str = "redis://localhost:6379"
    max_retries: int = 3
    request_timeout_ms: int = 5000
    allowed_origins: List[str] = field(default_factory=lambda: ["*"])
    enable_metrics: bool = False
    worker_concurrency: int = 4


def parse_bool(value: str) -> bool:
    return value.strip().lower() == "true"


def parse_int(value: str, name: str) -> int:
    try:
        return int(value)
    except ValueError:
        raise ValueError(f"{name} must be an integer, got: '{value}'")


def parse_string_list(value: str) -> List[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def load_config() -> Config:
    errors: List[str] = []

    # Parse all values
    try:
        port = parse_int(os.environ.get("PORT", "8080"), "PORT")
    except ValueError as e:
        errors.append(str(e))
        port = 0

    log_level = os.environ.get("LOG_LEVEL", "info").strip().lower()
    node_env = os.environ.get("NODE_ENV", "development").strip().lower()
    database_url = os.environ.get("DATABASE_URL", "")
    redis_url = os.environ.get("REDIS_URL", "redis://localhost:6379")

    try:
        max_retries = parse_int(os.environ.get("MAX_RETRIES", "3"), "MAX_RETRIES")
    except ValueError as e:
        errors.append(str(e))
        max_retries = 0

    try:
        request_timeout_ms = parse_int(
            os.environ.get("REQUEST_TIMEOUT_MS", "5000"), "REQUEST_TIMEOUT_MS"
        )
    except ValueError as e:
        errors.append(str(e))
        request_timeout_ms = 0

    allowed_origins = parse_string_list(os.environ.get("ALLOWED_ORIGINS", "*"))
    enable_metrics = parse_bool(os.environ.get("ENABLE_METRICS", "false"))

    try:
        worker_concurrency = parse_int(
            os.environ.get("WORKER_CONCURRENCY", "4"), "WORKER_CONCURRENCY"
        )
    except ValueError as e:
        errors.append(str(e))
        worker_concurrency = 0

    config = Config(
        port=port,
        log_level=log_level,
        node_env=node_env,
        database_url=database_url,
        redis_url=redis_url,
        max_retries=max_retries,
        request_timeout_ms=request_timeout_ms,
        allowed_origins=allowed_origins,
        enable_metrics=enable_metrics,
        worker_concurrency=worker_concurrency,
    )

    # Validate constraints
    if not (1024 <= config.port <= 65535):
        errors.append(f"PORT must be 1024-65535, got: {config.port}")

    allowed_log_levels = {"trace", "debug", "info", "warn", "error"}
    if config.log_level not in allowed_log_levels:
        errors.append(
            f"LOG_LEVEL must be one of {sorted(allowed_log_levels)}, "
            f"got: '{config.log_level}'"
        )

    allowed_envs = {"development", "staging", "production"}
    if config.node_env not in allowed_envs:
        errors.append(
            f"NODE_ENV must be one of {sorted(allowed_envs)}, "
            f"got: '{config.node_env}'"
        )

    if not database_url:
        errors.append("DATABASE_URL is required but not set")
    elif not database_url.startswith("postgres://"):
        errors.append(
            f"DATABASE_URL must start with 'postgres://', "
            f"got: '{database_url[:20]}...'"
        )

    if redis_url and not redis_url.startswith("redis://"):
        errors.append(
            f"REDIS_URL must start with 'redis://', "
            f"got: '{redis_url[:20]}...'"
        )

    if not (1 <= config.max_retries <= 10):
        errors.append(
            f"MAX_RETRIES must be 1-10, got: {config.max_retries}"
        )

    if not (100 <= config.request_timeout_ms <= 60000):
        errors.append(
            f"REQUEST_TIMEOUT_MS must be 100-60000, "
            f"got: {config.request_timeout_ms}"
        )

    if not (1 <= config.worker_concurrency <= 32):
        errors.append(
            f"WORKER_CONCURRENCY must be 1-32, "
            f"got: {config.worker_concurrency}"
        )

    if errors:
        print("FATAL: Configuration validation failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        sys.exit(1)

    # Log summary
    masked_db = database_url
    if "@" in masked_db:
        s = masked_db.index("://") + 3
        e = masked_db.index("@")
        masked_db = masked_db[:s] + "***" + masked_db[e:]

    print(f"[CONFIG] PORT={config.port}")
    print(f"[CONFIG] LOG_LEVEL={config.log_level}")
    print(f"[CONFIG] NODE_ENV={config.node_env}")
    print(f"[CONFIG] DATABASE_URL={masked_db}")
    print(f"[CONFIG] REDIS_URL={config.redis_url}")
    print(f"[CONFIG] MAX_RETRIES={config.max_retries}")
    print(f"[CONFIG] REQUEST_TIMEOUT_MS={config.request_timeout_ms}")
    print(f"[CONFIG] ALLOWED_ORIGINS={config.allowed_origins}")
    print(f"[CONFIG] ENABLE_METRICS={config.enable_metrics}")
    print(f"[CONFIG] WORKER_CONCURRENCY={config.worker_concurrency}")

    return config


if __name__ == "__main__":
    config = load_config()
    print("\nConfiguration is valid. Starting service...")
```
