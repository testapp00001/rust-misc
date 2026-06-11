# Contributing to DevOps Grandmaster

Thank you for helping build the ultimate DevOps learning resource.

## How to Contribute

### 1. Report Issues
- Broken examples or commands that don't work
- Outdated information or deprecated tools
- Typos, unclear explanations, or missing context
- Missing topics that should be covered

### 2. Improve Existing Modules
- Fix bugs in example code
- Add real-world context to explanations
- Improve exercises with better scenarios
- Add alternative approaches with trade-off analysis

### 3. Add New Content
- Fill empty `examples/` directories with working code
- Add exercises and solutions to modules
- Create new modules for missing topics
- Add cross-references between related modules

## Module Structure

Every module must follow this structure:

```
XX-module-name/
├── README.md          # The lesson (Problem → Naive → Right → Production → Lab → Limitation → Next)
├── cheatsheet.md      # Quick reference (50-150 lines)
├── examples/          # Working, runnable code
│   ├── README.md      # How to run the examples
│   └── ...            # Actual code files
├── exercises/         # Hands-on practice (5 exercises)
│   ├── README.md      # Exercise overview
│   ├── exercise-01.md # Conceptual
│   ├── exercise-02.md # Guided
│   ├── exercise-03.md # Independent
│   ├── exercise-04.md # Challenge
│   └── exercise-05.md # Integration
└── solutions/         # Reference solutions (5 solutions)
    ├── README.md      # Solution overview
    ├── solution-01.md # With detailed explanations
    └── ...
```

## Quality Standards

### README.md
- Follow the Problem → Naive Way → Right Way → Production Way → Hands-On Lab → Limitation → Next pattern
- Use ASCII diagrams for architecture visualization
- Include copy-pasteable code examples
- Target: 600-1400 lines

### cheatsheet.md
- One-page quick reference
- Commands, not explanations
- Grouped by task/scenario
- Target: 50-150 lines

### examples/
- Every example must be runnable (no pseudocode)
- Include a README.md with prerequisites and run instructions
- Use realistic scenarios
- Support Linux (primary) and macOS (secondary)

### exercises/
- 5 exercises per module, escalating difficulty
- Clear success criteria
- Hints in `<details>` tags

### solutions/
- Complete, working solutions
- Explain WHY, not just WHAT
- Reference relevant README sections
- Include common mistakes

## Code Style

### Bash
```bash
# Use shellcheck-clean scripts
set -euo pipefail
```

### Dockerfiles
```dockerfile
# Use specific base image tags
FROM python:3.12-slim
# Not: FROM python:latest
```

### YAML (Kubernetes, Docker Compose)
```yaml
# Use 2-space indentation
# Include comments explaining non-obvious fields
```

## Commit Messages

```
module-XX: Brief description of change

- What changed and why
- Any breaking changes
- Related issues
```

Examples:
```
module-04: Add Rust multi-stage build example
module-12: Fix health check endpoint in exercise-03
module-39: Update Prometheus config for v2.50
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b module-XX-improvement`
3. Make your changes following the quality standards
4. Test all examples are runnable
5. Submit a PR with a clear description

## Code of Conduct

- Be respectful and constructive
- Focus on helping learners
- Assume good intent
- Provide actionable feedback

## Questions?

Open an issue with the `question` label.
