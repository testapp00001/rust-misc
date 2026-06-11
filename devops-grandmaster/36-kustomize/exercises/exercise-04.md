# Exercise 04: Implement a Multi-Cluster Kustomize Strategy

**Type:** Challenge
**Time:** 50 minutes
**Objective:** Design a Kustomize structure that supports deploying the same
application across multiple clusters with different configurations, using
generators, transformers, and composition.

---

## Background

Your company runs Kubernetes in three regions:

| Cluster | Region | Characteristics |
|---------|--------|-----------------|
| `us-east-1` | Virginia | Primary. High CPU limits. Internal load balancer. |
| `us-west-2` | Oregon | Secondary. Standard limits. External load balancer. |
| `eu-west-1` | Ireland | GDPR region. Data residency enforced. Extra audit logging. |

Each cluster has:
- Different container registry mirrors (`us-east-1.registry.io`,
  `us-west-2.registry.io`, `eu-west-1.registry.io`)
- Different node taints (hence different tolerations)
- Different resource quotas (different CPU/memory limits)
- Different ConfigMaps with region-specific endpoints
- The EU cluster requires a `PodSecurityPolicy` equivalent and an audit
  ConfigMap sidecar

Your team wants a **single Git repository** that serves as the source of truth
for all three clusters, with no duplication of base manifests.

---

## Tasks

### Task 1: Design the Directory Structure

Design a Kustomize directory layout that supports:

1. A **base** with the common application manifests
2. **Environment overlays** for shared concerns (dev, staging, prod)
3. **Cluster overlays** that layer on top of environment overlays
4. A **components** directory for reusable pieces (e.g., monitoring sidecar,
   audit logging)

Your structure should look something like this (fill in the details):

```
exercise-04/
  base/
    ...
  components/
    monitoring/
    audit-logging/
  overlays/
    environments/
      dev/
      staging/
      production/
    clusters/
      us-east-1/
      us-west-2/
      eu-west-1/
```

### Task 2: Implement the Base

Create the base with:
- Deployment for `webapp` (image: `webapp:v1.0.0`)
- Service (ClusterIP, port 80 -> 8080)
- ConfigMap with `APP_ENV=production` and `REGION=unknown`
- ServiceAccount named `webapp-sa`

### Task 3: Implement the Monitoring Component

Create a Kustomize **component** (not a regular overlay) at
`exercise-04/components/monitoring/`.

The component should:
1. Add a `configMapGenerator` that creates a ConfigMap named
   `monitoring-config` with `PROMETHEUS_ENABLED=true` and `METRICS_PORT=9090`
2. Add a patch to the Deployment that injects a Prometheus annotations:
   ```yaml
   annotations:
     prometheus.io/scrape: "true"
     prometheus.io/port: "9090"
   ```

A component uses `kind: Kustomization` but is referenced with the `components`
field (not `resources`). This allows it to be composed with other components.

```yaml
# In the overlay kustomization.yaml
components:
  - ../../components/monitoring
```

### Task 4: Implement Cluster Overlays

For each cluster, create an overlay that:

**us-east-1:**
- Sets the registry to `us-east-1.registry.io`
- Adds tolerations for `region=us-east:NoSchedule`
- Sets CPU limit to `2000m`, memory limit to `2Gi`
- Includes the monitoring component
- Adds a node affinity for `topology.kubernetes.io/zone: us-east-1a`

**us-west-2:**
- Sets the registry to `us-west-2.registry.io`
- Adds tolerations for `region=us-west:NoSchedule`
- Sets CPU limit to `1000m`, memory limit to `1Gi`
- Includes the monitoring component

**eu-west-1:**
- Sets the registry to `eu-west-1.registry.io`
- Adds tolerations for `region=eu-west:NoSchedule`
- Sets CPU limit to `1000m`, memory limit to `1Gi`
- Includes the monitoring component
- Includes the audit-logging component (adds a sidecar container for audit
  logs)
