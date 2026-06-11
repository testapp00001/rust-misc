# Exercise 03: Set Up a Private Registry with TLS and Authentication

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

---

## Objective

Deploy a self-hosted Docker registry with TLS encryption and password authentication. Push and pull images to verify the private registry works end-to-end.

---

## Prerequisites

- Docker installed and running.
- `openssl` installed (for generating TLS certificates).
- `htpasswd` installed (for generating password files). On Debian/Ubuntu: `sudo apt-get install apache2-utils`. On macOS: it is pre-installed.
- A terminal.

---

## Instructions

### Step 1: Generate TLS Certificates

Create a directory for your registry setup and generate a self-signed TLS certificate.

```bash
# Create working directory
mkdir -p /tmp/private-registry/{certs,auth}
cd /tmp/private-registry

# Generate a self-signed certificate
# Replace "registry.local" with your desired hostname
openssl req -newkey rsa:4096 -nodes -sha256 \
  -keyout certs/domain.key \
  -x509 -days 365 \
  -out certs/domain.crt \
  -subj "/CN=registry.local"
```

**Your task:** Verify that `certs/domain.crt` and `certs/domain.key` were created.

### Step 2: Create Authentication Credentials

Use `htpasswd` to create a file with a username and password.

```bash
# Create a password file with a user
# You will be prompted to enter a password
htpasswd -Bc auth/htpasswd admin

# Verify the file was created
cat auth/htpasswd
# Should show: admin:$2y$05$... (bcrypt hashed password)
```

**Your task:** Create a second user called `developer` in the same file (without the `-c` flag, which creates a new file).

### Step 3: Start the Private Registry

Run the official Docker registry container with TLS and authentication enabled.

```bash
# Stop any existing registry container
docker rm -f private-registry 2>/dev/null

# Start the registry with TLS and auth
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

**Your task:** Verify the container is running with `docker ps`.

### Step 4: Configure Docker to Trust the Self-Signed Certificate

Docker does not trust self-signed certificates by default. You must add the certificate to Docker's trust store.

```bash
# Create the certificate directory for this registry
sudo mkdir -p /etc/docker/certs.d/localhost:5000

# Copy the certificate
sudo cp /tmp/private-registry/certs/domain.crt /etc/docker/certs.d/localhost:5000/ca.crt

# Restart Docker to pick up the new certificate
sudo systemctl restart docker

# Wait for Docker to restart
sleep 3

# Verify the registry container is still running
docker ps | grep private-registry
```

**Your task:** If the container stopped after the Docker restart, start it again with `docker start private-registry`.

### Step 5: Login to the Private Registry

```bash
# Login with the admin user you created
docker login localhost:5000
# Enter username: admin
# Enter password: (the password you set)
```

**Your task:** Verify the login succeeds. If it fails, check that the certificate is in the right place, Docker was restarted after adding the certificate, and the credentials are correct.

### Step 6: Push an Image to the Private Registry

```bash
# Pull a small image
docker pull alpine:3.19

# Tag it for your private registry
docker tag alpine:3.19 localhost:5000/alpine:3.19

# Push it
docker push localhost:5000/alpine:3.19
```

**Your task:** Verify the push succeeds without authentication errors or TLS errors.

### Step 7: Pull from the Private Registry

```bash
# Remove the local tagged image
docker rmi localhost:5000/alpine:3.19

# Pull it back
docker pull localhost:5000/alpine:3.19

# Verify it works
docker run --rm localhost:5000/alpine:3.19 cat /etc/os-release
```

**Your task:** Confirm the image runs and shows Alpine Linux information.

### Step 8: Verify Access Control

Test that unauthenticated access is blocked.

```bash
# Logout from the registry
docker logout localhost:5000

# Try to pull without authentication
docker rmi localhost:5000/alpine:3.19 2>/dev/null
docker pull localhost:5000/alpine:3.19
# This should fail with "unauthorized" or "denied"
```

**Your task:** Verify the pull fails with an authentication error. Then log back in and confirm it works again.

### Step 9: List Images in the Registry

```bash
# Use the registry API to list repositories
curl -u admin:YOUR_PASSWORD https://localhost:5000/v2/_catalog
# Should return: {"repositories":["alpine"]}

# List tags for a repository
curl -u admin:YOUR_PASSWORD https://localhost:5000/v2/alpine/tags/list
# Should return: {"name":"alpine","tags":["3.19"]}
```

**Your task:** Replace `YOUR_PASSWORD` with your actual password and verify the output.

---

## Success Criteria

- [ ] You generated a self-signed TLS certificate.
- [ ] You created an htpasswd file with two users.
- [ ] You started the registry container with TLS and authentication enabled.
- [ ] You configured Docker to trust the self-signed certificate.
- [ ] You logged in, pushed, and pulled images successfully.
- [ ] Unauthenticated pulls are rejected.
- [ ] You can query the registry API to list repositories and tags.

---

## Cleanup

```bash
# Stop and remove the registry container
docker rm -f private-registry

# Remove the working directory
rm -rf /tmp/private-registry

# Remove Docker's trust store entry
sudo rm -rf /etc/docker/certs.d/localhost:5000
```

---

## Hints

<details>
<summary>Hint 1: htpasswd Flag Meanings</summary>

- `-B`: Use bcrypt encryption (recommended, more secure).
- `-c`: Create a new file. Only use this for the FIRST user. If you use `-c` again, it overwrites the file.
- `-b`: Accept the password on the command line instead of prompting (less secure, but useful for scripts).

To add a second user: `htpasswd -B /tmp/private-registry/auth/htpasswd developer`

</details>

<details>
<summary>Hint 2: Certificate Trust on Different Systems</summary>

The path `/etc/docker/certs.d/<registry>/ca.crt` is the standard location for Docker daemon certificate trust on Linux. On macOS with Docker Desktop, the path is the same but managed differently -- you may need to add the certificate to your system keychain instead.

If you cannot restart Docker, you can also use the `--insecure-registry` flag in the Docker daemon configuration, but this disables TLS verification and is NOT recommended for production.

</details>

<details>
<summary>Hint 3: Common TLS Errors</summary>

- `x509: certificate signed by unknown authority` -- Docker does not trust your certificate. Make sure you copied it to `/etc/docker/certs.d/localhost:5000/ca.crt` and restarted Docker.
- `x509: certificate is valid for registry.local, not localhost` -- The CN in your certificate must match the hostname you use to connect. If you generate the cert with `/CN=registry.local`, you must connect as `registry.local`, not `localhost`. Either regenerate with `/CN=localhost` or add `localhost` as a Subject Alternative Name (SAN).

</details>

<details>
<summary>Hint 4: Registry API Authentication</summary>

The `curl` command for the registry API uses HTTPS because you configured TLS. If you get a certificate error with `curl`, add the `-k` flag to skip certificate verification (only for testing, never in production), or use `--cacert /tmp/private-registry/certs/domain.crt` to explicitly trust your certificate.

</details>
