# 36 — Kustomize: Configuration Management Without Templating

> **Previous:** [35 — Helm Charts](../35-helm-charts/) |
> **Next:** [37 — Kubernetes Networking Deep Dive](../37-kubernetes-networking-deep-dive/)

---

## Problem

You have a Kubernetes application that must be deployed across multiple environments
(dev, staging, production) and possibly multiple clusters or regions. Each environment
shares 90% of the same manifests but differs in replica counts, resource limits, image
tags, environment variables, and ingress hostnames. Copy-pasting manifests for each
environment creates drift. Templating engines (Helm) introduce a new DSL, require
maintaining `values.yaml` files, and can produce unrenderable output if a value is
missing.

You want a way to manage environment-specific configuration by composing and
patching plain Kubernetes manifests — no templating language, no Go templates,
no Tiller.

---

## Naive Way — Copy-Paste Per Environment

```
manifests/
  dev/
    deployment.yaml      # replicas: 1, image: myapp:dev
    service.yaml
    ingress.yaml
  staging/
    deployment.yaml      # replicas: 3, image: myapp:staging
    service.yaml
    ingress.yaml
  production/
    deployment.yaml      # replicas: 10, image: myapp:v1.2.0
    service.yaml
    ingress.yaml
```

**Why this fails:**

- Three copies of every manifest; a fix in one must be replicated to the others.
- Drift is inevitable: staging has an env var that dev does not, production has
  a different health check path.
- No single source of truth; diffs between environments are invisible.
- Onboarding a new environment means copying and editing everything again.

---

## Right Way — Kustomize Base + Overlays

### Directory Structure

```
kustomize-app/
  base/
    kustomization.yaml
    deployment.yaml
    service.yaml
    ingress.yaml
  overlays/
    dev/
      kustomization.yaml
      replicas-patch.yaml
      ingress-patch.yaml
    staging/
      kustomization.yaml
      replicas-patch.yaml
    production/
      kustomization.yaml
      replicas-patch.yaml
      hpa.yaml
```

### Base — The Common Manifests

```yaml
# base/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  labels:
    app: myapp
spec:
  replicas: 1
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
        - name: myapp
          image: myregistry/myapp
          ports:
            - containerPort: 8080
          env:
            - name: LOG_LEVEL
              value: "info"
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
```

```yaml
# base/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
metadata:
  name: myapp
commonLabels:
  app: myapp
  managed-by: kustomize
resources:
  - deployment.yaml
  - service.yaml
  - ingress.yaml
```

### Overlay — Environment-Specific Patches

```yaml
# overlays/dev/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base

namePrefix: dev-
commonLabels:
  environment: dev

# Simple field override without a full patch file
replicas:
  - name: myapp
    count: 1

images:
  - name: myregistry/myapp
    newTag: dev-latest

patches:
  - path: ingress-patch.yaml
```

```yaml
# overlays/dev/ingress-patch.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: myapp
spec:
  rules:
    - host: dev.myapp.company.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: dev-myapp
                port:
                  number: 80
```

```yaml
# overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../base
  - hpa.yaml

namePrefix: prod-
commonLabels:
  environment: production

replicas:
  - name: myapp
    count: 10

images:
  - name: myregistry/myapp
    newTag: v1.2.0

patches:
  - path: ingress-patch.yaml
  - target:
      kind: Deployment
      name: myapp
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/resources/limits/cpu
        value: "2"
      - op: replace
        path: /spec/template/spec/containers/0/resources/limits/memory
        value: "2Gi"
```

### Render and Apply

```bash
# Preview the rendered output
kubectl kustomize overlays/dev/

# Apply directly
kubectl apply -k overlays/dev/

# Compare environments
diff <(kubectl kustomize overlays/dev/) <(kubectl kustomize overlays/production/)
```

---

## Production Way — Advanced Kustomize Features

### 1. Components for Optional Features

```yaml
# components/monitoring/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1alpha1
kind: Component
resources:
  - service-monitor.yaml
patches:
  - target:
      kind: Deployment
    patch: |-
      - op: add
        path: /spec/template/metadata/annotations
        value:
          prometheus.io/scrape: "true"
          prometheus.io/port: "8080"
```

```yaml
# overlays/staging/kustomization.yaml
resources:
  - ../../base
components:
  - ../../components/monitoring
  - ../../components/tracing
```

### 2. ConfigMap/Secret Generation

```yaml
# overlays/production/kustomization.yaml
configMapGenerator:
  - name: app-config
    behavior: merge
    literals:
      - LOG_LEVEL=warn
      - DB_POOL_SIZE=20
      - CACHE_TTL=3600

secretGenerator:
  - name: app-secrets
    envs:
      - secrets.env
    type: Opaque
```

Kustomize hashes the generated ConfigMap/Secret, triggering a rolling update when
values change:

```
app-config-k2m5d8f   (auto-generated name with content hash)
```

### 3. Replacements (more powerful than patches)

```yaml
# kustomization.yaml
replacements:
  - source:
      kind: ConfigMap
      name: app-config
      fieldPath: data.DNS_NAME
    targets:
      - select:
          kind: Ingress
        fieldPaths:
          - spec.rules.0.host
      - select:
          kind: Deployment
        fieldPaths:
          - spec.template.spec.containers.0.env.[name=HOSTNAME].value
```

### 4. Remote Bases

