# Exercise 05: Runbook Repository and CI/CD

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete runbook repository structure, build a CI/CD pipeline that validates runbooks
on every commit, and create a service that links monitoring alerts to the correct runbooks. This
exercise integrates runbook authoring with DevOps practices: version control, automated testing,
and operational tooling.

## Scenario

Your organization has grown to 120+ runbooks spread across Google Docs, Confluence, Slack messages,
and random markdown files. You are tasked with creating a centralized, version-controlled, and
automatically validated runbook system.

```
                        +-------------------+
                        |  Monitoring/Alerts|
                        |  (Prometheus/     |
                        |   PagerDuty)      |
                        +--------+----------+
                                 |
                                 | alert with runbook annotation
                                 v
                        +-------------------+
                        |  Runbook Service  |
                        |  (links alerts to |
                        |   runbooks)       |
                        +--------+----------+
                                 |
                                 v
+-------------------+    +-------------------+    +-------------------+
|  Runbook Repo     |    |  CI/CD Pipeline   |    |  Runbook Portal   |
|  (Git)            |--->|  (validate, test) |--->|  (search, browse) |
|                   |    |                   |    |                   |
|  /runbooks/       |    |  - lint markdown  |    |  - by service     |
|  /templates/      |    |  - validate YAML  |    |  - by severity    |
|  /schemas/        |    |  - check links    |    |  - by alert       |
|  /scripts/        |    |  - test automation|    |                   |
+-------------------+    +-------------------+    +-------------------+
```

## Tasks

### Part A: Design the Repository Structure

Design the directory structure for the runbook repository. Consider:

- Runbooks organized by service/team (not by incident type, which creates duplication).
- Templates for common runbook patterns.
- Schema files that define the required structure of a runbook.
- Automation scripts that can be referenced from runbooks.
- A metadata file that indexes all runbooks for search.

```
runbooks/
|-- README.md
|-- .github/
|   +-- workflows/
|       +-- validate-runbooks.yml
|-- schemas/
|   +-- runbook-schema.json
|-- templates/
|   +-- standard-runbook.md
|   +-- database-runbook.md
|   +-- network-runbook.md
|-- scripts/
|   +-- validate.py
|   +-- link-checker.py
|   +-- alert-router.py
|-- services/
|   |-- payment-service/
|   |   |-- README.md
|   |   |-- rb-payment-001-pod-crashloop.md
|   |   |-- rb-payment-002-high-latency.md
|   |   +-- rb-payment-003-deploy-rollback.md
|   |-- user-service/
|   |   |-- README.md
|   |   +-- ...
|   +-- database/
|       |-- README.md
|       +-- ...
|-- index.yaml
+-- CHANGELOG.md
```

Complete this structure with:
1. At least 3 service directories with 2 runbooks each.
2. The `index.yaml` that maps alerts to runbooks.
3. A template for a standard runbook.
4. The validation schema (JSON Schema for runbook frontmatter).

<details><summary>Hint</summary>
Each runbook should have YAML frontmatter with metadata:
```yaml
---
id: RB-PAYMENT-001
title: "Pod CrashLoopBackOff"
service: payment-service
team: payments-sre
severity: P1
alerts:
  - alert: PodCrashLoopBackOff
    severity: critical
tags: [kubernetes, pods, crashloop]
last_reviewed: 2026-01-15
review_frequency: quarterly
---
```
The index.yaml can be generated from this frontmatter.
</details>

### Part B: Write the CI/CD Pipeline

Write a GitHub Actions workflow that validates runbooks on every pull request. The pipeline should:

1. **Lint Markdown**: Check that all runbook files are valid Markdown.
2. **Validate Frontmatter**: Check that required YAML frontmatter fields are present and valid.
3. **Validate Schema**: Check that frontmatter conforms to the JSON schema.
4. **Check Links**: Verify that internal links between runbooks are not broken.
5. **Check Freshness**: Warn if a runbook has not been reviewed in the last 90 days.
6. **Test Automation**: If a runbook has associated automation scripts, run them in a test environment.

```yaml
# .github/workflows/validate-runbooks.yml
name: Validate Runbooks

on:
  pull_request:
    paths:
      - 'runbooks/**'
      - 'schemas/**'
      - 'templates/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # TODO: Add validation steps
```

Implement at least 4 of the 6 validation steps.

<details><summary>Hint</summary>
For Markdown linting: use `markdownlint-cli2` or `mdl`.
For frontmatter validation: write a Python script that parses YAML frontmatter and validates
against the schema using `jsonschema`.
For link checking: write a script that extracts all `[text](path)` links and verifies the
target files exist.
For freshness: compare `last_reviewed` field against the current date.
</details>

### Part C: Write the Validation Script

Write `scripts/validate.py` that performs the validation checks from Part B. This script should:

1. Accept a directory path as input.
2. Recursively find all `.md` files.
3. Parse YAML frontmatter from each file.
4. Validate against the JSON schema.
5. Check for broken internal links.
6. Check review freshness.
7. Output a report in GitHub Actions annotation format.

