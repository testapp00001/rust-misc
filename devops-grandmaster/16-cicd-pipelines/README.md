# Module 16: CI/CD Pipelines — Automated Build, Test, Deploy

> **Previous Module (15):** Container Monitoring
> **Previous Limitation:** Building and deploying containers is a manual, error-prone process. Pushing code to git triggers nothing.
> **This Module Solves That:** Fully automated pipelines that build, test, scan, and deploy on every commit.

---

## 1. The Problem

Your team pushes code 20 times a day. For each push, someone must:

1. Pull the latest code
2. Build the Docker image
3. Run the tests
4. Scan for vulnerabilities
5. Push to a registry
6. Deploy to the correct environment
7. Verify the deployment
8. Roll back if something breaks

Doing this manually means:

- **Inconsistency.** Different people deploy differently. "It works on my machine."
- **Slow feedback.** A broken build sits unnoticed until someone tries to deploy.
- **Human error.** Forgetting to run tests, pushing to the wrong environment, typos in commands.
- **No audit trail.** Who deployed what, when, and why?
- **Fear of deploying.** Deployments become stressful events instead of routine operations.

**The core problem:** Manual processes do not scale. You need automation that runs the same steps, in the same order, every single time.

---

## 2. The Naive Way

### Manual deployment scripts

```bash
#!/bin/bash
# deploy.sh — the "it works on my laptop" approach

git pull
docker build -t myapp:latest .
docker run --rm myapp:latest npm test
docker push myapp:latest
ssh prod-server "docker pull myapp:latest && docker compose up -d"
echo "Deployed!"
```

### Why it fails

1. **No isolation.** The script runs on someone's laptop with unknown environment state.
2. **No parallelism.** Steps run sequentially. Testing, building, and scanning could run in parallel.
3. **No rollback.** If the deploy fails, you're stuck with a broken production.
4. **No environment separation.** The same script deploys to production. No staging gate.
5. **No secrets management.** Credentials are stored on the developer's machine.
6. **No notification.** Nobody knows the deploy happened (or failed) unless they're watching.

---

## 3. The Right Way

### CI/CD fundamentals

**Continuous Integration (CI):** Every code change triggers an automated build and test cycle. Broken code is caught within minutes, not days.

**Continuous Deployment (CD):** Every change that passes all tests is automatically deployed to production. No human intervention required.

**Continuous Delivery (CD):** Like deployment, but with a manual approval gate before production.

**The pipeline:**

```
Code Push → Build → Test → Scan → Push → Deploy (staging) → Deploy (prod)
    |         |       |      |      |          |                  |
    v         v       v      v      v          v                  v
  Git      Docker   Unit   CVE   Registry   Staging           Prod
           Build    Tests  Scan   Push       Smoke             Health
                                         Test              Check
```

### GitHub Actions for Docker

**.github/workflows/ci.yml:**

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  # Job 1: Build and test
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build image
        uses: docker/build-push-action@v5
        with:
          context: .
          load: true
          tags: myapp:test
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run unit tests
        run: |
          docker run --rm myapp:test npm test

      - name: Run integration tests
        run: |
          docker compose -f docker-compose.test.yml up --abort-on-container-exit
          docker compose -f docker-compose.test.yml down -v

  # Job 2: Security scan
  security-scan:
    runs-on: ubuntu-latest
    needs: build-and-test
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Build image
        uses: docker/build-push-action@v5
        with:
          context: .
          load: true
          tags: myapp:scan

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: 'myapp:scan'
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v2
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'

  # Job 3: Push to registry
  push:
    runs-on: ubuntu-latest
    needs: [build-and-test, security-scan]
    if: github.event_name == 'push'
    permissions:
      contents: read
      packages: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha
            type=ref,event=branch
            type=semver,pattern={{version}}
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  # Job 4: Deploy to staging
  deploy-staging:
    runs-on: ubuntu-latest
    needs: push
    if: github.ref == 'refs/heads/develop'
    environment: staging
    steps:
      - name: Deploy to staging
        run: |
          echo "Deploying to staging..."
          # ssh staging-server "docker pull $IMAGE && docker compose up -d"

      - name: Smoke test
        run: |
          sleep 10
          curl -f https://staging.example.com/health || exit 1

  # Job 5: Deploy to production
  deploy-production:
    runs-on: ubuntu-latest
    needs: push
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Deploy to production
        run: |
          echo "Deploying to production..."
          # ssh prod-server "docker pull $IMAGE && docker compose up -d"

      - name: Health check
        run: |
          sleep 15
          curl -f https://example.com/health || exit 1

      - name: Rollback on failure
        if: failure()
        run: |
          echo "Deployment failed! Rolling back..."
          # ssh prod-server "docker compose up -d --force-recreate"
