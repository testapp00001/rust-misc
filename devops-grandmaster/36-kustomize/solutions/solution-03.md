# Solution 03: Patches per Environment

---

## Task 1: Base Setup

### exercise-03/base/deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
spec:
  replicas: 1
  selector:
    matchLabels:
      app: webapp
  template:
    metadata:
      labels:
        app: webapp
    spec:
      containers:
        - name: webapp
          image: myregistry.io/webapp:v1.0.0
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
```

### exercise-03/base/service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: webapp
spec:
  type: ClusterIP
  selector:
    app: webapp
  ports:
    - port: 80
      targetPort: 8080
```

### exercise-03/base/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - deployment.yaml
  - service.yaml
```

---

## Task 2: Dev Overlay -- Strategic Merge Patch

### exercise-03/overlays/dev/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: dev

resources:
  - ../../base

patches:
  - path: patch-deployment.yaml
    target:
      group: apps
      version: v1
      kind: Deployment
      name: webapp
```

### exercise-03/overlays/dev/patch-deployment.yaml

```yaml
spec:
  template:
    metadata:
      annotations:
        team: platform-eng
    spec:
      nodeSelector:
        environment: dev
      containers:
        - name: webapp
          volumeMounts:
            - name: data
              mountPath: /data
      volumes:
        - name: data
          hostPath:
            path: /tmp/webapp-data
            type: DirectoryOrCreate
```

**Explanation:** The strategic merge patch merges with the existing Deployment.
The `containers` list merges by the `name` field, so `name: webapp` targets
the existing container rather than adding a new one. The `volumes` and
`nodeSelector` fields are additive -- they do not exist in the base, so they
are inserted. The annotation is added to the pod template metadata.

---

## Task 3: Staging Overlay -- Probes and PDB

### exercise-03/overlays/staging/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: staging

resources:
  - ../../base
  - pdb.yaml

patches:
  - path: patch-probes.yaml
    target:
      group: apps
      version: v1
      kind: Deployment
      name: webapp
  - path: patch-service-tls.yaml
    target:
      version: v1
      kind: Service
      name: webapp
```

### exercise-03/overlays/staging/patch-probes.yaml

```yaml
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
            timeoutSeconds: 3
            failureThreshold: 3
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
            timeoutSeconds: 3
            failureThreshold: 3
```

### exercise-03/overlays/staging/patch-service-tls.yaml

```yaml
metadata:
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-backend-protocol: https
```

### exercise-03/overlays/staging/pdb.yaml

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

**Explanation:** The probes patch targets only the `webapp` container by name.
The Service patch adds an annotation -- since the Service has no `spec` merge
key, only metadata fields, this is a simple metadata merge. The PDB is a new
resource added via `resources`, not a patch.

---

## Task 4: Production Overlay -- JSON Patches, HPA, Topology

### exercise-03/overlays/production/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: production

resources:
  - ../../base
  - hpa.yaml

patches:
  # JSON patch: add resource limits
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
  # Strategic merge patch: add topology spread constraints
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: webapp
    patch: |-
      spec:
        template:
          spec:
            topologySpreadConstraints:
              - maxSkew: 1
                topologyKey: topology.kubernetes.io/zone
                whenUnsatisfiable: DoNotSchedule
                labelSelector:
                  matchLabels:
                    app: webapp
```

### exercise-03/overlays/production/hpa.yaml

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: webapp-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: webapp
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

**Explanation:** The JSON patch uses `op: add` with path
`/spec/template/spec/containers/0/resources/limits`. The `0` index targets
the first container. If the container already has a `resources` object with
`requests`, the `add` operation inserts the `limits` key into the existing
object. The topologySpreadConstraints patch is a strategic merge patch that
adds a new list item (the list does not exist in the base, so it is created).

---

## Task 5: Validate Patch Output

```bash
# Dev
$ kustomize build exercise-03/overlays/dev | grep -A2 "nodeSelector"
      nodeSelector:
        environment: dev

$ kustomize build exercise-03/overlays/dev | grep "team"
        team: platform-eng

$ kustomize build exercise-03/overlays/dev | grep -A3 "hostPath"
        - hostPath:
            path: /tmp/webapp-data
            type: DirectoryOrCreate
          name: data

# Staging
$ kustomize build exercise-03/overlays/staging | grep -A5 "readinessProbe"
          readinessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10

$ kustomize build exercise-03/overlays/staging | grep "kind: PodDisruptionBudget"
kind: PodDisruptionBudget

$ kustomize build exercise-03/overlays/staging | grep "aws-load-balancer"
    service.beta.kubernetes.io/aws-load-balancer-backend-protocol: https

# Production
$ kustomize build exercise-03/overlays/production | grep -A3 "topologySpreadConstraints"
      topologySpreadConstraints:
        - labelSelector:
            matchLabels:
              app: webapp

$ kustomize build exercise-03/overlays/production | grep "kind: HorizontalPodAutoscaler"
kind: HorizontalPodAutoscaler

$ kustomize build exercise-03/overlays/production | grep -A2 "limits"
            limits:
              cpu: "1"
              memory: 1Gi
```

---

## Why It Works

**Strategic merge patches** use the Kubernetes API merge semantics. Lists of
objects with a merge key (like `containers` with `name` key) are merged by
matching on that key. Scalar fields are replaced. Maps are merged recursively.
This means you can add a `readinessProbe` to a container by specifying just
the container name and the probe config -- you do not need to copy the entire
container spec.

**JSON patches** (RFC 6902) are imperative operations: "add this value at this
path." They are more verbose but more precise. They are useful when the
strategic merge semantics do not do what you want -- for example, adding a
`limits` object to a `resources` object that already has `requests`.

**Target selectors** with `group`, `version`, `kind`, `name` tell Kustomize
exactly which resource to patch. Without a target, the patch file must contain
`apiVersion`, `kind`, and `metadata.name` and is matched by those fields. With
a target, the patch file is applied to the resource that matches the selector.

**New resources** (PDB, HPA) are not patches -- they are standalone resources.
They are added via the `resources` field in the overlay's `kustomization.yaml`.

---

## Common Mistakes

1. **Omitting the container `name` in a strategic merge patch.** If you patch
   `containers` without specifying `name`, Kustomize may add a new container
   entry instead of merging with the existing one. Always include the merge
   key.

2. **Using JSON patch `add` on an existing object.** If the path already
   exists, `add` replaces the entire value at that path. Use `replace` for
   existing paths or structure the path to target only the new key.

3. **Confusing patch types.** When you use `patches` in kustomization.yaml
   with a `target` selector, the patch type is inferred from the content. If
   the patch is a list (`- op: add ...`), it is treated as a JSON patch. If
   it is a map (`spec: ...`), it is treated as a strategic merge patch. Mixing
   them in the wrong context causes errors.

4. **Forgetting that `patches` with `target` replaces `patchesStrategicMerge`.**
   In older Kustomize versions, `patchesStrategicMerge` and `patchesJson6902`
   were separate fields. The unified `patches` field with `target` replaces
   both. Use the unified field for new projects.

5. **Not testing patches in isolation.** Always run `kustomize build` on each
   overlay independently. A patch that works on one overlay may conflict with
   another if they target the same field.
