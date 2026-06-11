# Solution 03: Design a Multi-Layer DDoS Defense

## Part A: Log Parser (`log_parser.py`)

```python
"""
log_parser.py -- Parse Nginx access log lines into structured data.
"""

import re
from datetime import datetime
from typing import Optional, Dict, Any

# Regex for Nginx combined log format with request time
LOG_PATTERN = re.compile(
    r'^(\S+)\s+\S+\s+\S+\s+\[([^\]]+)\]\s+"(\S+)\s+(\S+)\s+\S+"\s+(\d+)\s+(\d+)\s+"([^"]*)"\s+"([^"]*)"\s+rt=(\S+)'
)


def parse_log_line(line: str) -> Optional[Dict[str, Any]]:
    """
    Parse a single Nginx access log line into a structured dictionary.

    Returns None for malformed lines instead of raising exceptions.

    Args:
        line: A single line from an Nginx access log.

    Returns:
        A dictionary with keys: ip, timestamp, method, path, status,
        bytes_sent, user_agent, request_time.  None if the line is malformed.
    """
    line = line.strip()
    if not line:
        return None

    match = LOG_PATTERN.match(line)
    if not match:
        return None

    try:
        ip = match.group(1)
        timestamp_str = match.group(2)
        method = match.group(3)
        path = match.group(4)
        status = int(match.group(5))
        bytes_sent = int(match.group(6))
        user_agent = match.group(7) or "-"
        request_time = float(match.group(9))

        # Parse timestamp -- handle common Nginx format
        try:
            timestamp = datetime.strptime(timestamp_str, "%d/%b/%Y:%H:%M:%S %z")
        except ValueError:
            # Fallback: try without timezone
            timestamp = datetime.strptime(timestamp_str.split()[0], "%d/%b/%Y:%H:%M:%S")

        return {
            "ip": ip,
            "timestamp": timestamp,
            "method": method,
            "path": path,
            "status": status,
            "bytes_sent": bytes_sent,
            "user_agent": user_agent,
            "request_time": request_time,
        }
    except (ValueError, IndexError):
        return None


def parse_log_file(filepath: str) -> list:
    """
    Parse an entire Nginx access log file.

    Args:
        filepath: Path to the Nginx access log file.

    Returns:
        A list of parsed entry dictionaries (malformed lines are skipped).
    """
    entries = []
    with open(filepath, "r") as f:
        for line in f:
            entry = parse_log_line(line)
            if entry is not None:
                entries.append(entry)
    return entries
```

### Why This Works

- The regex captures each field from the Nginx combined log format. Group 8 (referer) is captured but not stored since it is not needed for anomaly detection.
- `parse_log_line` returns `None` for any line that does not match the expected format, rather than crashing. This is critical for production log parsing where log rotation, partial writes, or custom log formats can produce malformed lines.
- The timestamp parser has a fallback for logs that omit the timezone offset.
- `request_time` is parsed as a float to preserve sub-second precision (e.g., `rt=0.023`).

### Common Mistakes

- **Crashing on malformed lines** instead of returning `None`. Production logs always have some malformed entries due to log rotation or partial writes.
- **Using `re.match` without anchoring (`^`)**, which can match partial lines and produce garbage data.
- **Forgetting to handle empty `user_agent`**. Some clients send `""` which the regex captures as an empty string; downstream code that assumes a non-empty string will fail.

---

## Part B: Baseline Calculator (`baseline.py`)

