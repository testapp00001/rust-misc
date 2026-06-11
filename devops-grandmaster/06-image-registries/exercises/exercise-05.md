# Exercise 05: Design a Registry Strategy for a Multi-Team Organization

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

---

## Objective

Design a complete registry strategy for a fictional organization with multiple teams, compliance requirements, and CI/CD pipelines. Produce a written design document and implement key parts of it.

---

## The Scenario

You are the DevOps architect at **NovaCorp**, a mid-size company with the following structure:

**Teams:**
- **Platform Team** (3 engineers): Manages shared infrastructure, base images, and CI/CD.
- **Backend Team** (8 engineers): Builds 12 microservices in Go and Python.
- **Frontend Team** (5 engineers): Builds 4 React applications and a BFF (Backend for Frontend).
- **Data Team** (4 engineers): Builds data pipelines and ML model serving containers.

**Requirements:**
1. Each team must only be able to push to their own repositories.
2. All teams can pull shared base images maintained by the Platform team.
3. No image can be deployed to production without passing a vulnerability scan.
4. Every production image must be traceable to a Git commit and a CI pipeline run.
5. Old images must be automatically cleaned up after 90 days (except release tags).
6. The company has a hybrid cloud: some workloads run on AWS (EKS), some on-premises.
7. Registry costs must stay under $500/month.

---

## Instructions

### Part A: Registry Selection (Written Design)

Write a design document (`registry-design.md`) that answers:

1. **Which registry(ies) will you use?** Consider Docker Hub, ECR, GCR, ACR, GHCR, Harbor, and self-hosted options. Justify your choice based on the requirements above.

2. **How will you organize repositories?** Define the naming convention. Examples:
   - `nova-backend/user-service`
   - `nova-platform/base-node20`
   - How do you handle the hybrid cloud (AWS + on-premises)?

3. **How will you handle access control?** Define who can push, pull, and delete for each team. What about CI/CD service accounts?

4. **What is your tag strategy?** Define:
   - Tag format for development builds.
   - Tag format for staging builds.
   - Tag format for production releases.
   - How you handle semantic versioning.
   - Whether you use `latest` and under what conditions.

5. **What is your vulnerability scanning policy?** Define:
   - When scanning happens (build time, push time, or both).
   - What severity levels block deployment.
   - How you handle false positives.

6. **What is your image retention policy?** Define:
   - How long non-release images are kept.
   - How many recent images to keep per repository.
   - Which images are exempt from cleanup.

7. **What is your cost optimization strategy?** Estimate storage needs and define how you will stay under budget.

### Part B: Implement Access Control

Write the configuration or commands needed to set up the access control model you designed.

```bash
# If using Harbor, define:
# - Projects for each team
# - Robot accounts for CI/CD
# - RBAC rules

# If using ECR, define:
# - IAM policies for each team
# - Cross-account access (if needed)
# - Lifecycle policies for cleanup
```

### Part C: Implement the CI/CD Pipeline

Write a GitHub Actions workflow (or your CI system of choice) that:

1. Builds an image with the Git SHA embedded.
2. Scans the image for vulnerabilities.
3. Blocks the push if critical vulnerabilities are found.
4. Pushes to the correct registry with proper tags.
5. Promotes from staging to production (with approval).

```yaml
# .github/workflows/build-and-push.yml
# Write the complete workflow here
```

### Part D: Implement Retention Policies

Write the configuration for automated image cleanup.

```bash
# For ECR: lifecycle policy JSON
# For Harbor: retention rule configuration
# For Docker Hub: API script to clean old tags
```

### Part E: Implement Registry Mirroring (Bonus)

Design a solution for the on-premises environment to pull images without depending on the internet.

```bash
# Option 1: Harbor as a pull-through cache for ECR
# Option 2: Registry mirror in Docker daemon config
# Option 3: Image replication from cloud to on-premises Harbor
```

---

## Success Criteria

- [ ] Your design document addresses all 7 questions in Part A.
- [ ] Your access control model ensures teams cannot push to each other's repositories.
- [ ] Your CI/CD pipeline embeds the Git SHA and blocks critical vulnerabilities.
- [ ] Your retention policy automatically cleans up old images.
- [ ] Your cost estimate is realistic and stays under $500/month.
- [ ] Your design handles the hybrid cloud requirement.
- [ ] (Bonus) You have a solution for on-premises image mirroring.

---

## Hints

<details>
<summary>Hint 1: Repository Naming Convention</summary>

A good naming convention encodes the team and the application:

```
<org>/<team>/<app>:<tag>

Examples:
nova/backend/user-service:abc1234
nova/backend/order-service:1.2.3
nova/frontend/web-app:main-abc1234
nova/platform/base-node20:20.11
nova/data/ml-serving:v2.0.0
```

This makes it easy to set per-team access policies and retention rules.

</details>

<details>
<summary>Hint 2: ECR Lifecycle Policy</summary>

ECR lifecycle policies use rules based on tag patterns:

```json
{
  "rules": [
    {
      "rulePriority": 1,
      "description": "Keep last 10 images with any tag",
      "selection": {
        "tagStatus": "any",
        "countType": "imageCountMoreThan",
        "countNumber": 10
      },
      "action": {
        "type": "expire"
      }
    },
    {
      "rulePriority": 2,
      "description": "Keep release images for 90 days",
      "selection": {
        "tagStatus": "tagged",
        "tagPrefixList": ["v"],
        "countType": "sinceImagePushed",
        "countUnit": "days",
        "countNumber": 90
      },
      "action": {
        "type": "expire"
      }
    }
  ]
}
```

</details>

<details>
<summary>Hint 3: Harbor Project Structure</summary>

In Harbor, each team gets a project with its own access policies:

```
Projects:
  nova-platform:  (Platform team: admin, others: read-only)
  nova-backend:   (Backend team: admin, Platform: admin)
  nova-frontend:  (Frontend team: admin, Platform: admin)
  nova-data:      (Data team: admin, Platform: admin)
```

Robot accounts are created per project for CI/CD pipelines. Each robot account can only push to its project.

</details>

<details>
<summary>Hint 4: Cost Estimation</summary>

Estimate storage based on:
- Number of services: ~20
- Average image size: ~150MB (with multi-stage builds)
- Tags per service: ~15 (recent commits + releases)
- Total: 20 x 15 x 150MB = 45GB
- ECR cost: 45GB x $0.10/GB = $4.50/month

Add bandwidth costs for CI/CD pulls:
- 500 pulls/day x 150MB = 75GB/day
- Same-region ECR pulls: free
- Cross-region: ~$150/month
- On-premises to AWS: ~$675/month (consider a mirror)

</details>

<details>
<summary>Hint 5: GitHub Actions Multi-Stage Pipeline</summary>

Use separate jobs for build, scan, and deploy:

```yaml
jobs:
  build:
    # Build and push to staging
  scan:
    # Run Trivy on the built image
    needs: build
  deploy-staging:
    # Deploy to staging cluster
    needs: scan
  deploy-production:
    # Requires manual approval
    needs: deploy-staging
    environment: production
```

The `environment: production` keyword in GitHub Actions enables manual approval gates.

</details>
