# Module 54: Secrets Rotation - Solutions

## Overview

This directory contains reference solutions for all five exercises in Module 54. Each
solution includes complete explanations and working code where applicable.

**Important**: These solutions are one correct approach among many. If your solution
differs but meets the success criteria, it is valid.

## Solutions

| Solution | Exercise | Summary |
|----------|----------|---------|
| [solution-01.md](solution-01.md) | Why Secrets Rotation Matters | Threat analysis, strategy classification, risk mapping |
| [solution-02.md](solution-02.md) | Implement Basic Secret Rotation | Complete Rust implementation of `SecretRotator` |
| [solution-03.md](solution-03.md) | Zero-Downtime Rotation Strategy | Design doc + `DualSecretStore` with async consumers |
| [solution-04.md](solution-04.md) | Automated Rotation with Vault | Vault client, KV v2 + Transit integration |
| [solution-05.md](solution-05.md) | Secrets Rotation in CI/CD Pipeline | CLI tool with security guards and audit logging |

## How to Use

1. Attempt each exercise before reading the solution.
2. Compare your approach with the solution's approach.
3. Focus on understanding the *why* behind design decisions, not just the code.
4. If your solution is different but correct, that is fine -- there are multiple valid
   approaches to secrets rotation.
