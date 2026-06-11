# Solution 04: Zero Trust Incident Response

## Part A: Attack Flow Analysis

```
[ Attacker in product-svc ]
         |
         |--- Try 1: Connect to user-svc
         |    [ mTLS ] --> BLOCKED (product-svc cert is valid, but...)
         |    [ AuthZ ] --> BLOCKED (no AuthorizationPolicy allows product-svc -> user-svc)
         |    [ Alert ] --> Istio logs 403, Prometheus fires AuthorizationPolicyDenied alert
         |
         |--- Try 2: Connect to database directly
         |    [ Network Policy ] --> BLOCKED (no NetworkPolicy allows product-svc -> warehouse-db)
         |    [ mTLS ] --> BLOCKED (product-svc has no client cert for warehouse-db)
         |    [ Alert ] --> Falco detects unexpected outbound connection
         |
         |--- Try 3: Exfiltrate to external endpoint
         |    [ Egress Policy ] --> BLOCKED (no egress gateway allows external connections from product-svc)
         |    [ mTLS ] --> BLOCKED (external endpoint has no cert signed by internal CA)
         |    [ Alert ] --> Falco + Prometheus alert on unexpected egress
         |
         |--- Try 4: Modify order records
         |    [ AuthZ ] --> BLOCKED (no policy allows product-svc -> order-svc)
         |    [ OPA ] --> BLOCKED (even if somehow reached, product-svc identity is not authorized)
         |    [ Alert ] --> Istio logs 403, anomaly detection flags lateral movement
         |
         |--- Try 5: Steal credentials
         |    [ Vault ] --> BLOCKED (product-svc can only read its own secrets from Vault)
         |    [ No env vars ] --> BLOCKED (no static credentials in environment variables)
         |    [ Alert ] --> Vault audit log shows unauthorized secret access attempt
```

### Why This Works

Every attack attempt fails because of the layered zero trust controls:

1. **mTLS identity**: The compromised product-svc presents its own certificate during every TLS handshake. It cannot forge another service's identity because it does not have that service's private key.

