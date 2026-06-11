# Cheatsheet: Image Registries

## Docker Hub

```bash
# Login
docker login
docker login -u username

# Tag for Docker Hub
docker tag myapp:1.0 username/myapp:1.0

# Push
docker push username/myapp:1.0

# Pull
docker pull username/myapp:1.0
```

## Private Registries

```bash
# AWS ECR
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin 123456789.dkr.ecr.us-east-1.amazonaws.com
docker tag myapp:1.0 123456789.dkr.ecr.us-east-1.amazonaws.com/myapp:1.0
docker push 123456789.dkr.ecr.us-east-1.amazonaws.com/myapp:1.0

# Google GCR
gcloud auth configure-docker
docker tag myapp:1.0 gcr.io/my-project/myapp:1.0
docker push gcr.io/my-project/myapp:1.0

# Azure ACR
az acr login --name myregistry
docker tag myapp:1.0 myregistry.azurecr.io/myapp:1.0
docker push myregistry.azurecr.io/myapp:1.0

# Harbor
docker login harbor.example.com
docker tag myapp:1.0 harbor.example.com/myproject/myapp:1.0
docker push harbor.example.com/myproject/myapp:1.0
```

## Self-Hosted Registry

```bash
# Run a local registry
docker run -d -p 5000:5000 --name registry registry:2

# Tag and push
docker tag myapp:1.0 localhost:5000/myapp:1.0
docker push localhost:5000/myapp:1.0

# With TLS and auth (production)
docker run -d -p 5000:5000 \
  -v /certs:/certs \
  -v /auth:/auth \
  -e REGISTRY_HTTP_TLS_CERTIFICATE=/certs/domain.crt \
  -e REGISTRY_HTTP_TLS_KEY=/certs/domain.key \
  -e REGISTRY_AUTH=htpasswd \
  -e REGISTRY_AUTH_HTPASSWD_PATH=/auth/htpasswd \
  -e REGISTRY_AUTH_HTPASSWD_REALM="Registry Realm" \
  --name registry registry:2
```

## Tag Strategies

```bash
# Semantic versioning
myapp:1.0.0
myapp:1.0.1
myapp:1.1.0

# Git SHA (recommended for CI)
myapp:abc1234

# Environment
myapp:staging
myapp:production

# Latest (avoid in production)
myapp:latest
```

## Image Management

```bash
# List local images
docker images

# Remove image
docker rmi myapp:1.0

# Remove unused images
docker image prune -a

# Inspect image
docker inspect myapp:1.0

# Check image layers
docker history myapp:1.0

# Export/Import
docker save myapp:1.0 -o myapp.tar
docker load -i myapp.tar
```

## Registry Garbage Collection

```bash
# Harbor: Clean unused artifacts
docker exec -it harbor-core harbor gc --dry-run
docker exec -it harbor-core harbor gc

# Docker Registry: Garbage collect
docker exec -it registry bin/registry garbage-collect /etc/docker/registry/config.yml
```
