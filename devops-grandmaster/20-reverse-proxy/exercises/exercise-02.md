# Exercise 02: Guided — Nginx Reverse Proxy with SSL

## Type
Guided — step-by-step with checkpoints.

## Objective

Configure an Nginx reverse proxy that terminates SSL and forwards traffic to two backend services: a frontend static site and a JSON API. You will use self-signed certificates for local development.

## Prerequisites

- Docker and Docker Compose installed
- OpenSSL available on your machine

## Instructions

### Step 1: Create the Project Structure

Create the following directory layout:

```
exercise-02/
  docker-compose.yml
  nginx/
    nginx.conf
    certs/
  frontend/
    index.html
  api/
    app.py
    Dockerfile
```

### Step 2: Create the Frontend

Create `frontend/index.html`:

```html
<!DOCTYPE html>
<html>
<head><title>My App</title></head>
<body>
    <h1>Welcome to the Frontend</h1>
    <button id="btn">Call API</button>
    <pre id="output"></pre>
    <script>
        document.getElementById('btn').addEventListener('click', async () => {
            const res = await fetch('/api/status');
            const data = await res.json();
            document.getElementById('output').textContent = JSON.stringify(data, null, 2);
        });
    </script>
</body>
</html>
```

### Step 3: Create the API

Create `api/app.py`:

```python
from flask import Flask, jsonify
import os, datetime

app = Flask(__name__)

@app.route('/api/status')
def status():
    return jsonify({
        "service": "api",
        "status": "healthy",
        "timestamp": datetime.datetime.utcnow().isoformat(),
        "hostname": os.environ.get('HOSTNAME', 'unknown')
    })

@app.route('/api/health')
def health():
    return jsonify({"status": "ok"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Create `api/Dockerfile`:

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

### Step 4: Generate Self-Signed Certificates

Run this command from the project root:

```bash
mkdir -p nginx/certs
openssl req -x509 -nodes -days 365 \
  -newkey rsa:2048 \
  -keyout nginx/certs/server.key \
  -out nginx/certs/server.crt \
  -subj "/CN=localhost"
```

### Step 5: Write the Nginx Configuration

Create `nginx/nginx.conf`. It must:

1. Listen on port 80 and redirect all traffic to HTTPS (port 443).
2. Listen on port 443 with SSL using the self-signed certificates from Step 4.
3. Route `/` requests to the `frontend` service (port 80).
4. Route `/api/` requests to the `api` service (port 5000).
5. Forward the `X-Real-IP` and `X-Forwarded-For` headers to backends.
6. Forward the `X-Forwarded-Proto` header so backends know the original protocol.

**Your task:** Write the full `nginx/nginx.conf` file.

### Step 6: Write the Docker Compose File

Create `docker-compose.yml` that defines three services:

1. `frontend` — uses `nginx:alpine`, mounts `./frontend` as the web root.
2. `api` — builds from `./api`, exposes port 5000 internally.
3. `proxy` — uses `nginx:alpine`, mounts your `nginx.conf` and `certs/`, maps host ports 80 and 443, depends on both `frontend` and `api`.

### Step 7: Test

```bash
docker compose up -d

# Test HTTP redirect
curl -I http://localhost/
# Expected: 301 redirect to https://localhost/

# Test frontend over HTTPS (use -k for self-signed cert)
curl -k https://localhost/
# Expected: the HTML page

# Test API through proxy
curl -k https://localhost/api/status
# Expected: JSON with service, status, timestamp, hostname

# Test health endpoint
curl -k https://localhost/api/health
# Expected: {"status": "ok"}
```

## Success Criteria

- [ ] HTTP requests on port 80 return a 301 redirect to HTTPS.
- [ ] `https://localhost/` serves the frontend HTML page.
- [ ] `https://localhost/api/status` returns valid JSON from the API.
- [ ] The API response includes the correct `hostname` of its container.
- [ ] `X-Forwarded-Proto` header reaches the backend as `https`.
- [ ] The `proxy_set_header` directives include `Host`, `X-Real-IP`, `X-Forwarded-For`, and `X-Forwarded-Proto`.

## Hints

<details>
<summary>Hint 1 — HTTP to HTTPS redirect</summary>
Use a separate `server` block listening on port 80 with `return 301 https://$host$request_uri;`.
</details>

<details>
<summary>Hint 2 — SSL configuration</summary>
The `ssl_certificate` and `ssl_certificate_key` paths must match where you mount them inside the container. A common path is `/etc/nginx/certs/`.
</details>

<details>
<summary>Hint 3 — Proxy headers</summary>
Inside each `location` block, use `proxy_set_header` directives. `$proxy_add_x_forwarded_for` appends the client IP to any existing `X-Forwarded-For` header.
</details>

<details>
<summary>Hint 4 — Docker Compose networking</summary>
Services in the same Compose file can reach each other by service name. `proxy_pass http://api:5000;` works because Docker DNS resolves `api` to the container's IP.
</details>

<details>
<summary>Hint 5 — The API sees the wrong protocol</summary>
If the API reports `http` instead of `https`, you are missing `proxy_set_header X-Forwarded-Proto $scheme;` in the `/api/` location block.
</details>
