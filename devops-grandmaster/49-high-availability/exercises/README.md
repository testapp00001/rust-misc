# Module 49: High Availability -- Exercises

## Overview

These exercises build your understanding of high availability patterns from foundational concepts through production-grade architectures. You will work with active-passive and active-active configurations, consensus protocols, and zero-downtime operational patterns used in mission-critical systems.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | HA Pattern Selection | Conceptual | 15 min | Easy |
| 02 | Active-Passive Cluster Setup | Guided | 30 min | Easy-Medium |
| 03 | Active-Active Configuration | Independent | 30 min | Medium |
| 04 | Consensus Protocol Implementation | Challenge | 45 min | Medium-Hard |
| 05 | HA Architecture for Zero-Downtime Operations | Integration | 45 min | Hard |

## Getting Started

Work through the exercises in order. Each exercise builds on concepts from the previous one. Solutions are available in the `../solutions/` directory, but attempt each exercise before reviewing the solution.

## Prerequisites

- Two Linux VMs or containers for exercises 02-05
- PostgreSQL 15+ installed on both nodes
- `keepalived` package available for exercise 02
- `etcd` v3.5+ for exercise 04
- `patroni` Python package for exercise 04
- Basic understanding of TCP/IP networking and Linux system administration
