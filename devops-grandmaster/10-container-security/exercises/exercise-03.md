# Exercise 03: Scan and Fix Vulnerabilities in a Container Image

**Type:** Independent
**Time:** 30-45 minutes
**Difficulty:** Medium

## Objective

Use a container scanning tool (Trivy) to identify vulnerabilities in a container image, analyze the results, and produce a hardened Dockerfile that eliminates or reduces the critical and high-severity findings. You will learn to make informed decisions about base images, package management, and vulnerability remediation.

## Scenario

Your team's application Dockerfile has been flagged by the security team. You need to scan the image, understand the vulnerabilities, and fix them. The application is a Python Flask web server.

**The vulnerable Dockerfile:**

```dockerfile
FROM python:3.11

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

RUN python setup.py install

EXPOSE 5000

CMD ["python", "app.py"]
```

**requirements.txt:**

```
flask==2.3.2
requests==2.31.0
gunicorn==21.2.0
pyyaml==6.0.1
cryptography==41.0.3
Jinja2==3.1.2
```

## Tasks

### Part A: Build and Scan the Vulnerable Image

1. Create a working directory with the Dockerfile and requirements.txt above.
2. Create a minimal `app.py` so the image builds:

```python
from flask import Flask
app = Flask(__name__)

@app.route("/")
def hello():
    return "Hello, World!"

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
```

3. Build the image:

```bash
docker build -t vuln-app:original .
```

4. Scan it with Trivy:

```bash
trivy image vuln-app:original
```

5. Record the total number of vulnerabilities by severity (CRITICAL, HIGH, MEDIUM, LOW).

<details>
<summary>Hint</summary>

If you do not have Trivy installed, install it with:
```bash
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin
```
If Trivy is not available, you can use Docker Scout: `docker scout cves vuln-app:original`.

</details>

### Part B: Analyze the Scan Results

Answer these questions based on the scan output:

1. How many CRITICAL vulnerabilities were found? List their CVE IDs.
2. Which base image packages contribute the most vulnerabilities?
3. Are any vulnerabilities in the Python packages themselves (not the OS packages)?
4. What is the oldest vulnerability found (earliest CVE year)?

<details>
<summary>Hint</summary>

Trivy groups vulnerabilities by target: OS packages (Debian/Alpine) and Python packages (pip). Look at the "Target" column in the output. OS package vulnerabilities are often the majority and come from the base image.

</details>

### Part C: Fix the Dockerfile

Create an improved Dockerfile that addresses the vulnerabilities. Consider these strategies:

1. **Switch to a minimal base image.** Alpine or slim images have fewer installed packages and therefore fewer vulnerabilities.
2. **Pin versions precisely.** Ensure all Python packages use exact versions.
3. **Update vulnerable packages.** If a package has a known fix, use the fixed version.
4. **Add security hardening.** Non-root user, no-new-privileges, etc.

Your new Dockerfile should:
- Reduce CRITICAL vulnerabilities to zero (or as close as possible).
- Reduce HIGH vulnerabilities significantly.
- Still run the Flask application correctly.
- Include all the security hardening from Exercise 02.

<details>
<summary>Hint 1: Base image choice</summary>

Try `python:3.12-slim-alpine` or `python:3.12-alpine`. Alpine-based images typically have far fewer OS-level vulnerabilities because they use musl libc and BusyBox instead of glibc and GNU coreutils. However, some Python packages with C extensions may need additional build dependencies on Alpine.

</details>

<details>
<summary>Hint 2: Package versions</summary>

Run `pip install --upgrade` for packages that have known CVEs. For example, if `cryptography==41.0.3` has CVEs, check what the latest patched version is. Use `pip index versions <package>` or check PyPI for the latest version.

</details>

<details>
<summary>Hint 3: Multi-stage builds</summary>

A multi-stage build can separate build dependencies (compilers, headers) from the runtime image. This reduces the attack surface in the final image.

</details>

### Part D: Verify the Fix

1. Build the new image:

```bash
docker build -t vuln-app:hardened .
```

2. Scan it:

```bash
trivy image vuln-app:hardened
```

3. Compare the results:

```bash
# Generate JSON reports for comparison
trivy image --format json --output original.json vuln-app:original
trivy image --format json --output hardened.json vuln-app:hardened

# Count vulnerabilities by severity
echo "=== Original ==="
cat original.json | jq '.Results[]?.Vulnerabilities | group_by(.Severity) | map({severity: .[0].Severity, count: length})'
echo "=== Hardened ==="
cat hardened.json | jq '.Results[]?.Vulnerabilities | group_by(.Severity) | map({severity: .[0].Severity, count: length})'
```

4. Verify the application still works:

```bash
docker run --rm -d --name test-app -p 5000:5000 vuln-app:hardened
curl http://localhost:5000
docker rm -f test-app
```

---

## Success Criteria

- [ ] You have a scan report showing the vulnerability count of the original image.
- [ ] The hardened image has zero CRITICAL vulnerabilities.
- [ ] The hardened image has fewer HIGH vulnerabilities than the original.
- [ ] The hardened Dockerfile runs the application as a non-root user.
- [ ] The application responds correctly on port 5000 after hardening.

## What You Should Understand After This Exercise

Image scanning is not just about running a tool -- it is about understanding the results and making informed decisions. Switching to a minimal base image is often the single most effective action for reducing vulnerabilities. Pinned, up-to-date dependencies and a minimal runtime footprint form the foundation of container supply chain security.
