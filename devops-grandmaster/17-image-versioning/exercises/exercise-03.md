# Exercise 03: Git SHA-Based Tagging in a CI Pipeline

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Implement Git SHA-based image tagging in a GitHub Actions CI pipeline so that every image is traceable to the exact commit that produced it, with no possibility of tag collision or ambiguity.

## Scenario

Your team builds a container image on every push to `main`. Currently, the pipeline tags every image as `latest`, which means you cannot tell which commit produced which image. You need to change the pipeline so that every image is tagged with the full Git SHA, the short SHA, and the branch name -- and `latest` is only applied on the `main` branch.

## Tasks

### Part A: Understand SHA Tagging

Before writing any YAML, answer these questions:

1. What is the difference between `git rev-parse HEAD` and `git rev-parse --short HEAD`? When would you use each as a Docker tag?
2. Can two different commits ever produce the same short SHA? What are the implications for Docker tags?
3. If your pipeline runs on a pull request (not a push to `main`), should the image be pushed to the production registry? Why or why not?
4. What is the difference between tagging an image with a Git SHA and tagging it with a Git tag (e.g., `v1.2.3`)?

<details>
<summary>Hint</summary>

Short SHAs are truncated (7-12 characters). Collisions are extremely rare but theoretically possible. PR builds should validate but not publish. Git SHAs identify commits; Git tags identify releases.

</details>

### Part B: Write the CI Pipeline

Create a GitHub Actions workflow `.github/workflows/build.yml` that:

1. Triggers on pushes to `main` and on pull requests targeting `main`.
2. Builds a Docker image from the repository root.
3. Tags the image with:
   - The full 40-character SHA: `myapp:<full-sha>`
   - The short SHA: `myapp:<short-sha>`
   - The branch name: `myapp:<branch-name>`
   - `latest` (only on pushes to `main`, never on PRs)
4. Pushes the image to `ghcr.io` only on pushes to `main` (not on PRs).
5. Uses Docker Buildx with layer caching for faster builds.

The workflow must:

- Use `docker/build-push-action@v5` for the build and push.
- Use `docker/metadata-action@v5` to generate tags.
- Use `docker/login-action@v3` for registry authentication.
- Set appropriate `permissions:` for the workflow.

Write the complete workflow YAML.

<details>
<summary>Hint</summary>

Use `docker/metadata-action` with `type=sha` and `type=ref,event=branch` tag types. Use `type=raw,value=latest,enable={{is_default_branch}}` to conditionally apply the `latest` tag. Use `push: ${{ github.event_name == 'push' }}` to control when images are pushed.

</details>

### Part C: Add Build Metadata Labels

Extend the pipeline to add OCI-compliant labels to the image. The labels must include:

1. `org.opencontainers.image.version` -- Set to the short SHA.
2. `org.opencontainers.image.revision` -- Set to the full SHA.
3. `org.opencontainers.image.created` -- Set to the build timestamp in ISO 8601.
4. `org.opencontainers.image.source` -- Set to the repository URL.
5. `org.opencontainers.image.title` -- Set to the repository name.

Use `docker/metadata-action` to generate these labels automatically.

<details>
<summary>Hint</summary>

`docker/metadata-action` generates OCI labels automatically when you use its `labels` output. Pass `${{ steps.meta.outputs.labels }}` to `docker/build-push-action`.

</details>

### Part D: Add Image Summary

After the build, the pipeline must write a summary to `$GITHUB_STEP_SUMMARY` that includes:

1. The image name and all tags that were pushed.
2. The Git SHA and branch.
3. The image digest (content-addressable hash).
4. A link to the image in the GitHub Container Registry.
5. The build duration.

Write the step that generates this summary.

<details>
<summary>Hint</summary>

The `docker/build-push-action` outputs `digest` (the image's content hash) and `imageid`. Use `${{ steps.build.outputs.digest }}` in your summary. Measure duration by recording timestamps before and after the build step.

</details>

### Part E: Add a PR Comment with Image Details

For pull request builds, add a step that posts a comment on the PR with:

1. The short SHA of the commit.
2. The image tags that were generated (but not pushed).
3. A note that the image was built but not pushed (since it is a PR).
4. A link to the workflow run.

This comment should update (not duplicate) if the PR is updated.

<details>
<summary>Hint</summary>

Use `peter-evans/create-or-update-comment@v4` with a `comment-id` stored from a previous step to enable updating. Alternatively, use `github-script` to find and update an existing bot comment.

</details>

## Success Criteria

- [ ] The pipeline triggers on both pushes to `main` and PRs targeting `main`.
- [ ] Images are tagged with full SHA, short SHA, and branch name on every build.
- [ ] The `latest` tag is only applied on pushes to `main`.
- [ ] Images are pushed to `ghcr.io` only on pushes to `main`, not on PRs.
- [ ] OCI labels include version, revision, created date, source URL, and title.
- [ ] The GitHub Actions job summary shows image tags, digest, and build metadata.
- [ ] PR builds post a comment with image details.
- [ ] The workflow uses Docker Buildx with layer caching.

## What You Should Understand After This Exercise

Git SHA tagging creates a 1:1 mapping between commits and images. Every image can be traced back to its source code by reading the tag. The `latest` tag should only be applied to the main branch to avoid confusion. PR builds should validate the image (build and test) without pushing to the registry. OCI labels embed metadata directly in the image, making it self-describing even outside the CI system. The image digest is the true immutable identifier -- tags are human-friendly aliases that point to digests.
