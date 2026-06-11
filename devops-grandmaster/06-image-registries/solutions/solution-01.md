# Solution 01: Registry Architecture and Tag Strategies

## Part A: Registry Architecture

### Question 1: The Push-Pull Model

When you run `docker push myregistry.com/myapp:1.0`, the following happens at
the layer level:

1. **Docker contacts the registry.** The Docker client authenticates with
   `myregistry.com` and initiates a push for the repository `myapp` with tag
   `1.0`.

2. **Docker sends the image manifest.** The manifest is a JSON document that
   lists all the layers (by SHA256 digest) that make up the image. Think of it
   as a table of contents.

3. **For each layer, Docker checks if the registry already has it.** Before
   uploading, the client sends a HEAD request with the layer's SHA256 digest.
   If the registry responds with `200 OK`, the layer already exists and the
   upload is skipped. This is the key optimization -- shared base layers are
   never uploaded twice.

4. **Only new layers are uploaded.** Layers that the registry does not have are
   uploaded via the blob upload API. The registry stores them in its
   content-addressable storage backend.

5. **The tag is created.** Once all layers are present, the registry creates a
   tag entry that maps `1.0` to the manifest digest. The tag is just a pointer
   to a manifest, not a copy of the image.

**Concrete example:** If your image has three layers (base OS, dependencies,
application code) and the registry already has the base OS layer from a
previous push, only two layers are transferred. The base layer check happens
in milliseconds.

### Question 2: Content-Addressable Storage

**Why this matters for storage efficiency:**

Content-addressable storage means each layer is identified by its SHA256 hash,
not by a human-readable name. If two images contain an identical layer (same
files, same permissions, same timestamps), that layer has the same hash and is
stored only once. A registry with 100 images all based on `ubuntu:22.04` stores
the Ubuntu base layer once, not 100 times.

**How two different images can share layers:**

Consider `myapp:1.0` and `myapp:2.0`:

```
myapp:1.0                          myapp:2.0
  Layer 1: ubuntu:22.04 (sha256:aaa)  Layer 1: ubuntu:22.04 (sha256:aaa)  -- SAME
  Layer 2: apt-get install (sha256:bbb) Layer 2: apt-get install (sha256:bbb) -- SAME
  Layer 3: COPY v1 code (sha256:ccc)   Layer 3: COPY v2 code (sha256:ddd)   -- DIFFERENT
  Layer 4: RUN build v1 (sha256:eee)   Layer 4: RUN build v2 (sha256:fff)   -- DIFFERENT
```

Layers 1 and 2 are identical in both images because the Dockerfile
instructions produce the same output. The registry stores layers aaa and bbb
once, then stores ccc, ddd, eee, and fff separately.

**What happens when you push two images that share a base layer:**

When you push the first image, all four layers are uploaded. When you push the
second image, the client sends HEAD requests for each layer. The registry
responds `200 OK` for layers aaa and bbb (already stored), so those are
skipped. Only layers ddd and fff are uploaded. This is why the second push is
faster and why registry storage does not grow linearly with the number of
images.

### Question 3: Registry vs. Repository vs. Tag

```
Registry:   A server that stores and distributes container images.
            Examples: docker.io, ghcr.io, 123456789012.dkr.ecr.us-east-1.amazonaws.com

Repository: A named collection of related image tags within a registry.
            A repository holds all versions of a single application or base image.
            Example: library/nginx (on docker.io), mycompany/user-service (on ghcr.io)

Tag:        A human-readable label that points to a specific image manifest
            (and therefore a specific set of layers) within a repository.
            Example: 1.25, latest, alpine, main-abc1234
```

**Concrete example showing the relationship:**

```
Full reference:  ghcr.io/novacorp/backend/user-service:1.2.3
                 ─────────  ────────  ──────────────  ─────
                 Registry    Org      Repository      Tag
```

- `ghcr.io` is the registry (GitHub Container Registry).
- `novacorp/backend` is the organizational namespace.
- `user-service` is the repository (all versions of this service).
- `1.2.3` is the tag pointing to a specific image build.

A single repository can have many tags. Each tag points to exactly one image
manifest. Multiple tags can point to the same manifest (e.g., `1.2.3` and
`1.2` and `latest` might all point to the same image).

---

## Part B: Tag Strategies

### Question 4: The `latest` Tag

Pulling `latest` in production is risky for several reasons:

1. **`latest` is not automatically updated.** When you push `myapp:2.0`, the
   `latest` tag does not magically update to point to `2.0`. It still points
   to whatever it was last explicitly tagged to. If nobody ran
   `docker tag myapp:2.0 myapp:latest && docker push myapp:latest`, then
   `latest` still points to the old version.

2. **`latest` has no audit trail.** When you deploy `myapp:1.2.3`, you know
   exactly what you deployed. When you deploy `myapp:latest`, you have to
   check the registry to figure out what `latest` actually points to. Six
   months later, nobody remembers.

