# Module 11: Logging Strategy -- Exercises

## Overview

These exercises build your understanding of container logging from foundational
concepts through production-grade logging architecture. Complete them in order
-- each builds on the previous one.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | stdout vs stderr and Why Structured Logging Matters | Conceptual | 20 min | Easy |
| 02 | Convert Unstructured Prints to Structured JSON Logging | Guided | 30 min | Easy-Medium |
| 03 | Implement Log Levels and Filtering for a Multi-Service App | Independent | 45 min | Medium |
| 04 | Design a Logging Strategy with Rotation, Retention, and Cost Control | Challenge | 60 min | Medium-Hard |
| 05 | Set Up Docker Logging Drivers to Forward Logs to a Central System | Integration | 45 min | Hard |

## How to Use These Exercises

1. Work through them in order -- each builds on the previous one.
2. Read the objective and instructions carefully before starting.
3. Use the hints only after you have attempted the exercise yourself.
4. Check your work against the success criteria listed at the end of each exercise.
5. Compare your solution with the corresponding file in `../solutions/`.

## Prerequisites

- Docker Engine 24+ installed and running
- Docker Compose v2 installed
- Python 3.10+ installed
- Basic familiarity with `docker run`, `docker logs`, and `docker exec`
- A terminal and a text editor

## Getting Help

If you get stuck:

- Re-read the relevant section of the [module notes](../README.md).
- Check `docker logs --help` and `docker run --log-driver` documentation.
- Look at the hint sections (hidden behind `<details>` tags).
- Review the [cheatsheet](../cheatsheet.md) for quick command reference.
- As a last resort, review the solution file -- but try to understand *why*
  the solution works, not just copy it.

## Solutions

See the [solutions directory](../solutions/) for detailed solutions with
explanations.
