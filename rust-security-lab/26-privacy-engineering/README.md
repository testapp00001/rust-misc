# Module 26: Privacy Engineering

> "Privacy is not about having something to hide. Privacy is about having the right to control what you reveal about yourself and when." -- Bruce Schneier

## Overview

Privacy engineering is the practice of building systems that protect personal data by design. This module covers:
- **Differential Privacy**: Adding calibrated noise to queries so individual records cannot be identified
- **k-Anonymity**: Ensuring each record is indistinguishable from at least k-1 others
- **Data Minimization**: Collecting only what you need, deleting when done
- **Anonymization**: Removing PII, pseudonymization, re-identification risk assessment
- **GDPR Compliance**: Right to erasure, data portability, consent management
- **Privacy by Design**: Embedding privacy into system architecture from the start
- **Data Retention**: Automated deletion policies and compliance enforcement
- **Consent Management**: Granular consent, withdrawal, and audit trails
- **Privacy-Preserving Computation**: Secure aggregation without exposing individual data
- **Privacy Impact Assessment**: Identifying and mitigating privacy risks systematically

## Key Concepts

### Differential Privacy

Differential privacy provides a mathematical guarantee that the output of a query does not reveal whether any individual's data was included. The key parameter is **epsilon** (e):

```
Small epsilon  -->  more noise  -->  more privacy  -->  less accuracy
Large epsilon  -->  less noise  -->  less privacy  -->  more accuracy
```

A mechanism M satisfies (e, delta)-differential privacy if for all neighboring datasets D and D' (differing by one record):

```
P[M(D) in S] <= e^epsilon * P[M(D') in S] + delta
```

### k-Anonymity

A dataset satisfies k-anonymity if every combination of quasi-identifier values (age, zip code, gender, etc.) appears at least k times. This prevents linking attacks where an adversary uses external data to re-identify individuals.

```
k=1  -->  no protection (each record unique)
k=5  -->  each record matches at least 4 others on quasi-identifiers
k=100 --> strong protection but may lose data utility
```

### Data Minimization (GDPR Article 5(1)(c))

Collect only data that is:
1. **Adequate** -- sufficient for the stated purpose
2. **Relevant** -- connected to the purpose
3. **Limited** -- no more than necessary

### Privacy by Design (7 Foundational Principles)

1. Proactive not reactive; preventative not remedial
2. Privacy as the default setting
3. Privacy embedded into design
4. Full functionality -- positive-sum, not zero-sum
5. End-to-end security -- full lifecycle protection
6. Visibility and transparency
7. Respect for user privacy -- keep it user-centric

### GDPR Technical Requirements

| Right | Technical Implementation |
|-------|------------------------|
| Right to erasure (Art. 17) | Delete personal data on request, propagate to backups |
| Data portability (Art. 20) | Export data in machine-readable format (JSON/CSV) |
| Consent (Art. 7) | Record consent with timestamp, allow withdrawal |
| Data protection by design (Art. 25) | Implement minimization, pseudonymization, encryption |

## Attack Patterns

### Re-identification Attack

Even after removing names, individuals can be re-identified by combining quasi-identifiers (zip code + birth date + gender). The 1997 Massachusetts Governor health records attack demonstrated this -- 87% of Americans could be uniquely identified by these three fields.

### Linkage Attack

An adversary links an "anonymized" dataset with external data sources. Example: linking Netflix prize data with IMDB reviews to de-anonymize users.

### Inference Attack

Even with differential privacy, repeated queries on overlapping data can leak information. The privacy budget must be tracked across all queries.

## Rust-Specific Tips

1. Use `rand::distributions` for noise generation (Laplace, Gaussian)
2. Use `serde`/`serde_json` for data serialization and GDPR data export
3. Use `sha2` for pseudonymization (hashing identifiers with a secret salt)
4. Use `ring` for encryption of personal data at rest
5. Use `HashMap` for consent storage and audit trails
6. Use `chrono` for timestamps in retention policies (workspace dependency available)

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_differential_privacy.rs` | Differential Privacy | Laplace mechanism, epsilon budget |
| 02 | `p02_k_anonymity.rs` | k-Anonymity | Quasi-identifiers, generalization |
| 03 | `p03_data_minimization.rs` | Data Minimization | Collect only what's needed |
| 04 | `p04_anonymization.rs` | Anonymization | PII removal, pseudonymization |
| 05 | `p05_gdpr_technical.rs` | GDPR Technical | Erasure, portability, consent |
| 06 | `p06_privacy_by_design.rs` | Privacy by Design | Architecture-level privacy |
| 07 | `p07_data_retention.rs` | Data Retention | Automated deletion, compliance |
| 08 | `p08_consent_management.rs` | Consent Management | Granular consent, audit trail |
| 09 | `p09_privacy_preserving_computation.rs` | Privacy-Preserving Computation | Secure aggregation |
| 10 | `p10_privacy_impact.rs` | Privacy Impact Assessment | Risk identification and mitigation |

## Usage

```bash
# Test your implementation (exercise stubs)
cargo test -p privacy_engineering

# Test the reference solution
cargo test -p privacy_engineering --features solution
```

## References

- [Dwork, "The Algorithmic Foundations of Differential Privacy"](https://www.cis.upenn.edu/~aaroth/Papers/privacybook.pdf)
- [Sweeney, "k-Anonymity: A Model for Protecting Privacy"](https://dataprivacylab.org/dataprivacy/projects/kanonymity/kanonymity.pdf)
- [GDPR Full Text](https://gdpr-info.eu/)
- [ICO Privacy by Design Guide](https://ico.org.uk/for-organisations/guide-to-data-protection/guide-to-the-general-data-protection-regulation-gdpr/accountability-and-governance/data-protection-by-design-and-default/)
- [NIST Privacy Framework](https://www.nist.gov/privacy-framework)
