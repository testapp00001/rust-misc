# Solution 03: Set Up a Private Registry with TLS and Authentication

## Step-by-Step Walkthrough

### Step 1: Generate TLS Certificates

```bash
mkdir -p /tmp/private-registry/{certs,auth}
cd /tmp/private-registry

openssl req -newkey rsa:4096 -nodes -sha256 \
  -keyout certs/domain.key \
  -x509 -days 365 \
  -out certs/domain.crt \
  -subj "/CN=localhost"
```

**Why `/CN=localhost`:** The Common Name (CN) in the certificate must match
the hostname you use to connect. Since we connect as `localhost:5000`, the CN
must be `localhost`. If you use `/CN=registry.local`, you would need to
connect as `registry.local:5000` and add that hostname to `/etc/hosts`.

Verify the files exist:

```bash
ls -la certs/domain.crt certs/domain.key
# -rw------- 1 user user 1704 ... domain.key
# -rw-r--r-- 1 user user 3272 ... domain.crt
```

### Step 2: Create Authentication Credentials

```bash
# First user (creates the file)
htpasswd -Bc auth/htpasswd admin
# Enter a password when prompted

# Second user (adds to existing file -- note: no -c flag)
htpasswd -B auth/htpasswd developer
# Enter a password when prompted

# Verify
cat auth/htpasswd
```

Expected output:

```
admin:$2y$05$...bcrypt-hash...
developer:$2y$05$...bcrypt-hash...
```

**Why no `-c` for the second user:** The `-c` flag creates a new file,
overwriting any existing content. If you use `-c` for the second user, you
lose the first user. Always use `-c` only for the first entry.

### Step 3: Start the Private Registry

```bash
docker rm -f private-registry 2>/dev/null

docker run -d \
  --name private-registry \
  -p 5000:5000 \
  -v /tmp/private-registry/certs:/certs \
  -v /tmp/private-registry/auth:/auth \
  -e REGISTRY_HTTP_TLS_CERTIFICATE=/certs/domain.crt \
  -e REGISTRY_HTTP_TLS_KEY=/certs/domain.key \
  -e REGISTRY_AUTH=htpasswd \
  -e REGISTRY_AUTH_HTPASSWD_PATH=/auth/htpasswd \
  -e REGISTRY_AUTH_HTPASSWD_REALM="Registry Realm" \
  registry:2
```

Verify:

```bash
docker ps | grep private-registry
# Should show the container running with port 5000 mapped
```

### Step 4: Configure Docker to Trust the Self-Signed Certificate

```bash
sudo mkdir -p /etc/docker/certs.d/localhost:5000
sudo cp /tmp/private-registry/certs/domain.crt /etc/docker/certs.d/localhost:5000/ca.crt

sudo systemctl restart docker
sleep 3

# Check if the registry container is still running
docker ps | grep private-registry
```

**Why this step is necessary:** Docker validates TLS certificates when
communicating with registries. Self-signed certificates are not signed by a
trusted Certificate Authority (CA), so Docker rejects them by default. By
placing the certificate in `/etc/docker/certs.d/<registry>/ca.crt`, you tell
Docker to trust this specific certificate for this specific registry.

**If the container stopped after Docker restart:** This happens because
Docker restarts kill all containers. Start it again:

```bash
docker start private-registry
```

### Step 5: Login to the Private Registry

```bash
docker login localhost:5000
# Username: admin
# Password: (the password you set)
```

Expected output:

```
WARNING! Your password will be stored unencrypted in /home/user/.docker/config.json.
Login Succeeded
```

If login fails with `x509: certificate signed by unknown authority`, the
certificate was not copied to the right place or Docker was not restarted.

### Step 6: Push an Image to the Private Registry

```bash
docker pull alpine:3.19
docker tag alpine:3.19 localhost:5000/alpine:3.19
docker push localhost:5000/alpine:3.19
```

Expected output:

```
The push refers to repository [localhost:5000/alpine]
5f70bf18a086: Pushed
3.19: digest: sha256:abc123... size: 528
```

No TLS errors, no authentication errors. The push succeeded.

### Step 7: Pull from the Private Registry