```python
"""
baseline.py -- Calculate traffic baselines from historical log data.
"""

import statistics
from collections import Counter, defaultdict
from typing import List, Dict, Any


def calculate_baselines(entries: List[Dict[str, Any]]) -> Dict[str, Any]:
    """
    Calculate traffic baseline metrics from a list of parsed log entries.

    Args:
        entries: List of parsed log entry dictionaries from log_parser.

    Returns:
        A JSON-serializable dictionary of baseline metrics.
    """
    if not entries:
        return _empty_baseline()

    # --- Requests per second ---
    timestamps = [e["timestamp"] for e in entries]
    min_ts = min(timestamps)
    max_ts = max(timestamps)
    duration_seconds = max((max_ts - min_ts).total_seconds(), 1.0)
    rps = len(entries) / duration_seconds

    # --- Per-minute bucketing ---
    per_minute_counts = Counter()
    per_ip_per_minute = defaultdict(Counter)
    unique_ips_per_minute = defaultdict(set)
    status_counts = Counter()
    request_times = []
    path_counts = Counter()

    for entry in entries:
        minute_key = entry["timestamp"].replace(second=0, microsecond=0)
        per_minute_counts[minute_key] += 1
        per_ip_per_minute[minute_key][entry["ip"]] += 1
        unique_ips_per_minute[minute_key].add(entry["ip"])

        status = entry["status"]
        if 200 <= status < 300:
            status_counts["2xx"] += 1
        elif 300 <= status < 400:
            status_counts["3xx"] += 1
        elif 400 <= status < 500:
            status_counts["4xx"] += 1
        else:
            status_counts["5xx"] += 1

        request_times.append(entry["request_time"])
        path_counts[entry["path"]] += 1

    # --- Requests per IP per minute ---
    all_ip_counts = []
    for minute_bucket in per_ip_per_minute.values():
        all_ip_counts.extend(minute_bucket.values())
    rpipm_mean = statistics.mean(all_ip_counts) if all_ip_counts else 0.0
    rpipm_std = statistics.stdev(all_ip_counts) if len(all_ip_counts) > 1 else 0.0

    # --- Status distribution (as fractions) ---
    total = len(entries)
    status_distribution = {
        k: round(v / total, 4) for k, v in status_counts.items()
    }
    for key in ["2xx", "3xx", "4xx", "5xx"]:
        status_distribution.setdefault(key, 0.0)

    # --- Average request time ---
    rt_mean = statistics.mean(request_times) if request_times else 0.0
    rt_std = statistics.stdev(request_times) if len(request_times) > 1 else 0.0

    # --- Unique IPs per minute ---
    uipm_values = [len(s) for s in unique_ips_per_minute.values()]
    uipm_mean = statistics.mean(uipm_values) if uipm_values else 0.0
    uipm_std = statistics.stdev(uipm_values) if len(uipm_values) > 1 else 0.0

    # --- RPS per minute ---
    rps_values = list(per_minute_counts.values())
    rps_mean = statistics.mean(rps_values) if rps_values else 0.0
    rps_std = statistics.stdev(rps_values) if len(rps_values) > 1 else 0.0

    # --- Top paths ---
    top_paths = [path for path, _ in path_counts.most_common(10)]

    return {
        "rps": {"mean": round(rps_mean, 2), "std": round(rps_std, 2)},
        "requests_per_ip_per_minute": {"mean": round(rpipm_mean, 2), "std": round(rpipm_std, 2)},
        "status_distribution": status_distribution,
        "avg_request_time": {"mean": round(rt_mean, 4), "std": round(rt_std, 4)},
        "unique_ips_per_minute": {"mean": round(uipm_mean, 2), "std": round(uipm_std, 2)},
        "top_paths": top_paths,
    }


def _empty_baseline() -> Dict[str, Any]:
    """Return an empty baseline structure."""
    return {
        "rps": {"mean": 0.0, "std": 0.0},
        "requests_per_ip_per_minute": {"mean": 0.0, "std": 0.0},
        "status_distribution": {"2xx": 0.0, "3xx": 0.0, "4xx": 0.0, "5xx": 0.0},
        "avg_request_time": {"mean": 0.0, "std": 0.0},
        "unique_ips_per_minute": {"mean": 0.0, "std": 0.0},
        "top_paths": [],
    }
```

### Why This Works

- Per-minute bucketing uses `timestamp.replace(second=0, microsecond=0)` to create consistent time buckets. This allows calculating per-minute statistics even when the log spans hours.
- Standard deviation is calculated with `statistics.stdev` (sample std dev), which requires at least 2 data points. The code handles the single-sample case by returning 0.0.
- `per_ip_per_minute` tracks how many requests each IP makes per minute, giving a distribution of per-IP activity. This is the foundation for detecting IP concentration attacks.
- Status distribution is stored as fractions (not counts) so it can be compared across different time windows regardless of total volume.

### Common Mistakes

- **Using population standard deviation (`pstdev`) instead of sample standard deviation (`stdev`)**. For baseline calculation from a sample of traffic, `stdev` is correct.
- **Not handling empty input**. If the log file is empty or all lines are malformed, the function must not crash -- it should return the empty baseline structure.
- **Calculating RPS as `total_requests / total_seconds`** without also tracking per-minute variation. The mean RPS hides spikes that occur within individual minutes.

