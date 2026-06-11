# Module 30: ConfigMaps & Secrets — Configuration in Kubernetes

**Previous:** [Module 29: Ingress Controllers](../29-ingress-controllers/README.md)

---

## The Problem

Your application needs configuration: database hostnames, API keys, feature flags, log levels, and TLS certificates. In traditional deployments, you might:

- Hardcode values in the application code (wrong — requires rebuild for every change)
- Store them in a `.env` file on the server (wrong — not portable, not version controlled)
- Pass them as command-line arguments (fragile — gets lost in process management)

In Kubernetes, you need a way to inject configuration into containers without rebuilding images, while keeping sensitive data (passwords, keys) separate from non-sensitive data (feature flags, URLs).

---

## The Naive Way

Hardcode configuration in container images or deployment YAML:

```yaml
# In deployment.yaml — visible to anyone with kubectl access
spec:
  containers:
    - name: app
      image: myapp:v1
      env:
        - name: DB_HOST
          value: "prod-db.example.com"
        - name: DB_PASSWORD
          value: "supersecret123"    # Visible in plain text!
        - name: API_KEY
          value: "sk-abc123xyz"      # Visible in plain text!
```

**What goes wrong:**

- Secrets are visible in plain text in the deployment YAML
- Secrets are visible in `kubectl describe pod` output
- Changing config requires editing and reapplying the deployment
- Same image cannot be reused across environments (dev/staging/prod)
- Secrets may end up in version control (Git history)

---

## The Right Way

### ConfigMap — Non-Sensitive Configuration

A **ConfigMap** stores non-sensitive key-value pairs. It can be consumed as environment variables or mounted as files.

**Creating a ConfigMap:**

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  # Simple key-value pairs
  DB_HOST: "postgres-service"
  DB_PORT: "5432"
  DB_NAME: "myapp"
  LOG_LEVEL: "info"
  FEATURE_FLAG_NEW_UI: "true"

  # File-like entries (for mounting as files)
  application.properties: |
    server.port=8080
    spring.datasource.url=jdbc:postgresql://postgres-service:5432/myapp
    logging.level.root=INFO

  nginx.conf: |
    server {
      listen 80;
      server_name localhost;
      location / {
        proxy_pass http://backend:8080;
      }
    }
```

**Creating from the command line:**

```bash
# From literal values
kubectl create configmap app-config \
  --from-literal=DB_HOST=postgres-service \
  --from-literal=DB_PORT=5432 \
  --from-literal=LOG_LEVEL=info

# From a file
kubectl create configmap app-config --from-file=config.properties

# From a directory (each file becomes a key)
kubectl create configmap app-config --from-file=config/

# From an env file
kubectl create configmap app-config --from-env-file=config.env
```

**Consuming as environment variables:**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-config
spec:
  containers:
    - name: app
      image: myapp:v1
      envFrom:
        - configMapRef:
            name: app-config     # All keys become env vars
      env:
        - name: DB_HOST
          valueFrom:
            configMapKeyRef:
              name: app-config
              key: DB_HOST       # Single key from configmap
```

**Mounting as a volume:**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-config-files
spec:
  containers:
    - name: app
      image: myapp:v1
      volumeMounts:
        - name: config-volume
          mountPath: /etc/config
          readOnly: true
  volumes:
    - name: config-volume
      configMap:
        name: app-config
```

Files appear at `/etc/config/`:
- `/etc/config/application.properties`
- `/etc/config/nginx.conf`

### Secret — Sensitive Data

A **Secret** stores sensitive data (passwords, tokens, certificates). It is base64-encoded (not encrypted by default).

**Creating a Secret:**

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
data:
  DB_PASSWORD: c3VwZXJzZWNyZXQxMjM=    # base64 of "supersecret123"
  API_KEY: c2stYWJjMTIzeHl6              # base64 of "sk-abc123xyz"

# OR use stringData for plain text (Kubernetes encodes it)
stringData:
  DB_PASSWORD: "supersecret123"
  API_KEY: "sk-abc123xyz"
```

