# Cheatsheet: ConfigMaps & Secrets

## ConfigMap
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  APP_ENV: "production"
  LOG_LEVEL: "info"
  config.json: |
    {"key": "value"}
```

## Secret
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
data:
  PASSWORD: cGFzc3dvcmQ=    # echo -n 'password' | base64
```

## Using in Pods
```yaml
# As environment variables
env:
  - name: APP_ENV
    valueFrom:
      configMapKeyRef:
        name: app-config
        key: APP_ENV
  - name: PASSWORD
    valueFrom:
      secretKeyRef:
        name: app-secrets
        key: PASSWORD

# All keys from ConfigMap
envFrom:
  - configMapRef:
      name: app-config

# As mounted files
volumeMounts:
  - name: config-volume
    mountPath: /etc/config
volumes:
  - name: config-volume
    configMap:
      name: app-config
```

## Common Commands
```bash
# ConfigMap
kubectl create configmap app-config --from-literal=KEY=VALUE
kubectl create configmap app-config --from-file=config.yaml
kubectl get configmaps
kubectl describe configmap app-config

# Secret
kubectl create secret generic app-secrets --from-literal=KEY=VALUE
kubectl get secrets
kubectl describe secret app-secrets
kubectl get secret app-secrets -o jsonpath='{.data.KEY}' | base64 -d
```
