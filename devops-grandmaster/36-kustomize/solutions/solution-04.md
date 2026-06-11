# Solution 04: Multi-Cluster Kustomize Strategy

---

## Task 1: Directory Structure

```
exercise-04/
  base/
    kustomization.yaml
    deployment.yaml
    service.yaml
    configmap.yaml
    serviceaccount.yaml
  components/
    monitoring/
      kustomization.yaml
      patch-deployment.yaml
      configmap.yaml
    audit-logging/
      kustomization.yaml
      patch-sidecar.yaml
      configmap.yaml
  overlays/
    clusters/
      us-east-1/
        kustomization.yaml
        cluster-config.yaml
        patch-deployment.yaml
      us-west-2/
        kustomization.yaml
        cluster-config.yaml
        patch-deployment.yaml
      eu-west-1/
        kustomization.yaml
        cluster-config.yaml
        patch-deployment.yaml
```

---

## Task 2: Base Implementation

### base/deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: webapp
  template:
    metadata:
      labels:
        app: webapp
    spec:
      serviceAccountName: webapp-sa
      containers:
        - name: webapp
          image: registry.example.com/webapp:v1.0.0
          ports:
            - containerPort: 8080
          envFrom:
            - configMapRef:
                name: webapp-config
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
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
```

### base/configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: webapp-config
data:
  APP_ENV: production
  REGION: unknown
```

### base/serviceaccount.yaml

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: webapp-sa
```

### base/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - deployment.yaml
  - service.yaml
  - configmap.yaml
  - serviceaccount.yaml
```

---

## Task 3: Monitoring Component

### components/monitoring/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1alpha1
kind: Component

configMapGenerator:
  - name: monitoring-config
    literals:
      - PROMETHEUS_ENABLED=true
      - METRICS_PORT=9090

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
          metadata:
            annotations:
              prometheus.io/scrape: "true"
              prometheus.io/port: "9090"
```

**Key point:** This uses `kind: Component`, not `kind: Kustomization`. A
component is a reusable unit that can be included by multiple overlays via the
`components` field. Unlike resources, components do not create standalone
resource manifests -- they are composed into the overlay's build.

---

## Task 4: Cluster Overlays

### us-east-1/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: us-east-1

resources:
  - ../../base
  - cluster-config.yaml

components:
  - ../../components/monitoring

configMapGenerator:
  - name: webapp-config
    behavior: merge
    literals:
      - REGION=us-east-1

generatorOptions:
  disableNameSuffixHash: true

replacements:
  - source:
      kind: ConfigMap
      name: cluster-config
      fieldPath: data.REGISTRY
    targets:
      - select:
          kind: Deployment
          name: webapp
        fieldPaths:
          - spec.template.spec.containers.[name=webapp].image
        options:
          delimiter: '/'
          index: 0

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
            tolerations:
              - key: region
                operator: Equal
                value: us-east
                effect: NoSchedule
            affinity:
              nodeAffinity:
                requiredDuringSchedulingIgnoredDuringExecution:
                  nodeSelectorTerms:
                    - matchExpressions:
                        - key: topology.kubernetes.io/zone
                          operator: In
                          values:
                            - us-east-1a
            containers:
              - name: webapp
                resources:
                  limits:
                    cpu: "2"
                    memory: 2Gi
```

### us-east-1/cluster-config.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cluster-config
data:
  REGISTRY: us-east-1.registry.io
```

### us-west-2/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: us-west-2

resources:
  - ../../base
  - cluster-config.yaml

components:
  - ../../components/monitoring

configMapGenerator:
  - name: webapp-config
    behavior: merge
    literals:
      - REGION=us-west-2

generatorOptions:
  disableNameSuffixHash: true

replacements:
  - source:
      kind: ConfigMap
      name: cluster-config
      fieldPath: data.REGISTRY
    targets:
      - select:
          kind: Deployment
          name: webapp
        fieldPaths:
          - spec.template.spec.containers.[name=webapp].image
        options:
          delimiter: '/'
          index: 0

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
            tolerations:
              - key: region
                operator: Equal
                value: us-west
                effect: NoSchedule
            containers:
              - name: webapp
                resources:
                  limits:
                    cpu: "1"
                    memory: 1Gi
```

### us-west-2/cluster-config.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cluster-config
data:
  REGISTRY: us-west-2.registry.io
```

### eu-west-1/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: eu-west-1

resources:
  - ../../base
  - cluster-config.yaml

components:
  - ../../components/monitoring
  - ../../components/audit-logging

commonLabels:
  data-residency: eu

configMapGenerator:
  - name: webapp-config
    behavior: merge
    literals:
      - REGION=eu-west-1
      - GDPR_COMPLIANCE=true

generatorOptions:
  disableNameSuffixHash: true

replacements:
  - source:
      kind: ConfigMap
      name: cluster-config
      fieldPath: data.REGISTRY
    targets:
      - select:
          kind: Deployment
          name: webapp
        fieldPaths:
          - spec.template.spec.containers.[name=webapp].image
        options:
          delimiter: '/'
          index: 0

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
            tolerations:
              - key: region
                operator: Equal
                value: eu-west
                effect: NoSchedule
            containers:
              - name: webapp
                resources:
                  limits:
                    cpu: "1"
                    memory: 1Gi
```