```

### GitLab CI for Docker

**.gitlab-ci.yml:**

```yaml
stages:
  - build
  - test
  - scan
  - push
  - deploy

variables:
  IMAGE: $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  LATEST: $CI_REGISTRY_IMAGE:latest

build:
  stage: build
  image: docker:24-dind
  services:
    - docker:24-dind
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker build -t $IMAGE -t $LATEST .
    - docker push $IMAGE
    - docker push $LATEST

test:
  stage: test
  image: $IMAGE
  script:
    - npm test

scan:
  stage: scan
  image:
    name: aquasec/trivy:latest
    entrypoint: [""]
  script:
    - trivy image --exit-code 1 --severity CRITICAL $IMAGE

deploy-staging:
  stage: deploy
  image: alpine:latest
  environment:
    name: staging
  script:
    - apk add openssh-client
    - ssh $STAGING_SERVER "docker pull $IMAGE && docker compose up -d"
  only:
    - develop

deploy-production:
  stage: deploy
  image: alpine:latest
  environment:
    name: production
  script:
    - apk add openssh-client
    - ssh $PROD_SERVER "docker pull $IMAGE && docker compose up -d"
  only:
    - main
  when: manual
```

### Jenkins for Docker

**Jenkinsfile:**

```groovy
pipeline {
    agent any

    environment {
        REGISTRY = 'registry.example.com'
        IMAGE = "${REGISTRY}/myapp"
    }

    stages {
        stage('Build') {
            steps {
                script {
                    docker.build("${IMAGE}:${env.BUILD_NUMBER}")
                }
            }
        }

        stage('Test') {
            steps {
                sh "docker run --rm ${IMAGE}:${env.BUILD_NUMBER} npm test"
            }
        }

        stage('Scan') {
            steps {
                sh "trivy image --exit-code 1 --severity CRITICAL ${IMAGE}:${env.BUILD_NUMBER}"
            }
        }

        stage('Push') {
            steps {
                script {
                    docker.withRegistry("https://${REGISTRY}", 'registry-credentials') {
                        docker.image("${IMAGE}:${env.BUILD_NUMBER}").push()
                        docker.image("${IMAGE}:${env.BUILD_NUMBER}").push('latest')
                    }
                }
            }
        }

        stage('Deploy Staging') {
            when { branch 'develop' }
            steps {
                sh "ssh staging-server 'docker pull ${IMAGE}:${env.BUILD_NUMBER} && docker compose up -d'"
            }
        }

        stage('Deploy Production') {
            when { branch 'main' }
            input {
                message "Deploy to production?"
                ok "Deploy"
            }
            steps {
                sh "ssh prod-server 'docker pull ${IMAGE}:${env.BUILD_NUMBER} && docker compose up -d'"
            }
        }
    }

    post {
        failure {
            // Rollback logic
            sh "ssh prod-server 'docker compose up -d --force-recreate'"
        }
        always {
            sh "docker rmi ${IMAGE}:${env.BUILD_NUMBER} || true"
        }
    }
}
```

---

## 4. The Production Way

### Automated testing in containers

Tests should run in containers, identical to production:

```yaml
# docker-compose.test.yml
version: '3.8'

