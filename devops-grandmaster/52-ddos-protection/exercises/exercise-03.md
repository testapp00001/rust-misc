# Exercise 03: Traffic Analysis and Anomaly Detection

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Build a traffic analysis system that monitors request patterns, establishes baselines, detects anomalies indicative of DDoS attacks, and triggers automated alerts. You will write a Python-based traffic analyzer that processes Nginx access logs and identifies attack signatures.

## Scenario

Your production environment generates Nginx access logs in the following format:

```
192.168.1.10 - - [15/Jan/2024:10:30:00 +0000] "GET /api/data HTTP/1.1" 200 1234 "-" "Mozilla/5.0" rt=0.023
192.168.1.10 - - [15/Jan/2024:10:30:01 +0000] "POST /api/auth/login HTTP/1.1" 200 567 "-" "curl/7.88" rt=0.045
10.0.0.50 - - [15/Jan/2024:10:30:01 +0000] "GET /api/expensive HTTP/1.1" 200 8901 "-" "python-requests/2.31" rt=1.234
```

You need to build a system that can process these logs and detect when the traffic pattern deviates from normal.

## Tasks

### Part A: Log Parser

Write a Python module called `log_parser.py` that parses Nginx access log lines into structured data.

Each parsed entry should contain:
- `ip` -- client IP address
- `timestamp` -- datetime object
- `method` -- HTTP method (GET, POST, etc.)
- `path` -- request path
- `status` -- HTTP status code (integer)
- `bytes_sent` -- response size in bytes
- `user_agent` -- User-Agent string
- `request_time` -- request processing time in seconds (float)

Handle edge cases:
- Malformed log lines (return None, do not crash)
- Missing fields (use sensible defaults)
- Different timestamp formats

<details>
<summary>Hint: Parsing approach</summary>

Use a regular expression to parse the combined log format. The pattern is:
```
^(\S+) \S+ \S+ \[([^\]]+)\] "(\S+) (\S+) \S+" (\d+) (\d+) "([^"]*)" "([^"]*)" rt=(\S+)
```
Use `datetime.strptime` to parse the timestamp: `%d/%b/%Y:%H:%M:%S %z`. Wrap the parsing in a try/except to handle malformed lines gracefully.

</details>

### Part B: Baseline Calculator

Write a module called `baseline.py` that calculates traffic baselines from historical log data.

Calculate these baseline metrics from a set of parsed log entries:
1. **Requests per second (RPS)** -- total requests divided by the time span
2. **Requests per IP per minute** -- average and standard deviation
3. **Status code distribution** -- percentage of 2xx, 3xx, 4xx, 5xx responses
4. **Average request time** -- mean and standard deviation
5. **Unique IPs per minute** -- average count
6. **Top paths by request count** -- the 10 most-requested paths

Store baselines in a JSON-serializable dictionary with this structure:
```python
{
    "rps": {"mean": 150.0, "std": 30.0},
    "requests_per_ip_per_minute": {"mean": 20.0, "std": 15.0},
    "status_distribution": {"2xx": 0.85, "3xx": 0.05, "4xx": 0.08, "5xx": 0.02},
    "avg_request_time": {"mean": 0.15, "std": 0.05},
    "unique_ips_per_minute": {"mean": 50.0, "std": 10.0},
    "top_paths": ["/api/data", "/api/auth/login", ...]
}
```

<details>
<summary>Hint: Calculating standard deviation</summary>

Use Python's `statistics` module:
```python
import statistics
values = [count_per_minute_1, count_per_minute_2, ...]
mean = statistics.mean(values)
std = statistics.stdev(values) if len(values) > 1 else 0.0
```
Group log entries by minute using `timestamp.replace(second=0, microsecond=0)` to get per-minute buckets.

</details>

### Part C: Anomaly Detector

Write a module called `detector.py` that compares current traffic against baselines and flags anomalies.

Implement detection for these attack signatures:

1. **Volume spike** -- Current RPS exceeds baseline mean by more than 3 standard deviations
2. **IP concentration** -- A single IP exceeds 10x the average per-IP request rate
3. **Error rate spike** -- 5xx responses exceed 20% of total traffic
4. **Path hammering** -- A single path receives more than 50% of all requests
5. **Slow request flood** -- Average request time exceeds 2x the baseline mean
6. **New IP surge** -- Unique IPs per minute exceeds 3x the baseline mean
7. **User-Agent anomalies** -- More than 30% of requests have empty or suspicious User-Agents

For each detected anomaly, return a structured alert:
```python
{
    "timestamp": "2024-01-15T10:30:00Z",
    "detection_type": "volume_spike",
    "severity": "critical",  # critical, warning, info
    "description": "RPS is 450, baseline mean is 150 (3.0 std deviations above)",
    "metrics": {"current_rps": 450, "baseline_mean": 150, "std_deviations": 3.0},
    "recommended_action": "Enable aggressive rate limiting and review top IPs"
}
```