### eu-west-1/cluster-config.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cluster-config
data:
  REGISTRY: eu-west-1.registry.io
```

### Audit Logging Component (Bonus)

### components/audit-logging/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1alpha1
kind: Component

configMapGenerator:
  - name: audit-config
    literals:
      - AUDIT_ENABLED=true
      - AUDIT_LOG_PATH=/var/log/audit

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
                volumeMounts:
                  - name: audit-logs
                    mountPath: /var/log/audit
                    readOnly: false
              - name: audit-logger
                image: audit-logger:v1.0.0
                args:
                  - --log-path=/var/log/audit
                volumeMounts:
                  - name: audit-logs
                    mountPath: /var/log/audit
                    readOnly: true
            volumes:
              - name: audit-logs
                emptyDir: {}
```

---

## Task 5: Replacements Explanation

The `replacements` transformer works as follows:

1. **Source:** Read the value `us-east-1.registry.io` from
   `cluster-config` ConfigMap's `data.REGISTRY` field.

2. **Target:** Find the Deployment named `webapp` and locate the image field
   of the container named `webapp`:
   `spec.template.spec.containers.[name=webapp].image`

3. **Options:** Split the current image value `registry.example.com/webapp:v1.0.0`
   by `/` into `["registry.example.com", "webapp:v1.0.0"]`, replace index `0`
   with `us-east-1.registry.io`, and join back with `/`.

4. **Result:** `us-east-1.registry.io/webapp:v1.0.0`

---

## Task 6: Validate

```bash
# us-east-1: high limits, monitoring, correct registry
$ kustomize build exercise-04/overlays/clusters/us-east-1 | grep "image:"
    image: us-east-1.registry.io/webapp:v1.0.0

$ kustomize build exercise-04/overlays/clusters/us-east-1 | grep "cpu:"
            cpu: "2"

$ kustomize build exercise-04/overlays/clusters/us-east-1 | grep "prometheus.io"
    prometheus.io/port: "9090"
    prometheus.io/scrape: "true"

# eu-west-1: GDPR labels, audit sidecar, correct registry
$ kustomize build exercise-04/overlays/clusters/eu-west-1 | grep "data-residency"
    data-residency: eu

$ kustomize build exercise-04/overlays/clusters/eu-west-1 | grep "GDPR_COMPLIANCE"
    GDPR_COMPLIANCE: "true"

$ kustomize build exercise-04/overlays/clusters/eu-west-1 | grep "audit-logger"
    image: audit-logger:v1.0.0
```

---

## Why It Works

**Components** allow shared functionality to be defined once and included by
multiple overlays. Unlike regular resources, components can be composed -- you
can include both `monitoring` and `audit-logging` in the eu-west-1 overlay.
The `kind: Component` declaration tells Kustomize this is a composable unit.

**`configMapGenerator` with `behavior: merge`** adds new keys to an existing
ConfigMap without replacing it. This lets the base define `APP_ENV` and
`REGION`, and each cluster overlay adds or overrides only the keys it needs.

**Replacements** are the most powerful transformer for multi-cluster setups.
They decouple the configuration value (registry URL) from the resource spec.
Each cluster defines its registry in a ConfigMap, and the replacement
injects it into the Deployment. This is more maintainable than patching the
image field directly.

**`delimiter` and `index` options** in replacements let you target specific
parts of a composite value. The image string `registry.example.com/webapp:v1`
is split by `/`, and only the registry part (index 0) is replaced. The rest
of the string is preserved.

---

## Common Mistakes

1. **Using `behavior: replace` instead of `behavior: merge` on cluster
   ConfigMaps.** With `replace`, the base ConfigMap keys (APP_ENV, REGION)
   are discarded. With `merge`, new keys are added and existing keys with the
   same name are overwritten.

2. **Forgetting `generatorOptions.disableNameSuffixHash`.** Generated
   ConfigMaps get hash suffixes by default. The replacement source references
   `cluster-config`, but the actual name becomes `cluster-config-8k2m4x`.
   The replacement fails because it cannot find the source.

3. **Not including `cluster-config.yaml` in `resources`.** The replacement
   source ConfigMap must be a resource in the build. If it is only in
   `configMapGenerator`, the replacement may not find it in the resource graph.

4. **Confusing `components` and `resources`.** Components are declared with
   `kind: Component` and referenced via the `components` field. If you list a
   component directory in `resources`, Kustomize treats it as a regular
   kustomization, which may work but loses the composability benefits.

5. **Overlapping patches from components and overlays.** If the monitoring
   component patches the Deployment's annotations and the cluster overlay also
   patches annotations, Kustomize applies both patches. If they conflict on
   the same key, the last patch wins. Ensure component and overlay patches
   target different fields or are designed to compose.
