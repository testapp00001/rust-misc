# Module 42: Alerting Systems — Exercises

## Overview

Alerting is the bridge between monitoring and response. A well-designed alerting system pages the right person, at the right time, with enough context to start debugging immediately. A poorly designed one creates alert fatigue, where engineers ignore pages because 95% of them are noise. This module covers AlertManager routing and inhibition, PagerDuty integration, on-call rotations, escalation policies, alert grouping and deduplication, and strategies for reducing alert noise.

## Prerequisites

- Docker and Docker Compose
- Basic understanding of Prometheus (Module 39)
- Familiarity with YAML configuration
- A text editor for writing AlertManager and Prometheus configs

## Exercises

| Exercise | Type | Topic | Estimated Time |
|----------|------|-------|----------------|
| 01 | Conceptual | Alert fatigue and meaningful alerts | 30 min |
| 02 | Guided | AlertManager with routing and inhibition | 45 min |
| 03 | Independent | Escalation policies and on-call rotations | 60 min |
| 04 | Challenge | Alert deduplication and grouping | 60 min |
| 05 | Integration | Alerting strategy reducing noise by 80% | 90 min |

## Getting Started

Work through the exercises in order. Each exercise builds on concepts from the previous one. Solutions are available in the `solutions/` directory — attempt each exercise before reviewing the solution.

## Environment Setup

Install the base dependencies before starting:

```bash
docker pull prom/alertmanager:latest
docker pull prom/prometheus:latest
docker pull prom/node-exporter:latest
docker pull python:3.11-slim
```

For the webhook receiver used in several exercises:

```bash
pip install flask requests
```
