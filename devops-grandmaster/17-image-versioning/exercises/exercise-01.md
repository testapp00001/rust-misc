# Exercise 01: Why 'latest' Is Dangerous in Production

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Understand why the `latest` Docker tag is unreliable for production, identify the specific risks it introduces, and articulate what properties a production-grade tagging strategy must have.

## Scenario

Your team deploys a web application using `docker compose`. The `docker-compose.yml` references `myapp:latest` for every environment. A junior developer pushes a breaking change on Friday afternoon. By Monday morning, three separate production servers have pulled three different images -- some pulled the Friday build, one still has Thursday's image cached, and one pulled a Saturday hotfix. Nobody can answer the question: "What version is running in production right now?"

You have been asked to write an internal memo explaining why the `latest` tag must be banned in production and what should replace it.

## Tasks

### Part A: The 'latest' Misconception

Many developers believe `latest` means "the most recent stable release." Explain why this is wrong by answering the following questions:

1. What does `latest` actually mean in Docker's tag system?
2. If I run `docker pull myapp:latest` on Monday and again on Wednesday, am I guaranteed to get the same image? Why or why not?
3. A colleague says: "We always use `latest` so we automatically get bug fixes." Explain why this reasoning is dangerous.
4. Does `latest` have any special meaning to Docker's build or pull logic, or is it just a convention?

<details>
<summary>Hint</summary>

`latest` is simply a tag string -- it is the default tag Docker assigns when no tag is specified. It has no built-in semantic meaning. Every `docker push myapp` overwrites what `latest` points to.

</details>

### Part B: Failure Mode Analysis

For each scenario below, explain what goes wrong when using `latest` and how a proper tagging strategy would prevent the problem.

| # | Scenario | What goes wrong with 'latest'? | How does a version tag fix it? |
|---|----------|-------------------------------|-------------------------------|
| 1 | You deploy `myapp:latest` to production. Two hours later, a colleague pushes a new build that also gets tagged `latest`. | ? | ? |
| 2 | Your CI pipeline caches `myapp:latest` locally. A new version is pushed to the registry, but your pipeline uses the cached copy. | ? | ? |
| 3 | You need to roll back a production incident. The previous version was also tagged `latest` before being overwritten. | ? | ? |
| 4 | Three replicas of your service each pull `myapp:latest` at slightly different times during a rolling update. | ? | ? |
| 5 | A security auditor asks: "What exact image was running in production at 3:00 PM last Tuesday?" | ? | ? |

<details>
<summary>Hint</summary>

Think about mutability. A tag that changes its meaning over time breaks reproducibility, rollback, auditability, and consistency across replicas.

</details>

### Part C: Properties of a Good Tagging Strategy

You need to define the requirements for the new tagging strategy. For each property below, write a one-sentence definition and explain why it matters.

1. **Immutability** -- What does it mean for a tag to be immutable, and what breaks if tags are mutable?
2. **Traceability** -- What information should a tag let you recover, and why?
3. **Reproducibility** -- If you pull the same tag six months from now, what should you get?
4. **Consistency** -- If three replicas pull the same tag, what must be true?
5. **Human readability** -- Why should a tag convey something meaningful to a person looking at a deployment dashboard?

<details>
<summary>Hint</summary>

Each property addresses a specific failure mode from Part B. Immutability prevents overwrite problems. Traceability lets you answer "what code is this?" Reproducibility ensures `pull` gives the same result every time. Consistency ensures all replicas run the same bits. Readability helps during incident response.

</details>

### Part D: Tag Comparison

Evaluate each candidate tagging approach. For each one, rate it (Good / Partial / Bad) against the five properties from Part C and explain the tradeoff.

| Tag Strategy | Example Tag | Immutability | Traceability | Reproducibility | Consistency | Readability |
|-------------|-------------|-------------|-------------|----------------|-------------|-------------|
| `latest` only | `myapp:latest` | ? | ? | ? | ? | ? |
| Build number only | `myapp:47` | ? | ? | ? | ? | ? |
| Semantic version | `myapp:v1.2.3` | ? | ? | ? | ? | ? |
| Git SHA | `myapp:a1b2c3d` | ? | ? | ? | ? | ? |
| SemVer + Git SHA | `myapp:v1.2.3-a1b2c3d` | ? | ? | ? | ? | ? |
| Date-based | `myapp:20240115` | ? | ? | ? | ? | ? |

<details>
<summary>Hint</summary>

No single strategy scores "Good" on all five properties. That is why production systems often combine multiple tags -- for example, pushing both `myapp:v1.2.3` (readable) and `myapp:a1b2c3d` (traceable) for the same image.

</details>

### Part E: Draft the Policy

Write a short (5-10 bullet point) tagging policy for your team. The policy must:

1. Define which tags are allowed in production.
2. Define which tags are allowed in development/staging.
3. State what happens to `latest` (banned, restricted, or allowed only for a specific use).
4. Require that every production image be traceable to a git commit.
5. Address base image pinning (e.g., `FROM node:18-alpine` vs `FROM node:18.19.0-alpine3.19@sha256:...`).

<details>
<summary>Hint</summary>

A good policy is short enough to fit on a wiki page and specific enough to be enforced by automation. Consider: "Production images MUST be tagged with a semantic version AND a git SHA. The `latest` tag MUST NOT appear in any deployment manifest."

</details>

## Success Criteria

- [ ] You can explain in your own words why `latest` does not mean "most recent stable."
- [ ] You identified at least three concrete failure modes caused by mutable tags.
- [ ] You defined five properties that a production tagging strategy must satisfy.
- [ ] You evaluated at least four tagging approaches against those properties.
- [ ] You wrote a tagging policy that is specific enough to be automated and enforced.
- [ ] You can articulate why teams often combine multiple tags (e.g., SemVer + Git SHA) rather than picking one.

## What You Should Understand After This Exercise

The `latest` tag is a mutable pointer, not a version. It provides zero guarantees about what image you will get when you pull it. Production requires immutable, traceable, reproducible tags. Semantic versioning communicates the nature of changes. Git SHA tags trace images to source code. Combining both gives you readability and traceability. Base image pinning with digests prevents silent drift. A tagging policy turns these principles into enforceable rules.
