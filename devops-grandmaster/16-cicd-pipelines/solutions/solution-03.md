# Solution 03: Security Scanning and Image Push

## Part A: Add Trivy Vulnerability Scanning

```yaml
      - name: Run Trivy vulnerability scan (table output)
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: myapp:${{ github.sha }}
          format: table
          output: trivy-results.txt

      - name: Run Trivy vulnerability scan (SARIF output)
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: myapp:${{ github.sha }}
          format: sarif
          output: trivy-results.sarif

      - name: Upload Trivy scan results to GitHub Security tab
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: trivy-results.sarif
```

### Why This Works

Two Trivy runs are needed because a single invocation produces only one output format. The table format goes to a file for human review in the workflow logs. The SARIF format is uploaded to GitHub's code scanning dashboard, where it appears alongside CodeQL results in the Security tab. The `if: always()` on the upload step ensures SARIF results are uploaded even if the scan finds vulnerabilities (which would fail a later step).

### Common Mistake

- **Running Trivy only in table format.** Table output is useful for debugging but does not integrate with GitHub's security tooling. Always include SARIF for the Security tab.
- **Not specifying `output`.** Without `output:`, Trivy prints to stdout, which is hard to parse and does not produce a file for upload.

## Part B: Enforce a Vulnerability Threshold

```yaml
      - name: Run Trivy vulnerability scan with threshold
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: myapp:${{ github.sha }}
          format: table
          exit-code: 1
          severity: CRITICAL,HIGH
          ignore-unfixed: true
```

### Why This Works

`exit-code: 1` tells Trivy to return a non-zero exit code when vulnerabilities matching the severity filter are found. Since GitHub Actions treats any non-zero exit code as step failure, the pipeline stops. `severity: CRITICAL,HIGH` limits the check to high-impact vulnerabilities, allowing Medium and Low findings to be tracked without blocking the pipeline. `ignore-unfixed: true` excludes vulnerabilities that have no available fix, preventing the pipeline from being blocked by issues that cannot be resolved.

### Common Mistakes

- **Using `exit-code: 0` (the default).** With exit code 0, Trivy reports vulnerabilities but does not fail the step. The pipeline continues, and the image is pushed despite having Critical CVEs.
- **Not using `ignore-unfixed: true`.** Without this, the pipeline can be blocked by CVEs that have no patch available. This creates a deadlock: the image cannot be deployed, but there is no fix to apply.
- **Setting severity too low.** Including `MEDIUM` or `LOW` in the severity filter can block the pipeline on non-critical findings, creating noise and slowing development.

## Part C: Sign the Image with Cosign

```yaml
      - name: Install cosign
        uses: sigstore/cosign-installer@v3

      - name: Sign the container image
        run: cosign sign --yes ghcr.io/${{ github.repository }}@${{ steps.build-push.outputs.digest }}
        env:
          COSIGN_EXPERIMENTAL: 1
```

### Why This Works

Keyless signing with cosign uses the OIDC (OpenID Connect) identity of the GitHub Actions runner. No private key needs to be stored or managed. The `--yes` flag skips the interactive confirmation prompt. The `COSIGN_EXPERIMENTAL: 1` environment variable enables the keyless flow. The image is referenced by its digest (a content-addressable hash) rather than its tag, because tags are mutable but digests are immutable. The signature is stored alongside the image in the registry as an OCI attachment.

### Common Mistakes

- **Signing by tag instead of digest.** Tags can be reassigned to different images. Signing by digest ensures the signature is bound to the exact image content.
- **Storing a private key as a GitHub secret.** While this works, keyless signing via OIDC is more secure because there is no long-lived secret to rotate or leak.
- **Missing `id-token: write` permission.** The workflow must have `id-token: write` in its `permissions:` block for OIDC-based keyless signing to work.

## Part D: Conditional Push with Full Security Pipeline

```yaml
name: Secure CI Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-test-scan-push:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
      id-token: write
      security-events: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      # Stage 1: Build
      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          load: true
          tags: myapp:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      # Stage 2: Test
      - name: Run linting
        run: docker run --rm myapp:${{ github.sha }} npm run lint

      - name: Run tests
        run: docker run --rm myapp:${{ github.sha }} npm test

      # Stage 3: Scan
      - name: Run Trivy vulnerability scan (threshold)
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: myapp:${{ github.sha }}
          format: table
          exit-code: 1
          severity: CRITICAL,HIGH
          ignore-unfixed: true

      - name: Run Trivy vulnerability scan (SARIF)
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: myapp:${{ github.sha }}
          format: sarif
          output: trivy-results.sarif

      - name: Upload scan results to GitHub Security tab
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: trivy-results.sarif

      - name: Generate scan summary
        if: always()
        run: |
          echo "## Trivy Vulnerability Scan Results" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          docker run --rm aquasec/trivy:0.50.1 image --format table myapp:${{ github.sha }} >> $GITHUB_STEP_SUMMARY 2>&1 || true

      # Stage 4: Sign and Push
      - name: Log in to GitHub Container Registry
        if: github.event_name == 'push'
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate image metadata
        if: github.event_name == 'push'
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=sha,prefix=
            type=raw,value=latest

      - name: Build and push image
        if: github.event_name == 'push'
        id: build-push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Install cosign
        if: github.event_name == 'push'
        uses: sigstore/cosign-installer@v3

      - name: Sign the container image
        if: github.event_name == 'push'
        run: cosign sign --yes ghcr.io/${{ github.repository }}@${{ steps.build-push.outputs.digest }}
```

### Why This Works

The pipeline is organized into four sequential stages: Build, Test, Scan, Sign and Push. Each stage depends on the previous one succeeding. The `if: github.event_name == 'push'` condition on the publish and signing steps ensures images are only pushed on pushes to `main`, not on pull requests. The `id-token: write` permission enables OIDC-based keyless signing. The `security-events: write` permission allows uploading SARIF results. The scan summary is written to `$GITHUB_STEP_SUMMARY` for visibility in the GitHub Actions UI.

### Common Mistakes

- **Pushing before scanning.** If the push step runs before the scan step, vulnerable images can reach the registry. The ordering must be Build -> Test -> Scan -> Push.
- **Missing `security-events: write` permission.** Without this, the SARIF upload fails silently, and scan results do not appear in the Security tab.
- **Running all scan and push steps on PRs.** PRs should build, test, and scan, but not push. Use `if: github.event_name == 'push'` to gate push-related steps.

## Key Takeaway

A security-enhanced CI pipeline treats vulnerability scanning as a first-class pipeline stage with the same authority as testing: if the scan fails, the image does not get pushed. Trivy with `exit-code: 1` and `severity: CRITICAL,HIGH` enforces a "no critical vulnerabilities" policy automatically. Cosign with keyless OIDC signing provides cryptographic provenance without managing secrets. The pipeline enforces the principle that security is not a post-deployment audit -- it is a pre-deployment gate.
