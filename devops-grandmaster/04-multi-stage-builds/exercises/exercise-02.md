# Exercise 02: Convert Single-Stage to Multi-Stage

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Convert a single-stage Dockerfile into a multi-stage Dockerfile step by step.
By the end, you will understand the mechanics of `FROM ... AS`, `COPY --from=`,
and how to choose appropriate base images for each stage.

## Starting Point

You have this single-stage Dockerfile for a Node.js application that uses
TypeScript and needs to be compiled before running:

```dockerfile
FROM node:20

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

EXPOSE 3000
CMD ["node", "dist/index.js"]
```

The image built from this Dockerfile is 1.1GB. Your goal is to shrink it.

## Tasks

### Part A: Identify What Belongs Where

Split the Dockerfile instructions into two categories:

1. **Build-time instructions** -- things needed only to compile/transpile the code
2. **Runtime instructions** -- things needed to run the application

List each instruction and explain which category it belongs to.

<details>
<summary>Hint</summary>

Ask yourself: "Does this instruction produce artifacts that the running
application needs, or does it produce tools that are only used during the
build?" The `npm ci` command installs ALL dependencies, including
devDependencies like TypeScript. The running app only needs the compiled
output in `dist/` and production `node_modules`.

</details>

### Part B: Write the Builder Stage

Write the first stage of the multi-stage Dockerfile. This stage should:
- Use `node:20` as the base image (you need the full toolchain to build)
- Be named `builder`
- Install all dependencies
- Build the TypeScript application

<details>
<summary>Hint</summary>

```dockerfile
FROM node:20 AS builder

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build
```

The key difference from a single-stage build is the `AS builder` name.
This lets you reference this stage later.

</details>

### Part C: Write the Runtime Stage

Write the second stage. This stage should:
- Use a smaller base image (not the full `node:20`)
- Copy only the files needed to run the application from the builder stage
- Run as a non-root user

What files do you need to copy from the builder? List them explicitly.

<details>
<summary>Hint</summary>

You need:
1. `dist/` -- the compiled JavaScript output
2. `node_modules/` -- but only production dependencies
3. `package.json` -- for metadata and the start script

For the base image, `node:20-slim` is much smaller than `node:20`.
You should also run `npm prune --production` in the builder to remove
devDependencies before copying.

</details>

### Part D: Add the Prune Step

Before the runtime stage copies `node_modules`, you need to remove
devDependencies from the builder. Add `npm prune --production` to the
builder stage. Explain why this step matters for image size.

<details>
<summary>Hint</summary>

TypeScript, type definitions, test frameworks, and build tools are all
listed under `devDependencies`. A typical TypeScript project has
hundreds of megabytes of devDependencies. The running application
does not need any of them.

</details>

### Part E: Choose the Right Runtime Base

Compare these three runtime base images for this Node.js application:

| Base Image | Approximate Size |
|---|---|
| `node:20` | 1.1GB |
| `node:20-slim` | 200MB |
| `node:20-alpine` | 130MB |

Which would you choose for production and why? What are the trade-offs?

<details>
<summary>Hint</summary>

`slim` is a safe default -- it is Debian-based and has the same libc as
the full image. `alpine` is smaller but uses musl libc, which can cause
issues with some native Node.js modules. For most TypeScript/Express
applications, alpine works fine.

</details>

### Part F: Final Dockerfile

Combine all the parts into a complete multi-stage Dockerfile. Build it and
compare the image size to the original 1.1GB.

```bash
# Build the original (single-stage)
docker build -t exercise2-single --target=builder .
docker images exercise2-single

# Build the multi-stage version
docker build -t exercise2-multi .
docker images exercise2-multi
```

Record both sizes. Calculate the percentage reduction.

## Success Criteria

- [ ] You can split a Dockerfile into build-time and runtime instructions
- [ ] Your builder stage installs dependencies and compiles the application
- [ ] Your runtime stage uses a slim or alpine base image
- [ ] You copy only the necessary files from builder to runtime
- [ ] You run `npm prune --production` to remove devDependencies
- [ ] Your final image is at least 70% smaller than the original
- [ ] The application still runs correctly in the smaller image

## What You Should Understand After This Exercise

Multi-stage builds are not magic -- they are a mechanical process of
splitting build tools from runtime artifacts. The builder stage uses a
large image with compilers and tools. The runtime stage uses a small
image and copies only the finished output. The `--from=` flag is the
bridge between the two stages.
