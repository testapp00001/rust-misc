# Exercise 04: Design a DDoS Mitigation Strategy

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design a comprehensive DDoS mitigation strategy for a production e-commerce platform. You will create an incident response runbook, implement an emergency response script, configure upstream protections, and establish escalation procedures. This exercise tests your ability to make rapid decisions under pressure and coordinate multiple layers of defense.

## Scenario

You are the SRE on call for ShopFast, an e-commerce platform that handles 2,000 requests/second during normal operations. The platform runs on AWS with the following architecture:

```
Internet --> CloudFront CDN --> Application Load Balancer --> ECS Fargate (3 services)
                                      |
                                +-----+-----+
                                |           |
                           API Service   Search Service
                                |           |
                                +-----+-----+
                                      |
                                  RDS PostgreSQL
                                  ElastiCache Redis
```

At 14:30 UTC on Black Friday, your monitoring alerts fire:

```
ALERT: Request rate is 45,000 req/s (normal: 2,000 req/s)
ALERT: 5xx error rate at 35% (normal: 0.5%)
ALERT: ALB target response time p99 at 12s (normal: 200ms)
ALERT: ECS CPU utilization at 98% across all tasks
ALERT: RDS connections at maximum (200/200)
```

You need to design and implement a mitigation strategy.

## Tasks

### Part A: Classify the Attack and Prioritize

Based on the alert data, determine:

1. What type of DDoS attack is this? (volumetric, protocol, or application-layer)
2. What is the attack targeting? (bandwidth, connections, application resources)
3. What is the blast radius? (which services are affected)
4. What is the priority order for mitigation? (which action first)

Write your analysis as a structured incident report:

```
Attack Classification: ...
Target: ...
Affected Services: ...
Priority Actions:
  1. ...
  2. ...
  3. ...
```

<details>
<summary>Hint: Analyzing the alerts</summary>

The 45,000 req/s rate with 35% 5xx errors suggests an application-layer HTTP flood, not a volumetric attack (volumetric attacks would show bandwidth saturation, not application errors). The RDS connection exhaustion indicates the attack is hitting database-bound endpoints. The CPU saturation is a secondary effect of the request flood. Prioritize: first stop the bleeding (rate limiting), then protect the database (circuit breakers), then clean up (block IPs, enable upstream protection).

</details>

### Part B: Implement the Emergency Response Script

Write a bash script called `emergency-response.sh` that automates the first 5 minutes of incident response. The script should:

1. **Enable aggressive rate limiting** -- Deploy an emergency Nginx configuration with strict rate limits
2. **Protect the database** -- Enable connection pool limits and circuit breakers
3. **Block attacking IPs** -- Analyze recent logs and add iptables rules for top offenders
4. **Scale up infrastructure** -- Increase ECS task count to handle the load
5. **Enable upstream protection** -- Activate AWS Shield Advanced / Cloudflare Under Attack Mode
6. **Alert the team** -- Send notifications to Slack/PagerDuty
7. **Log the incident** -- Record all actions taken with timestamps

The script should accept arguments:
```bash
./emergency-response.sh --level critical --dry-run    # Show what would be done
./emergency-response.sh --level critical --execute     # Actually do it
./emergency-response.sh --rollback                     # Undo emergency changes
```

<details>
<summary>Hint 1: Emergency Nginx config</summary>

Create a temporary Nginx config snippet that overrides the normal rate limits:
```bash
cat > /etc/nginx/conf.d/emergency.conf << 'EOF'
limit_req_zone $binary_remote_addr zone=emergency:10m rate=5r/s;
limit_conn_zone $binary_remote_addr zone=emergency_conn:10m;

server {
    limit_req zone=emergency burst=10 nodelay;
    limit_conn emergency_conn 5;

    # Block empty User-Agents
    if ($http_user_agent = "") { return 403; }

    # Allow only essential paths
    location /health { proxy_pass http://backend; }
    location /api/orders { proxy_pass http://backend; }
    location /api/products { proxy_pass http://backend; }
    location / { return 503; }
}
EOF
nginx -t && nginx -s reload
```

</details>

<details>
<summary>Hint 2: Log analysis for IP blocking</summary>

Extract the top offending IPs from recent Nginx logs:
```bash
TOP_IPS=$(tail -50000 /var/log/nginx/access.log | \
    awk '{print $1}' | sort | uniq -c | sort -rn | head -50 | \
    awk '{print $2}')

for ip in $TOP_IPS; do
    iptables -A INPUT -s "$ip" -j DROP
done
```

</details>

<details>
<summary>Hint 3: AWS scaling</summary>

