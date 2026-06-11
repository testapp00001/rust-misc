# Solution 02: Push an Image to Docker Hub and Pull from Another Machine

## Step-by-Step Walkthrough

### Step 1: Create the Application Files

Create a directory and add `server.js` and `Dockerfile` as specified in the
exercise.

```bash
mkdir -p /tmp/registry-exercise && cd /tmp/registry-exercise

cat > server.js << 'EOF'
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
EOF

cat > Dockerfile << 'EOF'
FROM node:20-alpine
WORKDIR /app
COPY server.js .
ARG APP_VERSION=1.0.0
ENV APP_VERSION=${APP_VERSION}
EXPOSE 3000
CMD ["node", "server.js"]
EOF
```

### Step 2: Build and Test Locally

```bash
docker build --build-arg APP_VERSION=1.0.0 -t registry-exercise:1.0.0 .
docker run -d --name reg-test -p 3000:3000 registry-exercise:1.0.0
curl http://localhost:3000
```

Expected output:

```json
{
  "message": "Hello from the registry!",
  "version": "1.0.0",
  "hostname": "abc123def456",
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

The `"version": "1.0.0"` confirms the build arg was passed correctly through
the Dockerfile's `ARG` -> `ENV` chain.

```bash
docker stop reg-test && docker rm reg-test
```

### Step 3: Tag for Docker Hub

```bash
export DOCKER_USER="yourusername"  # Replace with your Docker Hub username

docker tag registry-exercise:1.0.0 ${DOCKER_USER}/registry-exercise:1.0.0
docker tag registry-exercise:1.0.0 ${DOCKER_USER}/registry-exercise:latest

# Verify both tags exist
docker images | grep registry-exercise
```

Expected output:

```
registry-exercise                   1.0.0        abc123def456   5 seconds ago   135MB
yourusername/registry-exercise      1.0.0        abc123def456   5 seconds ago   135MB
yourusername/registry-exercise      latest       abc123def456   5 seconds ago   135MB
```

Note that all three tags share the same image ID (`abc123def456`). Tags are
aliases, not copies.

### Step 4: Login and Push

```bash
docker login
# Enter your Docker Hub username and password (or access token)

docker push ${DOCKER_USER}/registry-exercise:1.0.0
docker push ${DOCKER_USER}/registry-exercise:latest
```

Expected output:

```
The push refers to repository [docker.io/yourusername/registry-exercise]
5f70bf18a086: Pushed
e12ab88d14dd: Pushed
1.0.0: digest: sha256:3a7b8f... size: 1345
```

Verify at `https://hub.docker.com/r/YOUR_USERNAME/registry-exercise` -- the
repository should appear with both tags listed.

### Step 5: Pull from a Clean Environment

```bash
# Remove ALL local copies
docker rmi registry-exercise:1.0.0
docker rmi ${DOCKER_USER}/registry-exercise:1.0.0
docker rmi ${DOCKER_USER}/registry-exercise:latest

# Verify nothing remains
docker images | grep registry-exercise
# (no output)

# Pull from Docker Hub
docker pull ${DOCKER_USER}/registry-exercise:1.0.0

# Run and test
docker run -d --name reg-pulled -p 3000:3000 ${DOCKER_USER}/registry-exercise:1.0.0
curl http://localhost:3000
```

The output is identical to Step 2. This proves the full push-pull cycle works:
the image on Docker Hub is byte-for-byte identical to what you built locally.

```bash
docker stop reg-pulled && docker rm reg-pulled
```

### Step 6: Push a Second Version

```bash
# Rebuild with a new version
docker build --build-arg APP_VERSION=1.0.1 -t registry-exercise:1.0.1 .

# Tag with semantic versioning
docker tag registry-exercise:1.0.1 ${DOCKER_USER}/registry-exercise:1.0.1
docker tag registry-exercise:1.0.1 ${DOCKER_USER}/registry-exercise:1.0

# Push both
docker push ${DOCKER_USER}/registry-exercise:1.0.1
docker push ${DOCKER_USER}/registry-exercise:1.0
```

### Verification

After pushing, verify the following:

```bash
# 1.0.1 returns version 1.0.1
docker pull ${DOCKER_USER}/registry-exercise:1.0.1
docker run --rm ${DOCKER_USER}/registry-exercise:1.0.1 cat /proc/1/environ | tr '\0' '\n' | grep APP_VERSION
# APP_VERSION=1.0.1

# 1.0 now also returns 1.0.1 (re-tagged)
docker pull ${DOCKER_USER}/registry-exercise:1.0
docker run --rm ${DOCKER_USER}/registry-exercise:1.0 cat /proc/1/environ | tr '\0' '\n' | grep APP_VERSION
# APP_VERSION=1.0.1

# 1.0.0 still returns 1.0.0 (original tag untouched)
docker pull ${DOCKER_USER}/registry-exercise:1.0.0
docker run --rm ${DOCKER_USER}/registry-exercise:1.0.0 cat /proc/1/environ | tr '\0' '\n' | grep APP_VERSION
# APP_VERSION=1.0.0
```

You can also verify by comparing image IDs:

```bash
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.ID}}" | grep registry-exercise
```

Expected:

```
yourusername/registry-exercise   1.0.1    def456abc789
yourusername/registry-exercise   1.0      def456abc789   # Same ID as 1.0.1
yourusername/registry-exercise   1.0.0    abc123def456   # Different ID (original)
yourusername/registry-exercise   latest   abc123def456   # Still points to 1.0.0
```

Key observation: `1.0` and `1.0.1` share the same image ID because `1.0` was
re-tagged to point at the `1.0.1` image. The `1.0.0` tag still points to the
original image. The `latest` tag was not updated and still points to `1.0.0`.

---

## Why This Works

The push-pull workflow works because of three registry properties:

1. **Content-addressable storage.** Layers are stored by hash. When you push,
   the registry skips layers it already has. When you pull, it downloads only
   the layers it needs.

2. **Tags are pointers.** A tag is a named reference to an image manifest.
   Running `docker tag A B` does not copy the image -- it creates a second
   pointer to the same manifest. This is why re-tagging `1.0` to point at
   `1.0.1` is instant and free.

3. **Manifests are immutable.** Once pushed, an image manifest (identified by
   its SHA256 digest) never changes. The tag `1.0.0` will always point to the
   same manifest, even after `1.0` is moved to a newer image. This is what
   makes exact version pinning reliable.

## Common Mistakes

1. **Forgetting to login before pushing.** This produces a `denied: requested
   access to the resource is denied` error. Always run `docker login` first.
   If you have 2FA enabled on Docker Hub, use an access token instead of your
   password.

2. **Not replacing `YOUR_USERNAME`.** The tag must include your Docker Hub
   username as the namespace. `docker tag myapp:1.0 myapp:1.0` will fail to
   push because Docker Hub does not know which account owns `myapp`.

3. **Expecting `latest` to auto-update.** In Step 6, we updated `1.0` but not
   `latest`. If someone pulls `latest` after Step 6, they still get `1.0.0`.
   To update `latest`, you must explicitly re-tag and push it.

4. **Confusing local tags with remote tags.** `docker rmi` only removes local
   copies. The image on Docker Hub is not affected. To remove images from
   Docker Hub, you must use the web UI or the API.

5. **Pushing before testing locally.** Always run the container locally and
   verify it works before pushing. Once an image is in a registry, others may
   pull it. Pushing a broken image wastes everyone's time.
