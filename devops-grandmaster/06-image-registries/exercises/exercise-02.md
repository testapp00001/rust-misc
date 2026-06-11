# Exercise 02: Push an Image to Docker Hub and Pull from Another Machine

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

---

## Objective

Build a container image, tag it correctly for Docker Hub, push it to a public repository, then pull it from a clean environment to verify the full registry workflow.

---

## Prerequisites

- Docker installed and running.
- A Docker Hub account (free at https://hub.docker.com).
- A terminal.

---

## Instructions

### Step 1: Create a Simple Application

Create a new directory and add the following files.

**server.js:**

```javascript
const http = require('http');
const os = require('os');

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    message: 'Hello from the registry!',
    version: process.env.APP_VERSION || 'unknown',
    hostname: os.hostname(),
    timestamp: new Date().toISOString()
  }));
});

server.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

**Dockerfile:**

```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY server.js .
ARG APP_VERSION=1.0.0
ENV APP_VERSION=${APP_VERSION}
EXPOSE 3000
CMD ["node", "server.js"]
```

### Step 2: Build and Test Locally

Build the image and verify it works on your machine before pushing to a registry.

```bash
# Build the image
docker build --build-arg APP_VERSION=1.0.0 -t registry-exercise:1.0.0 .

# Run it
docker run -d --name reg-test -p 3000:3000 registry-exercise:1.0.0

# Test it
curl http://localhost:3000

# Stop and remove
docker stop reg-test && docker rm reg-test
```

**Your task:** Verify the JSON output includes the correct version (`1.0.0`).

### Step 3: Tag for Docker Hub

Docker Hub requires the image name to follow the format `<dockerhub-username>/<repository>:<tag>`.

```bash
# Replace YOUR_USERNAME with your actual Docker Hub username
export DOCKER_USER="YOUR_USERNAME"

# Tag the image for Docker Hub
docker tag registry-exercise:1.0.0 ${DOCKER_USER}/registry-exercise:1.0.0
docker tag registry-exercise:1.0.0 ${DOCKER_USER}/registry-exercise:latest
```

**Your task:** Verify both tags exist by running `docker images | grep registry-exercise`.

### Step 4: Login and Push

```bash
# Login to Docker Hub
docker login

# Push both tags
docker push ${DOCKER_USER}/registry-exercise:1.0.0
docker push ${DOCKER_USER}/registry-exercise:latest
```

**Your task:** Open https://hub.docker.com/r/YOUR_USERNAME/registry-exercise in a browser and verify the repository exists with both tags.

### Step 5: Pull from a Clean Environment

Simulate pulling on a different machine by removing all local copies first.

```bash
# Remove ALL local copies of this image
docker rmi registry-exercise:1.0.0
docker rmi ${DOCKER_USER}/registry-exercise:1.0.0
docker rmi ${DOCKER_USER}/registry-exercise:latest

# Verify nothing remains
docker images | grep registry-exercise
# Should show nothing

# Pull from Docker Hub
docker pull ${DOCKER_USER}/registry-exercise:1.0.0

# Run it
docker run -d --name reg-pulled -p 3000:3000 ${DOCKER_USER}/registry-exercise:1.0.0

# Test it
curl http://localhost:3000
```

**Your task:** Confirm the output is identical to Step 2.

### Step 6: Push a Second Version

Now simulate a version update.

```bash
# Rebuild with a new version
docker build --build-arg APP_VERSION=1.0.1 -t registry-exercise:1.0.1 .

# Tag it
docker tag registry-exercise:1.0.1 ${DOCKER_USER}/registry-exercise:1.0.1
docker tag registry-exercise:1.0.1 ${DOCKER_USER}/registry-exercise:1.0

# Push
docker push ${DOCKER_USER}/registry-exercise:1.0.1
docker push ${DOCKER_USER}/registry-exercise:1.0
```

**Your task:** After pushing, verify that:
- `docker pull ${DOCKER_USER}/registry-exercise:1.0.1` returns version `1.0.1`.
- `docker pull ${DOCKER_USER}/registry-exercise:1.0` also returns version `1.0.1` (since `1.0` was re-tagged).
- `docker pull ${DOCKER_USER}/registry-exercise:1.0.0` still returns version `1.0.0` (the original tag was not overwritten).

---

## Success Criteria

- [ ] You built and tested the image locally before pushing.
- [ ] You tagged the image with both a version tag and `latest`.
- [ ] You pushed both tags to Docker Hub successfully.
- [ ] You removed all local copies and pulled from Docker Hub.
- [ ] The pulled image produces the same output as the locally built image.
- [ ] You pushed version 1.0.1 and verified that `1.0` now points to `1.0.1`.
- [ ] You understand that `1.0.0` still works independently.

---

## Hints

<details>
<summary>Hint 1: Docker Hub Username</summary>

Your Docker Hub username is visible at https://hub.docker.com after you log in. It is the part after `docker.io/` in your profile URL. If your username is `janedoe`, your images should be tagged as `janedoe/registry-exercise:1.0.0`.

</details>

<details>
<summary>Hint 2: Why `docker login` Might Fail</summary>

If `docker login` fails, check:
1. Are your credentials correct? Try logging in at https://hub.docker.com first.
2. Are you behind a corporate proxy? You may need to configure Docker proxy settings.
3. Is Docker running? Run `docker info` to check.

If you have 2FA enabled, you need a Docker Hub Access Token instead of your password. Create one at https://hub.docker.com/settings/security.

</details>

<details>
<summary>Hint 3: Understanding Tag Aliases</summary>

When you run `docker tag registry-exercise:1.0.1 ${DOCKER_USER}/registry-exercise:1.0`, you are not creating a copy of the image. Both tags point to the same image ID. This is why pushing `1.0.1` and then re-tagging `1.0` to point at `1.0.1` works -- `1.0` now points to the new image, while `1.0.0` still points to the original.

</details>

<details>
<summary>Hint 4: Verifying Tag Points</summary>

You can verify which image a tag points to by comparing image IDs:

```bash
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.ID}}" | grep registry-exercise
```

Tags that point to the same image will show the same ID.

</details>
