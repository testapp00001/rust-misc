# Cheatsheet: Pipeline Design

## Pipeline Stages
```
Code → Lint → Test → Build → Scan → Push → Deploy (Staging) → Deploy (Prod)
```

## GitHub Actions Template
```yaml
name: CI/CD Pipeline
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: ruff check .

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pytest

  build:
    needs: [lint, test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/build-push-action@v5
        with:
          push: false
          tags: my-app:${{ github.sha }}

  scan:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: aquasecurity/trivy-action@master
        with:
          image-ref: my-app:${{ github.sha }}

  deploy:
    needs: scan
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: production
    steps:
      - run: kubectl set image deployment/my-app my-app=my-app:${{ github.sha }}
```

## Best Practices
- Run tests in parallel
- Cache dependencies
- Use matrix builds
- Scan before push
- Deploy staging before production
- Use environments with approval gates
