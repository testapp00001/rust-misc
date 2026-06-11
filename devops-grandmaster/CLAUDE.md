# CLAUDE.md — DevOps Grandmaster Project Conventions

## Project Overview

This is a comprehensive DevOps learning curriculum — 81 modules from "it works on my machine"
to "I run the whole show." The goal: anyone who completes this becomes a DevOps Legendary
Grandmaster.

## Repository Structure

```
devops-grandmaster/
├── README.md                    # Master curriculum map
├── REVIEW.md                    # Project audit & transformation plan
├── CLAUDE.md                    # This file — project conventions
├── CONTRIBUTING.md              # Contributor guidelines
├── CHECKLIST.md                 # Progress tracking
└── XX-module-name/
    ├── README.md                # The lesson (read first)
    ├── cheatsheet.md            # Quick reference card
    ├── examples/                # Working, runnable code
    │   ├── README.md            # How to run the examples
    │   └── ...                  # Actual code files
    ├── exercises/               # Hands-on practice
    │   ├── README.md            # Exercise instructions
    │   ├── exercise-01.md       # Conceptual
    │   ├── exercise-02.md       # Guided
    │   ├── exercise-03.md       # Independent
    │   ├── exercise-04.md       # Challenge
    │   └── exercise-05.md       # Integration
    └── solutions/               # Reference solutions
        ├── README.md            # Solution overview
        ├── solution-01.md       # With detailed explanations
        ├── solution-02.md
        ├── solution-03.md
        ├── solution-04.md
        └── solution-05.md
```

## Content Standards

### README.md (The Lesson)
- Follow the pattern: Problem → Naive Way → Right Way → Production Way → Hands-On Lab → Limitation → Next
- Use ASCII diagrams for architecture visualization
- Include inline code examples that are copy-pasteable
- End with "Limitation" section that bridges to the next module
- Target: 600-1400 lines per module README

### cheatsheet.md
- One-page quick reference
- Commands, not explanations
- Grouped by task/scenario
- Target: 50-150 lines

### examples/
- Every example must be **runnable** — no pseudocode
- Include a README.md explaining prerequisites and how to run
- Use realistic scenarios, not "hello world"
- Support Linux (primary) and macOS (secondary)

### exercises/
- 5 exercises per module, escalating difficulty:
  1. **Conceptual** — verify understanding of the "why"
  2. **Guided** — step-by-step with hints
  3. **Independent** — solve from scratch
  4. **Challenge** — production-scenario difficulty
  5. **Integration** — combines this module with previous ones
- Each exercise has clear success criteria
- Include hints that students can reveal

### solutions/
- Complete, working solutions with detailed explanations
- Explain WHY, not just WHAT
- Include common mistakes and how to avoid them
- Reference the relevant README sections

## Formatting Rules

- Use `bash`, `yaml`, `python`, `go`, `rust`, `javascript` code fences (never generic)
- ASCII diagrams in ``` blocks with no language tag
- Module references: `[Module XX: Title](../XX-module-name/)`
- Internal links use relative paths
- No emojis in code blocks or file names
- Headers: `#` for title, `##` for sections, `###` for subsections

## Phases

| Phase | Modules | Theme |
|-------|---------|-------|
| 1 | 01-10 | Containerization Foundations |
| 2 | 11-17 | Production Containers |
| 3 | 18-23 | Scaling Basics |
| 4 | 24-38 | Deep Kubernetes |
| 5 | 39-44 | Observability |
| 6 | 45-50 | Database Operations |
| 7 | 51-56 | Security |
| 8 | 57-61 | CI/CD Mastery |
| 9 | 62-66 | Networking |
| 10 | 67-71 | Extreme Scaling |
| 11 | 72-75 | Cluster & Multi-Host |
| 12 | 76-80 | Production Mastery |
| Capstone | 81 | The Unbreakable System |

## Quality Checklist

Before marking any module as complete, verify:
- [ ] README follows the Problem→Naive→Right→Production→Lab→Limitation pattern
- [ ] cheatsheet.md exists and is concise
- [ ] examples/ has runnable code with a README
- [ ] exercises/ has 5 exercises with clear success criteria
- [ ] solutions/ has 5 solutions with explanations
- [ ] All code examples are tested and working
- [ ] Cross-references to related modules are present
- [ ] Difficulty and time estimate are stated

## Commands

```bash
# Count total lines across all markdown
find . -name '*.md' -not -path './.claude/*' -not -path './.git/*' | xargs wc -l

# List modules missing cheatsheets
for d in [0-9]*/; do [ ! -f "$d/cheatsheet.md" ] && echo "$d"; done

# List modules with empty examples
for d in [0-9]*/examples; do [ -z "$(ls -A "$d" 2>/dev/null)" ] && echo "$d"; done

# Check exercise/solution coverage
echo "Exercises: $(find */exercises -type f 2>/dev/null | wc -l)"
echo "Solutions: $(find */solutions -type f 2>/dev/null | wc -l)"
```