---

## Part C: Anomaly Detector (`detector.py`)

```python
"""
detector.py -- Compare current traffic against baselines and flag anomalies.
"""

from collections import Counter, defaultdict
from datetime import datetime
from typing import List, Dict, Any


def detect_anomalies(
    entries: List[Dict[str, Any]],
    baselines: Dict[str, Any],
) -> List[Dict[str, Any]]:
    """
    Analyze a window of log entries against precomputed baselines.

    Args:
        entries: Parsed log entries for the analysis window.
        baselines: Baseline dictionary from baseline.calculate_baselines().

    Returns:
        A list of alert dictionaries for detected anomalies.
    """
    if not entries:
        return []

    alerts = []
    now_str = entries[-1]["timestamp"].isoformat()

    # Aggregate current window metrics
    total_requests = len(entries)
    timestamps = [e["timestamp"] for e in entries]
    min_ts = min(timestamps)
    max_ts = max(timestamps)
    duration = max((max_ts - min_ts).total_seconds(), 1.0)
    current_rps = total_requests / duration

    # Per-IP counts
    ip_counts = Counter(e["ip"] for e in entries)

    # Status distribution
    status_counts = Counter()
    for e in entries:
        s = e["status"]
        if 200 <= s < 300:
            status_counts["2xx"] += 1
        elif 300 <= s < 400:
            status_counts["3xx"] += 1
        elif 400 <= s < 500:
            status_counts["4xx"] += 1
        else:
            status_counts["5xx"] += 1

    # Path distribution
    path_counts = Counter(e["path"] for e in entries)

    # Request times
    request_times = [e["request_time"] for e in entries]
    avg_rt = sum(request_times) / len(request_times) if request_times else 0

    # User-Agent analysis
    suspicious_ua = sum(
        1 for e in entries
        if not e["user_agent"] or e["user_agent"] == "-"
        or "bot" in e["user_agent"].lower()
        or "python" in e["user_agent"].lower()
        or "curl" in e["user_agent"].lower()
    )
    ua_suspicious_ratio = suspicious_ua / total_requests

    # Unique IPs
    unique_ips = len(ip_counts)

    # --- Detection 1: Volume Spike ---
    bl_rps = baselines["rps"]
    if bl_rps["std"] > 0:
        std_deviations = (current_rps - bl_rps["mean"]) / bl_rps["std"]
    else:
        std_deviations = (current_rps - bl_rps["mean"]) / max(bl_rps["mean"], 1.0)

    if std_deviations > 3:
        severity = _severity(std_deviations, 10 * bl_rps["mean"] <= current_rps)
        alerts.append({
            "timestamp": now_str,
            "detection_type": "volume_spike",
            "severity": severity,
            "description": (
                f"RPS is {current_rps:.0f}, baseline mean is "
                f"{bl_rps['mean']:.0f} ({std_deviations:.1f} std deviations above)"
            ),
            "metrics": {
                "current_rps": round(current_rps, 1),
                "baseline_mean": bl_rps["mean"],
                "std_deviations": round(std_deviations, 1),
            },
            "recommended_action": "Enable aggressive rate limiting and review top IPs",
        })

    # --- Detection 2: IP Concentration ---
    bl_rpipm = baselines["requests_per_ip_per_minute"]
    threshold_10x = bl_rpipm["mean"] * 10
    concentrated_ips = [
        (ip, count) for ip, count in ip_counts.most_common(10)
        if count > threshold_10x
    ]
    if concentrated_ips:
        top_ip, top_count = concentrated_ips[0]
        alerts.append({
            "timestamp": now_str,
            "detection_type": "ip_concentration",
            "severity": "warning",
            "description": (
                f"IP {top_ip} sent {top_count} requests "
                f"(baseline mean per IP: {bl_rpipm['mean']:.0f})"
            ),
            "metrics": {
                "top_ip": top_ip,
                "request_count": top_count,
                "baseline_mean_per_ip": bl_rpipm["mean"],
                "ratio": round(top_count / max(bl_rpipm["mean"], 1), 1),
            },
            "recommended_action": "Block or rate-limit the identified IPs",
        })

    # --- Detection 3: Error Rate Spike ---
    five_xx = status_counts.get("5xx", 0)
    error_rate = five_xx / total_requests
    if error_rate > 0.20:
        alerts.append({
            "timestamp": now_str,
            "detection_type": "error_rate_spike",
            "severity": "critical",
            "description": (
                f"5xx error rate is {error_rate:.1%} "
                f"({five_xx}/{total_requests} requests)"
            ),
            "metrics": {
                "error_rate": round(error_rate, 4),
                "five_xx_count": five_xx,
                "total_requests": total_requests,
            },
            "recommended_action": (
                "Check backend health; enable circuit breakers "
                "if database is overwhelmed"
            ),
        })

    # --- Detection 4: Path Hammering ---
    if path_counts:
        top_path, top_count = path_counts.most_common(1)[0]
        path_ratio = top_count / total_requests
        if path_ratio > 0.50:
            alerts.append({
                "timestamp": now_str,
                "detection_type": "path_hammering",
                "severity": "warning",
                "description": (
                    f"Path '{top_path}' received {path_ratio:.0%} "
                    f"of all requests ({top_count}/{total_requests})"
                ),
                "metrics": {
                    "path": top_path,
                    "request_count": top_count,
                    "ratio": round(path_ratio, 4),
                },
                "recommended_action": (
                    "Apply stricter rate limiting to the targeted path; "
                    "consider caching"
                ),
            })

    # --- Detection 5: Slow Request Flood ---
    bl_rt = baselines["avg_request_time"]
    if avg_rt > 2 * bl_rt["mean"]:
        alerts.append({
            "timestamp": now_str,
            "detection_type": "slow_request_flood",
            "severity": "warning",
            "description": (
                f"Average request time is {avg_rt:.3f}s, "
                f"baseline mean is {bl_rt['mean']:.3f}s"
            ),
            "metrics": {
                "current_avg_rt": round(avg_rt, 4),
                "baseline_mean_rt": bl_rt["mean"],
                "ratio": round(avg_rt / max(bl_rt["mean"], 0.001), 1),
            },
            "recommended_action": (
                "Check for slowloris or resource exhaustion; reduce timeouts"
            ),
        })

    # --- Detection 6: New IP Surge ---
    bl_uipm = baselines["unique_ips_per_minute"]
    ips_per_minute = defaultdict(set)
    for e in entries:
        mk = e["timestamp"].replace(second=0, microsecond=0)
        ips_per_minute[mk].add(e["ip"])
    avg_unique = (
        sum(len(s) for s in ips_per_minute.values())
        / max(len(ips_per_minute), 1)
    )

    if avg_unique > 3 * bl_uipm["mean"]:
        alerts.append({
            "timestamp": now_str,
            "detection_type": "new_ip_surge",
            "severity": "warning",
            "description": (
                f"Unique IPs per minute is {avg_unique:.0f}, "
                f"baseline mean is {bl_uipm['mean']:.0f}"
            ),
            "metrics": {
                "current_unique_ips_per_minute": round(avg_unique, 1),
                "baseline_mean": bl_uipm["mean"],
                "ratio": round(avg_unique / max(bl_uipm["mean"], 1), 1),
            },
            "recommended_action": (
                "Potential botnet; enable CAPTCHA or challenge pages for new IPs"
            ),
        })

    # --- Detection 7: User-Agent Anomalies ---
    if ua_suspicious_ratio > 0.30:
        alerts.append({
            "timestamp": now_str,
            "detection_type": "user_agent_anomaly",
            "severity": "warning",
            "description": (
                f"{ua_suspicious_ratio:.0%} of requests have empty or "
                f"suspicious User-Agents"
            ),
            "metrics": {
                "suspicious_ratio": round(ua_suspicious_ratio, 4),
                "suspicious_count": suspicious_ua,
                "total_requests": total_requests,
            },
            "recommended_action": (
                "Block requests with empty User-Agents; require valid UA header"
            ),
        })

    return alerts


def _severity(std_deviations: float, is_10x: bool) -> str:
    """Assign severity based on how far the metric deviates from baseline."""
    if is_10x or std_deviations >= 4:
        return "critical"
    if std_deviations >= 3:
        return "warning"
    return "info"
```