```python
#!/usr/bin/env python3
"""Validate runbook files in a repository."""

import sys
import os
import re
import yaml
import json
from pathlib import Path
from datetime import datetime, timedelta
from jsonschema import validate, ValidationError

def find_runbooks(directory: str) -> list[Path]:
    """Find all markdown files with runbook frontmatter."""
    pass

def parse_frontmatter(filepath: Path) -> dict | None:
    """Extract YAML frontmatter from a markdown file."""
    pass

def validate_schema(frontmatter: dict, schema: dict) -> list[str]:
    """Validate frontmatter against JSON schema. Return list of errors."""
    pass

def check_links(filepath: Path, content: str, all_files: set[Path]) -> list[str]:
    """Check that internal links point to existing files."""
    pass

def check_freshness(frontmatter: dict, max_age_days: int = 90) -> str | None:
    """Check if runbook has been reviewed recently enough."""
    pass

def main():
    """Run all validations and output a report."""
    pass

if __name__ == "__main__":
    main()
```

<details><summary>Hint</summary>
Frontmatter parsing: split file content on `---` and parse the first block as YAML.
Schema validation: load the schema from `schemas/runbook-schema.json` and use `jsonschema.validate()`.
Link checking: use regex `\[.*?\]\((.+?)\)` to find links, then check if the target path exists
relative to the file's directory.
GitHub Actions annotations use the format: `::error file=path,line=N::message`.
</details>

### Part D: Build the Alert-to-Runbook Router

Write `scripts/alert-router.py` -- a service that takes a Prometheus alert and returns the
relevant runbook URL. This integrates your runbook repository with your monitoring system.

The service should:
1. Read the runbook index (generated from frontmatter).
2. Accept an alert name (and optionally labels like service, severity).
3. Return the matching runbook(s) as a URL or file path.
4. Support a webhook endpoint for PagerDuty/Prometheus integration.

```python
#!/usr/bin/env python3
"""Route monitoring alerts to the appropriate runbook."""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import yaml
from pathlib import Path

class AlertRouter:
    def __init__(self, index_path: str):
        """Load the runbook index."""
        pass

    def find_runbook(self, alert_name: str, service: str = None, 
                     severity: str = None) -> list[dict]:
        """Find runbooks matching the given alert."""
        pass

class WebhookHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        """Handle incoming webhook from Prometheus/PagerDuty."""
        pass

def main():
    """Start the alert router service."""
    pass
```

Example integration with Prometheus:
```yaml
# In your Prometheus alert rules
groups:
  - name: kubernetes
    rules:
      - alert: PodCrashLoopBackOff
        expr: rate(kube_pod_container_status_restarts_total[15m]) > 0
        annotations:
          summary: "Pod {{ $labels.pod }} is crash looping"
          runbook_url: "http://runbook-service:8080/runbook?alert=PodCrashLoopBackOff&service={{ $labels.namespace }}"
```

<details><summary>Hint</summary>
The index should be a list of entries, each with alert names, service, severity, and the runbook
path. The `find_runbook` method should match on alert name first, then filter by service and
severity if provided. The webhook endpoint should parse the Prometheus alertmanager payload format.
</details>

### Part E: Write the Runbook Index Schema

Write the `index.yaml` schema and a script that generates it from the runbook frontmatter.
This index is the bridge between your repository and your tooling.

```yaml
# Example index.yaml structure
runbooks:
  - id: RB-PAYMENT-001
    title: "Pod CrashLoopBackOff"
    path: "services/payment-service/rb-payment-001-pod-crashloop.md"
    service: payment-service
    team: payments-sre
    severity: P1
    alerts:
      - PodCrashLoopBackOff
      - ContainerOOMKilled
    tags: [kubernetes, pods]
    last_reviewed: 2026-01-15
    review_due: 2026-04-15
  - id: RB-PAYMENT-002
    ...

metadata:
  generated_at: "2026-06-11T10:00:00Z"
  total_runbooks: 6
  services: [payment-service, user-service, database]
```

Write `scripts/generate-index.py` that:
1. Scans all runbook files.
2. Extracts frontmatter from each.
3. Generates the index.yaml.
4. Reports any runbooks with missing or invalid frontmatter.

## Success Criteria

- [ ] Repository structure is well-organized by service with clear conventions.
- [ ] At least 3 service directories exist with at least 2 runbooks each.
- [ ] Runbooks have complete YAML frontmatter with all required fields.
- [ ] JSON schema correctly defines the required frontmatter structure.
- [ ] CI/CD pipeline validates at least 4 of the 6 checks.
- [ ] `validate.py` correctly parses frontmatter, validates schema, checks links, and checks freshness.
- [ ] Alert router can match alerts to runbooks by name, service, and severity.
- [ ] Index generation script correctly aggregates all runbook metadata.
- [ ] The system end-to-end: alert fires -> router finds runbook -> engineer opens runbook.
- [ ] A new team can adopt this repository structure and start adding runbooks immediately.

## What You Should Understand After This Exercise

A runbook repository is not just a folder of markdown files -- it is an operational platform.
Version control gives you history, review, and rollback. CI/CD gives you validation and
freshness enforcement. An index gives you searchability. An alert router gives you the critical
integration between monitoring and response. The goal is that when an alert fires at 3 AM, the
on-call engineer gets a link to the exact runbook they need, and they can trust that it is
up-to-date and correct because the CI/CD pipeline enforces it.
