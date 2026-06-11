# Exercise 03: Design a Multi-Service Pipeline

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design a CI/CD pipeline strategy for a microservices architecture where
multiple services share common libraries. This exercise trains you to think
about pipeline dependencies, change detection, and build efficiency at scale.

## Scenario

Your organization has the following repository structure:

```
monorepo/
  libs/
    shared-auth/       # Authentication library used by all services
    shared-db/         # Database connection pool library
    shared-logging/    # Structured logging library
  services/
    api-gateway/       # Depends on shared-auth, shared-logging
    user-service/      # Depends on shared-auth, shared-db, shared-logging
    order-service/     # Depends on shared-db, shared-logging
    notification-svc/  # Depends on shared-logging
  Cargo.toml           # Workspace root
```

When a developer changes `shared-logging`, all four services need to be
tested. When a developer changes `user-service`, only `user-service` needs
to be tested. Building and testing everything on every push wastes CI
resources and slows down feedback.

## Tasks

### Part A: Design the Change Detection Strategy

Write a strategy (pseudocode or bash) that determines which services are
affected by a given commit. The output should be a list of services that
need to be built and tested.

<details>
<summary>Hint</summary>

Use `git diff` to find changed files, then map file paths to affected
services. For example, changes in `libs/shared-auth/*` affect `api-gateway`
and `user-service`. Changes in `services/order-service/*` only affect
`order-service`.

</details>

### Part B: Design the Pipeline Matrix

Create a GitHub Actions workflow that uses a dynamic matrix strategy. The
matrix should only include services that are affected by the current commit.
Each service should build and test in its own parallel job.

<details>
<summary>Hint</summary>

Use a preliminary job that outputs the matrix, then reference it in the
build jobs:

```yaml
jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.changes.outputs.matrix }}
    steps:
      - id: changes
        run: |
          # Determine affected services
          echo "matrix={\"service\":[\"user-service\",\"order-service\"]}" >> "$GITHUB_OUTPUT"

  build:
    needs: detect-changes
    strategy:
      matrix: ${{ fromJson(needs.detect-changes.outputs.matrix) }}
```

</details>

### Part C: Handle Shared Library Changes

When a shared library changes, you need to test all services that depend on
it. But you also need to test the library itself. Design the dependency
graph and explain the order of operations.

<details>
<summary>Hint</summary>

Draw the dependency graph as a tree or DAG. Libraries should be tested first
(in parallel if they are independent), then services that depend on them
should be tested after the libraries pass. If `shared-auth` and `shared-db`
both change, test them in parallel, then test all services that depend on
either one.

</details>

### Part D: Optimize for Speed

The current approach rebuilds every affected service from scratch. Propose
at least two optimizations that would reduce pipeline execution time when
only a shared library changes.

<details>
<summary>Hint</summary>

Consider:
- Caching compiled library artifacts so services do not recompile them
- Using a build matrix that only rebuilds the changed library and links
  pre-built artifacts for unchanged dependencies
- Incremental compilation strategies

</details>

## Success Criteria

- [ ] Change detection correctly maps file paths to affected services
- [ ] Pipeline uses a dynamic matrix to only build affected services
- [ ] Shared library changes trigger tests for all dependent services
- [ ] The dependency graph respects build order (libraries before services)
- [ ] At least two speed optimizations are proposed with trade-off analysis

## What You Should Understand After This Exercise

In a monorepo or microservices architecture, not every change affects every
service. A smart pipeline detects what changed and only builds/tests what
is affected. This requires a dependency graph, change detection logic, and
a dynamic build matrix -- but the payoff in CI speed and cost is enormous.
