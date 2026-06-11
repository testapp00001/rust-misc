# Exercise 01: Anatomy of a Dockerfile

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Analyze a Dockerfile line by line and explain what each instruction does,
why it is placed in that order, and what would happen if the order changed.
This exercise builds the mental model you need before writing your own Dockerfiles.

## The Dockerfile

You are handed this Dockerfile for a Python Flask application:

```dockerfile
FROM python:3.11-slim-bookworm

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

RUN adduser --disabled-password --gecos '' appuser
USER appuser

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:5000/health || exit 1

CMD ["python", "app.py"]
```

## Tasks

### Part A: Explain Each Instruction

For each line in the Dockerfile above, write a one-sentence explanation of what
it does and *why* it is there. Use your own words -- do not just copy the docs.

For example, for `FROM python:3.11-slim-bookworm` you might write:
"This sets the base image to Python 3.11 on a minimal Debian Bookworm variant,
giving us Python without unnecessary system packages."

Do this for every instruction: `FROM`, `WORKDIR`, `COPY requirements.txt .`,
`RUN pip install`, `COPY . .`, `RUN adduser`, `USER`, `EXPOSE`, `HEALTHCHECK`,
and `CMD`.

<details>
<summary>Hint</summary>

Focus on two things for each line:
1. What does this instruction do to the image?
2. What problem does it solve or what best practice does it follow?

For `EXPOSE 5000`, remember that it is documentation only -- it does not
actually publish the port.

</details>

### Part B: Why This Order?

The order of instructions in a Dockerfile is not arbitrary. Answer these questions:

1. Why does `COPY requirements.txt .` come *before* `COPY . .`?
2. Why does `RUN adduser` come *after* `COPY . .` and `RUN pip install`?
3. Why does `CMD` come at the very end?
4. Why does `USER appuser` come before `EXPOSE` and `CMD`?

<details>
<summary>Hint</summary>

Think about Docker's layer cache. When a layer changes, every layer after it
must be rebuilt. Which instructions change frequently? Which are stable?

Also think about what the `USER` instruction affects -- which processes run
as that user.

</details>

### Part C: What If We Change the Order?

For each scenario below, explain what would break or go wrong:

1. **Scenario 1:** Swap lines so `COPY . .` comes before `COPY requirements.txt .`

   ```dockerfile
   COPY . .
   COPY requirements.txt .
   RUN pip install --no-cache-dir -r requirements.txt
   ```

2. **Scenario 2:** Move `USER appuser` before `RUN pip install`

   ```dockerfile
   COPY requirements.txt .
   RUN adduser --disabled-password --gecos '' appuser
   USER appuser
   RUN pip install --no-cache-dir -r requirements.txt
   ```

3. **Scenario 3:** Move `EXPOSE 5000` to be the very first instruction after `FROM`

   ```dockerfile
   FROM python:3.11-slim-bookworm
   EXPOSE 5000
   WORKDIR /app
   # ... rest of Dockerfile
   ```

<details>
<summary>Hint</summary>

Scenario 1: Think about the build cache. What happens every time you change
any file in your project?

Scenario 2: Think about permissions. Can a non-root user install Python
packages into system directories?

Scenario 3: Think about what EXPOSE actually does (and does not do).

</details>

### Part D: Identifying Instructions

Without looking at the docs, write down what each of these instructions does
and whether they are required or optional in a Dockerfile:

| Instruction | What it does | Required? |
|-------------|-------------|-----------|
| FROM        |             |           |
| WORKDIR     |             |           |
| COPY        |             |           |
| RUN         |             |           |
| CMD         |             |           |
| ENTRYPOINT  |             |           |
| ENV         |             |           |
| EXPOSE      |             |           |
| ARG         |             |           |
| USER        |             |           |
| HEALTHCHECK |             |           |

<details>
<summary>Hint</summary>

Only `FROM` is strictly required. Every Dockerfile must start with a `FROM`
instruction (except when using `ARG` before the first `FROM`).

`CMD` and `ENTRYPOINT` define what runs when the container starts.
If neither is present, the container inherits the default from the base image.

</details>

## Success Criteria

- [ ] You can explain what every instruction in the Dockerfile does in your own words
- [ ] You can explain why the instructions are ordered the way they are
- [ ] You can predict what breaks when the order is changed (all 3 scenarios)
- [ ] You can distinguish required instructions from optional ones
- [ ] You understand the relationship between Docker layer caching and instruction order

## What You Should Understand After This Exercise

A Dockerfile is not just a list of commands -- it is an ordered sequence where
each instruction creates a layer. The order determines build speed (via caching),
image correctness (via permissions and dependencies), and runtime behavior
(via CMD, ENTRYPOINT, and USER). Understanding *why* each line is where it is
separates someone who can copy-paste a Dockerfile from someone who can write one.
