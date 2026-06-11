# Exercise 02: ConfigMaps as Files and Env Vars -- Hands-On

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Beginner

## Objective

Create a ConfigMap using multiple methods, consume it as both environment
variables and a mounted volume, and verify the values are correct inside a
running container.

---

## Background

You are deploying a Node.js application that reads configuration in two ways:

1. **Environment variables** for simple key-value settings: `APP_ENV`,
   `LOG_LEVEL`, `DB_HOST`, `DB_PORT`.
2. **A config file** at `/etc/app/application.properties` for structured
   settings that the application reads at startup.

You need to create a single ConfigMap that serves both consumption methods.

---

## Tasks

### Task 1: Create a ConfigMap

Create a ConfigMap named `app-config` in the `default` namespace that contains:

**Key-value pairs (for env vars):**

| Key | Value |
|-----|-------|
| APP_ENV | production |
| LOG_LEVEL | info |
| DB_HOST | postgres.default.svc |
| DB_PORT | 5432 |

**File entry (for volume mount):**

The key should be `application.properties` and the value should be:

```properties
server.port=8080
app.feature.darkMode=true
app.feature.newUI=false
app.log.format=json
```

Write the complete YAML manifest. Then apply it with `kubectl apply -f`.

<details>
<summary>Hint 1: ConfigMap data field</summary>

A ConfigMap's `data` field holds both simple key-value pairs and file-like
entries. A file entry is just a key that looks like a filename, with a
multi-line string value using the `|` YAML block scalar.

</details>

### Task 2: Consume as Environment Variables

Create a Pod named `configmap-env-pod` using the image `busybox:1.36` that:

1. Loads all keys from `app-config` as environment variables via `envFrom`.
2. Runs a command that prints each variable and then sleeps.
3. The command should print: `APP_ENV=<value> LOG_LEVEL=<value> DB_HOST=<value> DB_PORT=<value>`.

Apply the manifest, then verify by running:

```bash
kubectl exec configmap-env-pod -- env | grep -E "APP_ENV|LOG_LEVEL|DB_HOST|DB_PORT"
```

<details>
<summary>Hint 2: envFrom vs env</summary>

`envFrom` loads all keys from a ConfigMap as environment variables.
`env` with `valueFrom.configMapKeyRef` loads a single key. For this task,
use `envFrom` to load all four keys at once.

</details>

### Task 3: Consume as a Mounted Volume

Create a Pod named `configmap-volume-pod` using the image `busybox:1.36` that:

1. Mounts the `app-config` ConfigMap at `/etc/app` as a read-only volume.
2. Runs a command that prints the contents of `application.properties` and
   then sleeps.

Apply the manifest, then verify by running:

```bash
kubectl exec configmap-volume-pod -- cat /etc/app/application.properties
```

<details>
<summary>Hint 3: Volume mount structure</summary>

You need two sections in the Pod spec:
- `containers[].volumeMounts[]` with `name`, `mountPath`, and `readOnly: true`
- `volumes[]` with `name` and `configMap.name`

The names in both sections must match.

</details>

### Task 4: Verify Both Methods

After both Pods are running, answer:

1. Are the environment variables present in `configmap-volume-pod`? Why or
   why not?
2. Is the `application.properties` file present in `configmap-env-pod`? Why
   or why not?

---

## Success Criteria

- [ ] The ConfigMap `app-config` exists and contains all 5 entries.
- [ ] `configmap-env-pod` prints all 4 environment variables.
- [ ] `configmap-volume-pod` displays the contents of `application.properties`.
- [ ] You can explain why each Pod only has the configuration it was
      explicitly configured to consume.

---

## Hints

<details>
<summary>Hint 4: Pods are independent</summary>

Each Pod declares exactly which ConfigMaps it consumes and how. A Pod that
uses `envFrom` does not automatically get volume mounts, and vice versa.
The ConfigMap is a shared resource, but consumption is per-Pod.

</details>
