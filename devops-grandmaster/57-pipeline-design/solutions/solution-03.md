# Solution 03: Design a Multi-Service Pipeline

## Part A: Change Detection Strategy

```bash
#!/bin/bash
# detect-changes.sh - determines which services are affected by the current commit

set -euo pipefail

# Get list of changed files
CHANGED_FILES=$(git diff --name-only HEAD~1 HEAD)

# Define dependency map: library -> services that depend on it
declare -A LIB_TO_SERVICES
LIB_TO_SERVICES["libs/shared-auth"]="api-gateway user-service"
LIB_TO_SERVICES["libs/shared-db"]="user-service order-service"
LIB_TO_SERVICES["libs/shared-logging"]="api-gateway user-service order-service notification-svc"

# Track affected services (using associative array for deduplication)
declare -A AFFECTED

for file in $CHANGED_FILES; do
    # Check if the file is in a service directory
    if [[ "$file" =~ ^services/([^/]+)/ ]]; then
        service="${BASH_REMATCH[1]}"
        AFFECTED["$service"]=1
    fi

    # Check if the file is in a shared library
    for lib in "${!LIB_TO_SERVICES[@]}"; do
        if [[ "$file" =~ ^"$lib" ]]; then
            # Add all services that depend on this library
            for svc in ${LIB_TO_SERVICES[$lib]}; do
                AFFECTED["$svc"]=1
            done
        fi
    done
done

# Output as JSON array for GitHub Actions matrix
SERVICES=$(printf '"%s",' "${!AFFECTED[@]}" | sed 's/,$//')
echo "{\"service\":[$SERVICES]}"
```

### Why This Works

The script builds a dependency map from libraries to services. For each
changed file, it checks if the file is in a service directory (direct
change) or in a shared library (transitive change). The associative array
ensures each service appears only once even if multiple libraries change.

## Part B: Pipeline Matrix

```yaml
name: Multi-Service CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.changes.outputs.matrix }}
      has_changes: ${{ steps.changes.outputs.has_changes }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 2  # Need HEAD~1 for diff

      - name: Detect affected services
        id: changes
        run: |
          # Run the detection script
          MATRIX=$(bash .github/scripts/detect-changes.sh)
          echo "matrix=$MATRIX" >> "$GITHUB_OUTPUT"

          # Check if any services are affected
          if [ "$MATRIX" = '{"service":[]}' ]; then
            echo "has_changes=false" >> "$GITHUB_OUTPUT"
          else
            echo "has_changes=true" >> "$GITHUB_OUTPUT"
          fi

          echo "Affected services: $MATRIX"

  build-and-test:
    needs: detect-changes
    if: needs.detect-changes.outputs.has_changes == 'true'
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{ fromJson(needs.detect-changes.outputs.matrix) }}
      fail-fast: false
    steps:
      - uses: actions/checkout@v4

      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build service
        run: cargo build -p ${{ matrix.service }}

      - name: Test service
        run: cargo test -p ${{ matrix.service }}

      - name: Build Docker image
        run: |
          docker build -t ghcr.io/${{ github.repository }}/${{ matrix.service }}:${{ github.sha }} \
            -f services/${{ matrix.service }}/Dockerfile .
```

### Why This Works

The `detect-changes` job runs first and outputs a JSON matrix. The
`build-and-test` job uses `fromJson()` to parse the matrix and creates
one parallel job per affected service. The `fail-fast: false` setting
ensures all services are tested even if one fails.

The `fetch-depth: 2` is critical -- it fetches enough history for
`git diff HEAD~1 HEAD` to work. Without it, the default shallow clone
only has one commit and the diff would be empty.

## Part C: Handle Shared Library Changes

### Dependency Graph

```
shared-auth ──────┬──> api-gateway
                  └──> user-service

shared-db ────────┬──> user-service
                  └──> order-service

shared-logging ───┬──> api-gateway
                  ├──> user-service
                  ├──> order-service
                  └──> notification-svc
```

### Build Order

When `shared-logging` changes:

1. **Phase 1 (parallel):** Build and test `shared-logging` itself
2. **Phase 2 (parallel):** Build and test all four services that depend on it

