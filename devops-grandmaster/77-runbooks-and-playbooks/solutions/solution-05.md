# Solution 05: Runbook Repository and CI/CD

## Problem Statement

Design a runbook repository structure, build a GitHub Actions validation
pipeline, create a FastAPI service that links alerts to runbooks, and define
a runbook index YAML schema.

## Complete Solution

### Part 1: Repository Structure

```
runbooks/
├── .github/
│   └── workflows/
│       ├── validate-runbooks.yml      # CI: lint and validate on PR
│       └── publish-index.yml          # CD: rebuild index on merge
├── .vale.ini                          # Prose linter config
├── .vale/
│   └── styles/
│       └── Runbooks/
│           ├── HeadingCase.yml        # Custom linting rule
│           └── NoTBD.yml              # Flag TBD/TODO placeholders
├── scripts/
│   ├── validate_runbooks.py           # Schema + link validation
│   ├── build_index.py                 # Generate runbook-index.yaml
│   └── check_links.py                 # Verify cross-references
├── templates/
│   └── runbook-template.md            # Standard runbook template
├── index.yaml                         # Auto-generated runbook index
├── README.md
│
├── infrastructure/
│   ├── server/
│   │   ├── rb-server-001-disk-full.md
│   │   ├── rb-server-002-high-cpu.md
│   │   └── rb-server-003-high-memory.md
│   ├── network/
│   │   ├── rb-net-001-dns-failure.md
│   │   └── rb-net-002-load-balancer-down.md
│   └── storage/
│       └── rb-storage-001-nfs-unreachable.md
│
├── kubernetes/
│   ├── pods/
│   │   ├── rb-pod-001-crashloop.md
│   │   ├── rb-pod-002-imagepullbackoff.md
│   │   └── rb-pod-003-pending.md
│   ├── deployments/
│   │   └── rb-deploy-001-rollback.md
│   └── cluster/
│       ├── rb-cluster-001-node-not-ready.md
│       └── rb-cluster-002-etcd-down.md
│
├── database/
│   ├── postgres/
│   │   ├── rb-pg-001-failover.md
│   │   ├── rb-pg-002-replication-lag.md
│   │   └── rb-pg-003-connection-exhaustion.md
│   ├── redis/
│   │   └── rb-redis-001-high-memory.md
│   └── mongodb/
│       └── rb-mongo-001-replicaset-degraded.md
│
├── security/
│   ├── rb-sec-001-certificate-expiry.md
│   ├── rb-sec-002-unauthorized-access.md
│   └── rb-sec-003-ransomware-response.md
│
└── ci-cd/
    ├── rb-cicd-001-pipeline-failure.md
    └── rb-cicd-002-deploy-rollback.md
```

**Naming convention**: `rb-<domain>-<NNN>-<short-description>.md`

This provides:
- Alphabetical grouping by domain (`rb-db`, `rb-k8s`, `rb-server`)
- Numeric ordering within a domain (001, 002, 003)
- Human-readable description for quick identification

---

### Part 2: Runbook Template

```markdown
<!-- templates/runbook-template.md -->

# RUNBOOK: [Title]

<!-- REQUIRED FRONTMATTER - used by validation pipeline -->
<!-- id: rb-domain-NNN -->
<!-- severity: P1 | P2 | P3 | P4 -->
<!-- owner: team-name -->
<!-- tags: comma, separated, tags -->
<!-- created: YYYY-MM-DD -->
<!-- updated: YYYY-MM-DD -->
<!-- review_cycle: monthly | quarterly | biannual -->
<!-- alert_rules: [list of Alertmanager rule names that trigger this runbook] -->

---

## 1. Overview

<!-- 2-3 sentences: What does this runbook cover? When is it triggered? -->

## 2. Symptoms

<!-- How does the operator know this runbook applies?
     Include specific alerts, dashboard screenshots, log patterns. -->

## 3. Impact

<!-- What is the user/business impact? Which services are affected? -->

## 4. Prerequisites

<!-- What access, tools, and knowledge are needed before starting? -->

- [ ] Access requirement 1
- [ ] Access requirement 2

## 5. Diagnosis

<!-- Step-by-step diagnostic procedure. Include exact commands. -->

### Step 1: [Diagnostic action]

```bash
command here
```

**Expected output:** [what success looks like]

**If you see [X]:** Go to Step [N]

### Step 2: ...

## 6. Resolution

<!-- Multiple options if applicable. Clearly mark which is preferred. -->

### Option A: [Preferred resolution]

### Option B: [Alternative]

### Option C: [Temporary mitigation]

## 7. Verification

<!-- How to confirm the issue is resolved. Must be specific and testable. -->

- [ ] Verification criterion 1
- [ ] Verification criterion 2

## 8. Post-Resolution

<!-- Immediate and follow-up actions to prevent recurrence. -->

## 9. Escalation

<!-- When and who to escalate to if the runbook does not resolve the issue. -->

| Time Elapsed | Action |
|-------------|--------|
| 15 min | ... |

## 10. Related Runbooks

<!-- Cross-references to related procedures. -->

- [RB-XXX-NNN: Title](path/to/runbook.md)
```

