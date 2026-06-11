# Solution 04: Scan and Fix Vulnerabilities in a Production Image

## Step 1 -- Vulnerable Image Scan Results

Running `trivy image webapp:vulnerable` produces output similar to:

```
webapp:vulnerable (debian 11.8)
Total: 842 (UNKNOWN: 0, LOW: 456, MEDIUM: 263, HIGH: 97, CRITICAL: 26)

python:3.9 base image contributes the bulk of OS-level CVEs including:
- libssl1.1 (multiple CRITICAL CVEs)
- libcurl4 (HIGH CVEs)
- zlib1g (HIGH CVEs)

Python packages with known CVEs:
- flask==2.0.1  -> CVE-2023-30861 (HIGH) - session cookie handling
- Jinja2==3.0.1 -> CVE-2024-22195 (HIGH) - XSS in xmlattr filter
- PyYAML==5.4   -> CVE-2020-14343 (HIGH) - arbitrary code execution
- requests==2.25.0 -> CVE-2023-32681 (MEDIUM) - credential leak on redirect
```

*(Exact numbers vary based on scan date and database version.)*

## Step 2 -- Fixed Dockerfile

### Dockerfile.secure

```dockerfile
# Pin by digest for reproducibility
FROM python:3.12-slim-bookworm@sha256:a866731a6b71c tried

# Security labels for provenance
LABEL org.opencontainers.image.source="https://github.com/example/webapp"
LABEL org.opencontainers.image.description="Secure Python web service"
LABEL org.opencontainers.image.vendor="Example Corp"

WORKDIR /app

# Install dependencies first (layer caching)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Create non-root user and switch to it
RUN groupadd -r appuser && useradd -r -g appuser -d /app -s /sbin/nologin appuser \
    && chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["python", "app.py"]
```

*Note: Replace the sha256 digest with the actual current digest from
`docker pull python:3.12-slim-bookworm`. Example:*

```bash
docker pull python:3.12-slim-bookworm
# Note the Digest: sha256:... from the output
```

A practical version without the digest (still secure):

```dockerfile
FROM python:3.12-slim-bookworm

LABEL org.opencontainers.image.source="https://github.com/example/webapp"
LABEL org.opencontainers.image.description="Secure Python web service"

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

RUN groupadd -r appuser && useradd -r -g appuser -d /app -s /sbin/nologin appuser \
    && chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["python", "app.py"]
```

### Updated requirements.txt

```
flask>=3.0.0
requests>=2.32.0
Jinja2>=3.1.3
PyYAML>=6.0.1
```

## Step 3 -- Rescan Results

After rebuilding:

```bash
docker build -t webapp:secure -f Dockerfile.secure .
trivy image webapp:secure
```

Expected improvement:

```
webapp:secure (debian 12.x)
Total: 12 (UNKNOWN: 0, LOW: 8, MEDIUM: 3, HIGH: 1, CRITICAL: 0)
```

The slim base image eliminates most OS-level CVEs. The updated Python
packages eliminate the application-level CVEs.

## Step 4 -- Trivy Ignore File

```
# Accepted risk: CVE-2024-XXXXX in libxml2
# This is a LOW severity issue requiring local access to exploit.
# Our container does not process untrusted XML. Upstream fix pending.
# Reviewed by: security-team on 2024-01-15
# Tracking: JIRA-SEC-1234
CVE-2024-XXXXX
```

## Step 5 -- Kubernetes Deployment

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: webapp
  labels:
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
  namespace: webapp
  labels:
    app: webapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: webapp
  template:
    metadata:
      labels:
        app: webapp
    spec:
      securityContext:
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: webapp
          image: webapp:secure
          ports:
            - containerPort: 8080
          securityContext:
            allowPrivilegeEscalation: false
            runAsNonRoot: true
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
          resources:
            limits:
              memory: "128Mi"
              cpu: "500m"
            requests:
              memory: "64Mi"
              cpu: "250m"
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 3
            periodSeconds: 5
```

## Why It Works

### Base Image Change

`python:3.12-slim-bookworm` contains roughly 100 packages compared to 800+
in the full `python:3.9` image. Fewer packages means fewer potential
vulnerabilities. Bookworm (Debian 12) is the current stable release with
active security support.

### Dependency Updates

Each updated version addresses a specific CVE:

| Package | Old | New | CVE Fixed |
|---------|-----|-----|-----------|
| Flask | 2.0.1 | 3.0.0+ | CVE-2023-30861 |
| Jinja2 | 3.0.1 | 3.1.3+ | CVE-2024-22195 |
| PyYAML | 5.4 | 6.0.1+ | CVE-2020-14343 |
| requests | 2.25.0 | 2.32.0+ | CVE-2023-32681 |

### Non-root User

The `appuser` is created with no login shell (`/sbin/nologin`), no home
directory write access, and no root group membership. The `USER appuser`
directive ensures the process runs as this unprivileged user.

### OS vs Application Vulnerabilities

Trivy reports two categories:

- **OS vulnerabilities:** CVEs in packages installed by the base image
  (e.g. libssl, curl). Fixed by choosing a cleaner base image or running
  `apt-get upgrade` during build.
- **Application vulnerabilities:** CVEs in language-specific packages (e.g.
  Flask, Jinja2). Fixed by updating dependency versions.

## Common Mistakes

- **Using `latest` tag without a digest.** The `latest` tag is mutable and
  can change at any time. Pinning by digest ensures reproducibility.
- **Running `apt-get upgrade` instead of choosing a better base image.**
  Upgrading during build works but adds build time and may introduce
  incompatibilities. A minimal base image is the better first step.
- **Setting `runAsNonRoot: true` without creating a user.** If the base
  image has no non-root user, the container will fail to start. Always
  create a user explicitly.
- **Ignoring LOW severity CVEs.** Low-severity CVEs can be chained together
  for privilege escalation. Track them even if you do not fix them
  immediately.
- **Not testing after fixing dependencies.** Major version bumps (e.g.
  Flask 2.x to 3.x) can introduce breaking changes. Run your test suite
  after updating.
