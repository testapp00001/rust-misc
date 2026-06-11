# Module 54: Secrets Rotation - Exercises

## Overview

These exercises teach you how to implement automated secret rotation, design rotation
strategies, and achieve zero-downtime credential cycling in production systems.

## Prerequisites

- Docker and Docker Compose installed
- Rust toolchain (1.70+)
- Basic understanding of cryptography and hashing
- Familiarity with environment variables and configuration management

## Exercises

| Exercise | Title | Difficulty | Time |
|----------|-------|------------|------|
| 01 | Why Secrets Rotation Matters | Conceptual | 20 min |
| 02 | Implement Basic Secret Rotation | Guided | 40 min |
| 03 | Zero-Downtime Rotation Strategy | Independent | 50 min |
| 04 | Automated Rotation with Vault | Challenge | 60 min |
| 05 | Secrets Rotation in CI/CD Pipeline | Integration | 60 min |

## How to Use These Exercises

1. Read the objective and background for each exercise carefully.
2. Follow the instructions step by step.
3. Check your work against the success criteria before moving on.
4. Use hints only when you are stuck -- try first.
5. Compare your approach with the solutions in `solutions/`.

## Key Concepts Covered

- **Credential Lifecycle**: Creation, active use, rotation, revocation.
- **Rotation Strategies**: Time-based, event-based, and hybrid rotation.
- **Zero-Downtime Rotation**: Overlapping validity windows for seamless transitions.
- **Vault Integration**: Using HashiCorp Vault for automated secret management.
- **CI/CD Integration**: Injecting rotating secrets into build and deploy pipelines.
