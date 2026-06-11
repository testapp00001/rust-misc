# Solution 03: Git SHA-Based Tagging in a CI Pipeline

## Part A: Understand SHA Tagging

### 1. Full SHA vs Short SHA

`git rev-parse HEAD` returns the full 40-character SHA (e.g., `a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2`). `git rev-parse --short HEAD` returns a 7-character prefix (e.g., `a1b2c3d`). Use the full SHA when you need guaranteed uniqueness (e.g., signing, auditing). Use the short SHA for human-readable tags where 7 characters provide sufficient uniqueness for most repositories.

### 2. Short SHA collisions

Yes, two different commits can theoretically produce the same short SHA, but the probability is extremely low for 7-character prefixes in repositories with fewer than ~200,000 commits. For Docker tags, a collision would mean two different images share the same tag, which could cause the wrong image to be pulled. Using the full SHA eliminates this risk entirely.

### 3. PR builds and the registry

No. Pull request builds should validate the image (build and test) but should NOT push to the production registry. PRs represent unmerged, potentially unreviewed code. Pushing PR images to the registry creates noise, wastes storage, and could expose unreviewed code. PR builds should push to a temporary registry or skip the push entirely.

### 4. SHA tags vs Git tags

A Git SHA tag identifies a specific commit -- every commit gets one automatically. A Git tag (e.g., `v1.2.3`) is a human-created pointer to a commit, typically used for releases. Git tags are created manually or by release automation. Git SHA tags are created automatically by CI. Both are immutable, but they serve different purposes: SHA tags provide traceability; Git tags provide semantic meaning.

### Why This Works

Understanding the difference between full and short SHAs, and between SHA tags and Git tags, is essential for designing a tagging strategy that is both traceable and readable.

### Common Mistakes

- **Using short SHAs in security-critical contexts.** For image signing and audit trails, use the full SHA.
- **Pushing PR images to the production registry.** PR builds should validate, not publish.
- **Treating Git SHA tags and Git tags as interchangeable.** They serve different purposes.

## Part B: Write the CI Pipeline

### .github/workflows/build.yml

```yaml
name: Build and Push Image

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

permissions:
  contents: read
  packages: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        if: github.event_name == 'push'
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate image metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=sha,prefix=
            type=sha,format=long,prefix=
            type=ref,event=branch
            type=raw,value=latest,enable=${{ github.ref == format('refs/heads/{0}', 'main') && github.event_name == 'push' }}

      - name: Build and push Docker image
        id: build
        uses: docker/build-push-action@v5
        with:
          context: .
          push: ${{ github.event_name == 'push' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### Why This Works

The workflow triggers on both pushes to `main` and PRs targeting `main`. The `docker/metadata-action` generates tags automatically: `type=sha,prefix=` produces the short SHA, `type=sha,format=long,prefix=` produces the full SHA, `type=ref,event=branch` produces the branch name, and `type=raw,value=latest,enable=...` conditionally applies the `latest` tag only on the main branch. The `push: ${{ github.event_name == 'push' }}` ensures images are only pushed on merges to main, not on PRs. The `permissions:` block grants `packages: write` for pushing to GHCR.

### Common Mistakes

- **Using `enable={{is_default_branch}}` without the full condition.** The `is_default_branch` filter in `docker/metadata-action` works, but the explicit branch check is clearer.
- **Not setting `permissions:`.** Without `packages: write`, the push step fails with a 403 error.
- **Pushing on PRs.** The `push:` condition must check the event name, not just the branch.

## Part C: Add Build Metadata Labels

The `docker/metadata-action` automatically generates OCI labels when you use its `labels` output. The complete workflow from Part B already includes this -- `${{ steps.meta.outputs.labels }}` is passed to `docker/build-push-action`.

The labels generated include:

```yaml
org.opencontainers.image.version=a1b2c3d        # Short SHA
org.opencontainers.image.revision=a1b2c3d4e5f6   # Full SHA
org.opencontainers.image.created=2024-01-15T10:30:00Z  # Build timestamp
org.opencontainers.image.source=https://github.com/org/repo
org.opencontainers.image.title=repo-name
org.opencontainers.image.url=https://github.com/org/repo
org.opencontainers.image.description=...
```

### Why This Works

The `docker/metadata-action` reads the Git context (commit SHA, repository URL, timestamps) and generates OCI-compliant labels automatically. This is preferable to manually constructing labels in the Dockerfile because the metadata is derived from the CI environment, not from build arguments.

### Common Mistakes

- **Manually constructing labels in the Dockerfile.** This duplicates effort and may produce different values than the CI system.
- **Not passing `labels` to the build action.** The labels are generated but never applied to the image.

## Part D: Add Image Summary

```yaml
      - name: Generate build summary
        if: always()
        run: |
          START_TIME=${{ steps.build-start.outputs.time }}
          END_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)

          cat >> $GITHUB_STEP_SUMMARY << 'EOF'
          ## Image Build Summary

          | Property | Value |
          |----------|-------|
          EOF

          echo "| **Image** | \`ghcr.io/${{ github.repository }}\` |" >> $GITHUB_STEP_SUMMARY
          echo "| **Short SHA** | \`${{ github.sha }}\` |" >> $GITHUB_STEP_SUMMARY
          echo "| **Branch** | \`${{ github.ref_name }}\` |" >> $GITHUB_STEP_SUMMARY
          echo "| **Digest** | \`${{ steps.build.outputs.digest }}\` |" >> $GITHUB_STEP_SUMMARY
          echo "| **Registry** | [GitHub Container Registry](https://github.com/${{ github.repository }}/pkgs/container/${{ github.event.repository.name }}) |" >> $GITHUB_STEP_SUMMARY
          echo "| **Pushed** | ${{ github.event_name == 'push' }} |" >> $GITHUB_STEP_SUMMARY

          echo "" >> $GITHUB_STEP_SUMMARY
          echo "### Tags" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo '```' >> $GITHUB_STEP_SUMMARY
          echo "${{ steps.meta.outputs.tags }}" >> $GITHUB_STEP_SUMMARY
          echo '```' >> $GITHUB_STEP_SUMMARY

      - name: Record build start time
        id: build-start
        run: echo "time=$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> $GITHUB_OUTPUT
