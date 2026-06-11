# Cheatsheet: Load Balancing

## Algorithms

| Algorithm | Description | Best For |
|-----------|-------------|----------|
| Round Robin | Rotate through servers | Equal servers |
| Weighted Round Robin | Rotate with weights | Unequal servers |
| Least Connections | Send to least busy | Varying request times |
| IP Hash | Same client → same server | Sticky sessions |
| Random | Random selection | Simple, effective |

## Nginx Load Balancer
```nginx
upstream backend {
    # Round Robin (default)
    server backend1:8080;
    server backend2:8080;
    server backend3:8080;

    # Other methods
    # least_conn;
    # ip_hash;
}

server {
    listen 80;
    location / {
        proxy_pass http://backend;
    }
}
```

## Docker Compose Scaling
```yaml
services:
  api:
    image: my-api
    deploy:
      replicas: 3
    # Use nginx for load balancing
  nginx:
    image: nginx
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
```

## Health Checks in Load Balancer
```nginx
upstream backend {
    server backend1:8080 max_fails=3 fail_timeout=30s;
    server backend2:8080 max_fails=3 fail_timeout=30s;
}
```
