# Exercise 04: Design a Logging Strategy with Rotation, Retention, and Cost Control

**Type:** Challenge
**Time:** 60 minutes
**Difficulty:** Medium-Hard

## Objective

Design a complete logging strategy for a production system running 20
containers across 3 hosts. Your strategy must address log rotation, retention
policies, storage budgeting, and cost control. You will produce configuration
files and a written policy document.

## Background

Your company runs a microservices platform with these characteristics:

- **20 containers** across 3 Docker hosts
- Average log volume: **500 MB per container per day** (10 GB/day total)
- Peak traffic (Black Friday): **3x normal volume** (30 GB/day)
- Log storage budget: **$200/month** (self-hosted, disk cost only)
- Compliance requirement: retain logs for **90 days**
- Team size: 4 developers who need to search logs for debugging

Without a strategy, logs will fill disks, costs will spiral, and developers
will be unable to find the information they need.

---

## Instructions

### Part A: Calculate Storage Requirements

Answer these questions with actual numbers:

1. At 10 GB/day with 90-day retention, how much raw storage do you need?
2. If you achieve 10:1 compression (typical for text logs), how much disk
   space do you actually need?
3. At Black Friday volume (30 GB/day) for 3 days, how much additional
   storage do you need?
4. If 1 TB of SSD storage costs $50/month, does your plan fit within the
   $200/month budget?

<details>
<summary>Hint</summary>

- Raw: 10 GB x 90 days = 900 GB
- Compressed: 900 GB / 10 = 90 GB
- Black Friday surge: (30 - 10) GB x 3 days = 60 GB extra
- Total peak: 90 + 60 = 150 GB, well under 1 TB

</details>

### Part B: Configure Docker Log Rotation

Write the Docker daemon configuration (`daemon.json`) and individual
container run commands that enforce log rotation. Your configuration must:

1. Set a global default log driver with rotation
2. Ensure no single container can consume more than 50 MB of log storage
3. Keep enough log files to cover at least 2 hours of peak traffic
4. Allow individual containers to override the global default

Calculate: at peak volume, how fast does a single container fill a 10 MB
log file? How many files do you need to cover 2 hours?

<details>
<summary>Hint</summary>

- Peak volume per container: 500 MB/day x 3 = 1.5 GB/day = 62.5 MB/hour
- At 10 MB per file: 62.5 / 10 = ~6.25 files per hour
- For 2 hours: ~13 files. Round up to 15 for safety.
- Total per container: 15 files x 10 MB = 150 MB. Adjust `max-size` or
  `max-file` to fit the 50 MB constraint.

</details>

### Part C: Write the Daemon Configuration

Create a `/etc/docker/daemon.json` file that:

1. Uses the `json-file` log driver (or `local` for better compression)
2. Sets `max-size` and `max-file` as global defaults
3. Sets `compress: true` to enable gzip compression of rotated files
4. Sets `tag` to include the container name in log entries

<details>
<summary>Hint</summary>

The `json-file` driver supports `compress` and `tag` options. The `local`
driver has built-in compression and uses less disk, but is not as widely
supported by log collectors.

```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "...",
    "max-file": "...",
    "compress": "true",
    "tag": "{{.Name}}"
  }
}
```

</details>

### Part D: Write a docker-compose.yml with Per-Service Overrides

Create a `docker-compose.yml` for three representative services:

1. **web-frontend** -- high volume, low importance. Use `max-size: 5m`,
   `max-file: 3`. Consider using `log-driver: none` for static asset
   requests.
2. **api-backend** -- medium volume, high importance. Use `max-size: 20m`,
   `max-file: 5`.
3. **payment-service** -- low volume, critical importance. Use `max-size: 50m`,
   `max-file: 10`. Never lose these logs.

Each service must also set its `LOG_LEVEL` environment variable appropriately.

<details>
<summary>Hint</summary>

In `docker-compose.yml`, logging is configured per-service:

```yaml
services:
  web-frontend:
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "3"
```

</details>

### Part E: Write the Logging Policy Document

Create a file called `LOGGING_POLICY.md` that covers:

1. **Log levels** -- what each level means and when to use it
2. **Required fields** -- what every JSON log entry must contain
3. **Retention policy** -- how long logs are kept at each tier
4. **Storage budget** -- how much disk is allocated and how it is divided
5. **Escalation** -- who is notified for ERROR and CRITICAL logs
6. **Review schedule** -- how often the team reviews logging costs and volume

<details>
<summary>Hint</summary>

A good logging policy answers these questions before they are asked:
- "Should I log this at INFO or DEBUG?" (the policy defines the boundary)
- "How far back can I search?" (the retention policy answers this)
- "Why are we out of disk?" (the storage budget prevents this)
- "Who gets paged at 3 AM?" (the escalation policy answers this)

</details>

### Part F: Implement a Log Cleanup Script

Write a bash script called `cleanup-logs.sh` that:

1. Finds all Docker log files older than 7 days
2. Compresses any uncompressed log files
3. Deletes compressed log files older than 90 days
4. Reports how much space was reclaimed
5. Is safe to run via cron (no interactive prompts, proper error handling)

<details>
<summary>Hint</summary>

Docker log files live in `/var/lib/docker/containers/<id>/<id>-json.log*`.
Use `find` with `-mtime` for age-based selection. Use `gzip` for compression.
Always test with `-dry-run` (print before delete) before deploying.

</details>

---

## Success Criteria

- [ ] Your storage calculations are correct and fit within the $200/month
      budget
- [ ] Your `daemon.json` enforces log rotation globally
- [ ] Your `docker-compose.yml` has per-service log rotation overrides
      appropriate to each service's importance and volume
- [ ] Your `LOGGING_POLICY.md` covers all six required sections
- [ ] Your `cleanup-logs.sh` script handles compression, deletion, and
      reporting
- [ ] You can explain the trade-off between `max-size` and `max-file` and
      how they interact
- [ ] You understand why `log-driver: none` is sometimes the right choice

## What You Should Understand After This Exercise

Logging is not free. Every log line consumes disk, network bandwidth (if
forwarded), and money. A production logging strategy must balance
observability (keep everything) against cost (keep only what matters). Log
rotation is your first line of defense against runaway disk usage. Retention
policies are your second. Together, they turn an unpredictable cost into a
budgeted line item.