services:
  app:
    build:
      context: .
      target: test  # Multi-stage build with test stage
    depends_on:
      test-db:
        condition: service_healthy
      test-redis:
        condition: service_healthy
    environment:
      - NODE_ENV=test
      - DATABASE_URL=postgres://test:test@test-db:5432/test
      - REDIS_URL=redis://test-redis:6379
    command: npm test

  test-db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=test
      - POSTGRES_PASSWORD=test
      - POSTGRES_DB=test
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U test"]
      interval: 5s
      timeout: 3s
      retries: 5

  test-redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5
```

### Multi-stage build for testing

```dockerfile
# Stage 1: Build
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .

# Stage 2: Test
FROM builder AS test
RUN npm test
RUN npm run lint

# Stage 3: Production
FROM node:18-alpine AS production
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY --from=builder /app/dist ./dist
USER node
EXPOSE 3000
CMD ["node", "dist/server.js"]
```

### Image scanning in CI/CD

```yaml
# GitHub Actions with multiple scanners
- name: Run Snyk security scan
  uses: snyk/actions/docker@master
  env:
    SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
  with:
    image: myapp:test
    args: --severity-threshold=high

- name: Run Hadolint (Dockerfile linting)
  uses: hadolint/hadolint-action@v3.1.0
  with:
    dockerfile: Dockerfile

- name: Run Dockle (container best practices)
  uses: goodwithtech/dockle-action@main
  with:
    image: myapp:test
    exit-code: 1
    exit-level: warn
```

### Deploying to different environments

```yaml
# Environment-specific deployment
deploy:
  strategy:
    matrix:
      environment: [staging, production]
  environment: ${{ matrix.environment }}
  steps:
    - name: Deploy to ${{ matrix.environment }}
      run: |
        ENV=${{ matrix.environment }}
        IMAGE_TAG=${{ github.sha }}

        # Use environment-specific compose override
        docker compose \
          -f docker-compose.yml \
          -f docker-compose.${ENV}.yml \
          up -d
```

**docker-compose.staging.yml:**

```yaml
version: '3.8'

services:
  app:
    image: myapp:${IMAGE_TAG:-latest}
    environment:
      - NODE_ENV=staging
      - LOG_LEVEL=debug
    ports:
      - "3000:3000"
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
```

**docker-compose.production.yml:**

```yaml
version: '3.8'

services:
  app:
    image: myapp:${IMAGE_TAG:-latest}
    environment:
      - NODE_ENV=production
      - LOG_LEVEL=warn
    ports:
      - "3000:3000"
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 512M
          cpus: '1.0'
```

### Rollback strategies

**Strategy 1: Tag-based rollback**

```bash
# Rollback to previous version
PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD~1)
docker compose pull myapp:${PREVIOUS_TAG}
docker compose up -d
```

**Strategy 2: Automatic rollback in CI/CD**

```yaml
deploy-production:
  steps:
    - name: Deploy new version
      run: |
        docker compose pull
        docker compose up -d
        sleep 30

    - name: Health check
      id: health
      run: |
        for i in $(seq 1 10); do
          if curl -f https://example.com/health; then
            echo "healthy=true" >> $GITHUB_OUTPUT
            exit 0
          fi
          sleep 5
        done
        echo "healthy=false" >> $GITHUB_OUTPUT
        exit 1

    - name: Rollback if unhealthy
      if: steps.health.outputs.healthy == 'false'
      run: |
        echo "Rolling back to previous version..."
        docker compose down
        docker compose -f docker-compose.previous.yml up -d
```

**Strategy 3: Blue-green deployment**

```yaml
# Deploy to "green" alongside existing "blue"
deploy:
  steps:
    - name: Deploy green
      run: |
        docker compose -f docker-compose.green.yml up -d
        sleep 15

    - name: Health check green
      run: |
        curl -f http://localhost:3001/health

    - name: Switch traffic
      run: |
        # Update nginx upstream to point to green
        # Reload nginx
        nginx -s reload

    - name: Remove blue
      run: |
        docker compose -f docker-compose.blue.yml down