---

### Part 3: GitHub Actions Validation Pipeline

```yaml
# .github/workflows/validate-runbooks.yml

name: Validate Runbooks

on:
  pull_request:
    paths:
      - "**/*.md"
      - "index.yaml"
      - "scripts/**"
      - "templates/**"
  push:
    branches: [main]
    paths:
      - "**/*.md"

jobs:
  validate:
    name: Lint and Validate Runbooks
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - name: Install dependencies
        run: |
          pip install pyyaml jsonschema vale pygments

      - name: Validate runbook schema
        run: python scripts/validate_runbooks.py
        # Checks:
        # - Every .md file in the repo has the required frontmatter fields
        # - Severity is one of P1, P2, P3, P4
        # - review_cycle is one of monthly, quarterly, biannual
        # - Owner is a valid team name
        # - Tags are non-empty
        # - File follows the naming convention (rb-<domain>-<NNN>-<desc>.md)

      - name: Validate cross-references
        run: python scripts/check_links.py
        # Checks:
        # - Every linked runbook in "Related Runbooks" section exists
        # - Every runbook is referenced by at least one other runbook (no orphans)
        # - Internal markdown links are valid (anchors, relative paths)

      - name: Run prose linter (Vale)
        uses: errata-ai/vale-action@v2
        with:
          files: "**/*.md"
          vale_flags: "--config=.vale.ini"
        # Checks:
        # - No "TBD" or "TODO" placeholders in merged runbooks
        # - Consistent heading capitalization
        # - No passive voice in resolution steps
        # - Technical terms are consistently formatted

      - name: Check for required sections
        run: |
          # Verify every runbook contains all required sections
          REQUIRED_SECTIONS=(
            "Overview"
            "Symptoms"
            "Impact"
            "Prerequisites"
            "Diagnosis"
            "Resolution"
            "Verification"
            "Post-Resolution"
          )
          FAILURES=0
          for f in $(find . -name "rb-*.md" -not -path "./templates/*"); do
            for section in "${REQUIRED_SECTIONS[@]}"; do
              if ! grep -q "^## .*${section}" "$f"; then
                echo "::error file=${f}::Missing required section: ${section}"
                FAILURES=$((FAILURES + 1))
              fi
            done
          done
          if [ $FAILURES -gt 0 ]; then
            echo "::error::${FAILURES} validation error(s) found"
            exit 1
          fi
          echo "All runbooks have required sections."

      - name: Validate index.yaml
        run: |
          python -c "
          import yaml, sys
          with open('index.yaml') as f:
              index = yaml.safe_load(f)
          # Verify index structure
          assert 'runbooks' in index, 'Missing top-level runbooks key'
          for rb in index['runbooks']:
              for field in ['id', 'title', 'severity', 'owner', 'path', 'tags']:
                  assert field in rb, f'Runbook {rb.get(\"id\", \"?\")} missing {field}'
          print(f'Index validated: {len(index[\"runbooks\"])} runbooks')
          "

  build-index:
    name: Rebuild Runbook Index
    runs-on: ubuntu-latest
    needs: validate
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - name: Install dependencies
        run: pip install pyyaml

      - name: Build index
        run: python scripts/build_index.py

      - name: Commit updated index
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add index.yaml
          git diff --cached --quiet || git commit -m "chore: rebuild runbook index [skip ci]"
          git push
```

---

### Part 4: Validation Script

