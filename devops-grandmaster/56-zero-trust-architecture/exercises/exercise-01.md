# Exercise 01: Zero Trust vs Castle-and-Moat

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand the fundamental difference between traditional perimeter-based security (castle-and-moat) and zero trust architecture. Analyze a real-world security incident and explain how zero trust principles would have changed the outcome.

## Tasks

### Part A: Compare the Two Models

Study the following two architecture diagrams. For each security control listed, identify whether it exists in the castle-and-moat model, the zero trust model, or both.

**Castle-and-Moat Architecture:**

```
                        INTERNET
                           |
                    [ FireWall / DMZ ]
                           |
                   +-------+-------+
                   |               |
               [ Web App ]   [ VPN Gateway ]
                   |               |
                   +-------+-------+
                           |
                   INTERNAL NETWORK
                   (TRUSTED ZONE)
                   |       |       |
               [ API ] [ DB ] [ Admin ]
```

**Zero Trust Architecture:**

```
                        INTERNET
                           |
                    [ Identity Provider ]
                    [ Policy Engine    ]
                    [ mTLS Gateway     ]
                           |
           +-------+-------+-------+-------+
           |       |       |       |       |
       [ Front ] [ API ] [ DB  ] [ Admin ] [ Svc C ]
           |       |       |       |       |
        mTLS     mTLS    mTLS    mTLS    mTLS
        +Auth    +Auth   +Auth   +Auth   +Auth
```

Complete the comparison table:

| Security Control | Castle-and-Moat | Zero Trust | Why? |
|-----------------|-----------------|------------|------|
| Network perimeter firewall | ? | ? | ? |
| Mutual authentication between services | ? | ? | ? |
| VPN required for internal access | ? | ? | ? |
| Least privilege per service | ? | ? | ? |
| Encrypted internal traffic | ? | ? | ? |
| Continuous identity verification | ? | ? | ? |
| Micro-segmentation | ? | ? | ? |
| Assume breach mentality | ? | ? | ? |

<details><summary>Hint</summary>Castle-and-moat trusts everything inside the perimeter. Zero trust trusts nothing, regardless of network location. Think about what happens to traffic once it passes the firewall in each model.</details>

### Part B: Analyze a Breach Scenario

Read the following incident report:

```
INCIDENT REPORT: IR-2025-0847
Date: 2025-03-15
Summary:
  An attacker compromised a developer's laptop via a phishing email.
  The laptop was connected to the corporate VPN.
  From the laptop, the attacker scanned the internal network and
  discovered an unpatched Jenkins server (10.0.5.12:8080).
  The attacker exploited CVE-2024-XXXX to gain remote code execution
  on Jenkins. From Jenkins, the attacker accessed environment variables
  containing database credentials. The attacker exfiltrated 2M customer
  records from the production database (10.0.10.50:5432).

Timeline:
  09:15 - Phishing email opened
  09:22 - Malware installed on developer laptop
  10:05 - VPN connection established (attacker inherited trust)
  10:47 - Internal network scan completed
  11:30 - Jenkins server compromised
  11:45 - Database credentials extracted from Jenkins env vars
  12:10 - Data exfiltration began
  14:30 - Anomaly detected by database monitoring

Impact: 2M customer records exfiltrated
Root cause: Flat internal network with implicit trust
```

For each stage of the attack, identify:
1. Which zero trust principle was violated
2. What zero trust control would have prevented or detected the step
3. How the blast radius would have been reduced

<details><summary>Hint</summary>Map each attack step to zero trust principles: "never trust, always verify," "least privilege," and "assume breach." Consider mTLS, micro-segmentation, identity verification, and continuous monitoring.</details>

### Part C: Design the Zero Trust Alternative

Rewrite the incident timeline as if zero trust architecture had been in place. For each original attack step, describe:
- What the attacker would encounter
- Why the attack would fail or be detected
- What the security team would see

<details><summary>Hint</summary>Think about: device posture checks before VPN access, service identity verification via mTLS, micro-segmentation preventing lateral movement, secrets not stored in environment variables, continuous monitoring detecting anomalous behavior patterns.</details>

## Success Criteria

- [ ] Completed comparison table with correct categorizations for all 8 controls
- [ ] Identified at least 3 zero trust violations in the breach scenario
- [ ] Mapped each attack step to a specific zero trust control
- [ ] Described a realistic zero trust alternative timeline
- [ ] Can articulate why "network location != trust" is the core zero trust principle

## What You Should Understand After This Exercise

- The fundamental difference between perimeter-based and zero trust security models
- Why implicit trust based on network location is a critical vulnerability
- How the three pillars of zero trust (verify explicitly, least privilege, assume breach) apply to real incidents
- That zero trust is not a single product but an architectural philosophy
- How zero trust limits blast radius even when individual components are compromised
