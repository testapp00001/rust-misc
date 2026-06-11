# Solution 01: Layer Caching and Invalidation Cascading

## Part A: First Build (Cold Cache)

On a completely empty cache, every layer must be built from scratch. Docker
creates one layer per instruction:

```
Layer 1: FROM node:18-alpine          → BUILT (pulled from registry)
Layer 2: WORKDIR /app                  → BUILT
Layer 3: COPY package.json ...         → BUILT
Layer 4: RUN npm ci --only=production  → BUILT
Layer 5: COPY tsconfig.json ./         → BUILT
Layer 6: COPY src/ ./src/              → BUILT
Layer 7: RUN npm run build             → BUILT
Layer 8: CMD ["node", "dist/index.js"] → BUILT
```

Total: 8 layers, all built from scratch. This is the baseline.

## Part B: Code Change (src/routes.ts changes)

```
Layer 1: FROM node:18-alpine          → CACHED (base image unchanged)
Layer 2: WORKDIR /app                  → CACHED (instruction unchanged)
Layer 3: COPY package.json ...         → CACHED (package files unchanged)
Layer 4: RUN npm ci --only=production  → CACHED (Layer 3 is cached)
Layer 5: COPY tsconfig.json ./         → CACHED (tsconfig.json unchanged)
Layer 6: COPY src/ ./src/              → REBUILT (src/routes.ts changed!)
Layer 7: RUN npm run build             → REBUILT (Layer 6 changed)
Layer 8: CMD ["node", "dist/index.js"] → CACHED (instruction unchanged)
```

Only 2 layers rebuild. The expensive `npm ci` step is cached because
`package.json` and `package-lock.json` did not change.

## Part C: Dependency Change (package-lock.json changes)

```
Layer 1: FROM node:18-alpine          → CACHED (base image unchanged)
Layer 2: WORKDIR /app                  → CACHED (instruction unchanged)
Layer 3: COPY package.json ...         → REBUILT (package-lock.json changed!)
Layer 4: RUN npm ci --only=production  → REBUILT (Layer 3 changed)
Layer 5: COPY tsconfig.json ./         → REBUILT (Layer 4 changed)
Layer 6: COPY src/ ./src/              → REBUILT (Layer 5 changed)
Layer 7: RUN npm run build             → REBUILT (Layer 6 changed)
Layer 8: CMD ["node", "dist/index.js"] → CACHED (instruction unchanged)
```

5 layers rebuild. The cascade starts at Layer 3 (the first `COPY` that
touches the changed file) and propagates through every subsequent layer.
This is expected -- dependency changes are rare, so this cost is acceptable.

## Part D: The Bad Dockerfile

With the colleague's Dockerfile, changing `src/routes.ts` causes:

```
Layer 1: FROM node:18-alpine          → CACHED
Layer 2: WORKDIR /app                  → CACHED
Layer 3: COPY . .                      → REBUILT (src/routes.ts changed!)
Layer 4: RUN npm ci --only=production  → REBUILT (Layer 3 changed)
Layer 5: RUN npm run build             → REBUILT (Layer 4 changed)
Layer 6: CMD ["node", "dist/index.js"] → CACHED
```

4 layers rebuild, including the expensive `npm ci` step.

Comparison:

```
Original Dockerfile (code change):
  Rebuilt layers: 2 (COPY src, RUN build)
  npm ci: CACHED

Bad Dockerfile (code change):
  Rebuilt layers: 4 (COPY ., npm ci, RUN build, CMD)
  npm ci: REBUILT ← This is the expensive part!
```

The bad Dockerfile is worse because `COPY . .` copies everything -- including
source code. Any source code change invalidates `COPY . .`, which cascades
to `npm ci`. Even though `package.json` and `package-lock.json` did not
change, Docker cannot know that -- it only sees that the `COPY . .` layer
changed, so it rebuilds everything after it.

## Part E: The Cascade Diagram

Original Dockerfile -- `src/routes.ts` changes:

```
FROM node:18-alpine              CACHED
WORKDIR /app                     CACHED
COPY package.json ...            CACHED
RUN npm ci --only=production     CACHED  ← Dependencies preserved!
COPY tsconfig.json ./            CACHED
COPY src/ ./src/                 ── CASCADE STARTS HERE ──
RUN npm run build                REBUILT (cascade)
CMD ["node", "dist/index.js"]    CACHED
```

Bad Dockerfile -- `src/routes.ts` changes:

```
FROM node:18-alpine              CACHED
WORKDIR /app                     CACHED
COPY . .                         ── CASCADE STARTS HERE ──
RUN npm ci --only=production     REBUILT (cascade) ← Expensive!
RUN npm run build                REBUILT (cascade)
CMD ["node", "dist/index.js"]    CACHED
```

The key difference: in the original Dockerfile, the cascade starts after
the dependency installation. In the bad Dockerfile, the cascade starts
before it, forcing a full dependency reinstall.

## Common Mistakes

1. **Thinking `RUN` commands are cached based on their output.** They are
   not -- Docker caches based on the command string and the previous layer's
   hash. Even if the output would be identical, a changed previous layer
   forces a rebuild.

2. **Thinking cache invalidation goes backward.** It does not. Changing Layer 5
   only invalidates Layers 6, 7, 8 -- not Layers 1-4.

3. **Forgetting that `CMD` and `ENTRYPOINT` create layers.** They do, but
   they are almost always cached because the instruction string rarely changes.
