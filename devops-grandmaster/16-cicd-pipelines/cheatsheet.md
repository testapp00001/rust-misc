# Cheatsheet: CI/CD Pipelines

## Pipeline Stages
```
Code → Build → Test → Scan → Push → Deploy (Staging) → Deploy (Prod)
```

## GitHub Actions Docker Pipeline
```yaml
name: CI/CD
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/build-push-action@v5
        with:
          push: false
          tags: my-app:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  test:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - run: docker run my-app:${{ github.sha }} python -m pytest

  deploy:
    needs: test
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - run: kubectl set image deployment/my-app my-app=my-app:${{ github.sha }}
```

## Best Practices
- Run tests in parallel
- Cache dependencies
- Use matrix builds for multiple versions
- Scan for vulnerabilities before push
- Deploy to staging before production
- Use environments with approval gates