```python
# scripts/validate_runbooks.py

"""
Validates all runbook markdown files against the required schema.
Exits with code 1 if any validation errors are found.
"""

import os
import re
import sys
from pathlib import Path

REQUIRED_FRONTMATTER = {
    "id": re.compile(r"^rb-\w+-\d{3,}$"),
    "severity": {"P1", "P2", "P3", "P4"},
    "owner": None,          # Any non-empty string
    "tags": None,           # Any non-empty string
    "created": re.compile(r"^\d{4}-\d{2}-\d{2}$"),
    "updated": re.compile(r"^\d{4}-\d{2}-\d{2}$"),
    "review_cycle": {"monthly", "quarterly", "biannual"},
}

REQUIRED_SECTIONS = [
    "Overview", "Symptoms", "Impact", "Prerequisites",
    "Diagnosis", "Resolution", "Verification", "Post-Resolution",
]

FILE_PATTERN = re.compile(r"^rb-\w+-\d{3,}-.+\.md$")


def extract_frontmatter(content: str) -> dict[str, str]:
    """Extract HTML comment frontmatter from a markdown file."""
    frontmatter = {}
    for match in re.finditer(r"<!--\s*(\w+):\s*(.+?)-->", content):
        key, value = match.group(1).strip(), match.group(2).strip()
        frontmatter[key] = value
    return frontmatter


def extract_sections(content: str) -> list[str]:
    """Extract H2 section headings from a markdown file."""
    return re.findall(r"^##\s+\d+\.\s+(.+)$", content, re.MULTILINE)


def validate_runbook(filepath: Path) -> list[str]:
    """Validate a single runbook file. Returns list of errors."""
    errors = []
    rel_path = filepath.relative_to(Path("."))

    # Check filename convention
    if not FILE_PATTERN.match(filepath.name):
        errors.append(f"{rel_path}: Filename does not match pattern rb-<domain>-<NNN>-<desc>.md")

    content = filepath.read_text(encoding="utf-8")
    frontmatter = extract_frontmatter(content)
    sections = extract_sections(content)

    # Validate frontmatter
    for field, constraint in REQUIRED_FRONTMATTER.items():
        if field not in frontmatter:
            errors.append(f"{rel_path}: Missing frontmatter field '{field}'")
            continue
        value = frontmatter[field]
        if isinstance(constraint, set) and value not in constraint:
            errors.append(f"{rel_path}: {field}='{value}' not in {constraint}")
        elif isinstance(constraint, re.Pattern) and not constraint.match(value):
            errors.append(f"{rel_path}: {field}='{value}' does not match pattern {constraint.pattern}")
        elif constraint is None and not value:
            errors.append(f"{rel_path}: {field} is empty")

    # Validate sections
    for section in REQUIRED_SECTIONS:
        if not any(section in s for s in sections):
            errors.append(f"{rel_path}: Missing required section '{section}'")

    # Check for TODO/TBD placeholders (excluding template directory)
    if "templates" not in str(rel_path):
        for i, line in enumerate(content.splitlines(), 1):
            if re.search(r"\b(TBD|TODO|FIXME|PLACEHOLDER)\b", line, re.IGNORECASE):
                errors.append(f"{rel_path}:{i}: Contains placeholder text: {line.strip()}")

    return errors


def main():
    root = Path(".")
    runbook_files = [
        p for p in root.rglob("rb-*.md")
        if "templates" not in str(p)
    ]

    if not runbook_files:
        print("WARNING: No runbook files found.")
        sys.exit(0)

    all_errors = []
    for filepath in sorted(runbook_files):
        errors = validate_runbook(filepath)
        all_errors.extend(errors)

    if all_errors:
        print(f"VALIDATION FAILED: {len(all_errors)} error(s)\n")
        for error in all_errors:
            print(f"  ERROR: {error}")
        sys.exit(1)
    else:
        print(f"VALIDATION PASSED: {len(runbook_files)} runbook(s) validated successfully.")
        sys.exit(0)


if __name__ == "__main__":
    main()
```

---

### Part 5: Build Index Script

