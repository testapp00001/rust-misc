# Exercise 01: Classify the Attack

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Given real-world DDoS attack traffic patterns, classify each attack by type (volumetric, protocol, or application-layer), identify the OSI layer it targets, and map it to the appropriate mitigation strategy.

## Scenario

Your SOC team has received the following traffic alerts from your monitoring systems. For each alert, you must classify the attack and determine the correct response.

```
Alert Timeline:
  09:15 -- Inbound traffic spikes to 45 Gbps from 12,000 source IPs
           All traffic is UDP on random ports
           Legitimate traffic is 500 Mbps

  09:32 -- SYN packets spike to 800,000/sec on port 443
           TCP connections never complete (no ACK)
           Server connection table fills up

  09:47 -- HTTP GET requests spike to 50,000 req/sec to /api/search
           All requests have valid User-Agent headers
           Each request triggers a full-text database search
           Source IPs are residential (botnet)

  10:05 -- HTTP POST requests to /api/login at 5,000 req/sec
           Each request uses a different username/password pair
           Requests come from 3 IP addresses

  10:20 -- Slow HTTP connections: 10,000 connections open
           Each connection sends headers at 1 byte/sec
           Server thread pool exhausted
```

## Tasks

### Part A: Classify Each Attack

For each alert in the timeline, fill in the following table:

| Time | Attack Type | OSI Layer | Attack Name | Goal |
|------|------------|-----------|-------------|------|
| 09:15 | ? | ? | ? | ? |
| 09:32 | ? | ? | ? | ? |
| 09:47 | ? | ? | ? | ? |
| 10:05 | ? | ? | ? | ? |
| 10:20 | ? | ? | ? | ? |

<details>
<summary>Hint</summary>
Attack types are: Volumetric (saturate bandwidth), Protocol (exhaust server resources), and Application (exhaust application resources). The OSI layer tells you where the attack operates: L3 (network), L4 (transport), or L7 (application). A UDP flood is volumetric at L3/L4. A SYN flood is protocol at L4. An HTTP flood targeting a specific expensive endpoint is application-layer at L7.
</details>

### Part B: Identify Mitigation Layers

For each attack, identify which layer of defense should handle it:

- **Layer 1:** Cloud DDoS Shield (upstream scrubbing)
- **Layer 2:** CDN / Edge WAF
- **Layer 3:** Load Balancer (connection limits, SYN cookies)
- **Layer 4:** Application rate limiting (Nginx, API gateway)
- **Layer 5:** Application code (circuit breakers, caching)

Map each attack to the primary and secondary mitigation layers.

<details>
<summary>Hint</summary>
Volumetric attacks must be stopped upstream (Layer 1) before they reach your infrastructure. Protocol attacks are best handled at the load balancer (Layer 3) with SYN cookies and connection limits. Application attacks require rate limiting (Layer 4) and application-level defenses (Layer 5) like caching expensive queries.
</details>

### Part C: Determine the Blast Radius

For each attack, describe what happens if the mitigation fails:

- Which services are affected?
- What is the impact on legitimate users?
- How quickly does the impact escalate?

<details>
<summary>Hint</summary>
Think about resource exhaustion. A UDP flood that saturates bandwidth affects ALL services. A SYN flood that fills the connection table prevents new TCP connections to ALL ports. An HTTP flood to /api/search only affects that endpoint -- unless it exhausts the database connection pool, which then affects all endpoints that use the database.
</details>

### Part D: Design the Alert Thresholds

For each attack type, define the monitoring metrics and alert thresholds that would detect the attack within 60 seconds. Specify:

- The metric to monitor
- The normal baseline value
- The alert threshold (as a multiple of baseline)
- The severity level

<details>
<summary>Hint</summary>
For volumetric attacks, monitor inbound bandwidth (baseline: 500 Mbps, alert: 10x baseline). For SYN floods, monitor SYN packet rate and connection table utilization. For HTTP floods, monitor requests per second per endpoint. For slowloris, monitor average connection duration and active connection count.
</details>

## Success Criteria

- [ ] Each attack is correctly classified by type and OSI layer
- [ ] Mitigation layers are appropriate for each attack type
- [ ] Blast radius analysis considers cascading failures
- [ ] Alert thresholds are realistic and detect attacks within 60 seconds
- [ ] You can explain why volumetric attacks cannot be stopped at the application layer

## What You Should Understand After This Exercise

DDoS attacks are not all the same. Volumetric attacks overwhelm bandwidth and must be stopped upstream by your cloud provider or CDN. Protocol attacks exhaust server-level resources like connection tables and are best handled by the load balancer with SYN cookies and connection limits. Application-layer attacks are the hardest to detect because they look like legitimate traffic -- they require application-level rate limiting, caching, and behavioral analysis. Understanding the attack type determines where you invest your defense budget and which team is responsible for mitigation.