### Why This Works

- Each detection function is independent, so multiple anomalies can be detected in the same window. The severity escalation logic (3+ simultaneous anomalies escalating to critical) is handled by the alerter, not the detector.
- IP concentration uses the per-IP request count from the entire window, not per-minute. This catches sustained attacks from a single IP even if the per-minute rate is below the threshold.
- Path hammering calculates the ratio of the top path's requests to total requests. This catches both targeted attacks (one path gets 90% of traffic) and more subtle imbalances.
- User-Agent anomaly detection flags empty UAs, known bot signatures (`python-requests`, `curl`), and the literal string "bot".
- The `std_deviations` guard prevents division by zero when the baseline has zero standard deviation, falling back to a ratio of the mean instead.

### Common Mistakes

- **Only checking volume and ignoring behavioral signals.** A sophisticated attack from a botnet with valid User-Agents and distributed IPs will pass volume checks but fail on path hammering or error rate.
- **Using fixed thresholds instead of baseline-relative thresholds.** A threshold of 1000 RPS is meaningless without knowing what "normal" looks like.
- **Not handling the case where `std` is 0** (constant baseline). Division by zero crashes the detector.
- **Flagging suspicious User-Agents too aggressively.** Legitimate monitoring tools often use `curl` or `python-requests`. Use the 30% threshold to distinguish a small number of tools from an army of bots.

