# Solution 02: ConfigMaps as Files and Env Vars -- Hands-On

---

## Task 1: Create a ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  # Simple key-value pairs for env var consumption
  APP_ENV: "production"
  LOG_LEVEL: "info"
  DB_HOST: "postgres.default.svc"
  DB_PORT: "5432"

  # File-like entry for volume mount consumption
  application.properties: |
    server.port=8080
    app.feature.darkMode=true
    app.feature.newUI=false
    app.log.format=json
```

Apply it:

```bash
kubectl apply -f configmap.yaml
```

Verify:

```bash
kubectl get configmap app-config -o yaml
```

**Why this works:** The `data` field supports both simple string values and
multi-line file content. The key `application.properties` is treated as a
filename when the ConfigMap is mounted as a volume, and as an opaque key
when accessed via the API.

---

## Task 2: Consume as Environment Variables

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: configmap-env-pod
spec:
  containers:
    - name: app
      image: busybox:1.36
      command:
        - sh
        - -c
        - |
          echo "APP_ENV=$APP_ENV LOG_LEVEL=$LOG_LEVEL DB_HOST=$DB_HOST DB_PORT=$DB_PORT"
          sleep 3600
      envFrom:
        - configMapRef:
            name: app-config
```

Apply and verify:

```bash
kubectl apply -f pod-env.yaml
kubectl exec configmap-env-pod -- env | grep -E "APP_ENV|LOG_LEVEL|DB_HOST|DB_PORT"
```

Expected output:

```
APP_ENV=production
LOG_LEVEL=info
DB_HOST=postgres.default.svc
DB_PORT=5432
```

**Why this works:** `envFrom` with a `configMapRef` iterates over all keys in
the ConfigMap and creates an environment variable for each one. The key name
becomes the variable name, and the value becomes the variable value.

Note: the `application.properties` key also becomes an environment variable
with the entire file content as its value. This is usually not desirable for
multi-line values, which is why Task 3 uses a volume mount instead.

---

## Task 3: Consume as a Mounted Volume

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: configmap-volume-pod
spec:
  containers:
    - name: app
      image: busybox:1.36
      command:
        - sh
        - -c
        - |
          cat /etc/app/application.properties
          sleep 3600
      volumeMounts:
        - name: config-volume
          mountPath: /etc/app
          readOnly: true
  volumes:
    - name: config-volume
      configMap:
        name: app-config
```

Apply and verify:

```bash
kubectl apply -f pod-volume.yaml
kubectl exec configmap-volume-pod -- cat /etc/app/application.properties
```

Expected output:

```
server.port=8080
app.feature.darkMode=true
app.feature.newUI=false
app.log.format=json
```

You can also list all files:

```bash
kubectl exec configmap-volume-pod -- ls -la /etc/app/
```

This shows `APP_ENV`, `DB_HOST`, `DB_PORT`, `LOG_LEVEL`, and
`application.properties` as individual files.

**Why this works:** When a ConfigMap is mounted as a volume, each key in
`data` becomes a file in the mount directory. The filename is the key name,
and the file contents are the value. Keys with simple values become files
containing that string; keys with multi-line values become files with the
full content.

---

## Task 4: Verify Both Methods

### 1. Are the environment variables present in `configmap-volume-pod`?

**No.** The volume-mount Pod only has the ConfigMap mounted as files. It does
not use `envFrom` or `env`, so no environment variables are injected. The
Pod's environment contains only the default variables set by the container
runtime (PATH, HOME, etc.).

### 2. Is the `application.properties` file present in `configmap-env-pod`?

**No.** The env-var Pod only uses `envFrom` to inject environment variables.
It does not mount the ConfigMap as a volume, so no files appear at any mount
path. The configuration exists only as environment variables in the
container's process space.

**Key takeaway:** Each Pod explicitly declares how it consumes a ConfigMap.
Mounting as a volume and injecting as env vars are independent mechanisms.
Using both on the same ConfigMap is possible but creates duplication.

---

## Common Mistakes

1. **Forgetting `readOnly: true` on volume mounts.** ConfigMap volumes are
   read-only by default, but explicitly setting `readOnly: true` makes the
   intent clear and prevents confusion.

2. **Using `envFrom` with file-like ConfigMap entries.** If the ConfigMap
   contains multi-line values (like `application.properties`), `envFrom`
   creates an environment variable with the entire multi-line string as its
   value. This is rarely useful. Use `env` with `configMapKeyRef` to select
   specific keys, or mount as a volume.

3. **Expecting volume-mounted files to match directory structure.** ConfigMap
   keys are flat strings. A key named `subdir/config.properties` does NOT
   create a subdirectory. It creates a file literally named
   `subdir/config.properties`. Use `items` in the volume spec to control
   path mapping:

   ```yaml
   volumes:
     - name: config-volume
       configMap:
         name: app-config
         items:
           - key: application.properties
             path: app/application.properties
   ```

4. **Not verifying the ConfigMap before creating Pods.** Always run
   `kubectl get configmap <name> -o yaml` to confirm the data looks correct
   before referencing it in a Pod. Typos in key names cause silent failures.
