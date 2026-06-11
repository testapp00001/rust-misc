# Exercise 01: Why Secrets Rotation Matters

**Type:** Conceptual
**Difficulty:** Beginner
**Estimated Time:** 20 minutes

## Objective

Understand why secrets rotation is a critical security practice, identify the risks of
static credentials, and classify different rotation strategies by their trade-offs.

## Background

A "secret" is any credential that grants access: API keys, database passwords, TLS
private keys, JWT signing keys, encryption keys, and service account tokens. When a
secret is compromised and never rotated, the attacker has indefinite access. Rotation
limits the window of exposure.

## Instructions

### Part A: Threat Analysis

Read the following scenario and answer the questions.

> A microservice `payment-processor` uses a static database password set once during
> deployment. Six months later, a former employee's laptop is compromised. The laptop
> contains a `.env` file with that database password. The attacker now has full read/write
> access to the payments database.

1. What is the blast radius of this compromise?
2. How would the outcome differ if the password had been rotated every 24 hours?
3. What additional detection mechanisms would help even without rotation?

### Part B: Rotation Strategy Classification

Classify each rotation strategy below as **time-based**, **event-based**, or **hybrid**.
For each, state one advantage and one disadvantage.

| # | Strategy | Type | Advantage | Disadvantage |
|---|----------|------|-----------|--------------|
| 1 | Rotate API keys every 90 days automatically | | | |
| 2 | Rotate database password immediately after an employee offboards | | | |
| 3 | Rotate TLS certificates every 30 days AND on any detected anomaly | | | |
| 4 | Rotate encryption keys after every 1 million operations | | | |
| 5 | Rotate service tokens every 24 hours AND on deploy events | | | |

### Part C: Risk Mapping

For each secret type below, rank the rotation urgency from 1 (low) to 5 (critical) and
justify your ranking.

| Secret Type | Urgency (1-5) | Justification |
|-------------|---------------|---------------|
| Internal monitoring dashboard read-only API key | | |
| Production database admin password | | |
| JWT signing key for user authentication | | |
| CI/CD deploy token with write access to production | | |
| Staging environment test fixture key | | |

### Part D: Short Answer

1. Explain the concept of a "credential half-life" and why it matters for rotation policy design.
2. What is the difference between "rotation" and "rekeying"? Are they the same thing?
3. Why is it important to have an audit trail for every rotation event?

## Success Criteria

- [ ] You can explain at least three distinct risks of static credentials.
- [ ] You correctly classify all five rotation strategies in Part B.
- [ ] Your urgency rankings in Part C are justified with concrete reasoning.
- [ ] Your short answers demonstrate understanding of rotation vs. rekeying and the
      importance of audit trails.

## Hints

<details>
<summary>Hint 1: Blast radius</summary>

Think about what the database stores, what other systems trust it, and whether lateral
movement is possible from that database access.

</details>

<details>
<summary>Hint 2: Rotation vs. rekeying</summary>

Rotation replaces a credential with a new one. Rekeying may involve changing the
underlying encryption key that protects stored secrets. They are related but distinct.

</details>

<details>
<summary>Hint 3: Credential half-life</summary>

Consider: if a credential is stolen at a random point in its lifetime, how long is the
attacker's window of access on average? That is the half-life concept applied to
credentials.

</details>