---

## Part D: Alert Manager (`alerter.py`)

```python
"""
alerter.py -- Manage alert state, deduplication, escalation, and output.
"""

import json
import time
from datetime import datetime, timezone
from typing import List, Dict, Any, Optional
from collections import defaultdict


class AlertManager:
    """
    Manages DDoS detection alerts with deduplication, escalation,
    and multiple output formats.
    """

    def __init__(
        self,
        cooldown_seconds: int = 300,
        escalation_seconds: int = 300,
        output_handlers: Optional[List[str]] = None,
        webhook_url: Optional[str] = None,
    ):
        """
        Args:
            cooldown_seconds: Minimum seconds between duplicate alerts
                              (default 5 min).
            escalation_seconds: Seconds before a warning is escalated
                                to critical.
            output_handlers: List of output types:
                             "console", "json_file", "webhook".
            webhook_url: URL for webhook POST output.
        """
        self.cooldown_seconds = cooldown_seconds
        self.escalation_seconds = escalation_seconds
        self.output_handlers = output_handlers or ["console"]
        self.webhook_url = webhook_url

        # State tracking
        self._last_alert_times: Dict[str, float] = {}
        self._warning_start_times: Dict[str, float] = {}
        self._alert_history: List[Dict[str, Any]] = []

    def process_alerts(self, alerts: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """
        Process a batch of alerts: deduplicate, escalate, and output.

        Args:
            alerts: List of alert dictionaries from the detector.

        Returns:
            List of alerts that were actually emitted (after deduplication).
        """
        emitted = []
        now = time.time()

        for alert in alerts:
            dtype = alert["detection_type"]

            # --- Deduplication ---
            if not self._should_alert(dtype, now):
                continue

            # --- Escalation ---
            alert = self._check_escalation(dtype, alert, now)

            # --- Record ---
            alert["emitted_at"] = datetime.now(timezone.utc).isoformat()
            self._alert_history.append(alert)
            self._last_alert_times[dtype] = now
            emitted.append(alert)

            # --- Output ---
            self._output(alert)

        return emitted

    def _should_alert(self, detection_type: str, now: float) -> bool:
        """Return True if cooldown has elapsed since the last alert of
        this type."""
        last = self._last_alert_times.get(detection_type, 0)
        return (now - last) >= self.cooldown_seconds

    def _check_escalation(
        self, detection_type: str, alert: Dict[str, Any], now: float
    ) -> Dict[str, Any]:
        """
        If a warning has persisted longer than escalation_seconds,
        escalate it to critical.
        """
        if alert["severity"] == "warning":
            if detection_type not in self._warning_start_times:
                self._warning_start_times[detection_type] = now
            elif (now - self._warning_start_times[detection_type]
                  >= self.escalation_seconds):
                alert["severity"] = "critical"
                alert["description"] = (
                    f"[ESCALATED] {alert['description']} "
                    f"(warning persisted >{self.escalation_seconds}s)"
                )
        else:
            # Reset warning timer if severity is not warning
            self._warning_start_times.pop(detection_type, None)

        return alert

    def _output(self, alert: Dict[str, Any]) -> None:
        """Dispatch the alert to all configured output handlers."""
        for handler in self.output_handlers:
            if handler == "console":
                self._output_console(alert)
            elif handler == "json_file":
                self._output_json_file(alert)
            elif handler == "webhook":
                self._output_webhook(alert)

    def _output_console(self, alert: Dict[str, Any]) -> None:
        """Print alert to stderr in a human-readable format."""
        severity_colors = {
            "critical": "\033[91m",
            "warning": "\033[93m",
            "info": "\033[94m",
        }
        reset = "\033[0m"
        color = severity_colors.get(alert["severity"], "")

        print(
            f"{color}[{alert['severity'].upper()}]{reset} "
            f"{alert['detection_type']}: {alert['description']}"
        )
        if alert.get("recommended_action"):
            print(f"  Action: {alert['recommended_action']}")

    def _output_json_file(self, alert: Dict[str, Any]) -> None:
        """Append alert as a JSON line to alerts.jsonl."""
        with open("alerts.jsonl", "a") as f:
            f.write(json.dumps(alert, default=str) + "\n")

    def _output_webhook(self, alert: Dict[str, Any]) -> None:
        """POST alert to webhook URL."""
        if not self.webhook_url:
            return
        try:
            import urllib.request

            data = json.dumps(alert, default=str).encode("utf-8")
            req = urllib.request.Request(
                self.webhook_url,
                data=data,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            urllib.request.urlopen(req, timeout=5)
        except Exception as exc:
            print(f"[WARN] Webhook delivery failed: {exc}")

    def get_history(self) -> List[Dict[str, Any]]:
        """Return the full alert history."""
        return list(self._alert_history)
```

