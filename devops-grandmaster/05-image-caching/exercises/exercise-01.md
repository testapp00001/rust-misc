# Exercise 01: Layer Caching and Invalidation Cascading

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Verify your understanding of how Docker layer caching works and how cache
invalidation cascades through layers. This exercise trains you to predict
which layers will be cached and which will be rebuilt -- the foundation of
every Dockerfile optimization.

## Scenario

You have the following Dockerfile:

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci --only=production

COPY tsconfig.json ./
COPY src/ ./src/

RUN npm run build

CMD ["node", "dist/index.js"]
```

The project has these files:

```
myapp/
├── Dockerfile
├── package.json
├── package-lock.json
├── tsconfig.json
└── src/
    ├── index.ts
    ├── routes.ts
    └── utils.ts
```

## Tasks

### Part A: First Build

You run `docker build -t myapp .` for the first time with a completely empty cache.
How many layers does Docker create? List each layer and whether it must be built
from scratch or can be pulled from cache.

<details>
<summary>Hint</summary>

Count each instruction that produces a layer. `FROM`, `WORKDIR`, `COPY`, `RUN`,
and `CMD` all create layers. On a cold cache, every layer must be built.

</details>

### Part B: Code Change, No Dependency Change

You change a single line in `src/routes.ts`. You run `docker build -t myapp .` again.
For each layer, state whether Docker uses the cache or rebuilds it. Explain your
reasoning for each layer.

<details>
<summary>Hint</summary>

Docker checks each instruction's inputs. For `COPY`, Docker checks if the source
files changed. For `RUN`, Docker checks if the previous layer changed (or if
the command string changed).

</details>

### Part C: Dependency Change

You add a new package to `package.json` and regenerate `package-lock.json`.
You change nothing else. You run `docker build -t myapp .` again.
For each layer, state whether Docker uses the cache or rebuilds it.

<details>
<summary>Hint</summary>

Which `COPY` instruction touches `package-lock.json`? Everything from that
instruction onward will be rebuilt.

</details>

### Part D: The Bad Dockerfile

A colleague proposes this "simpler" Dockerfile:

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY . .
RUN npm ci --only=production
RUN npm run build

CMD ["node", "dist/index.js"]
```

You change a single line in `src/routes.ts`. Which layers does Docker rebuild
with this Dockerfile? Compare to the original Dockerfile and explain why this
is worse.

<details>
<summary>Hint</summary>

`COPY . .` copies everything -- including source code. Any source code change
invalidates this layer, which cascades to every layer after it.

</details>

### Part E: The Cascade Diagram

Draw an ASCII diagram showing what happens during cache invalidation for both
Dockerfiles when `src/routes.ts` changes. Your diagram should show:

1. Which layers are cached (mark with `CACHED`)
2. Which layers are rebuilt (mark with `REBUILT`)
3. Where the cascade starts

<details>
<summary>Hint</summary>

The cascade starts at the first layer whose input changed. Every layer after
that point is rebuilt, even if its own input did not change.

</details>

## Success Criteria

- [ ] You can identify which layers Docker creates for each Dockerfile instruction
- [ ] You can predict cache hits and misses for a given file change
- [ ] You understand that cache invalidation cascades forward, not backward
- [ ] You can explain why `COPY . .` before `RUN npm ci` is a caching problem
- [ ] You can draw a diagram showing the cascade for both Dockerfiles

## What You Should Understand After This Exercise

Docker layer caching is not magic -- it is a deterministic system based on
input hashing. Once you understand the rules, you can predict exactly which
layers will be cached for any file change. This prediction ability is what
lets you write Dockerfiles that rebuild fast.
