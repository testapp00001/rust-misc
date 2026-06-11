# Solution 01: stdout vs stderr and Why Structured Logging Matters

## Part A: stdout vs stderr

### 1. Fundamental purpose of stdout vs stderr

**stdout** (file descriptor 1) is for a program's primary output -- the
data that downstream consumers (other programs, files, users) are expected
to process. **stderr** (file descriptor 2) is for diagnostic messages --
errors, warnings, and operational information that is meant for human
consumption and should not interfere with the primary data stream.

The key insight: stdout is for *data*, stderr is for *noise around the data*.
A program that outputs JSON to stdout and error messages to stderr lets you
pipe its output to another program without error messages corrupting the
data stream.

### 2. Which stream does `print()` use?

`print("User logged in")` writes to **stdout** by default. Python's `print()`
function uses `sys.stdout` unless you specify otherwise.

`print("Disk full", file=sys.stderr)` writes to **stderr** because you
explicitly redirected the output.

### 3. Does `docker logs` show one or both?

`docker logs` shows **both** stdout and stderr, interleaved in the order
they were written. To filter:

```bash
# Show only stderr (redirect stdout to /dev/null)
docker logs my-app 1>/dev/null

# Show only stdout (redirect stderr to /dev/null)
docker logs my-app 2>/dev/null
```

### 4. Problems with sending everything to stdout

If errors and informational messages all go to stdout:

- Your log collector cannot route errors to an alerting system while sending
  info logs to cold storage. They are on the same stream with the same
  metadata.
- You cannot use Docker's logging driver options to treat the two streams
  differently.
- In a pipeline (`docker logs my-app | grep ERROR`), you are doing
  string-matching that the application should have done by choosing the
  right stream.
- Monitoring tools that watch stderr for unexpected output will see nothing,
  even when errors are occurring.

## Part B: Unstructured vs Structured Logs

### 1. Finding all ERROR logs across 50 containers

**Unstructured** -- you would use grep with a fragile regex:
```bash
docker logs container-1 2>&1 | grep "^[0-9-]* [0-9:]* ERROR"
docker logs container-2 2>&1 | grep "^[0-9-]* [0-9:]* ERROR"
# ... repeat for all 50 containers
```

This breaks if the timestamp format changes, if someone writes "Error"
instead of "ERROR", or if the message itself contains the word "ERROR".

**Structured** -- you use jq or a log aggregation query:
```bash
docker logs container-1 2>&1 | jq 'select(.level == "ERROR")'
# Or in Kibana/KQL: level: "ERROR"
# Or in LogQL: {service=~".+"} | json | level="ERROR"
```

This works regardless of message content or timestamp format because the
level is an explicit, separate field.

### 2. Calculating average errors per hour per user

**Unstructured** -- nearly impossible without writing a custom regex parser
that extracts "user 67890" from the message text. If someone changes "for
user" to "by user" or "user_id:", the parser breaks.

**Structured** -- straightforward:
```bash
docker logs container-1 2>&1 | jq 'select(.level == "ERROR")' | jq -s '
  group_by(.user_id) |
  map({user: .[0].user_id, count: length, per_hour: (length / 24)})
'
```

### 3. Which format is more self-documenting?

The **structured format** is more self-documenting. The field names
(`order_id`, `user_id`, `source_ip`, `service`) describe what each value
represents. In the unstructured format, you have to read the message and
interpret "order 12345 for user 67890 from 10.0.0.5" -- the meaning is
implicit in the sentence structure, not explicit in the data.

### 4. What happens when wording changes?

A regex parser designed to match `"Something went wrong while processing order"`
will silently stop matching when the message changes to `"Order processing
failed"`. You will lose visibility into those errors with no warning.

A JSON parser does not care about the message content -- it reads the `level`
field, which remains `"ERROR"` regardless of message wording.

## Part C: Identify the Bad Practices

### Snippet 1: Logging to a file

```python
logging.basicConfig(filename='/var/log/app.log')
```

**Problem:** When the container is removed, `/var/log/app.log` is gone.
The container filesystem is ephemeral (Module 09). Additionally, `docker logs`
cannot see output written to a file -- only stdout/stderr.

**Fix:**
```python
import sys
logging.basicConfig(stream=sys.stdout, level=logging.INFO)
```

### Snippet 2: Using print() with manual timestamps

```python
print(f"[{datetime.now()}] INFO: User {user_id} logged in from {ip}")
```

**Problem:** No log levels (you cannot filter ERROR from INFO without
parsing the string). Inconsistent timestamps (each call to `datetime.now()`
produces a slightly different format). No structured fields (user_id and ip
are embedded in the message string). Cannot be parsed by `jq` or log
aggregation tools.

**Fix:**
```python
import logging, json, sys

logger = logging.getLogger(__name__)
handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger.addHandler(handler)

logger.info("User logged in", extra={'user_id': user_id, 'ip': ip})
```

### Snippet 3: Logging to a file inside the container

```dockerfile
RUN mkdir -p /app/logs
CMD ["python", "app.py"]
# app.py writes logs to /app/logs/app.log
```

**Problem:** `docker logs` shows nothing because the application writes to
a file, not to stdout. The log file is on the ephemeral container filesystem.
Log rotation must be handled inside the application or container, which is
complex and error-prone.

**Fix:** Change the application to write to stdout. Remove the `mkdir` for
log files. Docker handles log rotation via `--log-opt max-size` and
`--log-opt max-file`.

## Part D: Sample Logging Contract

```
Logging Contract for All Services

1. ALL application logs go to stdout. Error and warning logs also go to
   stderr. No service writes logs to files.

2. Every log entry MUST be a single line of valid JSON containing at least:
   timestamp, level, message, service_name.

3. Timestamps MUST be in ISO 8601 format with UTC timezone:
   "2024-03-15T10:30:45.123Z"

4. Log levels are used as follows:
   - DEBUG: detailed diagnostic info (disabled in production)
   - INFO: normal operations (request received, response sent)
   - WARNING: unexpected but recoverable (retry, fallback used)
   - ERROR: operation failed (payment failed, DB unreachable)
   - CRITICAL: service cannot function (config missing, fatal error)

5. Stack traces go in a dedicated "exception" field, never embedded in
   the message string.

6. Request context (request_id, user_id, trace_id) is included in every
   log entry generated during a request.

7. PII (email addresses, passwords, credit card numbers) MUST NOT appear
   in log entries.

8. Log messages are written in English, present tense, and describe the
   event ("Order created") not the code ("Calling create_order()").
```

## Common Mistakes

- **Thinking stdout vs stderr does not matter** because Docker captures both.
  It matters for log routing, alerting, and pipeline compatibility.
- **Assuming structured logging is "more work."** The JSON formatter is 15
  lines of code written once. The time saved in debugging pays for itself
  the first time you need to search logs across multiple containers.
- **Embedding data in log messages** instead of as separate fields. This
  defeats the purpose of structured logging.
- **Not having a logging contract.** Without one, every developer makes
  different choices, and your logs become an inconsistent mess.