### Why This Works

- Deduplication uses a per-`detection_type` timestamp. If the same anomaly type fires again within the cooldown window (default 5 minutes), the duplicate is silently dropped. This prevents alert fatigue during sustained attacks.
- Escalation tracks when a warning first appeared. If the same warning type persists for more than 5 minutes without being resolved, the alerter promotes it to critical. This models real-world escalation: a brief spike is a warning; a sustained attack needs critical attention.
- Multiple output handlers allow the same alert to go to the console (for the on-call engineer), a JSON file (for post-incident analysis), and a webhook (for PagerDuty/Slack integration).

### Common Mistakes

- **Not deduplicating alerts.** During a sustained attack, the detector fires every analysis window (e.g., every 10 seconds). Without per-type deduplication, the on-call engineer receives hundreds of identical alerts.
- **Using a global cooldown instead of per-type cooldown.** A volume spike alert should not suppress a concurrent IP concentration alert -- they are different anomaly types that may require different responses.
- **Not resetting the warning escalation timer** when the alert severity changes. If a warning resolves and then reoccurs, the escalation timer should restart.

---

## Part E: Main Analysis Pipeline (`analyze_traffic.py`)

```python
#!/usr/bin/env python3
"""
analyze_traffic.py -- Main DDoS traffic analysis pipeline.

Usage:
    python analyze_traffic.py --log /var/log/nginx/access.log
    python analyze_traffic.py --log access.log --baseline baseline.json
    python analyze_traffic.py --log access.log --watch --interval 10
    python analyze_traffic.py --generate-sample --output sample.log
"""

import argparse
import json
import sys
import time
import random
from datetime import datetime, timedelta, timezone
from pathlib import Path

from log_parser import parse_log_line, parse_log_file
from baseline import calculate_baselines
from detector import detect_anomalies
from alerter import AlertManager


def analyze_log_file(log_path: str, baseline_path: str = None) -> None:
    """
    Analyze a log file: calculate baselines from the first 80% of entries,
    then detect anomalies in the remaining 20%.
    """
    print(f"[*] Parsing log file: {log_path}")
    entries = parse_log_file(log_path)
    print(f"[*] Parsed {len(entries)} valid log entries")

    if len(entries) < 10:
        print("[!] Not enough entries to establish a baseline (need at least 10)")
        return

    # --- Baselines ---
    if baseline_path and Path(baseline_path).exists():
        print(f"[*] Loading pre-computed baseline from {baseline_path}")
        with open(baseline_path) as f:
            baselines = json.load(f)
    else:
        split_idx = int(len(entries) * 0.8)
        baseline_entries = entries[:split_idx]
        analysis_entries = entries[split_idx:]
        print(
            f"[*] Calculating baselines from {len(baseline_entries)} "
            f"entries (first 80%)"
        )
        baselines = calculate_baselines(baseline_entries)

        baseline_out = "baseline.json"
        with open(baseline_out, "w") as f:
            json.dump(baselines, f, indent=2, default=str)
        print(f"[*] Baseline saved to {baseline_out}")

    # --- Analysis ---
    if baseline_path:
        analysis_entries = entries

    print(f"[*] Analyzing {len(analysis_entries)} entries against baselines")
    alerts = detect_anomalies(analysis_entries, baselines)

    # --- Output ---
    alerter = AlertManager(cooldown_seconds=0)  # No cooldown for file analysis
    emitted = alerter.process_alerts(alerts)

    # --- Summary ---
    print("\n" + "=" * 60)
    print("ANALYSIS SUMMARY")
    print("=" * 60)
    print(f"Total entries analyzed: {len(analysis_entries)}")
    print(f"Anomalies detected:     {len(emitted)}")
    if emitted:
        severity_counts = {}
        for a in emitted:
            severity_counts[a["severity"]] = (
                severity_counts.get(a["severity"], 0) + 1
            )
        for sev, count in sorted(severity_counts.items()):
            print(f"  {sev}: {count}")
        print("\nDetected anomaly types:")
        for a in emitted:
            print(
                f"  [{a['severity'].upper()}] {a['detection_type']}: "
                f"{a['description']}"
            )
    else:
        print("No anomalies detected -- traffic appears normal.")
    print("=" * 60)


def watch_log_file(log_path: str, baseline_path: str, interval: int) -> None:
    """
    Continuously monitor a log file, analyzing new entries every
    interval seconds.
    """
    print(f"[*] Watching {log_path} every {interval}s (Ctrl+C to stop)")

    with open(baseline_path) as f:
        baselines = json.load(f)

    alerter = AlertManager(cooldown_seconds=300)
    last_position = 0

    try:
        while True:
            with open(log_path) as f:
                f.seek(last_position)
                new_lines = f.readlines()
                last_position = f.tell()

            new_entries = []
            for line in new_lines:
                entry = parse_log_line(line)
                if entry:
                    new_entries.append(entry)

            if new_entries:
                print(f"[*] Processing {len(new_entries)} new entries")
                alerts = detect_anomalies(new_entries, baselines)
                alerter.process_alerts(alerts)
            else:
                print("[*] No new entries")

            time.sleep(interval)
    except KeyboardInterrupt:
        print("\n[*] Stopped watching")


def generate_sample(output_path: str) -> None:
    """Generate synthetic log data with normal traffic and injected attacks."""
    print(f"[*] Generating sample log data to {output_path}")

    start = datetime(2024, 1, 15, 10, 0, 0, tzinfo=timezone.utc)
    lines = []
    paths = [
        "/api/data", "/api/auth/login", "/api/search",
        "/health", "/api/products",
    ]
    user_agents = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Safari/605.1",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0) Mobile/15E148",
    ]

    def make_line(ip, ts, method, path, status, ua, rt):
        return (
            f'{ip} - - [{ts.strftime("%d/%b/%Y:%H:%M:%S +0000")}] '
            f'"{method} {path} HTTP/1.1" {status} '
            f'{random.randint(200, 5000)} "-" "{ua}" rt={rt:.3f}'
        )

    current = start
    end = start + timedelta(minutes=30)

    # Phase 1: Normal traffic (first 20 minutes)
    normal_end = start + timedelta(minutes=20)
    while current < normal_end:
        for _ in range(random.randint(80, 120)):
            ip = (
                f"192.168.{random.randint(1, 254)}."
                f"{random.randint(1, 254)}"
            )
            path = random.choice(paths)
            ua = random.choice(user_agents)
            rt = max(0.01, random.gauss(0.15, 0.05))
            lines.append(
                make_line(ip, current, "GET", path, 200, ua, rt)
            )
        current += timedelta(seconds=1)

    # Phase 2: Attack traffic (last 10 minutes)
    attack_ips = [
        f"10.0.{random.randint(1, 10)}.{random.randint(1, 254)}"
        for _ in range(20)
    ]
    while current < end:
        # Normal traffic continues
        for _ in range(random.randint(80, 120)):
            ip = (
                f"192.168.{random.randint(1, 254)}."
                f"{random.randint(1, 254)}"
            )
            path = random.choice(paths)
            ua = random.choice(user_agents)
            rt = max(0.01, random.gauss(0.15, 0.05))
            lines.append(
                make_line(ip, current, "GET", path, 200, ua, rt)
            )

        # Attack traffic: concentrated IPs, /api/search, high errors
        for _ in range(random.randint(300, 500)):
            ip = random.choice(attack_ips)
            ua = random.choice(["python-requests/2.31", "", "curl/7.88"])
            rt = max(0.01, random.gauss(1.5, 0.3))
            status = random.choice([200, 200, 200, 500, 500])
            lines.append(
                make_line(ip, current, "GET", "/api/search", status, ua, rt)
            )

        current += timedelta(seconds=1)

    random.shuffle(lines)

    with open(output_path, "w") as f:
        f.write("\n".join(lines) + "\n")

    total_seconds = (end - start).total_seconds()
    print(f"[*] Generated {len(lines)} log lines ({total_seconds:.0f}s)")
    print("[*] Normal traffic: first 20 minutes")
    print(
        "[*] Attack traffic: last 10 minutes "
        "(concentrated IPs, /api/search flood, high error rate)"
    )


def main():
    parser = argparse.ArgumentParser(description="DDoS Traffic Analyzer")
    parser.add_argument("--log", help="Path to Nginx access log file")
    parser.add_argument("--baseline", help="Path to pre-computed baseline JSON")
    parser.add_argument(
        "--watch", action="store_true",
        help="Continuously monitor the log file",
    )
    parser.add_argument(
        "--interval", type=int, default=10,
        help="Watch interval in seconds",
    )
    parser.add_argument(
        "--generate-sample", action="store_true",
        help="Generate sample log data",
    )
    parser.add_argument(
        "--output", default="sample.log",
        help="Output path for generated sample",
    )

    args = parser.parse_args()

    if args.generate_sample:
        generate_sample(args.output)
    elif args.log:
        if args.watch:
            if not args.baseline:
                print("[!] --watch requires --baseline")
                sys.exit(1)
            watch_log_file(args.log, args.baseline, args.interval)
        else:
            analyze_log_file(args.log, args.baseline)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
```

