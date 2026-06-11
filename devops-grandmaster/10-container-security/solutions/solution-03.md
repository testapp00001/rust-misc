# Solution 03: Scan and Fix Vulnerabilities in a Container Image

## Part A: Build and Scan

**Building the original image:**

```bash
docker build -t vuln-app:original .
```

**Scanning with Trivy:**

```bash
trivy image vuln-app:original
```

**Expected findings (approximate -- actual counts depend on when you run the scan):**

The `python:3.11` base image is a full Debian Bullseye image with many installed packages. Typical results:

- CRITICAL: 5-15 vulnerabilities (OpenSSL, libcurl, systemd, etc.)
- HIGH: 30-60 vulnerabilities
- MEDIUM: 100+ vulnerabilities
- LOW: 50+ vulnerabilities

Most vulnerabilities come from OS packages in the base image, not from the Python packages themselves.

---

## Part B: Analyze the Scan Results

**Answers (based on typical scan results):**

1. **CRITICAL vulnerabilities** typically include CVEs in OpenSSL (e.g., CVE-2023-5678), libcurl (e.g., CVE-2023-46218), and systemd components. The exact CVE IDs change over time as new vulnerabilities are discovered and patches are released.
2. **Base image packages** contribute the majority of vulnerabilities. The full Debian image includes hundreds of packages (compilers, libraries, system utilities) that a Python web application does not need. Each package is a potential vulnerability.
3. **Python package vulnerabilities** are usually fewer. Flask, requests, and gunicorn are well-maintained and patched quickly. However, `pyyaml==6.0.1` and `cryptography==41.0.3` in the requirements may have known CVEs depending on the scan date.
4. **Oldest vulnerabilities** are often in foundational libraries like glibc or OpenSSL, which may have CVEs from 2020-2022 that have not been patched in the base image version.

---

## Part C: Hardened Dockerfile

```dockerfile
# Build stage -- install dependencies in a separate stage
FROM python:3.12-slim AS builder

WORKDIR /build

COPY requirements.txt .

# Upgrade pip and install dependencies
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir --prefix=/install -r requirements.txt

# Runtime stage -- minimal image
FROM python:3.12-slim

# Install only the runtime dependencies needed
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      libssl3 \
      libcurl4 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --gid 1001 appgroup && \
    useradd --uid 1001 --gid appgroup --shell /bin/sh --create-home appuser

WORKDIR /app

# Copy installed Python packages from builder
COPY --from=builder /install /usr/local

# Copy application code
COPY --chown=appuser:appgroup app.py .

# Switch to non-root user
USER appuser

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD ["python", "-c", "import urllib.request; urllib.request.urlopen('http://localhost:5000/')"]

CMD ["python", "app.py"]
```

**Updated requirements.txt:**

```
flask==3.0.0
requests==2.31.0
gunicorn==21.2.0
pyyaml==6.0.1
cryptography==42.0.2
Jinja2==3.1.3
```

**Why this works:**

- **`python:3.12-slim`** is based on Debian Slim, which includes only the minimum runtime packages. It has significantly fewer vulnerabilities than the full Debian image.
- **Multi-stage build** separates build-time dependencies (pip, compilers) from the runtime image. The final image does not include pip, setuptools, or build tools -- only the installed packages.
- **`apt-get install --no-install-recommends`** installs only the packages explicitly listed, not their recommended dependencies. This keeps the image small and reduces the attack surface.
- **Updated package versions** address known CVEs in the Python packages. Always check PyPI for the latest versions when a vulnerability is reported.
- **Non-root user** ensures the application does not run as root, even if an attacker gains code execution through the application.
- **`HEALTHCHECK`** allows Docker to detect if the application is unresponsive.

---

## Part D: Verify the Fix

**Expected scan results after hardening:**

```bash
$ trivy image vuln-app:hardened
```

- CRITICAL: 0 (or very few, depending on scan date)
- HIGH: 5-15 (reduced from 30-60)
- MEDIUM: 20-40 (reduced from 100+)

**Comparison:**

| Severity | Original | Hardened | Reduction |
|----------|----------|----------|-----------|
| CRITICAL | 10 | 0 | 100% |
| HIGH | 45 | 8 | 82% |
| MEDIUM | 120 | 30 | 75% |

The remaining vulnerabilities are typically in core system libraries (glibc, openssl) that cannot be removed without breaking the system. These are mitigated by other security layers (non-root user, capabilities, seccomp).

**Application verification:**

```bash
$ docker run --rm -d --name test-app -p 5000:5000 vuln-app:hardened
$ curl http://localhost:5000
Hello, World!
$ docker rm -f test-app
```

The application responds correctly, confirming that the hardening did not break functionality.

---

## Common Mistakes

- **Using `python:3.12-alpine` without considering C extensions:** Alpine uses musl libc instead of glibc. Some Python packages with C extensions (like `cryptography`) may fail to install or behave differently on Alpine. If you use Alpine, you may need to install additional build dependencies (`gcc`, `musl-dev`, `libffi-dev`).
- **Updating packages without testing:** Always test your application after updating dependencies. A minor version bump can include breaking changes. Run your test suite before deploying.
- **Ignoring medium/low vulnerabilities:** While CRITICAL and HIGH should be the priority, medium vulnerabilities can sometimes be chained together for exploitation. Review them periodically.
- **Not re-scanning after fixes:** After hardening, always re-scan to verify the fixes worked. Some vulnerabilities may persist if the patched version is not available in the base image.
- **Assuming fewer vulnerabilities means secure:** A zero-CRITICAL image is not "secure" -- it just has fewer known vulnerabilities. Security is a continuous process, not a one-time scan.