Use the AWS CLI to scale ECS services:
```bash
aws ecs update-service \
    --cluster production \
    --service api-service \
    --desired-count 10

aws ecs update-service \
    --cluster production \
    --service search-service \
    --desired-count 6
```

</details>

### Part C: Create the Incident Runbook

Write a file called `ddos-runbook.md` that serves as the team's DDoS incident response runbook. It should include:

1. **Detection** -- What monitoring alerts indicate a DDoS attack
2. **Classification** -- How to determine the attack type and severity
3. **Immediate Response** (0-5 minutes) -- Automated actions and manual checks
4. **Short-term Mitigation** (5-30 minutes) -- Detailed steps to stabilize
5. **Recovery** (30-60 minutes) -- How to return to normal operations
6. **Post-Incident** -- What to do after the attack ends
7. **Escalation Matrix** -- When and whom to escalate to

For each step, specify:
- The action to take
- Who is responsible (SRE, Security, Management)
- How to verify the action worked
- What to do if the action fails

<details>
<summary>Hint: Escalation matrix</summary>

Structure the escalation as levels:
- **Level 1 (SRE on call)**: Rate limiting, log analysis, IP blocking (0-5 min)
- **Level 2 (SRE team lead)**: Infrastructure scaling, upstream provider contact (5-15 min)
- **Level 3 (Security team)**: Threat intelligence, forensic analysis (15-30 min)
- **Level 4 (Management)**: Customer communication, ISP escalation, legal (30+ min)

Each level should have clear triggers for escalation (e.g., "Escalate to Level 2 if 5xx rate does not drop below 10% within 5 minutes of Level 1 actions").

</details>

### Part D: Design Upstream Protection

Design the upstream protection configuration for AWS Shield Advanced and WAF. Write the Terraform configuration that:

1. Enables Shield Advanced on the ALB and CloudFront distribution
2. Creates WAF rules for rate limiting, geo-blocking, and bot detection
3. Configures CloudFront with appropriate cache behaviors and origin failover
4. Sets up CloudWatch alarms for attack detection

<details>
<summary>Hint: Shield + WAF integration</summary>

The key resources are:
- `aws_shield_protection` for each resource to protect
- `aws_wafv2_web_acl` with rate-based rules, IP reputation rules, and managed rule groups
- `aws_cloudfront_distribution` with `web_acl_id` associated
- `aws_cloudwatch_metric_alarm` for `DDoSDetected` and `DDoSAttackBitsPerSecond` metrics

Use AWS managed rule groups like `AWSManagedRulesCommonRuleSet` and `AWSManagedRulesKnownBadInputsRuleSet` for baseline protection.

</details>

### Part E: Implement Circuit Breakers for Database Protection

Write a Python module called `db_circuit_breaker.py` that protects the database during an attack:

1. Monitor database connection count and response time
2. Open the circuit when connections exceed 80% of the pool or response time exceeds 2 seconds
3. When the circuit is open, return cached responses or a graceful degradation message
4. Half-open state: allow 1 request per 10 seconds to test if the database has recovered
5. Close the circuit when 3 consecutive test requests succeed

Include integration with the Flask application from the module README.

<details>
<summary>Hint: Circuit breaker states</summary>

```
CLOSED (normal) --[failure_threshold reached]--> OPEN
OPEN --[recovery_timeout elapsed]--> HALF_OPEN
HALF_OPEN --[test request succeeds]--> CLOSED
HALF_OPEN --[test request fails]--> OPEN
```

In the OPEN state, the application should:
- Check Redis cache for the requested data
- Return a stale response with a `X-Served-From: cache` header
- If no cache exists, return a 503 with `Retry-After: 10`

</details>

---

## Success Criteria

- [ ] The attack classification correctly identifies this as an application-layer HTTP flood.
- [ ] The emergency response script has a dry-run mode and a rollback mode.
- [ ] The runbook covers all phases (detection through post-incident) with clear ownership.
- [ ] The Terraform configuration enables Shield Advanced and WAF with rate-based rules.
- [ ] The circuit breaker correctly transitions between states and returns cached data when open.
- [ ] The emergency response script logs all actions with timestamps for post-incident review.
- [ ] The escalation matrix has clear triggers and responsible parties for each level.

## What You Should Understand After This Exercise

DDoS mitigation is not just about technology -- it is about process, speed, and coordination. The first 5 minutes of an attack are critical. Automated scripts can handle the immediate response (rate limiting, IP blocking, scaling) while humans focus on classification and decision-making. The key layers are: upstream protection (Shield/Cloudflare) to absorb volumetric attacks, WAF rules to filter malicious application traffic, rate limiting to cap per-client abuse, circuit breakers to protect downstream services, and graceful degradation to maintain partial service availability. Every mitigation action should be reversible (rollback capability) and logged (for post-incident analysis).
