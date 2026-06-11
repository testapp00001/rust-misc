# Cheatsheet: Service Mesh

## What is Service Mesh
```
Without mesh:  App A → App B
With mesh:     App A → [Sidecar] → [Sidecar] → App B
```

## Istio Architecture
```
┌─────────────────────────────────────────┐
│              Control Plane               │
│  ┌──────────┬──────────┬──────────┐     │
│  │ istiod   │ Citadel  │ Pilot    │     │
│  └──────────┴──────────┴──────────┘     │
└─────────────────────────────────────────┘
                    │
┌───────────────────┼───────────────────┐
│                   │                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐
│  │ Pod A    │  │ Pod B    │  │ Pod C    │
│  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │
│  │ │ App  │ │  │ │ App  │ │  │ │ App  │ │
│  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │
│  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │
│  │ │Envoy│ │  │ │Envoy│ │  │ │Envoy│ │
│  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │
│  └──────────┘  └──────────┘  └──────────┘
└─────────────────────────────────────────┘
```

## Install Istio
```bash
# Download istioctl
curl -L https://istio.io/downloadIstio | sh -
export PATH=$PWD/bin:$PATH

# Install
istioctl install --set profile=demo -y

# Enable sidecar injection
kubectl label namespace default istio-injection=enabled
```

## Traffic Management
```yaml
# VirtualService
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: my-app
spec:
  hosts:
    - my-app
  http:
    - route:
        - destination:
            host: my-app
            subset: v1
          weight: 90
        - destination:
            host: my-app
            subset: v2
          weight: 10
```

## mTLS
```yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
spec:
  mtls:
    mode: STRICT
```