### Why This Works

- The 80/20 split ensures the baseline is calculated from a representative sample of normal traffic, while the remaining 20% is analyzed against that baseline. This simulates the real-world scenario where you have historical data and want to detect current anomalies.
- The `--watch` mode uses `file.seek` to read only new lines since the last read, avoiding re-processing the entire file on each iteration. This is efficient for large log files.
- The sample data generator produces two distinct phases: 20 minutes of normal traffic (random IPs, normal paths, low request times) followed by 10 minutes of attack traffic (concentrated IPs, target path flooding, high error rates, suspicious User-Agents). This guarantees the detector will find anomalies.
- The `AlertManager` is configured with `cooldown_seconds=0` for file analysis (show all anomalies) but `cooldown_seconds=300` for watch mode (deduplicate in real-time).

### Common Mistakes

- **Not splitting baseline and analysis data.** If you calculate the baseline from all data including the attack, the attack traffic inflates the baseline and makes detection harder.
- **Using `tail -f` instead of `seek`** for watch mode. `tail -f` is a shell concept; in Python, `seek` with a tracked position is the correct and portable approach.
- **Generating sample data where the attack is too subtle.** The attack traffic should be obviously anomalous (e.g., 5x the normal RPS from 20 IPs) to verify the detector works. In production, you tune thresholds to catch subtler attacks.

---

## Common Mistakes (Module-Level)

- **Not establishing a baseline before detecting anomalies.** Without a baseline, every threshold is arbitrary. A system that receives 1000 RPS at noon and 100 RPS at 3 AM needs time-of-day aware baselines.
- **Only monitoring volume.** Sophisticated DDoS attacks distribute traffic across many IPs and paths to stay below volume thresholds. IP concentration, path hammering, and User-Agent anomalies are critical secondary signals.
- **Alert fatigue from missing deduplication.** During a sustained attack, the detector fires every analysis window. Without per-type deduplication, the on-call engineer receives hundreds of identical alerts and starts ignoring them.
- **Not generating test data.** If you cannot reproduce an attack in a test environment, you cannot verify your detector works. Always build a sample data generator alongside the detector.