```python
# scripts/build_index.py

"""
Scans all runbook files and generates index.yaml.
Run this as part of CI/CD to keep the index in sync with the repository.
"""

import re
from pathlib import Path
from datetime import datetime, timezone

import yaml


def extract_frontmatter(content: str) -> dict[str, str]:
    frontmatter = {}
    for match in re.finditer(r"<!--\s*(\w+):\s*(.+?)-->", content):
        key, value = match.group(1).strip(), match.group(2).strip()
        frontmatter[key] = value
    return frontmatter


def extract_title(content: str) -> str:
    match = re.search(r"^#\s+RUNBOOK:\s+(.+)$", content, re.MULTILINE)
    return match.group(1).strip() if match else "Untitled"


def build_index(root: Path) -> dict:
    runbooks = []
    for filepath in sorted(root.rglob("rb-*.md")):
        if "templates" in str(filepath):
            continue

        content = filepath.read_text(encoding="utf-8")
        fm = extract_frontmatter(content)
        rel_path = str(filepath.relative_to(root))

        tags_raw = fm.get("tags", "")
        tags = [t.strip() for t in tags_raw.split(",") if t.strip()]

        runbooks.append({
            "id": fm.get("id", filepath.stem),
            "title": extract_title(content),
            "severity": fm.get("severity", "P3"),
            "owner": fm.get("owner", "unknown"),
            "path": rel_path,
            "tags": tags,
            "created": fm.get("created", ""),
            "updated": fm.get("updated", ""),
            "review_cycle": fm.get("review_cycle", "quarterly"),
            "alert_rules": [
                r.strip() for r in fm.get("alert_rules", "").split(",") if r.strip()
            ],
        })

    # Sort by severity (P1 first), then by ID
    severity_order = {"P1": 0, "P2": 1, "P3": 2, "P4": 3}
    runbooks.sort(key=lambda rb: (severity_order.get(rb["severity"], 9), rb["id"]))

    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "total_count": len(runbooks),
        "by_severity": {
            sev: len([rb for rb in runbooks if rb["severity"] == sev])
            for sev in ["P1", "P2", "P3", "P4"]
        },
        "runbooks": runbooks,
    }


def main():
    root = Path(".")
    index = build_index(root)

    output_path = root / "index.yaml"
    with open(output_path, "w") as f:
        yaml.dump(index, f, default_flow_style=False, sort_keys=False, allow_unicode=True)

    print(f"Generated {output_path}: {index['total_count']} runbooks indexed")
    for sev, count in index["by_severity"].items():
        print(f"  {sev}: {count}")


if __name__ == "__main__":
    main()
```

---

### Part 6: Generated Index YAML

