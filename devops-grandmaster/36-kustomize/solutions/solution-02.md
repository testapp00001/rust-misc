# Solution 02: Base Configuration with Environment Overlays

---

## Task 1: Base Directory Structure

### base/deployment.yaml

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
          image: myregistry.io/webapp:latest
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 200m
              memory: 256Mi
          env:
            - name: WEBAPP_LOG_LEVEL
              valueFrom:
                configMapKeyRef:
                  name: webapp-config
                  key: LOG_LEVEL
```

### base/service.yaml

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
      protocol: TCP
```

### base/configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: webapp-config
data:
  LOG_LEVEL: info
```

### base/kustomization.yaml

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

---

## Task 2: Validate the Base

```bash
$ kustomize build exercise-02/base
```

Expected output includes all three resources with `app.kubernetes.io/name: webapp`
and `app.kubernetes.io/managed-by: kustomize` labels on every resource.

---

## Task 3: Dev Overlay

### overlays/dev/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: dev

resources:
  - ../../base

images:
  - name: myregistry.io/webapp
    newTag: latest

replicas:
  - name: webapp
    count: 1

configMapGenerator:
  - name: webapp-config
    behavior: replace
    literals:
      - LOG_LEVEL=debug

generatorOptions:
  disableNameSuffixHash: true

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
                resources:
                  requests:
                    cpu: 100m
                    memory: 128Mi
                  limits:
                    cpu: 200m
                    memory: 256Mi
```

---

## Task 4: Staging Overlay

### overlays/staging/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: staging

resources:
  - ../../base

images:
  - name: myregistry.io/webapp
    newTag: v1.2.0-rc1

replicas:
  - name: webapp
    count: 2

configMapGenerator:
  - name: webapp-config
    behavior: replace
    literals:
      - LOG_LEVEL=info

generatorOptions:
  disableNameSuffixHash: true

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
                resources:
                  requests:
                    cpu: 250m
                    memory: 256Mi
                  limits:
                    cpu: 500m
                    memory: 512Mi
```

---

## Task 5: Production Overlay

### overlays/production/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: production

resources:
  - ../../base

images:
  - name: myregistry.io/webapp
    newTag: v1.2.0

replicas:
  - name: webapp
    count: 5

commonLabels:
  app.kubernetes.io/env: production

configMapGenerator:
  - name: webapp-config
    behavior: replace
    literals:
      - LOG_LEVEL=warn

generatorOptions:
  disableNameSuffixHash: true

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
                resources:
                  requests:
                    cpu: 500m
                    memory: 512Mi
                  limits:
                    cpu: "1"
                    memory: 1Gi
```

---

## Task 6: Validate All Overlays

```bash
# Dev
$ kustomize build exercise-02/overlays/dev | grep "namespace: dev"
namespace: dev
$ kustomize build exercise-02/overlays/dev | grep "replicas: 1"
  replicas: 1
$ kustomize build exercise-02/overlays/dev | grep "LOG_LEVEL"
    LOG_LEVEL: debug

# Staging
$ kustomize build exercise-02/overlays/staging | grep "namespace: staging"
namespace: staging
$ kustomize build exercise-02/overlays/staging | grep "replicas: 2"
  replicas: 2
$ kustomize build exercise-02/overlays/staging | grep "LOG_LEVEL"
    LOG_LEVEL: info

# Production
$ kustomize build exercise-02/overlays/production | grep "namespace: production"
namespace: production
$ kustomize build exercise-02/overlays/production | grep "replicas: 5"
  replicas: 5
$ kustomize build exercise-02/overlays/production | grep "LOG_LEVEL"
    LOG_LEVEL: warn
$ kustomize build exercise-02/overlays/production | grep "app.kubernetes.io/env"
    app.kubernetes.io/env: production
```

---

## Why It Works

**Base layer** defines the canonical resource specifications. The `commonLabels`
field ensures every resource carries identification metadata without manual
repetition.

**`images` transformer** scans all resource specs for containers using the
named image and replaces the tag. It does not require knowing which Deployment
or container -- it matches by image name.

**`replicas` transformer** targets a Deployment by name and sets the replica
count. This is a dedicated transformer, not a patch, because replica changes
are so common they warranted first-class support.

**`configMapGenerator` with `behavior: replace`** tells Kustomize to discard
the base ConfigMap and use only the overlay's version. Without `behavior:
replace`, Kustomize would merge the data keys, which could leave stale values.

**`generatorOptions.disableNameSuffixHash: true`** prevents Kustomize from
appending a content hash to generated ConfigMap names (e.g., `webapp-config-`
would become `webapp-config-8k2m4x`). This is necessary when the Deployment
references the ConfigMap by exact name.

**`namespace` field** sets the namespace on every resource in the output. This
is a global transformer that applies to all resources.

---

## Common Mistakes

1. **Forgetting `behavior: replace` on configMapGenerator.** Without it,
   Kustomize merges the overlay ConfigMap data with the base ConfigMap data.
   If the base has `LOG_LEVEL=info` and the overlay has `LOG_LEVEL=debug`,
   the result is `LOG_LEVEL=debug` (last wins for duplicate keys), but
   other keys from the base are still present.

2. **Forgetting `disableNameSuffixHash`.** The generated ConfigMap gets a hash
   suffix (e.g., `webapp-config-7k4m2x`), but the Deployment references
   `webapp-config`. The pod fails with "configmap not found."

3. **Using `newTag` on a non-existent image name.** The `images` transformer
   matches by `name`, which must be the full image name including registry
   prefix. If the base uses `myregistry.io/webapp` and you write `webapp` in
   the `images` field, it will not match.

4. **Not setting `namespace` in the base when using `configMapGenerator`.** If
   the base ConfigMap has no namespace and the overlay sets `namespace: dev`,
   the ConfigMap gets the namespace from the overlay. This is usually correct,
   but be aware that the base ConfigMap's namespace is not "dev" until the
   overlay applies it.

5. **Confusing `commonLabels` with `labels`.** In Kustomize, `commonLabels`
   adds labels to all resources and their selectors. `labels` (in newer
   versions) adds labels to metadata only. Using `commonLabels` incorrectly
   can change Service selectors, breaking traffic routing.
