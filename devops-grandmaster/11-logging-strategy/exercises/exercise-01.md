# Exercise 01: stdout vs stderr and Why Structured Logging Matters

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Explain the difference between stdout and stderr in the context of containers,
and articulate why structured logging (JSON) is superior to unstructured
plain-text logs for production systems.

## Background

Every process has three standard I/O streams: stdin (0), stdout (1), and
stderr (2). In containerized environments, Docker captures everything written
to stdout and stderr and makes it available through `docker logs`. How you use
these two streams -- and the format of what you write to them -- has a direct
impact on your ability to debug, monitor, and operate your applications.

---

## Tasks

### Part A: stdout vs stderr

Answer each question in two or three sentences.

1. What is the fundamental purpose of stdout versus stderr?
2. When a Python application calls `print("User logged in")`, which stream does
   the output go to? What about `print("Disk full", file=sys.stderr)`?
3. Docker captures both stdout and stderr. Does `docker logs` show one, the
   other, or both? How can you filter to see only stderr output?
4. A developer writes all log output to stdout, including error messages. What
   problems does this cause in a production environment with a log collection
   pipeline?

<details>
<summary>Hint</summary>

- stdout is for normal program output; stderr is for diagnostic/error output.
- `docker logs` shows both streams. Use `docker logs <container> 1>/dev/null`
  to see only stderr, or `2>/dev/null` to see only stdout.
- If errors and informational messages all go to the same stream, your log
  collector cannot route or alert on them differently.

</details>

### Part B: Unstructured vs Structured Logs

You are given two log lines from the same event:

**Unstructured:**
```
2024-03-15 10:30:45 ERROR Something went wrong while processing order 12345 for user 67890 from 10.0.0.5
```

**Structured (JSON):**
```json
{"timestamp": "2024-03-15T10:30:45Z", "level": "ERROR", "message": "Processing failed", "order_id": 12345, "user_id": 67890, "source_ip": "10.0.0.5", "service": "order-api"}
```

Answer the following:

1. If you need to find all ERROR-level logs across 50 containers, which format
   makes this easier? Write the command or query you would use for each format.
2. If you need to calculate the average number of errors per hour for a specific
   user, which format makes this possible without custom parsing?
3. A new developer joins the team. Which format is more self-documenting and
   why?
4. What happens to a regex-based parser when someone changes the log message
   wording from "Something went wrong while processing order" to "Order
   processing failed"?

<details>
<summary>Hint</summary>

- Think about `grep`, `jq`, and log aggregation query languages (KQL, Lucene,
  LogQL).
- Structured logs have explicit field names -- no ambiguity about what each
  value represents.
- Regex parsers are fragile; JSON parsers are not.

</details>

### Part C: Identify the Bad Practices

For each code snippet, identify what is wrong and explain how to fix it.

**Snippet 1:**
```python
import logging
logging.basicConfig(filename='/var/log/app.log')
logger = logging.getLogger(__name__)
logger.info("Application started")
```

**Snippet 2:**
```python
print(f"[{datetime.now()}] INFO: User {user_id} logged in from {ip}")
print(f"[{datetime.now()}] ERROR: Database connection failed")
```

**Snippet 3:**
```bash
# Dockerfile
RUN mkdir -p /app/logs
CMD ["python", "app.py"]
# app.py writes logs to /app/logs/app.log
```

<details>
<summary>Hint</summary>

- Where does `/var/log/app.log` live when the container is removed?
- What happens when you need to search for all ERROR logs across 20 containers
  using `print()` statements?
- Can `docker logs` see output written to a file?

</details>

### Part D: Design a Logging Contract

Your team runs five microservices. Write a short "logging contract" (5-8 bullet
points) that all services must follow. Cover:

- Which stream to use for what type of output
- Required fields in every log entry
- Timestamp format
- How to handle stack traces

<details>
<summary>Hint</summary>

- Consider using ISO 8601 for timestamps (e.g., `2024-03-15T10:30:45Z`).
- Every log entry should have at minimum: timestamp, level, message, service
  name.
- Stack traces should be a separate field, not embedded in the message string.

</details>

---

## Success Criteria

- [ ] You can explain when to use stdout versus stderr and why the distinction
      matters for log routing and alerting
- [ ] You can articulate at least three concrete advantages of structured
      (JSON) logging over plain-text logging
- [ ] You can identify common anti-patterns in container logging (logging to
      files, using print instead of a logger, unstructured output)
- [ ] You can define a basic logging contract for a multi-service system
- [ ] You understand why `docker logs` is the first tool to reach for and what
      its limitations are

## What You Should Understand After This Exercise

stdout and stderr are not just "two output streams" -- they are the foundation
of container observability. Everything your application writes to these streams
is captured by Docker and can be forwarded to centralized logging systems.
Structured logging transforms logs from human-readable strings into
machine-queryable data, which is essential once you operate more than one
container.
