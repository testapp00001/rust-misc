# Exercise 05: Complete Security Architecture

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Combine ALL security concepts from Modules 51-56 to design a comprehensive zero trust security architecture for a production application. This exercise requires you to integrate network security, DDoS protection, WAF, secrets management, security scanning, and zero trust principles into a cohesive, defense-in-depth architecture.

## Tasks

### Part A: Design the Architecture

You are building security for a production fintech application called "PayFlow" with these requirements:

```
PAYFLOW APPLICATION REQUIREMENTS:
- Payment processing (PCI DSS compliance required)
- 10,000 requests/second peak
- 99.99% availability SLA
- Multi-region deployment (US-East, EU-West, AP-South)
- Microservices architecture (12 services)
- Must handle PII and financial data
- Regulatory compliance: PCI DSS, SOC2, GDPR
```

Design the complete security architecture. Your design must include:

1. **Network Layer** (Module 51)
2. **DDoS Protection** (Module 52)
3. **WAF Layer** (Module 53)
4. **Secrets Management** (Module 54)
5. **Security Scanning** (Module 55)
6. **Zero Trust Controls** (Module 56)

Draw the complete architecture diagram using ASCII art:

```
INTERNET
    |
[ ??? ]              <-- Layer 1: DDoS Protection (Module 52)
    |
[ ??? ]              <-- Layer 2: WAF (Module 53)
    |
[ ??? ]              <-- Layer 3: Ingress + Zero Trust Gateway
    |
+---+---+---+---+
|   |   |   |   |     <-- Layer 4: Service Mesh (Module 56)
.   .   .   .   .
```

<details><summary>Hint</summary>Think about defense in depth. Each layer should independently protect against threats. If the WAF fails, the service mesh still enforces zero trust. If a service is compromised, micro-segmentation limits blast radius. No single layer is the "last line of defense."</details>

### Part B: Define Security Policies

For each layer, write the specific policies and configurations:

**1. Network Policies (Module 51):**

```yaml
# Write Kubernetes NetworkPolicy for the payment-processing namespace
# - Only ingress-controller can reach the API gateway
# - Only API gateway can reach payment-processor
# - Payment-processor can only reach payment-provider on port 443
# - DNS egress allowed for all pods
# - Default deny all other traffic
```

**2. DDoS Protection (Module 52):**

```yaml
# Write rate limiting configuration
# - Global rate limit: 50,000 req/s across all regions
# - Per-IP rate limit: 100 req/s
# - Per-user rate limit: 50 req/s for authenticated users
# - Burst allowance: 200 req/s for 10 seconds
# - Geographic distribution rules
```

**3. WAF Rules (Module 53):**

```yaml
# Write WAF rules for PayFlow
# - Block SQL injection attempts on /api/payments/*
# - Block XSS in all form submissions
# - Require Content-Type header on POST/PUT/PATCH
# - Block requests with suspicious User-Agent patterns
# - Rate limit login endpoint separately
# - PCI DSS specific rules (block credit card numbers in URLs/logs)
```

**4. Secrets Management (Module 54):**

```yaml
# Design secrets rotation strategy
# - Database credentials: rotate every 24 hours
# - API keys: rotate every 7 days
# - TLS certificates: rotate every 90 days
# - Payment provider keys: rotate every 30 days
# - Encryption keys: rotate every 90 days
```

Write the Vault policy for the payment service:

```hcl
# payment-service-policy.hcl
# TODO: Define what the payment service can and cannot access
# Consider: least privilege, path-based access, capabilities
```

<details><summary>Hint</summary>PCI DSS requires that encryption keys and database credentials are rotated regularly. Use Vault's dynamic secrets where possible -- generate credentials on demand rather than storing static ones. The payment service should NOT have access to user service secrets or admin credentials.</details>

### Part C: Implement Zero Trust Service Mesh

Write the complete Istio configuration for the PayFlow service mesh:

**1. PeerAuthentication (enforce mTLS):**

```yaml
# Require mTLS for ALL services in the payflow namespace
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: payflow
spec:
  mtls:
    mode: STRICT
```

**2. AuthorizationPolicies (write at least 5):**

```yaml
# Policy 1: Default deny all traffic
---
# Policy 2: Ingress gateway -> API gateway
---
# Policy 3: API gateway -> specific services
---
# Policy 4: Payment processor restrictions
---
# Policy 5: Admin service restrictions
---
```

