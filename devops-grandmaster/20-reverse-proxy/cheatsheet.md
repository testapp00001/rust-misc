# Cheatsheet: Reverse Proxy

## Forward vs Reverse Proxy

| | Forward Proxy | Reverse Proxy |
|---|---|---|
| Who | Client-side | Server-side |
| Purpose | Access control, anonymity | Load balancing, SSL, caching |
| Example | VPN, corporate proxy | Nginx, Traefik, HAProxy |

## Nginx Reverse Proxy
```nginx
server {
    listen 443 ssl http2;
    server_name example.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    # API backend
    location /api/ {
        proxy_pass http://api:3000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Frontend
    location / {
        root /usr/share/nginx/html;
        try_files $uri $uri/ /index.html;
    }
}
```

## Traefik (Auto-discovery)
```yaml
# docker-compose.yml
services:
  traefik:
    image: traefik:v2.10
    command:
      - --providers.docker=true
      - --entrypoints.web.address=:80
      - --entrypoints.websecure.address=:443
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro

  api:
    image: my-api
    labels:
      - traefik.enable=true
      - traefik.http.routers.api.rule=Host(`api.example.com`)
```

## Benefits
- SSL termination
- Load balancing
- Caching
- Compression
- Rate limiting
- Security headers