```

---

## 5. Hands-On Lab

### Lab: Build a complete GitHub Actions pipeline

**Objective:** Create a CI/CD pipeline that builds, tests, scans, and deploys a Docker application.

#### Step 1: Create the project

```bash
mkdir -p cicd-lab/.github/workflows && cd cicd-lab
```

#### Step 2: Create the application

**server.js:**

```javascript
const express = require('express');
const app = express();

app.get('/', (req, res) => {
  res.json({ message: 'CI/CD Lab', version: process.env.APP_VERSION || 'unknown' });
});

app.get('/health', (req, res) => {
  res.json({ status: 'healthy', timestamp: new Date().toISOString() });
});

app.get('/test-fail', (req, res) => {
  // Endpoint to test failure scenarios
  process.exit(1);
});

const port = process.env.PORT || 3000;
app.listen(port, () => console.log(`Listening on port ${port}`));

module.exports = app;
```

**server.test.js:**

```javascript
const request = require('supertest');
const app = require('./server');

describe('API Tests', () => {
  test('GET / returns message', async () => {
    const res = await request(app).get('/');
    expect(res.status).toBe(200);
    expect(res.body.message).toBe('CI/CD Lab');
  });

  test('GET /health returns healthy', async () => {
    const res = await request(app).get('/health');
    expect(res.status).toBe(200);
    expect(res.body.status).toBe('healthy');
  });
});
```

**package.json:**

```json
{
  "name": "cicd-lab",
  "version": "1.0.0",
  "scripts": {
    "start": "node server.js",
    "test": "jest --forceExit"
  },
  "dependencies": {
    "express": "^4.18.2"
  },
  "devDependencies": {
    "jest": "^29.7.0",
    "supertest": "^6.3.3"
  }
}
```

**Dockerfile:**

```dockerfile
# Build stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .

# Test stage
FROM builder AS test
RUN npm test

# Production stage
FROM node:18-alpine AS production
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY server.js .
USER node
EXPOSE 3000
CMD ["node", "server.js"]
```

#### Step 3: Create the CI/CD pipeline

**.github/workflows/ci-cd.yml:**

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  IMAGE_NAME: cicd-lab

jobs:
  # Job 1: Lint Dockerfile
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Hadolint
        uses: hadolint/hadolint-action@v3.1.0
        with:
          dockerfile: Dockerfile

  # Job 2: Build and test
  build-test:
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build test image
        uses: docker/build-push-action@v5
        with:
          context: .
          target: test
          load: true
          tags: ${{ env.IMAGE_NAME }}:test
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run tests in container
        run: |
          docker run --rm ${{ env.IMAGE_NAME }}:test

      - name: Build production image
        uses: docker/build-push-action@v5
        with:
          context: .
          target: production
          load: true
          tags: |
            ${{ env.IMAGE_NAME }}:${{ github.sha }}
            ${{ env.IMAGE_NAME }}:latest

      - name: Save image for next jobs
        run: |
          docker save ${{ env.IMAGE_NAME }}:${{ github.sha }} > /tmp/image.tar

      - name: Upload image artifact
        uses: actions/upload-artifact@v3
        with:
          name: docker-image
          path: /tmp/image.tar
          retention-days: 1

  # Job 3: Security scan
  security-scan:
    runs-on: ubuntu-latest
    needs: build-test
    steps:
      - uses: actions/checkout@v4

      - name: Download image
        uses: actions/download-artifact@v3
        with:
          name: docker-image
          path: /tmp

      - name: Load image
        run: docker load < /tmp/image.tar

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: '${{ env.IMAGE_NAME }}:${{ github.sha }}'
          format: 'table'
          exit-code: '0'
          severity: 'CRITICAL,HIGH'

  # Job 4: Deploy to staging
  deploy-staging:
    runs-on: ubuntu-latest
    needs: [build-test, security-scan]
    if: github.ref == 'refs/heads/develop'
    environment: staging
    steps:
      - name: Deploy to staging
        run: |
          echo "Deploying ${{ github.sha }} to staging..."
          # In real scenario: SSH to staging server, pull image, restart
          docker compose -f docker-compose.staging.yml up -d 2>/dev/null || true

      - name: Smoke test
        run: |
          sleep 10
          echo "Running smoke tests..."
          # curl -f http://staging.example.com/health || exit 1

  # Job 5: Deploy to production
  deploy-production:
    runs-on: ubuntu-latest
    needs: [build-test, security-scan]
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Deploy to production
        run: |
          echo "Deploying ${{ github.sha }} to production..."
          # In real scenario: SSH to prod server, pull image, restart
          docker compose -f docker-compose.production.yml up -d 2>/dev/null || true

      - name: Health check
        run: |
          sleep 15
          echo "Running health checks..."
          # curl -f https://example.com/health || exit 1

      - name: Notify team
        if: always()
        run: |
          echo "Deployment ${{ job.status }} for ${{ github.sha }}"
```

