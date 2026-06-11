# Module 09: Volumes and Data -- Exercises

These exercises build your understanding of Docker data persistence from
conceptual foundations through production-grade backup strategies.

## Exercise Overview

| # | Title | Type | Estimated Time |
|---|-------|------|----------------|
| 01 | Volume Types Explained | Conceptual | 20 min |
| 02 | Persisting Database Data | Guided | 30 min |
| 03 | Backup and Restore Strategy | Independent | 45 min |
| 04 | Stateful Application Data Design | Challenge | 60 min |
| 05 | Multi-Service Compose with Volumes | Integration | 45 min |

## How to Use These Exercises

1. Work through them in order -- each builds on the previous one.
2. Read the objective and instructions carefully before starting.
3. Use the hints only after you have attempted the exercise yourself.
4. Check your work against the success criteria listed at the end of each exercise.
5. Compare your solution with the corresponding file in `../solutions/`.

## Prerequisites

- Docker Engine 24+ installed and running
- Docker Compose v2 installed
- Basic familiarity with Dockerfile and `docker run`
- A terminal and a text editor

## Getting Help

If you get stuck:

- Re-read the relevant section of the module notes.
- Check `docker volume --help` and `docker run --mount` documentation.
- Look at the hint sections (hidden behind `<details>` tags).
- As a last resort, review the solution file -- but try to understand *why*
  the solution works, not just copy it.
