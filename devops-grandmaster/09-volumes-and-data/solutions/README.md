# Module 09: Volumes and Data -- Solutions

This directory contains detailed solutions for each exercise. Each solution
includes the complete answer, an explanation of why it works, and common
mistakes to avoid.

## Solutions Overview

| File | Exercise | Key Concepts |
|------|----------|--------------|
| `solution-01.md` | Volume Types Explained | Named volumes vs bind mounts vs tmpfs, `-v` vs `--mount` syntax |
| `solution-02.md` | Persisting Database Data | Named volume lifecycle, data surviving container removal |
| `solution-03.md` | Backup and Restore Strategy | Volume-to-tar backup, restore into new volume, read-only mounts |
| `solution-04.md` | Stateful Application Data Design | Choosing storage types, `pg_dump` vs tar, backup scripting |
| `solution-05.md` | Multi-Service Compose with Volumes | Compose volume syntax, profiles, healthchecks, tmpfs in Compose |

## How to Use These Solutions

1. **Attempt the exercise first.** These solutions are most useful when you
   have already struggled with the problem and understand what you got stuck on.
2. **Read the "Why It Works" section.** The commands are only half the value;
   understanding the reasoning is what makes you a better engineer.
3. **Review "Common Mistakes."** These are real errors that people make in
   production. Knowing them saves you from future debugging sessions.
