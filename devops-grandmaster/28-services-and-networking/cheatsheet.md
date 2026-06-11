# Cheatsheet: Services & Networking

## Service Types

| Type | Access | Use Case |
|------|--------|----------|
| ClusterIP | Internal only | Default, internal communication |
| NodePort | Node IP:Port | Development, testing |
| LoadBalancer | Cloud LB | Production external access |
| ExternalName | DNS CNAME | External service alias |

## Service Spec
```yaml
apiVersion: v1
kind: Service
metadata:
  name: my-service
spec:
  type: ClusterIP
  selector:
    app: my-app
  ports:
    - port: 80          # Service port
      targetPort: 8080   # Container port
```

## DNS Resolution
```
# Same namespace
my-service

# Different namespace
my-service.other-namespace

# Full FQDN
my-service.other-namespace.svc.cluster.local
```

## Common Commands
```bash
kubectl get services                  # List services
kubectl describe service my-service   # Details
kubectl get endpoints my-service      # See backend pods
kubectl port-forward svc/my-service 8080:80  # Port forward
```
