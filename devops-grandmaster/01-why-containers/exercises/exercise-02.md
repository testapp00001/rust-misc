# Exercise 02: Build a Reproducibility Problem, Then Fix It

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create a concrete, reproducible "it works on my machine" scenario using shell
scripts, then solve it using the principles behind containers -- bundling the
application with its exact runtime environment.

## Prerequisites

- A Linux or macOS terminal
- Python 3 installed (any version)
- `bash` or `zsh`

## Part 1: Create the Problem

### Step 1: Write an application that depends on a specific Python version feature

Create a file called `app.py`:

```python
#!/usr/bin/env python3
"""
A simple app that uses a feature introduced in Python 3.10+.
The `match` statement (structural pattern matching).
"""

import sys
import platform

def classify_status(code):
    match code:
        case 200:
            return "OK"
        case 404:
            return "Not Found"
        case 500:
            return "Internal Server Error"
        case _:
            return f"Unknown ({code})"

if __name__ == "__main__":
    print(f"Python {sys.version}")
    print(f"Platform: {platform.platform()}")
    print()

    for code in [200, 301, 404, 500]:
        print(f"  HTTP {code}: {classify_status(code)}")
```

### Step 2: Run it and note your Python version

```bash
python3 app.py
python3 --version
```

### Step 3: Create a `requirements.txt` that does NOT pin Python version

```
# requirements.txt
# This file pins packages but says nothing about the Python version.
requests==2.31.0
```

### Step 4: Write down why this setup will fail on a different machine

Answer these questions in a file called `analysis.md`:

1. What Python version does the `match` statement require?
2. If someone runs this on Python 3.8, what error will they see?
3. Does `requirements.txt` protect against this?
4. What is missing from this "reproducibility" setup?

<details>
<summary>Hint</summary>

The `match` statement was introduced in Python 3.10 (PEP 634).
On Python 3.8 or 3.9, you will get a `SyntaxError` because the parser
does not recognize the `match` keyword.
`requirements.txt` only pins *packages*, not the Python interpreter itself.

</details>

## Part 2: The Naive Fix -- Documentation

### Step 5: Create an `INSTALL.md` with setup instructions

Write a file called `INSTALL.md` that documents everything someone needs
to run `app.py` on a fresh machine.

<details>
<summary>Hint</summary>

Your instructions should cover:
- Required Python version (3.10+)
- How to install Python if missing
- How to create a virtual environment
- How to install dependencies
- How to run the app

</details>

### Step 6: Identify what can go wrong with INSTALL.md

Write in your `analysis.md` file at least **three** reasons why `INSTALL.md`
will still lead to "it works on my machine" problems.

<details>
<summary>Hint</summary>

Think about:
- People skipping steps or reading outdated instructions
- Different OS requiring different install commands
- System-level dependencies not mentioned in the file
- Python version managers (pyenv, brew, apt) giving different patch versions

</details>

## Part 3: The Container Idea

### Step 7: Write a Dockerfile (conceptually)

Even if you do not have Docker installed yet, write a file called `Dockerfile`
that would solve the reproducibility problem. It should:

1. Start from a specific Python version
2. Copy the application code
3. Install dependencies
4. Define the command to run

<details>
<summary>Hint</summary>

```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY app.py .
CMD ["python3", "app.py"]
```

The key insight: `FROM python:3.11-slim` pins the exact Python version
and OS in one line. No INSTALL.md needed.

</details>

### Step 8: Compare the approaches

Fill in this table in your `analysis.md`:

| Concern | INSTALL.md | Dockerfile |
|---------|-----------|------------|
| Python version pinned? | | |
| OS dependencies captured? | | |
| Works on macOS and Linux? | | |
| Requires human to follow steps? | | |
| Guaranteed identical runtime? | | |

## Success Criteria

- [ ] `app.py` runs on your machine and demonstrates a 3.10+ feature
- [ ] `analysis.md` explains why this fails on older Python
- [ ] `INSTALL.md` documents the setup steps
- [ ] `analysis.md` identifies at least 3 failure modes for INSTALL.md
- [ ] `Dockerfile` pins the Python version and installs dependencies
- [ ] The comparison table is filled in and accurate

## What You Should Understand After This Exercise

Documentation is necessary but not sufficient for reproducibility.
A Dockerfile captures the *entire runtime environment* in a single file --
OS, language version, packages, and configuration -- making "it works on my
machine" a solved problem rather than a recurring one.