```yaml
# index.yaml (auto-generated by scripts/build_index.py)

generated_at: "2026-06-11T14:30:00+00:00"
total_count: 18
by_severity:
  P1: 4
  P2: 8
  P3: 5
  P4: 1

runbooks:
  - id: rb-db-001
    title: "PostgreSQL Database Failover"
    severity: P1
    owner: dbre
    path: database/postgres/rb-pg-001-failover.md
    tags: [postgresql, patroni, failover, ha]
    created: "2026-01-15"
    updated: "2026-06-11"
    review_cycle: monthly
    alert_rules:
      - PostgreSQLDown
      - PatroniFailoverTriggered
      - PostgreSQLReplicationLagHigh

  - id: rb-sec-003
    title: "Ransomware Response"
    severity: P1
    owner: security
    path: security/rb-sec-003-ransomware-response.md
    tags: [security, ransomware, incident-response]
    created: "2026-03-01"
    updated: "2026-05-20"
    review_cycle: monthly
    alert_rules:
      - RansomwareDetected
      - AnomalousFileEncryption

  - id: rb-cluster-002
    title: "etcd Cluster Down"
    severity: P1
    owner: platform-eng
    path: kubernetes/cluster/rb-cluster-002-etcd-down.md
    tags: [kubernetes, etcd, consensus, ha]
    created: "2026-02-10"
    updated: "2026-06-01"
    review_cycle: monthly
    alert_rules:
      - EtcdClusterUnavailable
      - EtcdLeaderChanges

  - id: rb-net-002
    title: "Load Balancer Down"
    severity: P1
    owner: network-eng
    path: infrastructure/network/rb-net-002-load-balancer-down.md
    tags: [network, load-balancer, ha, traffic]
    created: "2026-01-20"
    updated: "2026-04-15"
    review_cycle: monthly
    alert_rules:
      - LoadBalancerHealthCheckFailed
      - LoadBalancerZeroBackends

  - id: rb-pod-001
    title: "Pod CrashLoopBackOff"
    severity: P2
    owner: platform-eng
    path: kubernetes/pods/rb-pod-001-crashloop.md
    tags: [kubernetes, pod, crashloop, restart]
    created: "2026-01-10"
    updated: "2026-06-11"
    review_cycle: quarterly
    alert_rules:
      - KubePodCrashLooping

  - id: rb-server-001
    title: "Disk Full"
    severity: P2
    owner: sre
    path: infrastructure/server/rb-server-001-disk-full.md
    tags: [server, disk, storage, cleanup]
    created: "2026-01-05"
    updated: "2026-05-30"
    review_cycle: quarterly
    alert_rules:
      - NodeFilesystemAlmostOutOfSpace
      - NodeFilesystemAlmostOutOfInodes

  - id: rb-server-002
    title: "High CPU Usage"
    severity: P2
    owner: sre
    path: infrastructure/server/rb-server-002-high-cpu.md
    tags: [server, cpu, performance, investigation]
    created: "2026-01-05"
    updated: "2026-05-15"
    review_cycle: quarterly
    alert_rules:
      - NodeHighCPUUsage
      - ProcessHighCPUUsage

  - id: rb-server-003
    title: "High Memory Usage"
    severity: P2
    owner: sre
    path: infrastructure/server/rb-server-003-high-memory.md
    tags: [server, memory, oom, performance]
    created: "2026-02-01"
    updated: "2026-05-15"
    review_cycle: quarterly
    alert_rules:
      - NodeHighMemoryUsage
      - ProcessHighMemoryUsage

  - id: rb-db-002
    title: "PostgreSQL Replication Lag"
    severity: P2
    owner: dbre
    path: database/postgres/rb-pg-002-replication-lag.md
    tags: [postgresql, replication, lag, performance]
    created: "2026-02-15"
    updated: "2026-05-01"
    review_cycle: quarterly
    alert_rules:
      - PostgreSQLReplicationLagHigh

  - id: rb-db-003
    title: "PostgreSQL Connection Exhaustion"
    severity: P2
    owner: dbre
    path: database/postgres/rb-pg-003-connection-exhaustion.md
    tags: [postgresql, connections, pgbouncer, pool]
    created: "2026-03-10"
    updated: "2026-06-01"
    review_cycle: quarterly
    alert_rules:
      - PostgreSQLConnectionsHigh
      - PgBouncerWaitingClients

  - id: rb-sec-001
    title: "SSL Certificate Expiry"
    severity: P2
    owner: security
    path: security/rb-sec-001-certificate-expiry.md
    tags: [security, ssl, tls, certificate, expiry]
    created: "2026-01-20"
    updated: "2026-04-10"
    review_cycle: quarterly
    alert_rules:
      - SSLCertificateExpiringSoon
      - SSLCertificateExpired

  - id: rb-pod-002
    title: "Pod ImagePullBackOff"
    severity: P2
    owner: platform-eng
    path: kubernetes/pods/rb-pod-002-imagepullbackoff.md
    tags: [kubernetes, pod, image, registry]
    created: "2026-02-01"
    updated: "2026-05-20"
    review_cycle: quarterly
    alert_rules:
      - KubePodImagePullBackOff

  - id: rb-cluster-001
    title: "Node Not Ready"
    severity: P3
    owner: platform-eng
    path: kubernetes/cluster/rb-cluster-001-node-not-ready.md
    tags: [kubernetes, node, kubelet, not-ready]
    created: "2026-01-15"
    updated: "2026-04-20"
    review_cycle: quarterly
    alert_rules:
      - KubeNodeNotReady
      - KubeNodeUnreachable

  - id: rb-pod-003
    title: "Pod Pending"
    severity: P3
    owner: platform-eng
    path: kubernetes/pods/rb-pod-003-pending.md
    tags: [kubernetes, pod, pending, scheduling]
    created: "2026-02-15"
    updated: "2026-05-10"
    review_cycle: quarterly
    alert_rules:
      - KubePodPendingLong

  - id: rb-net-001
    title: "DNS Failure"
    severity: P3
    owner: network-eng
    path: infrastructure/network/rb-net-001-dns-failure.md
    tags: [network, dns, resolution, core-dns]
    created: "2026-01-10"
    updated: "2026-04-05"
    review_cycle: quarterly
    alert_rules:
      - CoreDNSForwardFailure
      - DNSResolutionHighLatency

  - id: rb-cicd-001
    title: "CI/CD Pipeline Failure"
    severity: P3
    owner: devops
    path: ci-cd/rb-cicd-001-pipeline-failure.md
    tags: [cicd, pipeline, build, deployment]
    created: "2026-03-01"
    updated: "2026-06-01"
    review_cycle: quarterly
    alert_rules:
      - CIPipelineFailureRateHigh

  - id: rb-redis-001
    title: "Redis High Memory"
    severity: P3
    owner: platform-eng
    path: database/redis/rb-redis-001-high-memory.md
    tags: [redis, memory, cache, eviction]
    created: "2026-03-15"
    updated: "2026-05-25"
    review_cycle: quarterly
    alert_rules:
      - RedisMemoryHigh
      - RedisEvictionRate

  - id: rb-cicd-002
    title: "Deployment Rollback"
    severity: P4
    owner: devops
    path: ci-cd/rb-cicd-002-deploy-rollback.md
    tags: [cicd, deployment, rollback, kubernetes]
    created: "2026-04-01"
    updated: "2026-06-01"
    review_cycle: biannual
    alert_rules: []
```

---

### Part 7: FastAPI Alert-to-Runbook Linking Service

