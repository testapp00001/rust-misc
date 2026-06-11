# Exercise 01: Centralized Logging Architecture and Design Decisions

**Type:** Conceptual
**Time:** 25 minutes
**Difficulty:** Easy

## Objective

Explain the architecture of centralized logging systems, compare the major
stacks (ELK, EFK, Loki), and articulate the design trade-offs that drive
technology selection for log aggregation at scale.

## Background

When you run a single application on a single server, `tail -f /var/log/app.log`
is enough. When you run 50 microservices across 20 Kubernetes nodes, each
producing hundreds of log lines per second, you need a system that collects,
stores, indexes, and queries logs from every source in one place. The
architecture you choose has lasting implications for cost, operational
complexity, and debugging speed.

---

## Tasks

### Part A: Why Centralized Logging

Answer each question in two or three sentences.

1. A production incident occurs at 3 AM. Your application runs on 12 Kubernetes
   nodes. Without centralized logging, what does the debugging workflow look
   like? What specific problems does this introduce?
2. What is the difference between a log *aggregator* and a log *search engine*?
   Give one example of each from the ELK or Loki ecosystem.
3. Explain the concept of "structured logging" and why it is a prerequisite
   for effective centralized logging. What format is most commonly used?
4. Why do centralized logging systems separate the concepts of *collection*,
*processing*, *storage*, and *querying* into different components rather than
building one monolithic tool?

<details>
<summary>Hint</summary>

- Think about what you would have to do without centralized logging: SSH into
  each node, grep through files, manually correlate timestamps across servers.
- A log aggregator collects and forwards logs; a log search engine stores them
  in an indexed format that supports fast queries.
- Structured logging means emitting logs as key-value pairs (typically JSON)
  rather than free-form text strings.
- Separation of concerns lets you scale each layer independently, swap
  components, and handle failures in one layer without losing the others.

</details>

### Part B: Compare the Stacks

You are given three centralized logging architectures:

**Architecture 1: ELK Stack**
```
Applications --> Filebeat --> Logstash --> Elasticsearch --> Kibana
```

**Architecture 2: EFK Stack**
```
Applications --> Fluentd --> Elasticsearch --> Kibana
```

**Architecture 3: Loki + Grafana**
```
Applications --> Promtail --> Loki --> Grafana
```

For each architecture, answer the following:

1. What does each component do? Describe its role in one sentence.
2. Where does full-text indexing happen? What are the cost implications of
   indexing every field versus indexing only labels/metadata?
3. Which stack is most Kubernetes-native and why?
4. If your primary need is correlating logs with Prometheus metrics and traces
   in a single UI, which stack is the best fit? Justify your answer.

<details>
<summary>Hint</summary>

- Elasticsearch indexes the full content of every document. Loki indexes only
  labels (metadata) and stores log content in compressed chunks.
- Fluentd and Promtail both run as DaemonSets on Kubernetes nodes.
- Grafana natively supports Prometheus (metrics), Tempo (traces), and Loki
  (logs) with built-in correlation features.

</details>

### Part C: Identify the Architecture Problems

For each scenario, identify what is wrong with the logging architecture and
explain how to fix it.

**Scenario 1: Single Elasticsearch Node**

```
All 50 services --> Single Logstash --> Single Elasticsearch --> Kibana
```

The team reports that queries take 30+ seconds and the cluster runs out of
disk space every two weeks.

<details>
<summary>Hint</summary>

- Think about horizontal scaling, index lifecycle management, and retention
  policies.

</details>

**Scenario 2: Logging Everything**

A team configures their application to log at DEBUG level in production.
They ship all logs to Elasticsearch with no filtering. Daily ingest is 500 GB.
Storage costs are spiraling.

<details>
<summary>Hint</summary>

- Consider pre-ingest filtering, log levels, and what should be logged at
  DEBUG versus INFO in production.

</details>

**Scenario 3: No Labels or Structure**

Logs are shipped to Loki, but the Promtail configuration only sets
`{job="containers"}` as the label. All 50 services share the same label set.
Queries require scanning all logs and filtering by content.

<details>
<summary>Hint</summary>

- Loki's performance depends on labels. Think about what labels make queries
  efficient without creating high cardinality.

</details>

### Part D: Design a Logging Architecture

Your company runs the following services in Kubernetes:

- 3 frontend web servers (Node.js)
- 5 API services (Python/Go)
- 2 PostgreSQL databases
- 1 Redis cluster
- 1 RabbitMQ broker
- nginx ingress controller

Design a centralized logging architecture. Address:

1. Which stack you would choose (ELK, EFK, or Loki) and why
2. How logs are collected from each source
3. What labels or metadata you would attach to each log stream
4. How you would handle retention (hot, warm, cold tiers)
5. How you would reduce log volume without losing important information

<details>
<summary>Hint</summary>

- Consider that Prometheus and Grafana are already in use for metrics.
- Database logs and application logs have different formats and volumes.
- Not all logs need the same retention period.

</details>

---

## Success Criteria

- [ ] You can explain the role of each component in a centralized logging
      pipeline (collection, processing, storage, querying)
- [ ] You can compare ELK, EFK, and Loki architectures and articulate the
      trade-offs of each
- [ ] You can identify common architectural anti-patterns (single points of
      failure, no filtering, no labels) and propose fixes
- [ ] You can design a logging architecture for a multi-service Kubernetes
      environment with appropriate retention and cost controls
- [ ] You understand why Loki's label-only indexing model is cheaper but
      trades off full-text search speed

## What You Should Understand After This Exercise

Centralized logging is not just "send logs somewhere." The architecture you
choose -- which components, which indexing model, which labels, which retention
strategy -- determines whether your logging system helps you debug incidents
in minutes or becomes a costly, slow mess that nobody wants to use. The key
insight is that Loki's design philosophy (index labels, not content) trades
query flexibility for dramatically lower cost and operational complexity, which
is the right trade-off for most Kubernetes-native teams already using Grafana.
