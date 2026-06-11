# Exercise 03: Security Scanning and Image Push

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Extend a CI pipeline to include container image security scanning with Trivy, enforce a vulnerability threshold, sign the image with cosign, and push only images that pass all checks to a container registry.

## Scenario

Your organization has adopted a security-first policy. No container image may be pushed to the production registry unless:

1. It has been scanned for vulnerabilities.
2. It contains zero "Critical" or "High" severity CVEs.
3. It has been cryptographically signed.

You must implement this policy as a GitHub Actions workflow. The base pipeline already builds the image -- you need to add the security gates.

## Tasks

### Part A: Add Trivy Vulnerability Scanning

Add a Trivy scan step to your pipeline that:

1. Scans the built Docker image for OS and library vulnerabilities.
2. Outputs results in both table format (for human reading) and SARIF format (for GitHub Security tab integration).
3. Uploads the SARIF results to GitHub's code scanning dashboard using `github/codeql-action/upload-sarif@v3`.

The scan should run after the image is built but before it is pushed.

<details>
<summary>Hint</summary>

Use the `aquasecurity/trivy-action@master` action. Set `scan-type: image`, `image-ref: <your-image-tag>`, and `format: sarif` for the upload step. You may need to run Trivy twice: once for table output and once for SARIF.

</details>

### Part B: Enforce a Vulnerability Threshold

Modify the Trivy scan so that the pipeline fails if any "Critical" or "High" severity vulnerabilities are found. The pipeline should:

1. Fail the workflow (non-zero exit code) on Critical or High CVEs.
2. Allow Medium and Low severity findings to pass (but still report them).
3. Print a clear error message explaining why the pipeline failed.

<details>
<summary>Hint</summary>

Trivy supports `exit-code` and `severity` parameters. Set `exit-code: 1` to make Trivy return a non-zero exit code when vulnerabilities matching the specified severity are found. Use `severity: CRITICAL,HIGH` to filter.

</details>

### Part C: Sign the Image with Cosign

After the image passes the vulnerability scan, sign it using cosign:

1. Install cosign in the workflow using `sigstore/cosign-installer@v3`.
2. Sign the image using keyless signing (Sigstore OIDC, no private key needed).
3. Attach the signature to the image in the registry.
4. The signing step must only run after the vulnerability scan passes.

<details>
<summary>Hint</summary>

Keyless signing with cosign uses the OIDC identity of the GitHub Actions runner. Use `cosign sign <image-ref>` and it will automatically use the Sigstore bundle flow. You need `id-token: write` permission in the workflow.

</details>

### Part D: Conditional Push with Full Security Pipeline

Assemble the complete security-enhanced pipeline with these requirements:

1. The pipeline has four ordered stages: **Build** -> **Test** -> **Scan** -> **Sign and Push**.
2. The image is pushed to the registry only if all previous stages pass.
3. The image is tagged with both the Git SHA and `latest`.
4. A summary of the scan results is posted as a GitHub Actions job summary (using `$GITHUB_STEP_SUMMARY`).

Write the complete workflow YAML that ties all the stages together.

<details>
<summary>Hint</summary>

Use `needs:` to enforce stage ordering. The push step should `needs: [scan]`. For the job summary, write markdown to `$GITHUB_STEP_SUMMARY` in a step that parses Trivy's JSON output.

</details>

## Success Criteria

- [ ] Trivy scans the built image and reports vulnerabilities in table and SARIF formats.
- [ ] The pipeline fails when Critical or High severity CVEs are found.
- [ ] SARIF results appear in the GitHub Security tab after a successful run.
- [ ] The image is signed with cosign using keyless (OIDC) signing after passing the scan.
- [ ] The image is pushed to the registry only after build, test, scan, and sign all succeed.
- [ ] Image tags include both the Git SHA and `latest`.
- [ ] A scan summary is visible in the GitHub Actions job summary.

## What You Should Understand After This Exercise

Security scanning is not optional -- it is a pipeline stage with the same weight as building and testing. Trivy integrates directly into CI workflows and can enforce vulnerability thresholds that block bad images from reaching the registry. Cosign provides cryptographic provenance without managing private keys by leveraging Sigstore's OIDC-based keyless signing. The pipeline enforces a "no vulnerabilities, no deploy" policy automatically.
