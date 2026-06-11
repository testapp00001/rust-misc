# Exercise 03: Use Patches to Customize Resources per Environment

**Type:** Independent
**Time:** 40 minutes
**Objective:** Use strategic merge patches and JSON patches to make fine-grained
modifications to Kubernetes resources for different environments.

---

## Background

The base configuration from Exercise 02 is working, but the product team now
has environment-specific requirements that go beyond simple value changes:

- **Dev:** Add a `hostPath` volume mount for local file storage. Add a
  `nodeSelector` to pin the pod to a dev node. Add an annotation for the
  on-call team.
- **Staging:** Add a `readinessProbe` and `livenessProbe`. Add a
  `PodDisruptionBudget` with `minAvailable: 1`. Add a TLS annotation to the
  Service for the ingress controller.
- **Production:** Add resource limits, an `initContainer` that runs database
  migrations, a `topologySpreadConstraints` rule, and a `HorizontalPodAutoscaler`.

These changes require patches, not just value overrides.

---

## Tasks

### Task 1: Set Up the Working Directory

Create the directory structure:

```
exercise-03/
  base/
    kustomization.yaml
    deployment.yaml
    service.yaml
```

Use the same base deployment and service from Exercise 02, or create minimal
versions. The deployment should be named `webapp` with image
`myregistry.io/webapp:v1.0.0`.

### Task 2: Strategic Merge Patch -- Dev Overlay

Create `exercise-03/overlays/dev/kustomization.yaml` and a patch file
`exercise-03/overlays/dev/patch-deployment.yaml`.

The patch should:
1. Add a `nodeSelector` with `environment: dev`
2. Add an annotation `team: platform-eng` to the pod template metadata
3. Add a `hostPath` volume and volume mount at `/data` using the
   `patches` field with `target.kind: Deployment`

```yaml
# In kustomization.yaml
patches:
  - path: patch-deployment.yaml
    target:
      group: apps
      version: v1
      kind: Deployment
      name: webapp
```

### Task 3: Strategic Merge Patch -- Staging Overlay

Create `exercise-03/overlays/staging/` with patches that:

1. Add a `readinessProbe` (HTTP GET on `/healthz` port 8080, initial delay 5s)
2. Add a `livenessProbe` (HTTP GET on `/healthz` port 8080, initial delay 15s)
3. Add a TLS annotation to the Service:
   `service.beta.kubernetes.io/aws-load-balancer-backend-protocol: https`
4. Create a `PodDisruptionBudget` resource targeting the `webapp` deployment
   with `minAvailable: 1`

For the probes, use a strategic merge patch on the Deployment's first container.
For the PDB, create a new YAML file and add it to `resources`.

### Task 4: JSON Patch -- Production Overlay

Create `exercise-03/overlays/production/` using **JSON patches** (not strategic
merge patches) for at least one of the modifications:

1. Add resource limits using a JSON patch:
   ```yaml
   patches:
     - target:
         group: apps
         version: v1
         kind: Deployment
         name: webapp
       patch: |-
         - op: add
           path: /spec/template/spec/containers/0/resources/limits
           value:
             cpu: "1"
             memory: "1Gi"
   ```

2. Add a `topologySpreadConstraints` rule via a strategic merge patch that
   spreads pods across zones using `topology.kubernetes.io/zone`.

3. Create a `HorizontalPodAutoscaler` targeting the `webapp` deployment with:
   - minReplicas: 3
   - maxReplicas: 10
   - CPU target utilization: 70%

### Task 5: Validate Patch Output

For each overlay, run `kustomize build` and verify:

```bash
# Dev: should have nodeSelector, annotation, hostPath volume
kustomize build exercise-03/overlays/dev | grep -A5 "nodeSelector"
kustomize build exercise-03/overlays/dev | grep "team: platform-eng"

# Staging: should have probes, PDB, TLS annotation
kustomize build exercise-03/overlays/staging | grep -A10 "readinessProbe"
kustomize build exercise-03/overlays/staging | grep "kind: PodDisruptionBudget"

# Production: should have limits, topologySpreadConstraints, HPA
kustomize build exercise-03/overlays/production | grep -A5 "topologySpreadConstraints"
kustomize build exercise-03/overlays/production | grep "kind: HorizontalPodAutoscaler"
```

---

## Success Criteria

- [ ] Dev overlay adds nodeSelector, annotation, and hostPath volume via patch.
- [ ] Staging overlay adds readiness and liveness probes via patch.
- [ ] Staging overlay includes a PodDisruptionBudget as a new resource.
- [ ] Staging overlay adds TLS annotation to the Service.
- [ ] Production overlay uses at least one JSON patch.
- [ ] Production overlay includes a HorizontalPodAutoscaler.
- [ ] All overlays build without errors.
- [ ] Base files are not modified.

---

## Hints

<details>
<summary>Hint 1: Strategic Merge Patch Format</summary>

A strategic merge patch is a partial Kubernetes object. Kustomize merges it with
the matching resource. For adding a volume to a Deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
spec:
  template:
    spec:
      volumes:
        - name: data
          hostPath:
            path: /tmp/webapp-data
            type: DirectoryOrCreate
      containers:
        - name: webapp
          volumeMounts:
            - name: data
              mountPath: /data
```

The `containers` list merges by the `name` field, so specifying
`name: webapp` targets the existing container rather than adding a new one.

</details>

<details>
<summary>Hint 2: Patch Targeting</summary>

When using `patches` in kustomization.yaml with a `target` selector, the patch
file does not need `apiVersion`, `kind`, or `metadata`. For a strategic merge
patch targeting a Deployment container:

```yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: webapp
    patch: |-
      spec:
        template:
          spec:
            containers:
              - name: webapp
                readinessProbe:
                  httpGet:
                    path: /healthz
                    port: 8080
                  initialDelaySeconds: 5
                  periodSeconds: 10
```

You can inline the patch with `patch: |-` or use `path: patch-file.yaml`.

</details>

<details>
<summary>Hint 3: JSON Patch Syntax</summary>

JSON patches use RFC 6902 operations:
- `add` -- insert a value at the path
- `remove` -- delete a value at the path
- `replace` -- overwrite a value at the path
- `move` -- move a value from one path to another
- `copy` -- copy a value from one path to another
- `test` -- assert a value at the path

The path uses JSON Pointer syntax (`/spec/template/spec/containers/0`).
Array indices are zero-based.

For `add` on an existing path, the value is merged. For a path that does not
exist, the value is inserted.

</details>

<details>
<summary>Hint 4: Patching an Existing Container's Probes</summary>

When you use a strategic merge patch to add probes to a container, you must
include the container `name` field so the merge targets the correct container.
Without `name`, Kustomize may create a duplicate container entry or fail.

</details>

<details>
<summary>Hint 5: Creating a New Resource via Patch</summary>

PodDisruptionBudgets and HorizontalPodAutoscalers are not patches to existing
resources -- they are new resources. Create them as standalone YAML files and
add them to the overlay's `resources` list in `kustomization.yaml`.

PDB example:

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: webapp-pdb
spec:
  minAvailable: 1
  selector:
    matchLabels:
      app: webapp
```

</details>
