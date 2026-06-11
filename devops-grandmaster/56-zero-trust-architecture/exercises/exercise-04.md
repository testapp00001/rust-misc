# Exercise 04: Zero Trust Incident Response

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Given a scenario where a service in a zero-trust mesh is compromised, demonstrate how the architecture limits blast radius and walk through the complete detection, containment, and response process. Compare the zero trust response to what would happen in a traditional flat network.

## Tasks

### Part A: Analyze the Attack Scenario

Read the following incident scenario:

```
SCENARIO: Compromised Product Service

The Product Service (product-svc) in your zero-trust e-commerce mesh has been
compromised. An attacker exploited a dependency vulnerability (a malicious
crate published to crates.io that was included via a transitive dependency).

The compromised service now:
1. Attempts to connect to other services in the mesh
2. Tries to read customer data from the User Service
3. Attempts to access the database directly
4. Tries to exfiltrate data to an external endpoint
5. Attempts to modify order records

Your zero-trust architecture includes:
- mTLS between all services (identity: cluster.local/ns/ecommerce/sa/product-svc-sa)
- Istio AuthorizationPolicies (default deny + explicit allow)
- OPA sidecar policies for fine-grained authorization
- Service mesh observability (Kiali, Jaeger, Prometheus)
- Falco for runtime anomaly detection
- Certificate rotation every 24 hours
```

Draw the attack flow diagram showing what the attacker tries vs what the zero trust controls block:

```
[ Attacker in product-svc ]
         |
         |--- Try 1: Connect to user-svc
         |    [ mTLS ] --> ?
         |    [ AuthZ ] --> ?
         |
         |--- Try 2: Connect to database directly
         |    [ Network Policy ] --> ?
         |    [ mTLS ] --> ?
         |
         |--- Try 3: Exfiltrate to external endpoint
         |    [ Egress Policy ] --> ?
         |    [ mTLS ] --> ?
         |
         |--- Try 4: Modify order records
         |    [ AuthZ ] --> ?
         |    [ OPA ] --> ?
         |
         |--- Try 5: Steal credentials
         |    [ Vault ] --> ?
         |    [ No env vars ] --> ?
```

For each attack attempt, determine:
1. Would the attempt succeed or fail?
2. Which specific control blocks it?
3. What alert would be generated?
4. What would the security team see?

<details><summary>Hint</summary>In the communication matrix from Exercise 03, product-svc was only allowed to talk to inventory-svc on GET /stock/*. Any other connection attempt would be blocked by Istio's default-deny policy. mTLS means the attacker cannot impersonate another service.</details>

### Part B: Write the Detection Rules

Implement detection rules that would alert on the compromised service's behavior. Write rules for:

**1. Falco Runtime Rule:**

```yaml
# falco-rules.yaml
- rule: Unexpected Outbound Connection from Product Service
  desc: Detect when product-svc attempts to connect to services it should not communicate with
  condition: >
    # TODO: Define condition for unexpected outbound connections
    # Consider: container name, outbound connection destination, ports
  output: >
    Unexpected outbound connection from product-svc
    (user=%user.name command=%proc.cmdline connection=%fd.name
     container=%container.name image=%container.image.repository)
  priority: WARNING
  tags: [network, zero-trust, lateral-movement]

- rule: Product Service Accessing Sensitive Data
  desc: Detect when product-svc attempts to access data outside its scope
  condition: >
    # TODO: Define condition for data access anomaly
  output: >
    Product service accessing sensitive data
    (file=%fd.name user=%user.name command=%proc.cmdline)
  priority: CRITICAL
  tags: [data-access, zero-trust, exfiltration]
```

**2. Prometheus Alert Rules:**

```yaml
# prometheus-alerts.yaml
groups:
  - name: zero-trust-violations
    rules:
      - alert: AuthorizationPolicyDenied
        expr: |
          # TODO: Query Istio metrics for denied requests
          # hint: istio_requests_total with response_code 403
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Authorization policy denied request"

      - alert: AnomalousServiceCommunication
        expr: |
          # TODO: Detect unusual communication patterns
          # hint: compare current traffic to baseline
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Service showing anomalous communication patterns"
```

**3. Jaeger Trace Analysis:**

Write a query or description of the trace pattern that would indicate lateral movement from the compromised service.

<details><summary>Hint</summary>Look for: (1) product-svc making requests to services it has never communicated with before, (2) a spike in 403 responses from Istio, (3) requests from product-svc to external endpoints not in the allowlist, (4) unusual timing patterns (e.g., requests at 3 AM when traffic is normally zero).</details>

### Part C: Implement Automated Response

Write the automated response playbook that the zero trust architecture would execute:

```yaml
# response-playbook.yaml
incident:
  name: "Compromised Product Service"
  severity: critical

response_steps:
  - step: 1
    action: "Automatic Certificate Revocation"
    description: |
      # TODO: Describe how to revoke product-svc's certificate
      # How does this immediately cut off the attacker's access?
    command: |
      # Example: Revoke certificate via cert-manager
      # kubectl delete certificate product-svc -n ecommerce

  - step: 2
    action: "Enforce Quarantine Policy"
    description: |
      # TODO: Deploy an Istio policy that isolates product-svc
    yaml: |
      # Write an AuthorizationPolicy that blocks ALL traffic to/from product-svc

  - step: 3
    action: "Rotate Service Account"
    description: |
      # TODO: Rotate the service account to invalidate any cached credentials
    command: |
      # kubectl delete serviceaccount product-svc-sa -n ecommerce

  - step: 4
    action: "Deploy Clean Version"
    description: |
      # TODO: Deploy a known-good version of product-svc from a verified image
    command: |
      # kubectl rollout undo deployment/product-svc -n ecommerce

  - step: 5
    action: "Verify and Restore"
    description: |
      # TODO: Verify the clean version, restore limited connectivity
```

<details><summary>Hint</summary>The key insight is that in a zero trust architecture, response is fast because you can revoke trust for a specific identity without affecting any other service. Certificate revocation + policy update = immediate isolation. In a flat network, you would need to reconfigure firewalls and potentially take down the entire subnet.</details>

### Part D: Compare Zero Trust vs Flat Network Response

Create a side-by-side comparison of how the same incident would unfold in each architecture:

| Phase | Zero Trust | Flat Network |
|-------|-----------|--------------|
| Time to detect | ? | ? |
| Blast radius | ? | ? |
| Time to contain | ? | ? |
| Services affected | ? | ? |
| Data at risk | ? | ? |
| Recovery complexity | ? | ? |
| Customer impact | ? | ? |

<details><summary>Hint</summary>In a flat network, the attacker would have immediate access to the database, user service, payment service, and any other internal system. Detection would likely take hours or days. In zero trust, the attacker is confined to product-svc and blocked at every turn.</details>

## Success Criteria

- [ ] Correctly identified which attack attempts succeed and which are blocked
- [ ] Wrote at least 2 Falco detection rules for the scenario
- [ ] Wrote at least 2 Prometheus alert rules for zero trust violations
- [ ] Designed an automated response playbook with at least 4 steps
- [ ] Completed the comparison table with realistic timelines
- [ ] Can articulate why zero trust reduces both blast radius and response time

## What You Should Understand After This Exercise

- How zero trust architecture contains a breach to a single compromised service
- Why identity-based access control (mTLS + service accounts) is superior to network-based control
- How automated detection and response work together in a zero trust environment
- The operational difference between revoking trust and reconfiguring network rules
- Why "assume breach" leads to better detection and faster response
