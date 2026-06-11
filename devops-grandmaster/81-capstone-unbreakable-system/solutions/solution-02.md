# Solution 02: Core Infrastructure with Kubernetes

## Part A: Namespace and Resource Quota

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: payment-system
  labels:
    app.kubernetes.io/part-of: payment-platform
```

```yaml
# resource-quota.yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: payment-quota
  namespace: payment-system
spec:
  hard:
    pods: "20"
    requests.cpu: "10"
    requests.memory: 20Gi
    limits.cpu: "10"
    limits.memory: 20Gi
```

Apply with:

```bash
kubectl apply -f namespace.yaml
kubectl apply -f resource-quota.yaml
```

The resource quota ensures that no team or runaway deployment can
consume the entire cluster's resources within this namespace. When a
pod is created that would exceed the quota, the API server rejects it
with a `Forbidden` error.

## Part B: Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-api
  namespace: payment-system
  labels:
    app: payment-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: payment-api
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  template:
    metadata:
      labels:
        app: payment-api
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
      containers:
        - name: payment-api
          image: payment-api:1.0.0
          ports:
            - containerPort: 8080
              name: http
              protocol: TCP
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: payment-db-credentials
                  key: url
          resources:
            requests:
              cpu: 250m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            httpGet:
              path: /healthz
              port: http
            initialDelaySeconds: 10
            periodSeconds: 3
            timeoutSeconds: 2
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /readyz
              port: http
            initialDelaySeconds: 5
            periodSeconds: 5
            timeoutSeconds: 2
            failureThreshold: 3
          securityContext:
            readOnlyRootFilesystem: true
            allowPrivilegeEscalation: false
            capabilities:
              drop:
                - ALL
          volumeMounts:
            - name: tmp
              mountPath: /tmp
      volumes:
        - name: tmp
          emptyDir: {}
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: topology.kubernetes.io/zone
          whenUnsatisfiable: DoNotSchedule
          labelSelector:
            matchLabels:
              app: payment-api
```

**Why this works:**

- `maxUnavailable: 0` means Kubernetes will not terminate any old pod
  until a new pod passes its readiness probe. This guarantees zero
  downtime during rolling updates.
- `maxSurge: 1` means Kubernetes creates one extra pod at a time during
  the rollout. This limits resource spikes during deployment.
- The liveness probe restarts the container if it becomes unresponsive.
  The readiness probe removes it from the Service endpoint list if it
  cannot handle requests (e.g., database connection pool not ready).
- `topologySpreadConstraints` distributes pods across availability zones.
  If one AZ goes down, at most 1 of 3 pods is affected.
- `readOnlyRootFilesystem` prevents an attacker who compromises the
  container from writing malware to disk. The `/tmp` emptyDir volume
  is needed because many applications write temporary files.

## Part C: Service and Ingress

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: payment-api
  namespace: payment-system
spec:
  type: ClusterIP
  selector:
    app: payment-api
  ports:
    - port: 80
      targetPort: http
      protocol: TCP
      name: http
```

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: payment-api
  namespace: payment-system
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/limit-rps: "100"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "5"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - api.payments.example.com
      secretName: payment-api-tls
  rules:
    - host: api.payments.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: payment-api
                port:
                  number: 80
```

**Why this works:**

- The Service is ClusterIP (not NodePort or LoadBalancer) because the
  Ingress controller handles external traffic. ClusterIP is the most
  secure -- it is only reachable within the cluster.
- `cert-manager.io/cluster-issuer` triggers cert-manager to automatically
  provision and renew a Let's Encrypt TLS certificate.
- `ssl-redirect: "true"` sends a 301 redirect from HTTP to HTTPS,
  ensuring no traffic is served over plaintext.
- `limit-rps: "100"` with `limit-burst-multiplier: "5"` allows bursts
  of up to 500 requests but averages 100 RPS per IP. This provides
  basic application-layer DDoS protection.

## Part D: Horizontal Pod Autoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: payment-api
  namespace: payment-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: payment-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Pods
          value: 4
          periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

**Why this works:**

- Scale-up allows adding up to 4 pods per minute. Combined with a
  60-second stabilization window, this responds to traffic spikes
  within 1-2 minutes.
- Scale-down removes at most 10% of pods per minute with a 300-second
  stabilization window. This prevents flapping -- the HPA will not
  scale down until the lower load has been sustained for 5 minutes.
- The asymmetry (fast scale-up, slow scale-down) is deliberate. Being
  under-provisioned causes user-facing errors. Being over-provisioned
  only costs money. Always err on the side of over-provisioning.

## Part E: Pod Disruption Budget and Network Policy

```yaml
# pdb.yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: payment-api
  namespace: payment-system
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: payment-api
```

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: payment-api
  namespace: payment-system
spec:
  podSelector:
    matchLabels:
      app: payment-api
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: payment-api
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ingress-nginx
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: database
      ports:
        - protocol: TCP
          port: 5432
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: UDP
          port: 53
        - protocol: TCP
          port: 53
```

**Why this works:**

- `minAvailable: 2` ensures that during a voluntary disruption (cluster
  upgrade, node drain), at least 2 of 3 pods remain running. Kubernetes
  will refuse to evict a pod if it would bring the count below 2.
- The Network Policy uses a whitelist model: only explicitly allowed
  traffic is permitted. Pods can receive traffic from the ingress
  controller and from other pods in the same namespace. They can only
  send traffic to the database (port 5432) and to DNS (port 53).
- The `namespaceSelector` with `kubernetes.io/metadata.name` is a
  built-in label that Kubernetes adds to every namespace. It is more
  reliable than custom labels.

## Common Mistakes

1. **Setting `maxUnavailable: 1` instead of `0`.** This causes one old
   pod to terminate before the new pod is ready, creating a brief window
   where capacity is reduced. Under load, this can cause request failures.
   Always use `maxUnavailable: 0` for zero-downtime deployments.

2. **Forgetting the readiness probe.** Without a readiness probe, the
   Service sends traffic to the pod immediately, before the application
   has initialized its database connection pool. This causes connection
   errors for the first few seconds of every deployment.

3. **Setting resource limits without requests.** Without requests, the
   scheduler does not know how much resource the pod needs. It may place
   too many pods on a single node, causing OOM kills. Always set both
   requests and limits.

4. **Using `memory` as an HPA metric without understanding OOM behavior.**
   When a pod approaches its memory limit, the kernel's OOM killer
   terminates it. The HPA does not react fast enough to prevent this.
   Set memory limits generously and use CPU as the primary scaling metric.

5. **Not applying the Network Policy.** By default, Kubernetes allows all
   pod-to-pod traffic. Without a Network Policy, a compromised pod can
   reach every other pod in the cluster. Apply restrictive policies to
   every namespace.