**Creating from the command line:**

```bash
# From literal values
kubectl create secret generic app-secrets \
  --from-literal=DB_PASSWORD=supersecret123 \
  --from-literal=API_KEY=sk-abc123xyz

# From files
kubectl create secret generic tls-secret \
  --from-file=tls.crt \
  --from-file=tls.key

# TLS secret (special type)
kubectl create secret tls my-tls-secret \
  --cert=tls.crt \
  --key=tls.key

# Docker registry secret
kubectl create secret docker-registry regcred \
  --docker-server=registry.example.com \
  --docker-username=user \
  --docker-password=pass \
  --docker-email=user@example.com
```

**Consuming as environment variables:**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-secrets
spec:
  containers:
    - name: app
      image: myapp:v1
      envFrom:
        - secretRef:
            name: app-secrets    # All keys become env vars
      env:
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: app-secrets
              key: DB_PASSWORD
```

**Mounting as a volume:**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-secret-files
spec:
  containers:
    - name: app
      image: myapp:v1
      volumeMounts:
        - name: secret-volume
          mountPath: /etc/secrets
          readOnly: true
  volumes:
    - name: secret-volume
      secret:
        secretName: app-secrets
```

### ConfigMap vs Secret

| Feature | ConfigMap | Secret |
|---------|-----------|--------|
| **Purpose** | Non-sensitive config | Sensitive data |
| **Data format** | Plain text | Base64-encoded |
| **Size limit** | 1 MiB | 1 MiB |
| **Mounting** | File or env var | File or env var |
| **Encryption at rest** | No (by default) | Yes (if configured) |
| **Visibility** | Anyone with read access | Anyone with read access |

**Important:** Base64 encoding is NOT encryption. Anyone who can read the Secret can decode it. For real security, you need encryption at rest or an external secret manager.

---

## The Production Way

### Encoding vs Encryption

**Encoding (base64):**

```bash
# Encode
echo -n "supersecret123" | base64
# Output: c3VwZXJzZWNyZXQxMjM=

# Decode
echo "c3VwZXJzZWNyZXQxMjM=" | base64 -d
# Output: supersecret123
```

**Encryption at rest:**

Kubernetes can encrypt secrets in etcd using an `EncryptionConfiguration`:

```yaml
# /etc/kubernetes/encryption-config.yaml
apiVersion: apiserver.config.k8s.io/v1
kind: EncryptionConfiguration
resources:
  - resources:
      - secrets
    providers:
      - aescbc:
          keys:
            - name: key1
              secret: <base64-encoded-32-byte-key>
      - identity: {}    # Fallback (plaintext)
```

```bash
# Restart kube-apiserver with the encryption config
# (varies by installation method)
```

### External Secret Managers

For production, use external secret managers:

**HashiCorp Vault:**

```yaml
# Install External Secrets Operator
helm install external-secrets external-secrets/external-secrets

# Create a SecretStore
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: vault-backend
spec:
  provider:
    vault:
      server: "http://vault:8200"
      path: "secret"
      version: "v2"
      auth:
        kubernetes:
          mountPath: "kubernetes"
          role: "myapp"

---
# Create an ExternalSecret
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: app-secrets
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: app-secrets
  data:
    - secretKey: DB_PASSWORD
      remoteRef:
        key: secret/myapp
        property: db_password
    - secretKey: API_KEY
      remoteRef:
        key: secret/myapp
        property: api_key
```

**AWS Secrets Manager:**

```yaml
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: aws-backend
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-east-1
      auth:
        jwt:
          serviceAccountRef:
            name: external-secrets-sa
```

### Immutable ConfigMaps and Secrets

Prevent accidental changes:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
immutable: true     # Cannot be updated after creation
data:
  DB_HOST: "postgres-service"
  LOG_LEVEL: "info"
