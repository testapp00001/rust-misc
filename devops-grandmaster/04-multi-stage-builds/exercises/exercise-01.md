# Exercise 01: Anatomy of a Bloated Image

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Understand *why* single-stage Dockerfiles produce bloated images and *why* cleaning
up inside a single stage does not reduce image size. This exercise trains you to think
in terms of Docker layers -- the fundamental concept that makes multi-stage builds
necessary.

## Scenario

A developer on your team wrote this Dockerfile for a Python web application:

```dockerfile
FROM python:3.12

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

# "Clean up to reduce image size"
RUN apt-get remove -y gcc make && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /root/.cache

CMD ["python", "app.py"]
```

The `requirements.txt` contains packages that require C compilation (like `numpy`,
`cryptography`, or `psycopg2`). The developer expected the final image to be small
because they removed the compilers after installing the packages.

The image is still 924MB. The developer is confused.

## Tasks

### Part A: List What Is Inside

Without running any commands, estimate what is inside the 924MB image. Break it
down into categories: base image, build tools, runtime dependencies, application
code, and anything else. Give rough size estimates for each category.

<details>
<summary>Hint</summary>

Look at what `FROM python:3.12` brings in. Then think about what `pip install`
adds. The application code is tiny -- the bulk comes from the base image and
the build toolchain.

</details>

### Part B: Explain Why Cleanup Fails

The developer ran `apt-get remove -y gcc make` and deleted caches. Why is the
image still 924MB? Explain using the concept of Docker layers.

<details>
<summary>Hint</summary>

Each `RUN` instruction creates a new layer. Layers are additive. Think about
what happens to the bytes that were written in an earlier layer when a later
layer tries to delete them.

</details>

### Part C: Draw the Layer Stack

Draw a diagram showing each layer in this Dockerfile, its approximate size,
and what it contains. Show why the "deletion layer" does not reduce the total.

<details>
<summary>Hint</summary>

Your diagram should have 5 layers (base, pip install, COPY, cleanup RUN, CMD).
The key insight is that layers are stacked, not merged.

</details>

### Part D: The Chained-RUN Approach

A senior developer suggests this alternative:

```dockerfile
FROM python:3.12
RUN apt-get update && \
    apt-get install -y gcc make libffi-dev && \
    pip install --no-cache-dir numpy cryptography && \
    apt-get purge -y --auto-remove gcc make libffi-dev && \
    rm -rf /var/lib/apt/lists/*
COPY . /app
WORKDIR /app
CMD ["python", "app.py"]
```

Does this reduce image size compared to the original Dockerfile? Why or why not?
What are the trade-offs of this approach compared to multi-stage builds?

<details>
<summary>Hint</summary>

When install and cleanup happen in the SAME `RUN` instruction, they share a
single layer. Think about what that means for the layer's final size. Then
think about Docker cache behavior.

</details>

### Part E: The Multi-Stage Alternative

Without looking at the module README, sketch what a multi-stage Dockerfile for
this application would look like. You do not need the exact syntax -- describe
the structure: how many stages, what each stage does, and what gets copied
between them.

<details>
<summary>Hint</summary>

Think about two separate containers: one that has all the build tools and
compiles everything, and one that has only the runtime. The second container
copies the finished artifacts from the first.

</details>

## Success Criteria

- [ ] You can explain what a Docker layer is and why layers are additive
- [ ] You can explain why `apt-get remove` in a new layer does not reduce image size
- [ ] You can draw the layer stack and show where the waste accumulates
- [ ] You can explain when chaining RUN commands helps and when it does not
- [ ] You can describe the multi-stage build concept at a high level

## What You Should Understand After This Exercise

Docker images are stacks of layers. Each layer records only additions and
modifications -- it cannot remove data from a previous layer. This is why
cleaning up build tools after installing them does not shrink the image.
Multi-stage builds solve this by using separate images for building and
running, so the build tools never exist in the final image at all.