```yaml
# kustomization.yaml
resources:
  - https://github.com/company/k8s-base-manifests//base/myapp?ref=v2.0.0
  - https://github.com/external-ns/ingress-nginx//deploy/static/provider/cloud?ref=controller-v1.8.0
```

### 5. Multi-Cluster Strategy

```
kustomize-app/
  base/
    ...
  components/
    monitoring/
    tracing/
    network-policies/
  overlays/
    dev/
    staging/
    production-us-east/
      kustomization.yaml    # + region-specific configmap, ingress
    production-eu-west/
      kustomization.yaml    # + region-specific configmap, ingress
```

### 6. CI/CD Integration

```yaml
# .github/workflows/deploy.yaml
- name: Render manifests
  run: |
    kubectl kustomize overlays/${{ matrix.environment }} > rendered.yaml

- name: Validate
  run: |
    kubeconform -strict rendered.yaml

- name: Diff against cluster
  run: |
    kubectl diff -f rendered.yaml || true

- name: Apply
  if: github.ref == 'refs/heads/main'
  run: |
    kubectl apply -k overlays/${{ matrix.environment }}
```

### Kustomize vs Helm Decision Matrix

| Concern | Kustomize | Helm |
|---|---|---|
| Templating language | None (YAML only) | Go templates |
| Learning curve | Low | Medium-High |
| Chart repository | Not needed | OCI, HTTP repos |
| Library reuse | Components, remote bases | Subcharts, library charts |
| Secret management | Built-in generator | External (Sealed Secrets, Vault) |
| Lifecycle management | No built-in rollback | `helm rollback` |
| Best for | Patching existing manifests | Distributing reusable software |

---

## Hands-On Lab

### Lab: Kustomize Multi-Environment App

```bash
# 1. Create the project structure
mkdir -p kustomize-lab/{base,overlays/{dev,staging,production}}

# 2. Write base manifests
cat > kustomize-lab/base/deployment.yaml <<'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: echo-server
spec:
  replicas: 1
  selector:
    matchLabels:
      app: echo-server
  template:
    metadata:
      labels:
        app: echo-server
    spec:
      containers:
        - name: echo
          image: hashicorp/http-echo
          args: ["-text=hello from base"]
          ports:
            - containerPort: 5678
EOF

cat > kustomize-lab/base/service.yaml <<'EOF'
apiVersion: v1
kind: Service
metadata:
  name: echo-server
spec:
  ports:
    - port: 80
      targetPort: 5678
  selector:
    app: echo-server
EOF

cat > kustomize-lab/base/kustomization.yaml <<'EOF'
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - deployment.yaml
  - service.yaml
EOF

# 3. Create dev overlay
cat > kustomize-lab/overlays/dev/kustomization.yaml <<'EOF'
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
namePrefix: dev-
commonLabels:
  environment: dev
patches:
  - target:
      kind: Deployment
      name: echo-server
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/args
        value: ["-text=hello from DEV"]
EOF

# 4. Create production overlay
cat > kustomize-lab/overlays/production/kustomization.yaml <<'EOF'
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
namePrefix: prod-
commonLabels:
  environment: production
replicas:
  - name: echo-server
    count: 5
patches:
  - target:
      kind: Deployment
      name: echo-server
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/args
        value: ["-text=hello from PRODUCTION"]
EOF

# 5. Preview rendered output for each environment
echo "=== DEV ==="
kubectl kustomize kustomize-lab/overlays/dev/

echo "=== PRODUCTION ==="
kubectl kustomize kustomize-lab/overlays/production/

# 6. Diff between environments
diff <(kubectl kustomize kustomize-lab/overlays/dev/) \
     <(kubectl kustomize kustomize-lab/overlays/production/)

# 7. Apply dev to a cluster
kubectl apply -k kustomize-lab/overlays/dev/

# 8. Verify
kubectl get deployments -l environment=dev
kubectl get all -l app=echo-server

# 9. Use built-in kustomize (kubectl 1.14+)
kubectl kustomize kustomize-lab/overlays/dev/ | kubectl apply -f -
```

### Lab: ConfigMap Generation and Image Overrides

```bash
# 1. Add a ConfigMap generator
cat >> kustomize-lab/overlays/dev/kustomization.yaml <<'EOF'
configMapGenerator:
  - name: app-config
    literals:
      - LOG_LEVEL=debug
      - FEATURE_FLAG_V2=true
EOF

# 2. Override the image tag
cat >> kustomize-lab/overlays/dev/kustomization.yaml <<'EOF'
images:
  - name: hashicorp/http-echo
    newTag: "0.2.3"
EOF

# 3. Re-render and observe the generated ConfigMap name with hash
kubectl kustomize kustomize-lab/overlays/dev/ | grep -A5 "kind: ConfigMap"
# Notice: app-config-<hash> — changing literals triggers rolling update
```

---

## Limitation

Kustomize solves configuration management cleanly, but **Kubernetes networking
remains complex** regardless of how you generate your manifests. Kustomize does
not help you understand CNI plugins, choose the right NetworkPolicy strategy,
debug DNS resolution failures, or configure service mesh sidecars. Once your
multi-environment manifests are in order, the next challenge is making sure Pods
can actually communicate securely and efficiently.

---

## Next Topic

**[37 — Kubernetes Networking Deep Dive](../37-kubernetes-networking-deep-dive/)** —
CNI plugins, NetworkPolicies, CoreDNS, service discovery, and debugging network
issues in production clusters.
