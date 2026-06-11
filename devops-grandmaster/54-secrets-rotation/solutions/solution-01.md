# Solution 01: Why Secrets Rotation Matters

## Part A: Threat Analysis

### 1. Blast Radius

The blast radius includes:

- **Direct access**: Full read/write to the payments database, including PII, payment
  records, and transaction history.
- **Lateral movement**: If the database password is reused elsewhere (common in
  microservices), the attacker may gain access to adjacent systems.
- **Data exfiltration**: Attacker can dump the entire database at leisure since the
  credential is valid indefinitely.
- **Data manipulation**: Attacker can alter payment records, create fraudulent
  transactions, or delete audit logs.
- **Compliance violation**: PCI-DSS requires credential management controls; a static
  compromised password is a direct violation.
- **Trust chain compromise**: Downstream systems that trust payment data integrity are
  now unreliable.

### 2. With 24-Hour Rotation

- The compromised credential would expire within 24 hours at most.
- The attacker's window of access is bounded to 0-24 hours depending on when the laptop
  was compromised relative to the last rotation.
- Average exposure window: 12 hours (half the rotation interval).
- The former employee's credential becomes inert automatically -- no manual intervention
  needed.
- Detection systems have a bounded window to identify anomalous activity.

### 3. Additional Detection Mechanisms

Even without rotation, these help:

- **Connection anomaly detection**: Alert on database connections from unusual IP
  addresses or at unusual times.
- **Query pattern analysis**: Detect bulk SELECT statements or unusual write patterns.
- **Credential usage monitoring**: Track which credentials are used from where; alert on
  the same credential used from multiple locations.
- **Honeypot records**: Insert fake records that trigger alerts when accessed.
- **Network segmentation**: Ensure the database is not directly accessible from the
  internet or untrusted networks.

---

## Part B: Rotation Strategy Classification

| # | Strategy | Type | Advantage | Disadvantage |
|---|----------|------|-----------|--------------|
| 1 | Rotate API keys every 90 days | Time-based | Predictable schedule, easy to automate | 90-day window still allows significant exposure |
| 2 | Rotate DB password after employee offboards | Event-based | Immediate response to specific threat | Only covers one threat vector; misses others |
| 3 | TLS certs every 30 days AND on anomaly | Hybrid | Combines predictable schedule with reactive response | More complex to implement; anomaly detection has false positives |
| 4 | Rotate encryption keys after 1M operations | Event-based | Tied to actual usage volume, not time | Unpredictable timing; hard to plan maintenance windows |
| 5 | Service tokens every 24 hours AND on deploy | Hybrid | Very short exposure window; deploy-triggered covers code changes | High operational overhead; requires robust automation |

---

## Part C: Risk Mapping

| Secret Type | Urgency (1-5) | Justification |
|-------------|---------------|---------------|
| Internal monitoring dashboard read-only API key | 2 | Low privilege (read-only), internal only, but still a vector for reconnaissance |
| Production database admin password | 5 | Full access to all data, can modify or destroy records, highest blast radius |
| JWT signing key for user authentication | 5 | Compromise allows forging tokens for any user; undetectable without key rotation |
| CI/CD deploy token with write access to production | 5 | Direct path to production code execution; supply chain attack vector |
| Staging environment test fixture key | 1 | No production data, low privilege, but may reveal architecture to attacker |

**Key insight**: Urgency correlates with (privilege level * blast radius * likelihood of
targeting). A monitoring key with low privilege but high exposure (internet-facing) might
rank higher than a staging key with low exposure.

---

## Part D: Short Answers

### 1. Credential Half-Life

The credential half-life is the expected time an attacker has access if a credential is
compromised at a uniformly random point during its validity period. For a credential with
a rotation interval of T, the half-life is T/2 on average.

This matters for rotation policy design because:

- It quantifies the **expected** exposure, not the worst case.
- It helps set rotation intervals proportional to the sensitivity of the secret.
- A 90-day rotation yields an average 45-day exposure window -- often unacceptable for
  high-privilege credentials.
- It justifies shorter rotation intervals for critical secrets: a 1-hour rotation gives
  a 30-minute average exposure, which may be acceptable even for highly sensitive secrets.

### 2. Rotation vs. Rekeying

They are **not the same**:

- **Rotation** replaces a credential (e.g., password, API key) with a new one. The old
  credential is revoked. Data encrypted with the old key may need re-encryption.
- **Rekeying** changes the encryption key used to protect stored secrets. The underlying
  secrets may not change, but the key that encrypts them does. Rekeying is often done to
  comply with key management policies or to recover from a suspected key compromise.

In practice, rotation of an encryption key often involves rekeying the data it protects.
But rotating a database password does not involve rekeying anything -- it is purely a
credential swap.

### 3. Audit Trail Importance

Audit trails for rotation events are critical because:

- **Forensics**: If a breach occurs, the audit trail shows when secrets changed, helping
  determine the exposure window.
- **Compliance**: Standards like PCI-DSS, SOC 2, and ISO 27001 require evidence of
  credential lifecycle management.
- **Debugging**: If an application fails after rotation, the audit trail shows which
  secret version was active at the time of failure.
- **Accountability**: Shows who triggered manual rotations and why.
- **Automation verification**: Proves that automated rotation is actually happening on
  schedule.
