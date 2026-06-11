# Cheatsheet: Ingress Controllers

## Ingress Spec
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: my-ingress
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - myapp.example.com
      secretName: myapp-tls
  rules:
    - host: myapp.example.com
      http:
        paths:
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api-service
                port:
                  number: 80
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-service
                port:
                  number: 80
```

## Ingress Controllers

| Controller | Best For |
|------------|----------|
| Nginx Ingress | General purpose, widely used |
| Traefik | Auto-discovery, Let's Encrypt |
| ALB Ingress | AWS-native |
| Istio Gateway | Service mesh integration |

## Common Commands
```bash
kubectl get ingress                    # List ingresses
kubectl describe ingress my-ingress    # Details
kubectl get ingressclass               # Available controllers
```
