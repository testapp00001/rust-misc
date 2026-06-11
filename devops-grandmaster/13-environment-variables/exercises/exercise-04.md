# Exercise 04: Env Var Validation and Defaults

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Build a configuration module that reads environment variables, validates their types and values, provides sensible defaults, and fails fast on invalid configuration. This exercise trains you to treat configuration as a contract, not a loose bag of strings.

## Scenario

You are building a payment processing microservice. Misconfiguration is not a minor inconvenience -- a wrong port means the service is unreachable; a wrong log level means you miss critical errors in production; a missing API key means payments silently fail. You need a configuration system that validates everything at startup and refuses to run with bad config.

## Requirements

The service needs the following configuration:

| Variable | Type | Required | Default | Constraints |
|----------|------|----------|---------|-------------|
| `PORT` | integer | no | 8080 | 1024-65535 |
| `LOG_LEVEL` | string | no | info | One of: trace, debug, info, warn, error |
| `NODE_ENV` | string | no | development | One of: development, staging, production |
| `DATABASE_URL` | string | **yes** | none | Must start with `postgres://` |
| `REDIS_URL` | string | no | redis://localhost:6379 | Must start with `redis://` |
| `MAX_RETRIES` | integer | no | 3 | 1-10 |
| `REQUEST_TIMEOUT_MS` | integer | no | 5000 | 100-60000 |
| `ALLOWED_ORIGINS` | string list | no | * (allow all) | Comma-separated, no spaces |
| `ENABLE_METRICS` | boolean | no | false | `true` or `false` |
| `WORKER_CONCURRENCY` | integer | no | 4 | 1-32 |

## Tasks

### Part A: Define the Config struct (or class)

Write a data structure that holds all ten configuration values with their proper types (integer, boolean, string, string list). Use the language of your choice -- Python dataclass, Rust struct, JavaScript class, or Go struct.

<details>
<summary>Hint</summary>

In Python, use `@dataclass` with type annotations. In Rust, use a struct with `pub` fields. The `ALLOWED_ORIGINS` field should be a list of strings, not a single string. The `ENABLE_METRICS` field should be a boolean, not a string.

</details>

### Part B: Implement the parser and validator

Write a function that:

1. Reads each variable from the environment.
2. Converts it to the correct type (string to int, string to bool, comma-separated string to list).
3. Applies the default if the variable is not set.
4. Validates against the constraints (range, allowed values, format).
5. Returns the Config struct on success or exits with a clear error message on failure.

<details>
<summary>Hint</summary>

For boolean parsing, check for the string `"true"` (case-insensitive). For list parsing, split on commas. For integer parsing, catch `ValueError` (Python) or `parse::<u16>()` failure (Rust). Print the variable name, the invalid value, and the expected format in the error message.

</details>

### Part C: Implement fail-fast validation

The application should exit immediately at startup if any required variable is missing or any value is invalid. Write a `validate_config()` function that:

1. Checks that `DATABASE_URL` is set and starts with `postgres://`.
2. Checks that `PORT` is in the valid range.
3. Checks that `LOG_LEVEL` is one of the allowed values.
4. Checks that `NODE_ENV` is one of the allowed values.
5. Prints all errors at once (do not stop at the first error).

<details>
<summary>Hint</summary>

Collect errors into a list. After checking all variables, if the error list is non-empty, print each error on its own line and exit with code 1. This lets the developer fix all problems in one pass instead of fixing one, redeploying, discovering the next, and repeating.

</details>

### Part D: Write a config summary logger

After successful validation, print a summary of the resolved configuration. Mask any sensitive values (like `DATABASE_URL` -- show only the host, not the password).

Example output:

```
[CONFIG] PORT=8080
[CONFIG] LOG_LEVEL=info
[CONFIG] NODE_ENV=development
[CONFIG] DATABASE_URL=postgres://***@db-host:5432/myapp
[CONFIG] REDIS_URL=redis://localhost:6379
[CONFIG] MAX_RETRIES=3
[CONFIG] REQUEST_TIMEOUT_MS=5000
[CONFIG] ALLOWED_ORIGINS=["*"]
[CONFIG] ENABLE_METRICS=false
[CONFIG] WORKER_CONCURRENCY=4
```

<details>
<summary>Hint</summary>

Parse the DATABASE_URL to extract the host and database name while replacing the password portion. A simple approach: split on `@` and replace the password segment between `://` and `@` with `***`.

</details>

### Part E: Test your validation

Write test commands (or a test script) that exercises each validation rule:

1. Start with all valid values -- should succeed.
2. Set `PORT=99999` -- should fail with range error.
3. Set `LOG_LEVEL=verbose` -- should fail with allowed-values error.
4. Unset `DATABASE_URL` -- should fail with required-variable error.
5. Set `DATABASE_URL=mysql://...` -- should fail with format error.
6. Set multiple invalid values -- should report all errors, not just the first.

<details>
<summary>Hint</summary>

Use a shell script that exports variables, runs the app, checks the exit code, and captures stderr. Compare the exit code and error output against expected values. This is essentially a test suite for your config module.

</details>

## Success Criteria

- [ ] All ten variables are parsed to their correct types.
- [ ] Missing optional variables use the specified defaults.
- [ ] Missing required variables cause an immediate exit with a clear error.
- [ ] Invalid values (out of range, wrong format, not in allowed set) cause an immediate exit.
- [ ] Multiple errors are reported in a single run, not one at a time.
- [ ] The config summary masks sensitive values like database passwords.
- [ ] The test script exercises all validation rules and checks exit codes.

## What You Should Understand After This Exercise

Configuration validation is a defense mechanism. By validating at startup, you catch misconfigurations in seconds instead of discovering them when a production request fails. The pattern is always the same: read, parse, validate, fail fast. A well-designed config module turns environment variables from a source of runtime surprises into a typed, validated contract.