<details>
<summary>Hint: Severity levels</summary>

Assign severity based on how far the metric deviates from baseline:
- **info**: 2-3 std deviations above mean, or single metric anomaly
- **warning**: 3-4 std deviations, or 2+ simultaneous anomalies
- **critical**: 4+ std deviations, or 3+ simultaneous anomalies, or any volume spike that is 10x baseline

Use the `recommended_action` field to provide specific guidance based on the anomaly type.

</details>

### Part D: Alert Manager

Write a module called `alerter.py` that manages alert state and prevents alert fatigue.

Implement:
1. **Alert deduplication** -- Do not send the same alert type more than once per 5-minute window
2. **Alert escalation** -- If a warning persists for more than 5 minutes, escalate to critical
3. **Alert history** -- Keep a log of all alerts with timestamps
4. **Multiple output formats** -- Support console output, JSON file, and webhook (HTTP POST)

The alerter should expose a `process_alerts(alerts: list)` method that handles deduplication, escalation, and output.

<details>
<summary>Hint: Deduplication</summary>

Use a dictionary keyed by `detection_type` to track the last alert time for each type:
```python
last_alert_times = {}

def should_alert(detection_type, cooldown_seconds=300):
    now = time.time()
    last = last_alert_times.get(detection_type, 0)
    if now - last >= cooldown_seconds:
        last_alert_times[detection_type] = now
        return True
    return False
```

</details>

### Part E: Main Analysis Pipeline

Write a main script called `analyze_traffic.py` that ties everything together:

1. Accepts a log file path as a command-line argument (or reads from stdin)
2. Parses all log entries
3. Calculates baselines from the first 80% of entries
4. Analyzes the remaining 20% against the baselines
5. Outputs detected anomalies and a summary report

Usage:
```bash
# Analyze a log file
python analyze_traffic.py --log /var/log/nginx/access.log

# Analyze with a pre-computed baseline
python analyze_traffic.py --log /var/log/nginx/access.log --baseline baseline.json

# Continuous monitoring (read new lines as they appear)
python analyze_traffic.py --log /var/log/nginx/access.log --watch --interval 10

# Generate sample data for testing
python analyze_traffic.py --generate-sample --output sample.log
```

Include a `--generate-sample` flag that creates synthetic log data containing both normal traffic and simulated attack patterns for testing.

<details>
<summary>Hint: Sample data generation</summary>

Generate normal traffic with random IPs, paths from the top paths list, and Gaussian-distributed request times. Then inject attack traffic: a burst of requests from a small set of IPs targeting a specific path. This makes it easy to verify the detector catches the anomalies.

```python
import random
from datetime import datetime, timedelta

def generate_normal_traffic(start_time, duration_minutes, base_rps=100):
    entries = []
    current = start_time
    end = start_time + timedelta(minutes=duration_minutes)
    paths = ['/api/data', '/api/auth/login', '/api/search', '/health']
    while current < end:
        for _ in range(random.randint(int(base_rps*0.8), int(base_rps*1.2))):
            ip = f"192.168.{random.randint(1,254)}.{random.randint(1,254)}"
            path = random.choice(paths)
            entries.append((ip, current, 'GET', path, 200))
        current += timedelta(seconds=1)
    return entries

def inject_attack(entries, attack_start, attack_ips, target_path, rps_multiplier=20):
    # Add a burst of requests from a small number of IPs
    for second in range(60):
        t = attack_start + timedelta(seconds=second)
        for _ in range(rps_multiplier * 10):
            ip = random.choice(attack_ips)
            entries.append((ip, t, 'GET', target_path, 200))
```

</details>

---

## Success Criteria

- [ ] The log parser correctly handles valid log lines and gracefully handles malformed input.
- [ ] The baseline calculator computes RPS, per-IP rates, status distributions, and request time statistics.
- [ ] The anomaly detector identifies at least 5 of the 7 defined attack signatures.
- [ ] The alert manager deduplicates alerts and does not flood the output with repeated notifications.
- [ ] The main pipeline can analyze a log file and produce a summary report with detected anomalies.
- [ ] The sample data generator produces data that triggers the anomaly detector.
- [ ] All modules have clear docstrings and handle edge cases without crashing.

## What You Should Understand After This Exercise

Traffic analysis is the foundation of DDoS detection. You cannot defend against what you cannot see. The key insight is that DDoS detection is about comparing current traffic to a baseline -- not looking for a single magic number. A burst of 500 req/s might be normal on Black Friday but an attack at 3 AM. By calculating baselines from historical data and using statistical thresholds (standard deviations), you can detect anomalies regardless of absolute traffic volume. The hardest attacks to detect are application-layer floods that look like legitimate traffic -- these require behavioral analysis (IP concentration, path hammering, User-Agent patterns) rather than simple volume thresholds.