```

**Benefits:**
- Protects against accidental modifications
- Improves performance (kubelet doesn't watch for changes)
- Must be deleted and recreated to change

**Drawbacks:**
- Requires pod restart to pick up changes
- No rolling updates for config changes

### ConfigMap and Secret Updates

When a ConfigMap or Secret is updated:

**Mounted as volumes:** Files are updated automatically (may take up to 60 seconds due to kubelet sync period).

**Used as environment variables:** NOT updated. Pods must be restarted.

```bash
# Force a rolling restart when config changes
kubectl rollout restart deployment myapp

# Or use a hash annotation to trigger automatic restart
```

**Using hash annotation to trigger automatic restart:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  template:
    metadata:
      annotations:
        checksum/config: {{ include (print $.Template.BasePath "/configmap.yaml") . | sha256sum }}
    spec:
      containers:
        - name: app
          image: myapp:v1
          envFrom:
            - configMapRef:
                name: app-config
```

### SubPath Mounts

Mount a single file from a ConfigMap or Secret instead of the entire volume:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: subpath-pod
spec:
  containers:
    - name: app
      image: nginx:1.25
      volumeMounts:
        - name: config-volume
          mountPath: /etc/nginx/nginx.conf
          subPath: nginx.conf    # Mount only this file
  volumes:
    - name: config-volume
      configMap:
        name: app-config
```

**Note:** SubPath mounts are NOT updated automatically when the ConfigMap changes.

---

## Hands-On Lab

### Exercise 1: Create and Use a ConfigMap

```bash
# Create a ConfigMap
kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  APP_ENV: "production"
  LOG_LEVEL: "debug"
  DB_HOST: "postgres-service"
  DB_PORT: "5432"
  application.properties: |
    server.port=8080
    app.feature.newUI=true
    app.feature.darkMode=false
EOF

# View the ConfigMap
kubectl get configmap app-config -o yaml

# Use as environment variables
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: configmap-env-pod
spec:
  containers:
    - name: app
      image: busybox:1.36
      command: ['sh', '-c', 'echo "ENV: \$APP_ENV, LOG: \$LOG_LEVEL, DB: \$DB_HOST:\$DB_PORT"; sleep 3600']
      envFrom:
        - configMapRef:
            name: app-config
EOF

# Verify environment variables
kubectl exec configmap-env-pod -- env | grep -E "APP_ENV|LOG_LEVEL|DB_HOST|DB_PORT"
```

### Exercise 2: Mount ConfigMap as Volume

```bash
# Mount ConfigMap as files
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: configmap-volume-pod
spec:
  containers:
    - name: app
      image: busybox:1.36
      command: ['sh', '-c', 'cat /etc/config/application.properties; sleep 3600']
      volumeMounts:
        - name: config-volume
          mountPath: /etc/config
          readOnly: true
  volumes:
    - name: config-volume
      configMap:
        name: app-config
EOF

# Verify the mounted files
kubectl exec configmap-volume-pod -- ls -la /etc/config/
kubectl exec configmap-volume-pod -- cat /etc/config/application.properties
```

### Exercise 3: Create and Use a Secret

```bash
# Create a Secret
kubectl create secret generic app-secrets \
  --from-literal=DB_PASSWORD=supersecret123 \
  --from-literal=API_KEY=sk-abc123xyz \
  --from-literal=JWT_SECRET=my-jwt-secret-key

# View the Secret (base64 encoded)
kubectl get secret app-secrets -o yaml

# Decode a value
kubectl get secret app-secrets -o jsonpath='{.data.DB_PASSWORD}' | base64 -d

# Use as environment variables
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: secret-env-pod
spec:
  containers:
    - name: app
      image: busybox:1.36
      command: ['sh', '-c', 'echo "DB_PASS: \$DB_PASSWORD, API_KEY: \$API_KEY"; sleep 3600']
      envFrom:
        - secretRef:
            name: app-secrets
EOF