3. **`latest` is ambiguous across environments.** Your staging `latest` might
   point to `1.2.3` while production `latest` points to `1.2.1`. They look
   like the same tag but run different code.

4. **Reproducibility is impossible.** If a production incident occurs and you
   need to roll back, "go back to the previous `latest`" is not a meaningful
   instruction. You need an exact version number.

5. **Cache confusion.** If a machine has `myapp:latest` cached locally, it
   will not pull the updated version from the registry unless you explicitly
   force it. Different machines end up running different versions of "latest."

**What `latest` actually means:** It is just a default tag name. Docker uses
`latest` when you omit the tag (e.g., `docker pull myapp` is equivalent to
`docker pull myapp:latest`). It has no special behavior beyond that.

### Question 5: Compare Tag Strategies

| Strategy | When to Use | Advantage | Disadvantage |
|----------|-------------|-----------|--------------|
| **Semantic Versioning** (`1.2.3`) | Production releases, public APIs, any artifact that consumers depend on. | Users can pin at different precision levels (`1`, `1.2`, `1.2.3`). Communicates the nature of changes (breaking vs. patch). Clear rollback path. | Requires discipline to follow semver correctly. A wrong version bump (e.g., breaking change in a patch release) breaks trust. Not suitable for every-commit builds. |
| **Git SHA** (`abc1234`) | Every CI/CD build, development, staging, any build that needs exact traceability. | Uniquely identifies the source code that produced the image. Impossible to confuse two builds. Perfect audit trail. No version bumping discipline required. | Not human-readable. Users cannot tell if `abc1234` is newer than `def5678` without checking Git. Requires a mapping tool or CI system to navigate. |
| **Branch Name** (`main`, `develop`) | Staging environments, development branches, "always latest from this branch" use cases. | Easy to understand what code the image contains. Useful for auto-deploying the latest commit on a branch. | The meaning changes every time someone pushes to the branch. Two deploys of `main` can run completely different code. No history -- you lose the previous image reference when the tag moves. |

### Question 6: Identify the Bad Practices

**`company/myapp:production`**

- **Problem:** This tags an image with an environment name. The same image
  should be promoted from staging to production -- the environment is a
  deployment target, not an image property. If you tag `myapp:1.2.3` as
  `myapp:production` and then deploy a new version, you must re-tag
  `production` to the new image. Now `production` means different things at
  different times.
- **Fix:** Use immutable version tags (`1.2.3` or git SHA) and let your
  deployment system track which version is deployed where. The image tag
  should describe *what* the image contains, not *where* it runs.

**`company/myapp:latest`**

- **Problem:** In production, `latest` is ambiguous and dangerous. You cannot
  tell what version is running without checking the registry. Rollback is
  impossible because "the previous latest" is not a meaningful reference.
- **Fix:** For production, pin to an exact version (`1.2.3` or git SHA).
  Reserve `latest` only as a convenience for local development, and even
  then, document what it points to.

**`company/myapp:test-works-finally`**

- **Problem:** This is a descriptive, emotional tag. It tells you the
  developer's state of mind, not what the image contains. It is not
  reproducible, not sortable, and not parseable by any automation. What
  happens when the next test also "finally works"?
- **Fix:** Use a CI-generated tag based on git SHA or version number. If you
  need to mark that tests passed, use a CI status check or a registry
  label/metadata -- not the tag name.

**`company/myapp:DO-NOT-DELETE`**

- **Problem:** This is a plea disguised as a tag. It indicates that someone
  is afraid of losing this image, which means there is no proper versioning
  or retention policy. Tags should describe content, not express fear. If
  this image is important, it should have a proper version tag that is
  protected by a retention policy.
- **Fix:** Use a proper version tag (`1.2.3`). Configure retention policies
  that protect release tags while cleaning up old development builds. If the
  image must be preserved, use registry-side tag immutability rules (Harbor
  and ECR both support this).

---

## Common Mistakes

1. **Confusing registry with repository.** Saying "push to Docker Hub" is
   correct (that is the registry). Saying "push to nginx" is ambiguous -- do
   you mean the `library/nginx` repository on Docker Hub, or an `nginx`
   repository on your private registry? Always specify the full reference.

2. **Thinking tags are immutable copies.** Tags are mutable pointers. Running
   `docker tag myapp:1.0.1 myapp:1.0` changes what `1.0` points to -- it does
   not copy the image. Both tags now reference the same image ID.

3. **Assuming `latest` is the most recent build.** It is not. It is just a
   tag name with no special behavior. If you push `myapp:2.0` without
   updating `latest`, then `latest` still points to whatever it previously
   pointed to.

4. **Using `docker pull myapp` without a tag in scripts.** This implicitly
   uses `latest`, which makes builds non-reproducible. Always specify an
   explicit tag in scripts, CI pipelines, and deployment manifests.

5. **Creating tags that encode mutable state.** Tags like `production`,
   `stable`, or `approved` change meaning over time. This defeats the
   purpose of tagging, which is to provide a fixed reference to a specific
   build.