```python
# services/alert_router/main.py

"""
FastAPI service that maps incoming alerts to relevant runbooks.

Deployment:
    uvicorn services.alert_router.main:app --host 0.0.0.0 --port 8080

Usage:
    # Alertmanager webhook receiver
    POST /api/v1/alerts
    {
      "alerts": [{
        "labels": {"alertname": "KubePodCrashLooping", "namespace": "production"},
        "status": "firing"
      }]
    }

    # Response
    {
      "matched_runbooks": [
        {
          "id": "rb-pod-001",
          "title": "Pod CrashLoopBackOff",
          "path": "kubernetes/pods/rb-pod-001-crashloop.md",
          "severity": "P2",
          "relevance": "exact_match"
        }
      ]
    }
"""

import os
import re
from pathlib import Path
from typing import Optional

import yaml
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

# ---------------------------------------------------------------------------
# Models
# ---------------------------------------------------------------------------

class AlertLabels(BaseModel):
    alertname: str
    severity: Optional[str] = None
    namespace: Optional[str] = None
    pod: Optional[str] = None
    node: Optional[str] = None
    service: Optional[str] = None
    job: Optional[str] = None

    class Config:
        extra = "allow"   # Allow arbitrary additional labels


class Alert(BaseModel):
    labels: AlertLabels
    annotations: Optional[dict[str, str]] = None
    status: str = "firing"
    startsAt: Optional[str] = None
    endsAt: Optional[str] = None


class AlertmanagerWebhook(BaseModel):
    alerts: list[Alert]
    receiver: Optional[str] = None
    status: Optional[str] = None
    externalURL: Optional[str] = None


class MatchedRunbook(BaseModel):
    id: str
    title: str
    path: str
    severity: str
    owner: str
    tags: list[str]
    relevance: str  # "exact_match" | "tag_match" | "keyword_match"
    match_reason: str


class AlertResponse(BaseModel):
    alert_name: str
    status: str
    matched_runbooks: list[MatchedRunbook]
    message: str


# ---------------------------------------------------------------------------
# Runbook Index
# ---------------------------------------------------------------------------

class RunbookIndex:
    """
    Loads and queries the runbook index.

    Matching strategy:
    1. Exact match: alertname matches alert_rules in frontmatter
    2. Tag match:   alertname tokens match runbook tags
    3. Keyword match: alertname tokens appear in runbook title or path
    """

    def __init__(self, index_path: str = "index.yaml"):
        self.index_path = index_path
        self.runbooks: list[dict] = []
        self._alert_rule_map: dict[str, list[dict]] = {}  # alertname -> runbooks
        self._load_index()

    def _load_index(self):
        """Load index.yaml and build lookup structures."""
        path = Path(self.index_path)
        if not path.exists():
            raise FileNotFoundError(f"Runbook index not found: {self.index_path}")

        with open(path) as f:
            index_data = yaml.safe_load(f)

        self.runbooks = index_data.get("runbooks", [])

        # Build alert rule -> runbook mapping
        for rb in self.runbooks:
            for rule in rb.get("alert_rules", []):
                rule_lower = rule.lower()
                if rule_lower not in self._alert_rule_map:
                    self._alert_rule_map[rule_lower] = []
                self._alert_rule_map[rule_lower].append(rb)

    def reload(self):
        """Reload the index from disk (for hot-reloading)."""
        self._alert_rule_map.clear()
        self._load_index()

    def match_alert(self, alert: Alert) -> list[MatchedRunbook]:
        """Find runbooks matching the given alert."""
        matched: list[MatchedRunbook] = []
        seen_ids: set[str] = set()
        alert_name = alert.labels.alertname

        # Strategy 1: Exact match on alert_rules
        for rb in self._alert_rule_map.get(alert_name.lower(), []):
            if rb["id"] not in seen_ids:
                matched.append(MatchedRunbook(
                    id=rb["id"],
                    title=rb["title"],
                    path=rb["path"],
                    severity=rb["severity"],
                    owner=rb["owner"],
                    tags=rb.get("tags", []),
                    relevance="exact_match",
                    match_reason=f"Alert '{alert_name}' is listed in runbook alert_rules",
                ))
                seen_ids.add(rb["id"])

        # Strategy 2: Tag match
        alert_tokens = set(re.split(r"[-_\s]+", alert_name.lower()))
        for rb in self.runbooks:
            if rb["id"] in seen_ids:
                continue
            rb_tags = set(t.lower() for t in rb.get("tags", []))
            common = alert_tokens & rb_tags
            if common:
                matched.append(MatchedRunbook(
                    id=rb["id"],
                    title=rb["title"],
                    path=rb["path"],
                    severity=rb["severity"],
                    owner=rb["owner"],
                    tags=rb.get("tags", []),
                    relevance="tag_match",
                    match_reason=f"Shared tags: {', '.join(sorted(common))}",
                ))
                seen_ids.add(rb["id"])

        # Strategy 3: Keyword match in title
        for rb in self.runbooks:
            if rb["id"] in seen_ids:
                continue
            title_lower = rb["title"].lower()
            if any(token in title_lower for token in alert_tokens if len(token) > 3):
                matched.append(MatchedRunbook(
                    id=rb["id"],
                    title=rb["title"],
                    path=rb["path"],
                    severity=rb["severity"],
                    owner=rb["owner"],
                    tags=rb.get("tags", []),
                    relevance="keyword_match",
                    match_reason=f"Keyword match in title: '{rb['title']}'",
                ))
                seen_ids.add(rb["id"])

        # Sort: exact matches first, then by severity
        relevance_order = {"exact_match": 0, "tag_match": 1, "keyword_match": 2}
        severity_order = {"P1": 0, "P2": 1, "P3": 2, "P4": 3}
        matched.sort(key=lambda m: (
            relevance_order.get(m.relevance, 9),
            severity_order.get(m.severity, 9),
        ))

        return matched


# ---------------------------------------------------------------------------
# FastAPI Application
# ---------------------------------------------------------------------------

app = FastAPI(
    title="Runbook Alert Router",
    description="Maps incoming alerts to relevant runbooks",
    version="1.0.0",
)

# Load index at startup
INDEX_PATH = os.environ.get("RUNBOOK_INDEX_PATH", "index.yaml")
index: Optional[RunbookIndex] = None


@app.on_event("startup")
async def startup():
    global index
    index = RunbookIndex(index_path=INDEX_PATH)


@app.get("/health")
async def health():
    return {
        "status": "healthy",
        "runbooks_loaded": len(index.runbooks) if index else 0,
    }


@app.get("/api/v1/runbooks")
async def list_runbooks(
    severity: Optional[str] = None,
    owner: Optional[str] = None,
    tag: Optional[str] = None,
):
    """List all runbooks with optional filters."""
    if not index:
        raise HTTPException(status_code=503, detail="Index not loaded")

    results = index.runbooks
    if severity:
        results = [rb for rb in results if rb["severity"] == severity.upper()]
    if owner:
        results = [rb for rb in results if rb["owner"] == owner.lower()]
    if tag:
        results = [rb for rb in results if tag.lower() in [t.lower() for t in rb.get("tags", [])]]

    return {"count": len(results), "runbooks": results}


@app.get("/api/v1/runbooks/{runbook_id}")
async def get_runbook(runbook_id: str):
    """Get details for a specific runbook."""
    if not index:
        raise HTTPException(status_code=503, detail="Index not loaded")

    for rb in index.runbooks:
        if rb["id"] == runbook_id:
            return rb

    raise HTTPException(status_code=404, detail=f"Runbook {runbook_id} not found")


@app.post("/api/v1/alerts", response_model=list[AlertResponse])
async def receive_alerts(webhook: AlertmanagerWebhook):
    """
    Receive alerts from Alertmanager and return matched runbooks.

    This endpoint is designed to be used as an Alertmanager webhook receiver
    or called by a custom alerting pipeline.
    """
    if not index:
        raise HTTPException(status_code=503, detail="Index not loaded")

    responses = []
    for alert in webhook.alerts:
        if alert.status != "firing":
            continue

        matched = index.match_alert(alert)

        message = (
            f"Found {len(matched)} runbook(s) for alert '{alert.labels.alertname}'"
            if matched
            else f"No runbooks found for alert '{alert.labels.alertname}'. "
                 f"Consider creating one."
        )

        responses.append(AlertResponse(
            alert_name=alert.labels.alertname,
            status=alert.status,
            matched_runbooks=matched,
            message=message,
        ))

    return responses


@app.post("/api/v1/reload")
async def reload_index():
    """Hot-reload the runbook index from disk."""
    if not index:
        raise HTTPException(status_code=503, detail="Index not loaded")

    index.reload()
    return {
        "status": "reloaded",
        "runbooks_loaded": len(index.runbooks),
    }
```

