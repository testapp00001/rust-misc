# Exercise 04: Scan and Fix Vulnerabilities in a Production Image

**Type:** Challenge
**Estimated time:** 40 minutes

## Objective

Use `trivy` to scan a container image for vulnerabilities, analyse the
results, fix the Dockerfile, rebuild, and verify that the vulnerabilities
are resolved.

## Prerequisites

- `trivy` installed (`curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin`)
- `docker` or `podman` installed
- Internet access to pull images and vulnerability databases

## Instructions

### Step 1 -- Scan the Vulnerable Image

A developer has committed the following Dockerfile for a Python web service.
Save it as `Dockerfile.vulnerable`:

```dockerfile
FROM python:3.9

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 8080

CMD ["python", "app.py"]
```

And `requirements.txt`:

```
flask==2.0.1
requests==2.25.0
Jinja2==3.0.1
PyYAML==5.4
```

Build the image:

```bash
docker build -t webapp:vulnerable -f Dockerfile.vulnerable .
```

Scan it with trivy:

```bash
trivy image webapp:vulnerable
```

Record the output. Answer the following questions:

1. How many CRITICAL vulnerabilities were found?
2. How many HIGH vulnerabilities were found?
3. Which base image package(s) contribute the most vulnerabilities?
4. Which Python package(s) have known CVEs?

### Step 2 -- Fix the Dockerfile

Create a new file called `Dockerfile.secure` that addresses the following:

1. **Use a minimal base image.** Switch from `python:3.9` to a slim or
   distroless variant.
2. **Pin the base image by digest** for reproducibility (use
   `@sha256:...` syntax -- you can look up the digest with
   `docker pull python:3.12-slim` and note the digest).
3. **Do not run as root.** Add a non-root user.
4. **Update the Python dependencies** to their latest patched versions.
   Research the minimum safe versions for each package.
5. **Add security labels** for image provenance.

Update `requirements.txt` with fixed versions.

### Step 3 -- Rebuild and Rescan

Build the new image:

```bash
docker build -t webapp:secure -f Dockerfile.secure .
trivy image webapp:secure
```

Compare the results with the vulnerable scan.

### Step 4 -- Write a Trivy Ignore File

Some vulnerabilities cannot be fixed immediately (e.g. upstream has no
patch yet). Create a `.trivyignore` file that suppresses one specific CVE
you have identified as accepted risk. Explain in a comment why it is
acceptable.

### Step 5 -- Create a Kubernetes Security Context

Write a Kubernetes Deployment manifest for the secure image that:

1. Runs in a namespace with the `baseline` PSA standard enforced.
2. Sets the container to run as non-root.
3. Uses a read-only root filesystem.
4. Drops all capabilities.
5. Sets a seccomp profile.

## Success Criteria

- [ ] The vulnerable image scan shows CRITICAL and/or HIGH CVEs.
- [ ] The secure image has zero CRITICAL vulnerabilities.
- [ ] The secure image has fewer HIGH vulnerabilities than the vulnerable one.
- [ ] The Dockerfile.secure uses a non-root user.
- [ ] The Dockerfile.secure uses a slim or distroless base image.
- [ ] The `.trivyignore` file contains a valid CVE ID with a comment.
- [ ] The Kubernetes manifest passes the `baseline` PSA standard.
- [ ] You can explain the difference between OS-level and application-level
      vulnerabilities in trivy output.

## Hints

<details>
<summary>Hint 1 -- Choosing a base image</summary>
`python:3.12-slim-bookworm` is a good choice. It has far fewer installed
packages than the full `python:3.9` image, which reduces the attack surface.
</details>

<details>
<summary>Hint 2 -- Finding safe package versions</summary>
Check the CVE details for each vulnerable package. For example, Flask 2.0.1
has CVE-2023-30861. The fix is in Flask >= 2.3.2. Use `pip install --upgrade`
or check PyPI for the latest version.
</details>

<details>
<summary>Hint 3 -- Trivy exit codes</summary>
`trivy image --exit-code 1 --severity CRITICAL webapp:secure` will exit
with code 1 if any CRITICAL vulnerability is found. This is useful for CI
pipelines.
</details>

<details>
<summary>Hint 4 -- .trivyignore format</summary>
The file contains one CVE ID per line. Comments start with `#`.
Example:
```
# Accepted risk: upstream patch not yet available, network-only exploit
CVE-2023-XXXXX
```
</details>