2. **AuthorizationPolicies**: Even with a valid certificate, the default-deny policy blocks all connections unless an explicit ALLOW policy exists. Product-svc was only authorized to communicate with inventory-svc on GET /stock/*.

3. **Network Policies**: Kubernetes NetworkPolicies provide an additional layer of defense at the network level, preventing pods from sending packets to unauthorized destinations.

4. **Egress controls**: Istio's egress gateway restricts which external endpoints services can reach. The attacker cannot simply open a socket to their command-and-control server.

5. **Secrets management**: Vault provides dynamic, scoped credentials. Product-svc can only access its own Vault paths. There are no static credentials in environment variables to steal.

### Common Mistakes

- **Assuming the attacker can impersonate other services.** mTLS prevents identity spoofing. The attacker would need the private key of the target service.
- **Thinking network policies are redundant with Istio.** Defense in depth means both layers operate independently. If Istio has a bug, NetworkPolicies still block unauthorized traffic.
- **Forgetting about egress.** Many implementations focus on service-to-service (east-west) traffic but forget about egress (north-south) traffic. The attacker wants to exfiltrate data, which requires outbound connections.

---

## Part B: Detection Rules

### Falco Runtime Rules

```yaml
# falco-rules.yaml
- rule: Unexpected Outbound Connection from Product Service
  desc: Detect when product-svc attempts to connect to services it should not communicate with
  condition: >
    outbound and
    container and
    container.name = "product-svc" and
    not (fd.sip.name = "inventory-svc" and fd.sport = 8080) and
    not (fd.sip.name = "kubernetes.default.svc" and fd.sport = 443) and
    not (fd.sip.name =~ ".*vault.*" and fd.sport = 8200)
  output: >
    Unexpected outbound connection from product-svc
    (user=%user.name command=%proc.cmdline connection=%fd.name
     container=%container.name image=%container.image.repository
     destination=%fd.sip destination_port=%fd.sport)
  priority: WARNING
  tags: [network, zero-trust, lateral-movement]

- rule: Product Service Accessing Sensitive Files
  desc: Detect when product-svc attempts to access files outside its expected scope
  condition: >
    open_read and
    container and
    container.name = "product-svc" and
    not fd.name startswith "/app/" and
    not fd.name startswith "/etc/product-svc/" and
    not fd.name startswith "/tmp/"
  output: >
    Product service accessing unexpected file
    (file=%fd.name user=%user.name command=%proc.cmdline
     container=%container.name image=%container.image.repository)
  priority: CRITICAL
  tags: [file-access, zero-trust, data-exfiltration]

- rule: Unexpected Process Execution in Product Service
  desc: Detect when product-svc spawns unexpected processes (sign of compromise)
  condition: >
    spawned_process and
    container and
    container.name = "product-svc" and
    not proc.name in (product-svc, sh, bash) and
    not proc.pname in (product-svc, sh, bash)
  output: >
    Unexpected process in product-svc
    (process=%proc.name parent=%proc.pname command=%proc.cmdline
     user=%user.name container=%container.name)
  priority: CRITICAL
  tags: [process, zero-trust, compromise-indicator]

- rule: Product Service Credential Access Attempt
  desc: Detect when product-svc attempts to access credentials not assigned to it
  condition: >
    (open_read or open_write) and
    container and
    container.name = "product-svc" and
    (fd.name contains "credentials" or
     fd.name contains "secret" or
     fd.name contains ".env" or
     fd.name contains "token")
  output: >
    Credential access attempt from product-svc
    (file=%fd.name user=%user.name command=%proc.cmdline
     container=%container.name)
  priority: CRITICAL
  tags: [credentials, zero-trust, secret-access]
```

### Prometheus Alert Rules

```yaml
# prometheus-alerts.yaml
groups:
  - name: zero-trust-violations
    rules:
      - alert: AuthorizationPolicyDenied
        expr: |
          sum(rate(istio_requests_total{
            response_code="403",
            source_workload="product-svc"
          }[5m])) > 0
        for: 1m
        labels:
          severity: warning
          category: zero-trust
        annotations:
          summary: "Authorization policy denied request from product-svc"
          description: "product-svc is making requests that violate authorization policies"

      - alert: AnomalousServiceCommunication
        expr: |
          # Detect product-svc communicating with services it has never talked to before
          (
            sum(rate(istio_requests_total{
              source_workload="product-svc",
              destination_workload!="inventory-svc",
              destination_workload!="istio-ingressgateway"
            }[5m])) > 0
          )
        for: 1m
        labels:
          severity: critical
          category: lateral-movement
        annotations:
          summary: "Product service showing anomalous communication patterns"
          description: "product-svc is attempting to communicate with unexpected services"

      - alert: HighRateOfDeniedRequests
        expr: |
          sum(rate(istio_requests_total{response_code="403"}[5m]))
          /
          sum(rate(istio_requests_total[5m]))
          > 0.1
        for: 2m
        labels:
          severity: critical
          category: zero-trust
        annotations:
          summary: "High rate of denied requests detected"
          description: "More than 10% of requests are being denied, indicating possible compromise"

      - alert: UnexpectedEgressTraffic
        expr: |
          sum(rate(istio_tcp_sent_bytes_total{
            source_workload="product-svc",
            destination_workload="istio-egressgateway"
          }[5m])) > 1000
        for: 1m
        labels:
          severity: critical
          category: data-exfiltration
        annotations:
          summary: "Unexpected egress traffic from product-svc"
          description: "product-svc is sending data through the egress gateway"

      - alert: CertificateRevocationDetected
        expr: |
          sum(rate(istio_requests_total{
            response_code="403",
            connection_security_policy="mutual_tls"
          }[1m])) > 5
        for: 30s
        labels:
          severity: info
          category: certificate-management
        annotations:
          summary: "Certificate revocation in effect"
          description: "Multiple mTLS failures detected, likely due to certificate revocation"
```

### Jaeger Trace Analysis

Trace patterns indicating lateral movement from the compromised product-svc:

```
SUSPICIOUS TRACE PATTERNS:

1. Request chain: product-svc -> (unknown service)
   - Normal traces show: product-svc -> inventory-svc
   - Suspicious: product-svc -> user-svc, product-svc -> order-svc
   - Detection: Compare against baseline communication patterns

2. Request frequency anomaly:
   - Normal: 10-50 requests/minute from product-svc
   - Suspicious: 1000+ requests/minute (scanning behavior)
   - Detection: Statistical deviation from 30-day rolling average

3. Request timing anomaly:
   - Normal: Requests during business hours
   - Suspicious: Requests at 3 AM (attacker working off-hours)
   - Detection: Time-series analysis of request patterns

4. Error rate spike:
   - Normal: <1% error rate
   - Suspicious: >50% error rate (403s from denied connections)
   - Detection: Error rate threshold alert

5. New destination services:
   - Normal: product-svc only talks to inventory-svc
   - Suspicious: First-ever connection to user-svc, payment-svc, etc.
   - Detection: New edge in service communication graph
```

### Why This Works

Detection in a zero trust environment leverages the fact that every connection attempt is logged and verified. Unlike a flat network where traffic flows freely, zero trust generates a rich audit trail:

1. **Istio metrics**: Every request is tagged with source/destination workload, response code, and mTLS status. Denied requests (403) are immediately visible.
2. **Falco runtime monitoring**: Detects process execution, file access, and network connections at the container level. Catches behaviors that Istio cannot see (e.g., file system access).
3. **Prometheus alerting**: Aggregates metrics over time to detect patterns. A single 403 is normal; a spike of 403s from one service indicates compromise.
4. **Jaeger tracing**: Provides distributed tracing across the service mesh. Anomalous trace patterns reveal lateral movement attempts.

### Common Mistakes

- **Only alerting on successful attacks.** Alert on attempts, not just successes. A 403 from an authorization policy is a failed attack, but it is still important signal.
- **Ignoring false positives.** Too many alerts cause alert fatigue. Tune thresholds based on baseline behavior.
- **Not correlating across systems.** A Falco alert for unexpected process + Istio 403 spike + Jaeger anomalous trace = high-confidence compromise signal. Each alone might be noise.
- **Forgetting about normal traffic patterns.** You need a baseline of "normal" to detect "abnormal." Collect baseline metrics for at least 30 days before enabling anomaly detection.

---

## Part C: Automated Response Playbook

```yaml
# response-playbook.yaml
incident:
  name: "Compromised Product Service"
  severity: critical
  category: service-compromise
  zero-trust-principle: "assume-breach"

response_steps:
  - step: 1
    action: "Automatic Certificate Revocation"
    description: |
      Immediately revoke product-svc's mTLS certificate. This cuts off
      the attacker's ability to authenticate as product-svc to any service
      in the mesh. Since mTLS is required for all connections, the attacker
      is effectively network-isolated within seconds.
    command: |
      # Revoke via cert-manager
      kubectl delete certificate product-svc-cert -n ecommerce
      # Trigger immediate re-issuance for the replacement pod
      kubectl annotate certificate product-svc-cert -n ecommerce \
        cert-manager.io/renew-before="0s" --overwrite
    automation: true
    expected_time: "< 30 seconds"

  - step: 2
    action: "Enforce Quarantine Policy"
    description: |
      Deploy a restrictive AuthorizationPolicy that blocks ALL traffic
      to and from product-svc. This is more aggressive than the default-deny
      because it explicitly targets the compromised service.
    yaml: |
      apiVersion: security.istio.io/v1
      kind: AuthorizationPolicy
      metadata:
        name: quarantine-product-svc
        namespace: ecommerce
      spec:
        selector:
          matchLabels:
            app: product-svc
        action: DENY
        rules:
          - {}  # Deny ALL traffic
    automation: true
    expected_time: "< 15 seconds"

  - step: 3
    action: "Rotate Service Account"
    description: |
      Rotate the product-svc Kubernetes service account. This invalidates
      any cached tokens or credentials the attacker may have obtained.
      The new service account will have a fresh identity.
    command: |
      kubectl delete serviceaccount product-svc-sa -n ecommerce
      kubectl apply -f - <<EOF
      apiVersion: v1
      kind: ServiceAccount
      metadata:
        name: product-svc-sa
        namespace: ecommerce
        annotations:
          security.payflow.io/rotated: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
      EOF
    automation: true
    expected_time: "< 1 minute"

  - step: 4
    action: "Deploy Clean Version"
    description: |
      Deploy a known-good version of product-svc from a verified,
      signed container image. Use image digest pinning to ensure
      the exact image that was scanned and approved.
    command: |
      # Roll back to the last known-good deployment
      kubectl rollout undo deployment/product-svc -n ecommerce
      # Verify the rollback
      kubectl rollout status deployment/product-svc -n ecommerce
      # Pin to specific image digest
      kubectl set image deployment/product-svc \
        product-svc=registry.payflow.io/product-svc@sha256:KNOWN_GOOD_DIGEST \
        -n ecommerce
    automation: true
    expected_time: "< 3 minutes"

  - step: 5
    action: "Verify and Restore Limited Connectivity"
    description: |
      After the clean version is running, gradually restore connectivity.
      Start with health checks, then restore read-only access to inventory-svc,
      then full access.
    command: |
      # Remove quarantine policy
      kubectl delete authorizationpolicy quarantine-product-svc -n ecommerce
      # Verify pod is healthy
      kubectl wait --for=condition=ready pod -l app=product-svc -n ecommerce --timeout=60s
      # Test connectivity to inventory-svc
      kubectl exec -n ecommerce deploy/product-svc -- \
        curl -s https://inventory-svc/stock/health
    automation: false  # Manual verification required
    expected_time: "< 5 minutes"

  - step: 6
    action: "Post-Incident Forensics"
    description: |
      Collect logs, traces, and memory dumps from the compromised pod
      before it is destroyed. Analyze the attack vector and update
      detection rules.
    command: |
      # Collect pod logs
      kubectl logs -n ecommerce -l app=product-svc --previous > /tmp/product-svc-logs.txt
      # Collect Falco alerts
      kubectl logs -n falco -l app=falco --since=1h > /tmp/falco-alerts.txt
      # Export Jaeger traces
      curl "http://jaeger:16686/api/traces?service=product-svc&limit=100" > /tmp/traces.json
      # Collect network flow logs
      kubectl logs -n istio-system -l app=istio-ingressgateway --since=1h > /tmp/istio-logs.txt
    automation: false
    expected_time: "Manual"
```

### Why This Works

The response playbook leverages zero trust architecture for rapid containment:

1. **Certificate revocation (30s)**: The most powerful tool in zero trust. Revoking a certificate immediately invalidates the service's identity across the entire mesh. No other service will accept connections from the compromised service.

2. **Quarantine policy (15s)**: Even before certificate revocation takes effect, a DENY policy blocks all traffic. This is an application-level isolation that works regardless of the certificate state.

3. **Service account rotation (1m)**: Prevents the attacker from using any cached Kubernetes tokens or Vault credentials associated with the old service account.

4. **Clean deployment (3m)**: Ensures the running code is verified and trusted. Image digest pinning prevents supply chain attacks on the recovery itself.

5. **Gradual restoration (5m)**: Restores service incrementally with verification at each step. This prevents re-compromise and builds confidence that the system is clean.

The total time from detection to full containment is under 2 minutes for the automated steps. Compare this to flat network response times of 30 minutes to several hours.

### Common Mistakes

- **Restoring too quickly.** The urge to restore service is strong, but restoring before verifying the clean version risks re-compromise.
- **Not preserving forensic evidence.** Logs, traces, and memory dumps are critical for understanding the attack. Collect them before destroying the compromised pod.
- **Forgetting to update detection rules.** Every incident is a learning opportunity. Update Falco rules, Prometheus alerts, and OPA policies based on what you learned.
- **Not rotating all credentials.** The attacker may have obtained credentials for other services. Rotate all credentials that the compromised service had access to.

---

## Part D: Zero Trust vs Flat Network Response Comparison

| Phase | Zero Trust | Flat Network |
|-------|-----------|--------------|
| Time to detect | < 5 minutes (automated alerts) | Hours to days (manual log review) |
| Blast radius | Single service (product-svc only) | Entire internal network |
| Time to contain | < 2 minutes (cert revocation + policy) | Hours (firewall reconfiguration) |
| Services affected | 1 (product-svc isolated) | All internal services potentially compromised |
| Data at risk | None (product-svc has no access to sensitive data) | All customer data, payment data, credentials |
| Recovery complexity | Redeploy single service, restore policies | Rebuild entire infrastructure, rotate all credentials |
| Customer impact | Minimal (product catalog temporarily unavailable) | Severe (payment processing down, data breach notification required) |
| Regulatory impact | No data breach, no notification required | PCI DSS violation, GDPR notification, SOC2 findings |
| Forensic complexity | Single service to analyze | Entire network to analyze |
| Recurrence risk | Low (policies prevent same attack vector) | High (flat network allows same lateral movement) |

### Why This Works

The comparison demonstrates the fundamental value proposition of zero trust:

**Zero Trust Scenario:**
- The attacker is confined to product-svc from the moment of compromise.
- Every attempt to move laterally is blocked and logged.
- Automated response isolates the compromised service in under 2 minutes.
- No sensitive data is accessible from product-svc.
- Recovery is a simple redeployment of a single service.

**Flat Network Scenario:**
- The attacker has unrestricted access to the internal network.
- Lateral movement is trivial -- scan, discover, exploit.
- Detection happens when data is already exfiltrating.
- All services and data are potentially compromised.
- Recovery requires rebuilding the entire infrastructure.

The difference is not just speed -- it is the fundamental architecture. Zero trust assumes breach and designs for containment. Flat networks assume perimeter security and design for convenience.

### Common Mistakes

- **Underestimating detection time in flat networks.** Many organizations assume they would detect a breach quickly. The average detection time for flat networks is 197 days (IBM Cost of Data Breach Report).
- **Overestimating zero trust complexity.** While zero trust requires more initial setup, the operational complexity during an incident is dramatically lower.
- **Ignoring the regulatory implications.** In a flat network compromise, all data is at risk, triggering mandatory breach notifications. In zero trust, the blast radius may be small enough that no notification is required.

## Key Takeaway

Zero trust does not prevent breaches -- it contains them. The assume-breach principle means designing systems where a compromised component cannot harm the rest of the architecture. Certificate revocation, policy enforcement, and micro-segmentation work together to limit blast radius to a single service. The result is not just faster detection and response, but a fundamentally different risk profile: instead of "one breach = total compromise," zero trust delivers "one breach = one service, contained in minutes."
