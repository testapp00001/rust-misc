# Exercise 02: Create a Base Configuration with Environment Overlays

**Type:** Guided
**Time:** 35 minutes
**Objective:** Build a complete Kustomize base with deployment, service, and
configmap resources, then create dev, staging, and production overlays that
customize each environment.

---

## Background

Your team owns a web application called `webapp`. It needs to run in three
environments with different configurations:

| Setting | Dev | Staging | Production |
|---------|-----|---------|------------|
| Replicas | 1 | 2 | 5 |
| CPU request | 100m | 250m | 500m |
| Memory request | 128Mi | 256Mi | 512Mi |
| CPU limit | 200m | 500m | 1000m |
| Memory limit | 256Mi | 512Mi | 1Gi |
| Image tag | latest | v1.2.0-rc1 | v1.2.0 |
| Namespace | dev | staging | production |
| Log level | debug | info | warn |

---

## Tasks

### Task 1: Create the Base Directory

Create the following directory structure and files:

```
exercise-02/
  base/
    kustomization.yaml
    deployment.yaml
    service.yaml
    configmap.yaml
```

**deployment.yaml** should define a Deployment with:
- `app: webapp` label
- 1 container named `webapp` using image `myregistry.io/webapp:latest`
- Resource requests and limits (any reasonable defaults)
- A `WEBAPP_LOG_LEVEL` environment variable sourced from the `webapp-config`
  ConfigMap

**service.yaml** should define a ClusterIP Service targeting port 80 on the
container port 8080.

**configmap.yaml** should define a ConfigMap named `webapp-config` with a
single key `LOG_LEVEL` set to `info`.

**kustomization.yaml** should list all three resources and apply common labels:
- `app.kubernetes.io/name: webapp`
- `app.kubernetes.io/managed-by: kustomize`

### Task 2: Validate the Base

Run Kustomize to verify the base renders correctly:

```bash
kustomize build exercise-02/base
# or
kubectl kustomize exercise-02/base
```

Verify the output contains one Deployment, one Service, and one ConfigMap with
the common labels applied to every resource.

### Task 3: Create the Dev Overlay

Create `exercise-02/overlays/dev/kustomization.yaml` that:

1. References the base via `resources: [../../base]`
2. Sets the namespace to `dev`
3. Overrides the image tag to `latest`
4. Sets replicas to 1
5. Overrides resource requests/limits for dev
6. Overrides the `LOG_LEVEL` to `debug` by replacing the ConfigMap

### Task 4: Create the Staging Overlay

Create `exercise-02/overlays/staging/kustomization.yaml` that:

1. References the base
2. Sets the namespace to `staging`
3. Sets the image tag to `v1.2.0-rc1`
4. Sets replicas to 2
5. Sets resource requests/limits for staging
6. Sets `LOG_LEVEL` to `info`

### Task 5: Create the Production Overlay

Create `exercise-02/overlays/production/kustomization.yaml` that:

1. References the base
2. Sets the namespace to `production`
3. Sets the image tag to `v1.2.0`
4. Sets replicas to 5
5. Sets resource requests/limits for production
6. Sets `LOG_LEVEL` to `warn`
7. Adds the label `app.kubernetes.io/env: production`

### Task 6: Validate All Overlays

Run the build for each overlay and confirm the output differs as expected:

```bash
kustomize build exercise-02/overlays/dev
kustomize build exercise-02/overlays/staging
kustomize build exercise-02/overlays/production
```

For each, verify:
- The namespace is correct
- The replica count is correct
- The image tag is correct
- The ConfigMap LOG_LEVEL value is correct

---

## Success Criteria

- [ ] `kustomize build exercise-02/base` produces valid YAML with all resources.
- [ ] Common labels appear on all resources in the base output.
- [ ] Each overlay builds without errors.
- [ ] Dev overlay has namespace `dev`, 1 replica, image tag `latest`, log level `debug`.
- [ ] Staging overlay has namespace `staging`, 2 replicas, image tag `v1.2.0-rc1`, log level `info`.
- [ ] Production overlay has namespace `production`, 5 replicas, image tag `v1.2.0`, log level `warn`.
- [ ] No base files were modified when creating overlays.

---

## Hints

<details>
<summary>Hint 1: kustomization.yaml Structure</summary>

The base `kustomization.yaml` should look like this:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

commonLabels:
  app.kubernetes.io/name: webapp
  app.kubernetes.io/managed-by: kustomize

resources:
  - deployment.yaml
  - service.yaml
  - configmap.yaml
```

</details>

<details>
<summary>Hint 2: Changing the Image Tag</summary>

Use the `images` field in the overlay's `kustomization.yaml`:

```yaml
images:
  - name: myregistry.io/webapp
    newTag: v1.2.0
```

This is a transformer that patches the image tag in any container that
references `myregistry.io/webapp`.

</details>

<details>
<summary>Hint 3: Setting Replicas</summary>

Use the `replicas` field in the overlay:

```yaml
replicas:
  - name: webapp
    count: 5
```

The `name` here refers to the Deployment name, not the container name.

</details>

<details>
<summary>Hint 4: Overriding the ConfigMap</summary>

You have two options:

**Option A -- Replace the entire ConfigMap** by including a new ConfigMap YAML
in the overlay and referencing it in `resources`.

**Option B -- Use configMapGenerator** in the overlay with the same name. If
the overlay generates a ConfigMap with the same name as one in the base, the
overlay's version replaces the base's version.

```yaml
configMapGenerator:
  - name: webapp-config
    literals:
      - LOG_LEVEL=debug
```

Note: `configMapGenerator` appends a hash suffix to the ConfigMap name by
default. You need `generatorOptions` to disable that:

```yaml
generatorOptions:
  disableNameSuffixHash: true
```

</details>

<details>
<summary>Hint 5: Setting the Namespace</summary>

Simply add this to each overlay's `kustomization.yaml`:

```yaml
namespace: dev
```

Kustomize will set the namespace on every resource in the output.

</details>
