# Solution 01: The 12-Factor Config Philosophy

## Part A: Map the 12 Factors to Configuration

The four factors most relevant to environment variable configuration:

| Factor | Name | What It Requires | Why It Matters for Containers |
|--------|------|-----------------|-------------------------------|
| **III. Config** | Store config in the environment | All configuration that varies between deployments must be stored in environment variables, not in code. | Containers are immutable artifacts. Baking config into the image means you need a different image per environment. |
| **V. Build, Release, Run** | Strictly separate build and run | The build stage produces a release. The release is combined with config at run time. | The Docker image is the "release." Environment variables are applied at "run." Mixing them violates this separation. |
| **VI. Processes** | Execute the app as one or more stateless processes | Processes must not store local state. Config must not live on the filesystem. | Environment variables are injected by the orchestrator at process start. No filesystem dependency means the container is truly stateless. |
| **X. Dev/Prod Parity** | Keep development, staging, and production as similar as possible | Minimize the time gap, personnel gap, and tool gap between environments. | Using the same image with different env vars means the only difference is *values*, not *artifacts*. This maximizes parity. |

### Why this matters

These four factors form a chain: Factor III says config lives in the environment. Factor V says the build artifact (image) is separate from runtime config. Factor VI says processes get config from the environment, not from files. Factor X says the same artifact should run everywhere with different environment values. Break any link and the chain falls apart -- you end up with environment-specific images, config files checked into git, or hardcoded values in code.

---

## Part B: Config Files vs. Environment Variables

**Response to the junior developer:**

Storing configuration in a mounted `config.json` file has several problems that environment variables avoid. First, you need a different config file per environment, which means either maintaining multiple files or templating them at deploy time -- this reintroduces the exact complexity we are trying to eliminate. Second, a JSON syntax error (a missing comma, a trailing comma, a misplaced quote) causes a runtime crash that is hard to debug, whereas environment variables are simple key-value pairs with no parsing rules. Third, file-based config requires the application to include a JSON parser and handle file-not-found errors, adding code that environment variables do not need. Finally, most orchestrators (Docker, Kubernetes, CI/CD platforms) have native support for injecting environment variables, but mounting files requires additional volume or secret configuration that is more complex and less portable.

### Common mistakes

- Thinking config files are "easier" because they feel familiar. The complexity shifts from the config mechanism to the file management and parsing layer.
- Forgetting that JSON does not support comments. You cannot document your config in the file itself.
- Not realizing that a mounted file creates a dependency on the filesystem, which violates Factor VI (stateless processes).

---

## Part C: Classify Configuration Values

| Variable | Classification | Reasoning |
|----------|---------------|-----------|
| `DATABASE_URL` (with embedded password) | **Secret** | Contains credentials. Should be in a secrets manager, not an env var. |
| `LOG_LEVEL` | **Config** | Controls behavior, not security. Safe to expose. |
| `API_KEY` | **Secret** | A credential for a third-party service. Exposure means financial or data risk. |
| `PORT` | **Config** | A numeric setting with no security implications. |
| `JWT_SECRET` | **Secret** | Used to sign tokens. Exposure means anyone can forge authentication tokens. |
| `NODE_ENV` | **Config** | A string flag that controls runtime behavior. |
| `SMTP_PASSWORD` | **Secret** | A credential for an email service. |
| `MAX_UPLOAD_SIZE` | **Config** | A numeric limit with no security implications. |

### Key distinction

Config values control *how* the application behaves. Secrets control *who* can access what. If a value's exposure would allow unauthorized access, data breaches, or financial loss, it is a secret. Environment variables are appropriate for config but not ideal for secrets because `docker inspect` and `/proc/<pid>/environ` expose them.

---

## Part D: The Configuration Hierarchy

```
Highest priority
     |
     |  1. docker run -e         -- Runtime overrides (operator debugging in production)
     |  2. environment:          -- In docker-compose.yml (platform-specific overrides)
     |  3. .env.local            -- Gitignored env file (developer's local secrets)
     |  4. .env                  -- Default env file (team-wide defaults, committed)
     |  5. ENV in Dockerfile     -- Build-time defaults (rarely used for config)
     |  6. Code defaults         -- process.env.X || "default" (last resort)
     |
Lowest priority
```

| Level | Who Sets It | Example Use Case |
|-------|-------------|-----------------|
| 1. `docker run -e` | Operator / CI pipeline | Temporarily enabling debug logging to troubleshoot a production issue. |
| 2. `environment:` in compose | Platform team | Setting `NODE_ENV=production` for all services in the staging stack. |
| 3. `.env.local` | Individual developer | Setting a personal database password that should not be committed. |
| 4. `.env` | Team (committed to git) | Setting `DB_PORT=5432` as the default for all developers. |
| 5. `ENV` in Dockerfile | Image author | Setting `PATH` or other OS-level variables. Rarely used for app config. |
| 6. Code defaults | Application developer | `const port = process.env.PORT \|\| 3000` as a fallback. |

### Common mistakes

- Setting a value in `.env` and wondering why it does not take effect -- a higher-priority source is overriding it.
- Using `ENV` in the Dockerfile for application config, then being unable to override it at runtime (you can, but it is confusing and defeats the purpose of a clean image).
- Relying on code defaults for critical values like `DATABASE_URL` -- if the variable is not set, the app should fail, not silently connect to localhost.
