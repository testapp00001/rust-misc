# Solution 02: Build a Reproducibility Problem, Then Fix It

## Part 1: Create the Problem

### app.py

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

### Expected Output (Python 3.10+)

```
Python 3.11.4 (main, Jun  7 2023, 10:13:09) [GCC 12.3.0]
Platform: Linux-6.2.0-generic-x86_64-with-glibc2.35

  HTTP 200: OK
  HTTP 301: Unknown (301)
  HTTP 404: Not Found
  HTTP 500: Internal Server Error
```

### Expected Output (Python 3.8 or 3.9)

```
  File "app.py", line 12
    match code:
          ^
SyntaxError: invalid syntax
```

### analysis.md -- Part 1 Answers

1. **What Python version does the `match` statement require?**
   Python 3.10 or later. It was introduced in PEP 634 (Structural Pattern Matching).

2. **If someone runs this on Python 3.8, what error will they see?**
   `SyntaxError: invalid syntax` -- the `match` keyword does not exist in
   Python 3.8's parser.

3. **Does `requirements.txt` protect against this?**
   No. `requirements.txt` pins *packages*, not the Python interpreter version.
   You can install `requests==2.31.0` on Python 3.8 just fine.

4. **What is missing from this reproducibility setup?**
   The Python version itself is not captured anywhere in the project.
   There is no mechanism to enforce which Python version is used.

## Part 2: The Naive Fix -- Documentation

### INSTALL.md

```markdown
# Installation Guide

## Prerequisites

- Python 3.10 or later (required for structural pattern matching)
- pip (Python package manager)

## Setup

1. Check your Python version:
   ```bash
   python3 --version
   # Must show 3.10.x or later
   ```

2. If you need to install Python 3.10+:
   - macOS: `brew install python@3.11`
   - Ubuntu: `sudo add-apt-repository ppa:deadsnakes/ppa && sudo apt install python3.11`
   - Windows: Download from python.org

3. Create a virtual environment:
   ```bash
   python3 -m venv venv
   source venv/bin/activate  # Linux/macOS
   # venv\Scripts\activate   # Windows
   ```

4. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

5. Run the application:
   ```bash
   python3 app.py
   ```
```

### Why INSTALL.md Still Fails (Three Reasons)

**Reason 1: People do not read instructions.**
A new developer clones the repo and runs `python3 app.py` immediately.
They skip the version check, get a confusing `SyntaxError`, and waste
an hour debugging.

**Reason 2: The instructions are platform-specific and get outdated.**
The `brew install python@3.11` command works today, but in 6 months
Python 3.12 might be the current version and the Homebrew formula name
might change. Someone on Fedora needs `dnf install python3.11`, which
is not documented.

**Reason 3: System-level dependencies are not captured.**
If the app later adds a dependency on `pillow` (image processing),
it requires `libjpeg-dev` and `zlib-dev` on Ubuntu but not on macOS.
The INSTALL.md does not mention these, and `pip install pillow` will
fail with a cryptic C compiler error.

## Part 3: The Container Idea

### Dockerfile

```dockerfile
FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY app.py .

CMD ["python3", "app.py"]
```

### Comparison Table

| Concern | INSTALL.md | Dockerfile |
|---------|-----------|------------|
| Python version pinned? | No (text only) | Yes (FROM python:3.11-slim) |
| OS dependencies captured? | No | Yes (base image includes them) |
| Works on macOS and Linux? | Requires different commands | Same Dockerfile everywhere |
| Requires human to follow steps? | Yes | No (automated build) |
| Guaranteed identical runtime? | No | Yes (same image = same runtime) |

### Why This Works

The `FROM python:3.11-slim` line does three things at once:
1. Pins the Python version to 3.11.x
2. Pins the OS (Debian-based slim image)
3. Includes all system libraries needed to run Python

No human has to read instructions, check versions, or install system
packages. The `docker build` command produces an identical image every
time, regardless of the host machine.

### Common Mistakes to Avoid

- **Using `FROM python:latest`.** This is the same problem as not pinning
  -- the version changes over time. Always pin: `FROM python:3.11-slim`.
- **Forgetting to copy files in the right order.** Copy `requirements.txt`
  first, then `pip install`, then copy the rest. This enables Docker's
  layer caching (covered in later modules).
- **Running as root.** The default Docker user is root. For production,
  add `RUN useradd appuser` and `USER appuser` (covered in security modules).