---

### Part 8: Integration Architecture

```
┌──────────────┐     webhook     ┌─────────────────────┐     query      ┌──────────────┐
│ Alertmanager │ ──────────────> │  FastAPI Alert      │ ─────────────> │ index.yaml   │
│              │                 │  Router Service     │                │ (runbook     │
│  firing:     │                 │                     │                │  registry)   │
│  - alertname │                 │  POST /api/v1/alerts│                │              │
│  - severity  │                 │                     │                │              │
│  - labels    │                 │  Returns:           │                │              │
│              │                 │  - matched runbooks │                │              │
│              │                 │  - relevance score  │                │              │
└──────────────┘                 └────────┬────────────┘                └──────────────┘
                                          │
                                          │ matched runbook URLs
                                          v
                                 ┌─────────────────────┐
                                 │  Incident Response  │
                                 │  Platform           │
                                 │                     │
                                 │  - Slack/PagerDuty  │
                                 │  - Auto-open ticket │
                                 │  - Link to runbook  │
                                 └─────────────────────┘

┌──────────────┐    PR/Push     ┌─────────────────────┐    auto-commit  ┌──────────────┐
│ Engineer     │ ─────────────> │  GitHub Actions     │ ─────────────> │ index.yaml   │
│ edits        │                │  CI/CD Pipeline     │                │ (updated)    │
│ rb-xxx.md    │                │                     │                │              │
│              │                │  1. Schema validate │                │              │
│              │                │  2. Link check      │                │              │
│              │                │  3. Prose lint      │                │              │
│              │                │  4. Build index     │                │              │
└──────────────┘                └─────────────────────┘                └──────────────┘
```