```

### Why This Works

The summary writes markdown to `$GITHUB_STEP_SUMMARY`, which GitHub renders in the Actions UI. The image digest from `${{ steps.build.outputs.digest }}` is the content-addressable hash -- the true immutable identifier. The link to the GHCR package page allows quick navigation to the published image.

### Common Mistakes

- **Not using `$GITHUB_STEP_SUMMARY`.** Writing to stdout creates log noise. The summary is a dedicated markdown section in the Actions UI.
- **Missing the digest.** The digest is the most important piece of metadata for traceability and rollback.

## Part E: Add a PR Comment with Image Details

```yaml
      - name: Find existing bot comment
        if: github.event_name == 'pull_request'
        uses: peter-evans/find-comment@v3
        id: find-comment
        with:
          issue-number: ${{ github.event.pull_request.number }}
          comment-author: 'github-actions[bot]'
          body-includes: '## Image Build Preview'

      - name: Create or update PR comment
        if: github.event_name == 'pull_request'
        uses: peter-evans/create-or-update-comment@v4
        with:
          comment-id: ${{ steps.find-comment.outputs.comment-id }}
          issue-number: ${{ github.event.pull_request.number }}
          edit-mode: replace
          body: |
            ## Image Build Preview

            | Property | Value |
            |----------|-------|
            | **Commit** | `${{ github.sha }}` |
            | **Short SHA** | `${{ steps.meta.outputs.version }}` |
            | **Branch** | `${{ github.ref_name }}` |
            | **Status** | Built (not pushed -- this is a PR) |
            | **Workflow Run** | [View logs](${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}) |

            > This image was built for validation only. It will be pushed to the registry when merged to `main`.
```

### Why This Works

The `find-comment` action searches for an existing comment by the bot with a specific marker. If found, `create-or-update-comment` updates it instead of creating a duplicate. The `edit-mode: replace` ensures the comment is fully replaced, not appended. This keeps PR comments clean even when the PR is updated multiple times.

### Common Mistakes

- **Creating duplicate comments on each push.** Without `find-comment`, every push to the PR creates a new comment.
- **Not noting that the image was not pushed.** Developers may assume the image is available in the registry.
- **Missing the workflow run link.** The link helps reviewers quickly access the build logs.

## Key Takeaway

Git SHA tagging in CI creates an automatic, 1:1 mapping between commits and images. The `docker/metadata-action` generates all necessary tags and labels from the Git context, eliminating manual tag construction. PR builds validate without publishing, keeping the registry clean. OCI labels embed metadata directly in the image, making it self-describing. The image digest is the true immutable identifier -- tags are human-friendly aliases. Job summaries and PR comments provide visibility into what was built and how.