```bash
docker rmi localhost:5000/alpine:3.19
docker pull localhost:5000/alpine:3.19
docker run --rm localhost:5000/alpine:3.19 cat /etc/os-release
```

Expected output shows Alpine Linux:

```
NAME="Alpine Linux"
ID=alpine
VERSION_ID=3.19.0
PRETTY_NAME="Alpine Linux v3.19"
```

This confirms the full push-pull cycle works through your private registry.

### Step 8: Verify Access Control

```bash
# Logout
docker logout localhost:5000

# Remove local copy
docker rmi localhost:5000/alpine:3.19 2>/dev/null

# Try to pull without authentication
docker pull localhost:5000/alpine:3.19
```

Expected error:

```
Error response from daemon: unauthorized: authentication required
```

This proves the registry blocks unauthenticated access. Log back in:

```bash
docker login localhost:5000
docker pull localhost:5000/alpine:3.19
# Works again
```

### Step 9: List Images in the Registry

```bash
curl -u admin:YOUR_PASSWORD https://localhost:5000/v2/_catalog
```

Expected output:

```json
{"repositories":["alpine"]}
```

List tags:

```bash
curl -u admin:YOUR_PASSWORD https://localhost:5000/v2/alpine/tags/list
```

Expected output:

```json
{"name":"alpine","tags":["3.19"]}
```

If `curl` gives a certificate error, add the `-k` flag (skip TLS verification)
or use `--cacert /tmp/private-registry/certs/domain.crt` to trust the
self-signed certificate:

```bash
curl --cacert /tmp/private-registry/certs/domain.crt \
  -u admin:YOUR_PASSWORD https://localhost:5000/v2/_catalog
```

---

## Why This Works

The private registry setup combines three components:

1. **The `registry:2` image.** This is the official Docker Registry v2 image.
   It implements the OCI Distribution Specification, which is the same API
   that Docker Hub, ECR, and other registries use. Any Docker client can
   interact with it.

2. **TLS encryption.** The `REGISTRY_HTTP_TLS_CERTIFICATE` and
   `REGISTRY_HTTP_TLS_KEY` environment variables tell the registry to serve
   over HTTPS. Docker refuses to communicate with registries over plain HTTP
   (unless explicitly configured with `--insecure-registry`). The certificate
   in `/etc/docker/certs.d/` tells the Docker daemon to trust this specific
   certificate.

3. **htpasswd authentication.** The `REGISTRY_AUTH=htpasswd` setting enables
   HTTP Basic Authentication backed by a bcrypt-hashed password file. Every
   request to the registry API must include valid credentials. This is the
   simplest authentication method -- production setups typically use token-
   based authentication with LDAP or OIDC backends.

## Common Mistakes

1. **Using `/CN=registry.local` but connecting as `localhost`.** The CN must
   match the hostname. Either generate the cert with `/CN=localhost` or add
   `127.0.0.1 registry.local` to `/etc/hosts` and connect as
   `registry.local:5000`.

2. **Using `-c` flag on the second `htpasswd` call.** This overwrites the
   file. The `-c` flag means "create," not "create if not exists." Use `-c`
   only for the very first user.

3. **Forgetting to restart Docker after adding the certificate.** Docker reads
   the certificate trust store at startup. If you add a certificate and do not
   restart Docker, it will not be trusted. On some systems, you can reload
   with `sudo systemctl reload docker` instead of a full restart.

4. **Using `--insecure-registry` instead of proper TLS.** The
   `--insecure-registry` flag tells Docker to skip TLS verification entirely.
   This is acceptable for local testing but should never be used in
   production or across a network. It disables encryption and authentication
   verification.

5. **Not persisting registry data.** The registry container stores data inside
   the container's filesystem. If you remove the container (`docker rm`), all
   pushed images are lost. For persistence, add a volume mount:
   `-v /tmp/private-registry/data:/var/lib/registry`.

6. **Using `-k` with `curl` in production.** The `-k` flag disables
   certificate verification in curl. It is fine for testing self-signed certs,
   but in production you should use `--cacert` with the CA certificate or
   install the certificate in the system trust store.

---

## Cleanup

```bash
docker rm -f private-registry
rm -rf /tmp/private-registry
sudo rm -rf /etc/docker/certs.d/localhost:5000
```