# Verify (note: env vars are visible in the pod)
kubectl exec secret-env-pod -- env | grep -E "DB_PASSWORD|API_KEY"
```

### Exercise 4: Mount Secret as Volume

```bash
# Mount Secret as files
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: secret-volume-pod
spec:
  containers:
    - name: app
      image: busybox:1.36
      command: ['sh', '-c', 'ls -la /etc/secrets/; cat /etc/secrets/DB_PASSWORD; sleep 3600']
      volumeMounts:
        - name: secret-volume
          mountPath: /etc/secrets
          readOnly: true
  volumes:
    - name: secret-volume
      secret:
        secretName: app-secrets
EOF

# Verify the mounted files
kubectl exec secret-volume-pod -- cat /etc/secrets/DB_PASSWORD
kubectl exec secret-volume-pod -- cat /etc/secrets/API_KEY
```

### Exercise 5: ConfigMap and Secret in a Real Application

```bash
# Create a full application with ConfigMap and Secret
kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: nginx-config
data:
  nginx.conf: |
    server {
      listen 80;
      server_name localhost;
      location / {
        root /usr/share/nginx/html;
        index index.html;
      }
      location /health {
        return 200 'OK';
        add_header Content-Type text/plain;
      }
    }

---
apiVersion: v1
kind: Secret
metadata:
  name: nginx-secrets
type: Opaque
stringData:
  htpasswd: |
    admin:\$apr1\$xyz\$hashedpasswordhere

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nginx-app
  template:
    metadata:
      labels:
        app: nginx-app
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
          volumeMounts:
            - name: config-volume
              mountPath: /etc/nginx/conf.d/default.conf
              subPath: nginx.conf
            - name: secret-volume
              mountPath: /etc/nginx/secrets
              readOnly: true
          livenessProbe:
            httpGet:
              path: /health
              port: 80
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 80
            initialDelaySeconds: 3
            periodSeconds: 5
      volumes:
        - name: config-volume
          configMap:
            name: nginx-config
        - name: secret-volume
          secret:
            secretName: nginx-secrets

---
apiVersion: v1
kind: Service
metadata:
  name: nginx-app-service
spec:
  selector:
    app: nginx-app
  ports:
    - port: 80
      targetPort: 80
EOF

# Test the application
kubectl port-forward service/nginx-app-service 8080:80 &
curl http://localhost:8080/
curl http://localhost:8080/health
```

### Exercise 6: Update ConfigMap and See the Effect

```bash
# Update the ConfigMap
kubectl edit configmap nginx-config
# Change the nginx.conf to add a new location

# Check if the mounted file updated (may take up to 60 seconds)
kubectl exec $(kubectl get pods -l app=nginx-app -o jsonpath='{.items[0].metadata.name}') -- cat /etc/nginx/conf.d/default.conf

# For env var changes, restart the pods
kubectl rollout restart deployment nginx-app

# Clean up
kubectl delete deployment nginx-app
kubectl delete service nginx-app-service
kubectl delete configmap nginx-config app-config
kubectl delete secret nginx-secrets app-secrets
kubectl delete pod configmap-env-pod configmap-volume-pod secret-env-pod secret-volume-pod
```

---

## Verification Checklist

- [ ] Understand the difference between ConfigMap and Secret
- [ ] Can create ConfigMaps and Secrets from files, literals, and env files
- [ ] Know how to consume them as environment variables and mounted volumes
- [ ] Understand that base64 encoding is NOT encryption
- [ ] Know that mounted volumes update automatically, env vars do not
- [ ] Can use SubPath to mount individual files
- [ ] Understand when to use immutable ConfigMaps/Secrets

---

## Limitation

You have learned how to configure applications with ConfigMaps and Secrets. But in a production cluster, you have multiple teams, multiple environments (dev, staging, prod), and multiple applications. How do you:

- Isolate resources between teams?
- Prevent one team from accessing another team's secrets?
- Limit resource consumption per team?
- Restrict who can create, read, or modify resources?

Without access control, anyone with `kubectl` access can do anything in the cluster.

**Next:** [Module 31: Namespaces & RBAC](../31-namespaces-and-rbac/README.md) — Multi-tenancy and access control.