When `shared-auth` AND `shared-db` both change:

1. **Phase 1 (parallel):** Build and test `shared-auth` and `shared-db`
2. **Phase 2 (parallel):** Build and test `api-gateway`, `user-service`,
   and `order-service` (the union of dependents)

### Pipeline Implementation

```yaml
  # Phase 1: Test changed libraries
  test-libraries:
    needs: detect-changes
    if: needs.detect-changes.outputs.libs_changed == 'true'
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{ fromJson(needs.detect-changes.outputs.lib_matrix) }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Test library
        run: cargo test -p ${{ matrix.library }}

  # Phase 2: Test affected services (only after libraries pass)
  test-services:
    needs: [detect-changes, test-libraries]
    if: |
      always() &&
      needs.detect-changes.outputs.has_changes == 'true' &&
      (needs.test-libraries.result == 'success' || needs.test-libraries.result == 'skipped')
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{ fromJson(needs.detect-changes.outputs.matrix) }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Test service
        run: cargo test -p ${{ matrix.service }}
```

### Why This Works

Libraries are tested first because a broken library makes service tests
meaningless. The `always()` condition combined with the library result
check ensures services are tested when libraries pass OR when no libraries
changed (the `skipped` case).

## Part D: Optimize for Speed

### Optimization 1: Shared Build Cache with Rust Compilation Units

Instead of each service compiling shared libraries independently, use a
shared cache of compiled library artifacts:

```yaml
- name: Cache compiled libraries
  uses: actions/cache@v4
  with:
    path: |
      target/release/deps/libshared_*
      target/release/.fingerprint/shared-*
    key: libs-${{ hashFiles('libs/**/Cargo.toml', 'libs/**/*.rs') }}
```

**Trade-off:** Cache invalidation is coarse-grained. If any file in `libs/`
changes, all cached library artifacts are invalidated. But when no library
files change, services skip recompiling shared code entirely.

**Estimated savings:** 2-4 minutes per service job when libraries are unchanged.

### Optimization 2: Pre-built Library Artifacts

Build shared libraries once and upload them as artifacts for service jobs
to download:

```yaml
  build-libs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build libraries
        run: |
          cargo build -p shared-auth -p shared-db -p shared-logging --release
      - name: Upload library artifacts
        uses: actions/upload-artifact@v4
        with:
          name: compiled-libs
          path: target/release/deps/libshared_*

  test-services:
    needs: build-libs
    steps:
      - name: Download library artifacts
        uses: actions/download-artifact@v4
        with:
          name: compiled-libs
          path: target/release/deps/
      # Service build now skips library compilation
```

**Trade-off:** Adds an upload/download step (~30 seconds), but saves 2-4
minutes of compilation per service. Net savings increase with the number
of services.

### Optimization 3: Conditional Library Testing

Only test shared libraries if their source code actually changed. If only
a service changed, skip library testing entirely:

```yaml
  test-libraries:
    if: needs.detect-changes.outputs.libs_changed == 'true'
```

**Trade-off:** Saves the full library test job runtime when no libraries
changed. Risk is minimal because library tests are already covered by
previous commits.

## Common Mistakes to Avoid

- **Building everything on every push.** Without change detection, a
  one-line change to `notification-svc` triggers builds for all four
  services and three libraries. This wastes CI minutes and slows feedback.
- **Not respecting dependency order.** If you test services before their
  shared libraries pass, a broken library causes all service tests to fail
  with confusing compilation errors. Test libraries first.
- **Shallow clones with diff-based detection.** `git diff` requires at
  least two commits. The default `actions/checkout` with `fetch-depth: 1`
  breaks change detection. Always use `fetch-depth: 2` or more.
- **Forgetting the union of dependents.** When `shared-auth` changes, you
  must test both `api-gateway` and `user-service`. Forgetting one means
  a broken library can ship without detection.

## Key Takeaway

In a monorepo, smart change detection and a dynamic build matrix are
essential for pipeline efficiency. The dependency graph determines build
order, change detection determines what to build, and the matrix strategy
determines how to build it. Together, they turn a 30-minute "build
everything" pipeline into a 5-minute "build what changed" pipeline.