**3. OPA Policy for Payment Validation:**

```rego
package payflow.payments

import future.keywords.in

default allow = false

# Payment validation rules:
# 1. Max transaction amount: $50,000 (amounts over require manual approval)
# 2. Daily transaction limit per user: $100,000
# 3. Suspicious pattern detection (velocity checks)
# 4. PCI DSS: never log full card numbers
# 5. Geographic restrictions (sanctioned countries)
```

<details><summary>Hint</summary>For PCI DSS compliance, the payment service must be isolated from other services. It should only communicate with the API gateway (for receiving requests) and the payment provider (for processing). All payment data must be encrypted at rest and in transit. Use Istio's RequestAuthentication to validate JWT tokens before they reach the payment service.</details>

### Part D: Design the CI/CD Security Pipeline

Integrate Module 55 (Security Scanning) into the deployment pipeline:

```
Developer commits code
         |
    [ ??? ]          <-- SAST (Static Application Security Testing)
         |
    [ ??? ]          <-- SCA (Software Composition Analysis)
         |
    [ ??? ]          <-- Secret scanning
         |
    [ ??? ]          <-- Container image scanning
         |
    [ ??? ]          <-- IaC scanning (Terraform/K8s manifests)
         |
    [ ??? ]          <-- Policy validation (OPA/Gatekeeper)
         |
    Deploy to staging
         |
    [ ??? ]          <-- DAST (Dynamic Application Security Testing)
         |
    [ ??? ]          <-- Integration tests with security checks
         |
    Deploy to production (with zero trust policies active)
```

Write the GitHub Actions pipeline configuration:

```yaml
# .github/workflows/security-pipeline.yaml
name: PayFlow Security Pipeline
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  # TODO: Define security scanning jobs
  # Include: SAST, SCA, secret scanning, container scanning, IaC scanning
```

<details><summary>Hint</summary>PCI DSS requires that code is scanned before deployment. Use a gates approach: if any critical vulnerability is found, the pipeline fails. For zero trust, also validate that the Istio/OPA policies are syntactically correct and enforce the required security posture before deploying.</details>

### Part E: Create the Incident Response Matrix

Create a comprehensive incident response matrix that covers all security layers:

| Threat | Detection Layer | Detection Method | Response | Containment Time |
|--------|----------------|------------------|----------|-----------------|
| DDoS Attack | L1 (DDoS) | Rate anomaly | Auto-scale + block | < 30s |
| SQL Injection | L2 (WAF) | Pattern match | Block + alert | Immediate |
| Compromised Service | L4 (Mesh) | Anomaly detection | Revoke cert + isolate | < 2min |
| Stolen Credentials | L5 (Secrets) | Vault audit log | Rotate + revoke | < 5min |
| Vulnerable Dependency | L6 (Scanning) | SCA scan | Block deploy + patch | < 1hr |
| Data Exfiltration | L4 (Mesh) | Egress monitoring | Block egress + alert | < 1min |
| Insider Threat | All Layers | Behavioral analysis | Investigate + restrict | < 30min |

<details><summary>Hint</summary>Each row should represent a different threat category. The key insight of zero trust is that even if one detection layer fails, other layers provide defense in depth. The containment time should be realistic for automated vs manual responses.</details>

## Success Criteria

- [ ] Complete architecture diagram showing all 6 security layers
- [ ] At least 3 security policies per layer (Network, DDoS, WAF, Secrets, Scanning, Zero Trust)
- [ ] At least 5 Istio AuthorizationPolicies with correct least-privilege rules
- [ ] OPA policy implementing at least 3 payment validation rules
- [ ] CI/CD pipeline with at least 5 security scanning stages
- [ ] Incident response matrix covering at least 7 threat categories
- [ ] All configurations are PCI DSS compliant where applicable
- [ ] Design demonstrates defense in depth -- no single point of failure

## What You Should Understand After This Exercise

- How to integrate multiple security layers into a cohesive architecture
- Why defense in depth means each layer operates independently
- How zero trust principles apply at every level of the stack
- The relationship between detection, response, and containment in a zero trust environment
- How PCI DSS and other compliance frameworks map to zero trust controls
- That security architecture is not a product you buy but a design philosophy you implement
