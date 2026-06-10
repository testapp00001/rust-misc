# Module 23: Threat Modeling

> "Threat modeling is the practice of systematically identifying what can go wrong in your system's security — before an attacker shows you."

## Overview

Threat modeling is the foundational discipline of security engineering. It answers four questions:

1. **What are we building?** — Understand the system (data flows, trust boundaries, entry points)
2. **What can go wrong?** — Identify threats (STRIDE, attack trees, threat enumeration)
3. **What are we going to do about it?** — Define mitigations (defense in depth, security requirements)
4. **Did we do a good enough job?** — Validate the model (risk assessment, design review)

This module teaches you to think like a threat modeler — decomposing systems, enumerating threats systematically, scoring risks, and deriving testable security requirements.

## Key Concepts

### STRIDE Threat Model

STRIDE is Microsoft's mnemonic for six categories of threats:

```
S — Spoofing            Impersonating a user or system
T — Tampering           Modifying data or code
R — Repudiation         Denying an action was performed
I — Information Disclosure  Exposing data to unauthorized parties
D — Denial of Service   Making a system unavailable
E — Elevation of Privilege  Gaining unauthorized access level
```

Every threat in a system maps to at least one STRIDE category. The model ensures you don't overlook entire classes of attacks.

### Attack Trees

Attack trees decompose a high-level attack goal into sub-goals, forming a tree structure:

```
                    [Steal User Data]
                    /       |        \
           [SQL Injection] [XSS]  [Social Engineering]
           /        \        |
    [Union Query] [Blind]  [Stored XSS]
```

Each leaf node is a concrete attack step. By evaluating the cost and feasibility of leaf nodes, you can identify the **weakest path** — the cheapest, easiest attack route.

### DREAD Scoring

DREAD scores threats on five dimensions (each 1-10):

```
D — Damage             How bad is the impact?
R — Reproducibility    How easy is it to reproduce?
E — Exploitability     How much skill/effort is needed?
A — Affected Users     How many users are impacted?
D — Discoverability    How easy is it to find the vulnerability?

Risk Score = (D + R + E + A + D) / 5
```

### Data Flow Diagrams

A Data Flow Diagram (DFD) for security identifies:

- **External entities** (users, APIs, third-party services)
- **Processes** (application components, services)
- **Data stores** (databases, files, caches)
- **Data flows** (connections between components)
- **Trust boundaries** (where privilege levels change)

Threats arise at trust boundary crossings — where data moves from a less-trusted to a more-trusted zone.

### Security by Design

Threat modeling is not a one-time activity. It should be:
- **Continuous**: Re-evaluated when architecture changes
- **Collaborative**: Involves developers, ops, and security teams
- **Documented**: Threat models are living documents
- **Actionable**: Every identified threat has a mitigation or accepted risk

## Attack Patterns

### Missing Threat Category

Without STRIDE, teams often focus only on confidentiality (Information Disclosure) and forget about:
- **Tampering**: An attacker modifies a config file to elevate privileges
- **Repudiation**: A user denies performing a malicious action; no audit trail exists
- **Spoofing**: An attacker forges authentication tokens

### Shallow Attack Tree

An incomplete attack tree misses the cheapest attack path. For example, focusing on complex technical attacks while ignoring:
- **Social engineering**: Tricking an employee into revealing credentials
- **Physical access**: Stealing an unlocked laptop
- **Supply chain**: Compromising a dependency

### Ignoring Trust Boundaries

When you don't draw trust boundaries, you can't see where validation is missing:
- External input crosses into internal processing without sanitization
- A low-privilege service can call a high-privilege API directly
- Database credentials are embedded in client-side code

### Risk Score Manipulation

DREAD scores can be gamed by:
- **Anchoring**: Starting from a biased estimate
- **Optimism bias**: Underestimating exploitability
- **Scope creep**: Changing "Affected Users" definition to minimize scores

Defense: Use calibrated scoring with historical data and multiple reviewers.

## Rust-Specific Tips

1. **Serde for threat models**: Use `serde` + `serde_json` to serialize threat models, attack trees, and DREAD scores as structured data
2. **Enums for STRIDE categories**: Rust's `enum` with `match` ensures exhaustive handling of all threat categories
3. **Type-safe trust boundaries**: Use newtypes to distinguish data from different trust zones
4. **Builder pattern for attack trees**: Use the builder pattern to construct trees ergonomically
5. **Test-driven threat modeling**: Write tests that verify all STRIDE categories are covered
6. **Display trait for reports**: Implement `Display` to generate human-readable threat reports

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_stride_model.rs` | STRIDE threat model | Spoofing, Tampering, Repudiation, Information Disclosure, DoS, Elevation of Privilege |
| 02 | `p02_attack_trees.rs` | Attack trees | Decompose attacks into sub-goals, find weakest path |
| 03 | `p03_dread_scoring.rs` | DREAD scoring | Damage, Reproducibility, Exploitability, Affected Users, Discoverability |
| 04 | `p04_data_flow_analysis.rs` | Data flow analysis | Trust boundaries, data flows, entry points |
| 05 | `p05_asset_identification.rs` | Asset identification | What needs protection, value assessment |
| 06 | `p06_threat_enumeration.rs` | Threat enumeration | Systematically list all possible attacks |
| 07 | `p07_mitigation_strategies.rs` | Mitigation strategies | Defense in depth, layered security |
| 08 | `p08_security_requirements.rs` | Security requirements | Derive testable requirements from threat model |
| 09 | `p09_risk_assessment.rs` | Risk assessment | Likelihood x impact, risk tolerance |
| 10 | `p10_security_design_review.rs` | Security design review | Architecture review, threat model validation |

## Quick Test

```bash
# Test your implementation
cargo test -p 23-threat-modeling

# Test reference solution
cargo test -p 23-threat-modeling --features solution
```

## References

- [Microsoft STRIDE Threat Model](https://learn.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [Attack Trees (Schneier, 1999)](https://www.schneier.com/academic/archives/1999/12/attack_trees.html)
- [OWASP Threat Modeling](https://owasp.org/www-community/Threat_Modeling)
- [NIST SP 800-154: Guide to Data-Centric System Threat Modeling](https://csrc.nist.gov/publications/detail/sp/800-154/draft)
- [Threat Modeling Manifesto](https://www.threatmodelingmanifesto.org/)
- [Adam Shostack's Threat Modeling: Designing for Security](https://www.wiley.com/en-us/Threat+Modeling%3A+Designing+for+Security-p-9781118809969)
