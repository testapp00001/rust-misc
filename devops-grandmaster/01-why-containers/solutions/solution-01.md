# Solution 01: The "It Works On My Machine" Autopsy

## Part A: Identify the Differences

Here are the differences between the developer's machine and the production server:

| Aspect | Developer (macOS) | Production (Ubuntu) |
|--------|-------------------|---------------------|
| **OS** | macOS 14 (Sonoma) | Ubuntu 20.04 LTS |
| **Python version** | 3.11.4 | 3.6.9 |
| **Python source** | Likely Homebrew or pyenv | System Python (`apt`) |
| **Flask version** | 3.0.0 | 1.1.2 |
| **Available stdlib modules** | `datetime.timezone` available | `datetime.timezone` unavailable |
| **System libraries** | macOS frameworks | Ubuntu/Debian packages |
| **Path conventions** | `/usr/local/`, Homebrew paths | `/usr/`, system paths |
| **Shell** | zsh (default on macOS) | bash (default on Ubuntu) |
| **File system** | Case-insensitive (default) | Case-sensitive |

### Why This Matters

The Python version difference is the *proximate* cause of the crash, but it
is not the *root* cause. The root cause is that nothing in the project pins
or validates the Python version. The version difference is a symptom of a
system that allows environment drift.

## Part B: Explain the Failure

The `datetime.timezone` class was added in **Python 3.2** (PEP 3154), but
the `match` statement used in some patterns requires Python 3.10+.

In this specific case, the error is:

```
ModuleNotFoundError: No module named 'datetime.timezone'
```

Wait -- this is interesting. The `datetime.timezone` class exists in Python 3.2+,
so on Python 3.6.9 it *should* be available. Let me re-examine the scenario.

The actual error in the scenario is that Python 3.6.9 is the *system* Python on
Ubuntu 20.04. The developer's code might import `datetime.timezone` correctly,
but the real issue is likely that `flask==3.0.0` requires Python 3.8+, and on
Python 3.6 the package installation itself fails or produces import errors.

**The deeper lesson:** The error message can be misleading. The real problem is
the Python version gap (3.6 vs 3.11), which causes cascading failures in
package compatibility, stdlib features, and syntax support.

### Common Mistake

Many students focus only on the `datetime.timezone` import and miss the
broader picture: the entire dependency chain (Flask 3.0.0 requires Python 3.8+)
is incompatible with the production Python version.

## Part C: List All the Fixes

### Fix 1: Upgrade Python on the server (Quick and Dirty)

```bash
# On the production server
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt update
sudo apt install python3.11
sudo update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.11 1
```

**What it changes:** The production server now has Python 3.11.
**Prevents next problem?** No. The next developer might use Python 3.12 features.
**Effort:** 10 minutes.

### Fix 2: Pin Python version in `runtime.txt` or `.python-version`

```
# runtime.txt
python-3.11.4
```

Some platforms (Heroku, pyenv) respect this file. Combined with a CI check
that validates the Python version, this reduces drift.

**What it changes:** Documents the required version in code.
**Prevents next problem?** Partially. Only works if the deployment platform
respects the file, and someone still has to install the right version.
**Effort:** 5 minutes.

### Fix 3: Use a container (Dockerfile)

```dockerfile
FROM python:3.11.4-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python3", "app.py"]
```

**What it changes:** The Python version, OS, and all dependencies are
captured in a single file. The *same image* runs on the developer's laptop,
the CI server, and production.
**Prevents next problem?** Yes. This eliminates the entire class of
environment mismatch problems.
**Effort:** 30 minutes (including learning Docker basics).

### Comparison

| Approach | Prevents this issue? | Prevents future issues? | Effort |
|----------|---------------------|------------------------|--------|
| Upgrade Python | Yes | No | Low |
| Pin version in file | Partially | Partially | Very low |
| Container (Dockerfile) | Yes | Yes | Medium |

## Part D: The Deeper Question

Three additional environment differences that could cause future failures:

### 1. System Library Versions

macOS ships with LibreSSL, while Ubuntu ships with OpenSSL. If the Python
`ssl` module behaves differently (TLS version support, certificate handling),
HTTPS connections might work locally but fail in production.

### 2. File System Differences

macOS's default file system (APFS) is case-insensitive. Linux (ext4) is
case-sensitive. A file named `Config.py` vs `config.py` works on macOS
but causes `ModuleNotFoundError` on Linux.

### 3. Environment Variables

The developer might have `DATABASE_URL`, `API_KEY`, or `PATH` variables
set in their `.zshrc` that do not exist on the production server. The
application might silently fall back to defaults locally but crash in
production when the variable is missing.

### Common Mistakes to Avoid

- **Focusing only on the immediate error.** The Python version is the
  proximate cause, but the real problem is the lack of environment control.
- **Assuming "just upgrade Python" fixes everything.** It fixes this one
  issue but leaves the system vulnerable to the next environment drift.
- **Blaming the developer.** The system allowed this to happen. The fix
  should be systemic (containers, CI checks), not behavioral ("be more careful").

## Key Takeaway

The "it works on my machine" problem is not a single problem -- it is a
*category* of problems caused by uncontrolled environment state. Containers
solve this category by making the environment part of the deployable artifact.
