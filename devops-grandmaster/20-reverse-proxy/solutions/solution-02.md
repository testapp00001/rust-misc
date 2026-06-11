# Solution 02: Nginx Reverse Proxy with SSL

## Complete Solution

### `nginx/nginx.conf`

```nginx
events {
    worker_connections 1024;
}

http {
    # Upstream definitions
    upstream frontend {
        server frontend:80;
    }

    upstream api {
        server api:5000;
    }

    # Redirect all HTTP traffic to HTTPS
    server {
        listen 80;
        server_name localhost;

        return 301 https://$host$request_uri;
    }

    # HTTPS server with SSL termination
    server {
        listen 443 ssl;
        server_name localhost;

        # SSL certificate paths (mounted via Docker volumes)
        ssl_certificate     /etc/nginx/certs/server.crt;
        ssl_certificate_key /etc/nginx/certs/server.key;

        # SSL protocol and cipher configuration
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;

        # Frontend: serve static files
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # API: forward to backend
        location /api/ {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

### `docker-compose.yml`

```yaml
version: "3.8"

services:
  frontend:
    image: nginx:alpine
    volumes:
      - ./frontend:/usr/share/nginx/html:ro

  api:
    build: ./api
    expose:
      - "5000"

  proxy:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/certs:/etc/nginx/certs:ro
    depends_on:
      - frontend
      - api
```

### `frontend/index.html`

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

### `api/app.py`

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

### `api/Dockerfile`

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

### Certificate Generation

```bash
mkdir -p nginx/certs
openssl req -x509 -nodes -days 365 \
  -newkey rsa:2048 \
  -keyout nginx/certs/server.key \
  -out nginx/certs/server.crt \
  -subj "/CN=localhost"
```

---

## Why It Works

### HTTP to HTTPS redirect

The first `server` block listens on port 80 and returns a `301` redirect to the HTTPS URL. The `$host` variable preserves the original hostname, and `$request_uri` preserves the full original path and query string. This ensures that even if a user types `http://...`, they end up on HTTPS.

### SSL termination

The second `server` block listens on port 443 with `ssl` enabled. The `ssl_certificate` and `ssl_certificate_key` directives point to the self-signed cert and private key mounted into the container. SSL termination happens here — backend services receive plain HTTP, which is simpler and faster for them.

### Proxy headers

- `Host $host` — tells the backend which hostname the client requested. Without this, the backend sees the internal Docker hostname (e.g., `proxy`), which breaks virtual hosting and any host-dependent logic.
- `X-Real-IP $remote_addr` — the actual client IP, not the proxy's IP. Without this, every request appears to come from `172.x.x.x` (the Docker network).
- `X-Forwarded-For $proxy_add_x_forwarded_for` — appends the client IP to any existing chain of proxies. If there are multiple proxies, this preserves the full chain.
- `X-Forwarded-Proto $scheme` — tells the backend whether the original request was HTTP or HTTPS. This is critical for the API to generate correct URLs and for frameworks that enforce HTTPS redirects.

### Upstream blocks

Defining `upstream frontend` and `upstream api` groups allows Nginx to load-balance in the future by adding more `server` lines. Even with a single server, using upstreams is a best practice for readability and future-proofing.

### Docker Compose networking

All three services are in the same Compose network. The proxy service references `frontend` and `api` by name — Docker's built-in DNS resolves these to container IPs. The `expose` directive on the API service makes port 5000 available to other containers but not to the host.

---

## Common Mistakes

### 1. Certificate path mismatch

**Symptom:** Nginx fails to start with `cannot load certificate "/etc/nginx/certs/server.crt": BIO_new_file() failed`.

**Cause:** The volume mount path does not match the `ssl_certificate` path in `nginx.conf`. If you mount to `/etc/ssl/certs/` but configure `/etc/nginx/certs/`, Nginx cannot find the file.

**Fix:** Ensure the volume mount destination and the `ssl_certificate` path are identical.

### 2. Missing `proxy_set_header Host`

**Symptom:** Frontend loads but API calls fail, or the backend logs show `Host: frontend` instead of `Host: localhost`.

**Cause:** Without `proxy_set_header Host $host;`, Nginx forwards the internal upstream hostname as the `Host` header. Flask and other frameworks may reject or misroute these requests.

### 3. Forgetting `X-Forwarded-Proto`

**Symptom:** The API reports the request protocol as `http` even though the client used `https`.

**Cause:** The backend receives plain HTTP from the proxy (SSL is terminated at the proxy). Without `X-Forwarded-Proto $scheme`, the backend has no way to know the original protocol was HTTPS.

### 4. Redirect loop

**Symptom:** Browser shows "too many redirects" error.

**Cause:** Both the HTTP and HTTPS server blocks redirect to HTTPS, or a backend framework also tries to redirect HTTP to HTTPS, creating an infinite loop.

**Fix:** Only the port 80 server block should redirect. The port 443 block should serve content.

### 5. Self-signed certificate browser warning

**Symptom:** Browser shows "Your connection is not private" warning.

**Cause:** This is expected with self-signed certificates. Browsers only trust certificates signed by a Certificate Authority.

**Fix:** For local development, use `curl -k` to skip verification, or add the certificate to your system's trust store. For production, use Let's Encrypt or a commercial CA.
