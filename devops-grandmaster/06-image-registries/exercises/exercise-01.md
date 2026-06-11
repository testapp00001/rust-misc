# Exercise 01: Registry Architecture and Tag Strategies

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

---

## Objective

Demonstrate your understanding of how image registries work, how images are stored and distributed, and why tag strategies matter for production systems.

---

## Part A: Registry Architecture

Answer the following questions in writing. Be specific -- use concrete examples rather than vague descriptions.

### Question 1: The Push-Pull Model

Explain what happens when you run `docker push myregistry.com/myapp:1.0`. Describe the journey of an image from your local machine to the registry. What happens at the layer level?

### Question 2: Content-Addressable Storage

Docker registries use content-addressable storage, where layers are identified by their content hash (SHA256), not by name. Explain:

- Why this matters for storage efficiency.
- How two different images (e.g., `myapp:1.0` and `myapp:2.0`) can share layers.
- What happens when you push two images that share a base layer.

### Question 3: Registry vs. Repository vs. Tag

Define each of these terms precisely. Give a concrete example that shows the relationship between all three.

```
Registry:   ???
Repository: ???
Tag:        ???
```

---

## Part B: Tag Strategies

### Question 4: The `latest` Tag

A developer runs:

```bash
docker pull myregistry.com/myapp:latest
```

Explain why this is risky for production. What does `latest` actually mean? Does it always point to the most recent build?

### Question 5: Compare Tag Strategies

Fill in the table comparing the three main tag strategies. For each strategy, describe: when to use it, its main advantage, and its main disadvantage.

| Strategy | When to Use | Advantage | Disadvantage |
|----------|-------------|-----------|--------------|
| Semantic Versioning (`1.2.3`) | ??? | ??? | ??? |
| Git SHA (`abc1234`) | ??? | ??? | ??? |
| Branch Name (`main`, `develop`) | ??? | ??? | ??? |

### Question 6: Identify the Bad Practices

A team uses the following tag conventions. For each one, explain why it is problematic and suggest a better approach.

```bash
docker tag myapp:1.0 company/myapp:production
docker tag myapp:1.0 company/myapp:latest
docker tag myapp:1.0 company/myapp:test-works-finally
docker tag myapp:1.0 company/myapp:DO-NOT-DELETE
```

---

## Success Criteria

- [ ] You can explain the push-pull model at the layer level.
- [ ] You understand content-addressable storage and layer sharing.
- [ ] You can distinguish between registry, repository, and tag.
- [ ] You can explain why `latest` is dangerous in production.
- [ ] You can compare semantic versioning, git SHA, and branch-based tagging.
- [ ] You can identify bad tagging practices and suggest fixes.

---

## Hints

<details>
<summary>Hint 1: Push-Pull at the Layer Level</summary>

When you push an image, Docker does not upload the entire image as a single blob. It uploads each layer individually. Before uploading a layer, the registry checks if it already has a layer with that SHA256 digest. If it does, the upload is skipped. This is called "content-addressable deduplication."

</details>

<details>
<summary>Hint 2: Registry vs. Repository vs. Tag</summary>

Think of it like a library system:
- A registry is the library building (e.g., `docker.io`, `ghcr.io`).
- A repository is a specific book title within the library (e.g., `library/nginx`).
- A tag is a specific edition of that book (e.g., `1.25`, `latest`, `alpine`).

The full image reference is: `<registry>/<repository>:<tag>`

</details>

<details>
<summary>Hint 3: The `latest` Tag</summary>

The `latest` tag is just a default name. It is not automatically updated when you push new images. If you push `myapp:2.0` without also tagging it as `latest`, the `latest` tag still points to whatever it previously pointed to. There is no built-in "most recent" concept.

</details>

<details>
<summary>Hint 4: Environment Names in Tags</summary>

Tags should describe what the image contains, not where it runs. The same image (`myapp:1.2.3`) should be promoted from staging to production. The environment is a deployment concern, not an image property.

</details>