#### Step 4: Create environment-specific compose files

**docker-compose.staging.yml:**

```yaml
version: '3.8'

services:
  app:
    image: cicd-lab:${GITHUB_SHA:-latest}
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=staging
      - APP_VERSION=${GITHUB_SHA:-unknown}
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.25'
```

**docker-compose.production.yml:**

```yaml
version: '3.8'

services:
  app:
    image: cicd-lab:${GITHUB_SHA:-latest}
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - APP_VERSION=${GITHUB_SHA:-unknown}
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
```

#### Step 5: Create a test compose file

**docker-compose.test.yml:**

```yaml
version: '3.8'

services:
  app-test:
    build:
      context: .
      target: test
    environment:
      - NODE_ENV=test
```

#### Step 6: Test locally

```bash
# Build and test locally (simulating CI)
docker build --target test -t cicd-lab:test .
docker run --rm cicd-lab:test

# Build production image
docker build --target production -t cicd-lab:latest .

# Run it
docker run -d -p 3000:3000 --name cicd-lab cicd-lab:latest

# Test endpoints
curl http://localhost:3000/
curl http://localhost:3000/health

# Clean up
docker stop cicd-lab && docker rm cicd-lab
```

#### Step 7: Initialize git and push

```bash
git init
git add .
git commit -m "Initial commit with CI/CD pipeline"

# Create a GitHub repository
gh repo create cicd-lab --private --source=. --push

# Create develop branch
git checkout -b develop
git push -u origin develop
```

#### Step 8: Trigger the pipeline

```bash
# Make a change and push to develop (triggers staging deploy)
echo "# CI/CD Lab" >> README.md
git add README.md
git commit -m "Add README"
git push

# Merge to main (triggers production deploy)
git checkout main
git merge develop
git push
```

#### Step 9: Monitor the pipeline

```bash
# Watch the workflow run
gh run list

# Watch a specific run
gh run watch

# View logs
gh run view --log
```

#### Cleanup

```bash
gh repo delete cicd-lab --yes
```

---

## 6. Limitation

Your pipeline is automated. Every push builds, tests, scans, and deploys. But there is a subtle problem lurking in your Dockerfiles:

```dockerfile
FROM node:18-alpine
```

What version of Node.js 18? What version of Alpine? If you build this today and rebuild it in 6 months, you might get completely different base image versions. Your "reproducible build" is not actually reproducible.

Worse, what if `node:18-alpine` gets a breaking update? Your pipeline breaks, and you don't know why.

**You need a versioning strategy for your images that is predictable, traceable, and rollback-safe.**

---

## 7. Next Topic

**Module 17: Image Versioning** — We will learn tag strategies (semantic versioning, git SHA, date-based), tag immutability, base image pinning, and rollback by tag. The `latest` tag is a trap, and we will learn why.