---

### Part 9: Alertmanager Configuration

```yaml
# alertmanager.yml -- relevant snippet

route:
  receiver: default
  routes:
    - match:
        severity: critical
      receiver: pagerduty-critical
      continue: true

    - match_re:
        alertname: ".*"
      receiver: runbook-router
      group_wait: 30s
      group_interval: 5m

receivers:
  - name: default
    slack_configs:
      - channel: "#alerts"
        title: "{{ .GroupLabels.alertname }}"
        text: "{{ .CommonAnnotations.description }}"

  - name: pagerduty-critical
    pagerduty_configs:
      - service_key: "<key>"
        severity: critical

  - name: runbook-router
    webhook_configs:
      - url: "http://runbook-router:8080/api/v1/alerts"
        send_resolved: false
        http_config:
          basic_auth:
            username: alertmanager
            password_file: /etc/alertmanager/secrets/router-password

# Additionally, a custom receiver that enriches alerts with runbook links:
  - name: alerts-with-runbooks
    slack_configs:
      - channel: "#incidents"
        title: "{{ .GroupLabels.alertname }}"
        text: |
          {{ .CommonAnnotations.description }}

          *Runbooks:*
          {{ range .CommonAnnotations.runbook_links }}
          - {{ . }}
          {{ end }}
```

## Why This Works

1. **Convention over configuration**: The repository structure enforces naming
   conventions (`rb-<domain>-<NNN>-<desc>.md`) and required sections. The CI
   pipeline validates these automatically, preventing drift.

2. **Frontmatter as metadata**: Using HTML comment frontmatter (`<!-- key: value -->`)
   keeps runbooks valid markdown that renders correctly everywhere, while still
   being machine-parseable. This avoids the problem of YAML frontmatter breaking
   some markdown renderers.

3. **Three-tier matching**: The alert-to-runbook service uses exact match (highest
   confidence), tag match (medium confidence), and keyword match (lowest
   confidence). This ensures relevant runbooks surface even when alert rules
   are not explicitly configured.

4. **Hot-reloadable index**: The FastAPI service can reload `index.yaml` without
   restart via `POST /api/v1/reload`. Combined with the CI pipeline that
   auto-commits updated indexes, new runbooks become available within minutes
   of merging.

5. **Validation prevents bad runbooks from merging**: The CI pipeline checks
   schema, cross-references, prose quality, and placeholder text. A runbook
   with "TBD" in the resolution section will fail the build.

## Common Mistakes

1. **No index regeneration on merge**: If `build_index.py` only runs on PRs
   but never commits the result, the index becomes stale. The workflow must
   commit the updated `index.yaml` on push to main.

2. **Hardcoding alert-to-runbook mappings**: Mapping `alertname: "DiskFull"`
   to `runbook: "rb-server-001"` in code is brittle. Using the index with
   `alert_rules` frontmatter keeps the mapping co-located with the runbook.

3. **No link validation**: A runbook that references `rb-pg-001-failover.md`
   in its "Related Runbooks" section but the file does not exist will mislead
   engineers during incidents. The `check_links.py` script prevents this.

4. **Skipping prose linting**: A runbook with passive voice ("the server should
   be restarted") is harder to follow during an incident than active voice
   ("restart the server"). Vale enforces this.

5. **Treating the index as manually maintained**: The index should be entirely
   auto-generated. Manual edits will be overwritten on the next merge and
   create confusion about which version is authoritative.

6. **No severity-based sorting**: When an alert matches 5 runbooks, the most
   relevant one (highest severity, exact match) should appear first. Without
   sorting, engineers waste time reading the wrong runbook.