- Adds a label `data-residency: eu`
- Overrides the ConfigMap with `GDPR_COMPLIANCE=true`

### Task 5: Implement Replacements for Cluster-Specific Values

Use the **replacements** transformer to inject the cluster-specific registry
into the image reference. Create a ConfigMap in each cluster overlay with the
registry URL, then use replacements to patch the Deployment's image field.

```yaml
# replacement source
source:
  kind: ConfigMap
  name: cluster-config
  fieldPath: data.REGISTRY
# replacement target
targets:
  - select:
      kind: Deployment
      name: webapp
    fieldPaths:
      - spec.template.spec.containers.[name=webapp].image
    options:
      delimiter: '/'
      index: 0
```

### Task 6: Validate the Full Build

```bash
# Should produce output for us-east-1 with monitoring, high limits
kustomize build exercise-04/overlays/clusters/us-east-1

# Should produce output for eu-west-1 with monitoring, audit, GDPR label
kustomize build exercise-04/overlays/clusters/eu-west-1 | grep "data-residency"
kustomize build exercise-04/overlays/clusters/eu-west-1 | grep "GDPR_COMPLIANCE"
```

---

## Success Criteria

- [ ] Base builds independently and contains all common resources.
- [ ] Monitoring component can be included via `components` field.
- [ ] Each cluster overlay builds without errors.
- [ ] us-east-1 output shows higher CPU limits than eu-west-1.
- [ ] eu-west-1 output includes the audit sidecar and GDPR labels.
- [ ] Each cluster has the correct registry in the image reference.
- [ ] Replacements transformer correctly splits and replaces the registry
      portion of the image string.
- [ ] No duplication of base manifests across cluster overlays.

---

## Hints

<details>
<summary>Hint 1: Components vs. Overlays</summary>

A **component** is a reusable Kustomize unit that can be included by multiple
overlays. Unlike regular resources, components can be included multiple times
in a build tree. Define a component with:

```yaml
# components/monitoring/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1alpha1
kind: Component
# ... resources, patches, etc.
```

Reference it in an overlay with:
```yaml
components:
  - ../../components/monitoring
```

</details>

<details>
<summary>Hint 2: Layering Order</summary>

Kustomize processes the `kustomization.yaml` fields in a specific order:
1. `resources` and `components` are loaded first
2. `patchesStrategicMerge` and `patches` are applied next
3. `configMapGenerator` and `secretGenerator` run
4. `replacements` run last

This order matters: replacements can reference generated ConfigMaps because
they run after generators.

</details>

<details>
<summary>Hint 3: Replacements Syntax</summary>

The `replacements` field copies values from a source resource to target
resources. For splitting an image string like `us-east-1.registry.io/webapp:v1`:

```yaml
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
```

`delimiter: '/'` splits the image string into parts, and `index: 0` replaces
only the first part (the registry).

</details>

<details>
<summary>Hint 4: Audit Sidecar Pattern</summary>

For the eu-west-1 audit logging component, add a sidecar container via a
strategic merge patch:

```yaml
spec:
  template:
    spec:
      containers:
        - name: audit-logger
          image: audit-logger:v1.0.0
          args:
            - --log-path=/var/log/audit
          volumeMounts:
            - name: audit-logs
              mountPath: /var/log/audit
      volumes:
        - name: audit-logs
          emptyDir: {}
```

The main container also needs a volumeMount for the shared `audit-logs` volume.

</details>

<details>
<summary>Hint 5: Combining Environment and Cluster Layers</summary>

If you need both environment and cluster customization, have the cluster
overlay reference the environment overlay as its base:

```
overlays/clusters/us-east-1/kustomization.yaml:
  resources:
    - ../environments/production
```

Or have the cluster overlay reference the base directly and include both
environment patches and cluster patches. The simpler approach is to have
cluster overlays contain everything they need, referencing the base and
applying all patches in one place.

</details>
