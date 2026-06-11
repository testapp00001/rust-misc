# Solution 05: Expose Services with LoadBalancer and Ingress

## Part A: Deploy the Application

**web-app.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
        tier: frontend
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args:
            - >
              echo '<h1>Frontend</h1>' > /usr/share/nginx/html/index.html &&
              nginx -g 'daemon off;'
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: web-app-service
spec:
  type: ClusterIP
  selector:
    app: web-app
  ports:
    - port: 80
      targetPort: 80
```

**api-server.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  replicas: 2
  selector:
    matchLabels:
      app: api-server
  template:
    metadata:
      labels:
        app: api-server
        tier: backend
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args:
            - >
              echo '<h1>API Server</h1>' > /usr/share/nginx/html/index.html &&
              nginx -g 'daemon off;'
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: api-server-service
spec:
  type: ClusterIP
  selector:
    app: api-server
  ports:
    - port: 80
      targetPort: 80
```

**Verification:**

```bash
kubectl apply -f web-app.yaml
kubectl apply -f api-server.yaml

kubectl get deployments
kubectl get services
kubectl get pods -l tier=frontend
kubectl get pods -l tier=backend
```

Both deployments should have 2 running pods each. Both ClusterIP services should have endpoints listing 2 pod IPs.

## Part B: Expose with LoadBalancer Services

**loadbalancer-services.yaml:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-app-lb
spec:
  type: LoadBalancer
  selector:
    app: web-app
  ports:
    - port: 80
      targetPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: api-server-lb
spec:
  type: LoadBalancer
  selector:
    app: api-server
  ports:
    - port: 80
      targetPort: 80
```

Apply and check:

```bash
kubectl apply -f loadbalancer-services.yaml
kubectl get services
```

On a cloud provider, the EXTERNAL-IP column will show assigned IPs (or a hostname). On minikube, run `minikube tunnel` in a separate terminal first.

Test access:

```bash
curl http://<web-app-external-ip>:80
# Returns: <h1>Frontend</h1>

curl http://<api-server-external-ip>:80
# Returns: <h1>API Server</h1>
```

**Answers to the questions:**

1. **How many external IP addresses were provisioned?** Two. One for `web-app-lb` and one for `api-server-lb`. Each LoadBalancer Service provisions a separate cloud load balancer with its own external IP.

2. **What is the cost implication?** On AWS, each LoadBalancer Service creates an ELB (Elastic Load Balancer) that costs roughly $16-22/month plus data transfer charges. With 2 services, that is $32-44/month. In a real microservices architecture with 20-50 services, this approach costs hundreds of dollars per month just for load balancers. This is the primary motivation for Ingress.

3. **Can you route `/api` to the API server and `/` to the frontend using just LoadBalancer Services?** No. LoadBalancer Services operate at Layer 4 (TCP/UDP). They forward all traffic on a given port to the backend pods regardless of the URL path. A request to `/api` and a request to `/` are both just HTTP over TCP -- the LoadBalancer cannot distinguish them. For path-based routing, you need a Layer 7 component like an Ingress controller.

## Part C: Expose with Ingress (Single Load Balancer)

**Step 1: Install the Ingress controller:**

For minikube:

```bash
minikube addons enable ingress
```

For other clusters:

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml
```

Wait for readiness:

```bash
kubectl get pods -n ingress-nginx
```

The ingress-nginx controller pod should be in Running state.

**Step 2: Delete the LoadBalancer Services:**

```bash
kubectl delete -f loadbalancer-services.yaml
```

**Step 3: Create the Ingress resource:**

**app-ingress.yaml:**

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: app-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /$2
spec:
  ingressClassName: nginx
  rules:
    - host: app.local
      http:
        paths:
          - path: /api(/|$)(.*)
            pathType: ImplementationSpecific
            backend:
              service:
                name: api-server-service
                port:
                  number: 80
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-app-service
                port:
                  number: 80
```

Apply and verify:

```bash
kubectl apply -f app-ingress.yaml
kubectl get ingress
kubectl describe ingress app-ingress
```

Add the hostname to `/etc/hosts` (pointing to the ingress controller's IP):

```bash
# Get the ingress controller's external IP
kubectl get svc -n ingress-nginx
# Add to /etc/hosts
echo "<ingress-controller-ip> app.local" | sudo tee -a /etc/hosts
```

Test access:

```bash
curl http://app.local/
# Returns: <h1>Frontend</h1>

curl http://app.local/api
# Returns: <h1>API Server</h1>
```

The Ingress routes `/` to the frontend and `/api` to the API server, all through a single external IP.

## Part D: Compare and Reflect

| Aspect | LoadBalancer per Service | Ingress |
|--------|------------------------|---------|
| Number of external IPs | One per service (N services = N IPs) | One for the ingress controller (serves all services) |
| Path-based routing | No -- operates at Layer 4, cannot inspect URLs | Yes -- routes based on URL path (`/api` vs `/`) |
| Host-based routing | No -- all traffic on a port goes to one service | Yes -- routes based on hostname (`api.example.com` vs `web.example.com`) |
| TLS termination | No -- forwards raw TCP, cannot decrypt HTTPS | Yes -- terminates TLS at the ingress controller and forwards plain HTTP to backends |
| Cost (cloud) | High -- one cloud load balancer per service | Low -- one cloud load balancer for all services |
| Complexity | Low -- simple YAML, no additional components | Medium -- requires installing an ingress controller and configuring Ingress resources |
| Best for | Non-HTTP protocols (TCP, UDP, gRPC, databases), simple setups with few services | HTTP/HTTPS web applications, microservices with many services behind one IP |

**Answers to the questions:**

**1. When would you still use a LoadBalancer Service instead of Ingress?**

LoadBalancer Services are the right choice when:

- **The protocol is not HTTP.** Ingress only understands HTTP/HTTPS. For TCP services (databases, Redis, Kafka, gRPC without HTTP/2), you need a LoadBalancer Service or a TCP-aware ingress controller configuration.
- **You have very few services.** If you have only 1-2 external services, the simplicity of a LoadBalancer Service may outweigh the cost savings of Ingress.
- **You need non-standard ports.** Ingress typically exposes port 80 and 443. If you need to expose a service on a custom port, a LoadBalancer Service is more straightforward.
- **You need UDP.** Ingress controllers generally do not support UDP. For UDP services (DNS, gaming), use a LoadBalancer Service.

**2. What happens to the Ingress if the ingress controller pod crashes?**

All traffic through the Ingress stops. The ingress controller is the actual proxy that handles incoming requests and routes them to backend services. If the controller pod crashes:

- Existing connections are dropped.
- New connections fail (the ingress controller's LoadBalancer Service has no healthy backends).
- The Ingress resource itself still exists in the API server -- it is just a configuration object. But without a running controller to enforce it, it is inert.

Kubernetes will attempt to restart the controller pod (assuming its Deployment has `restartPolicy: Always`). Once the new pod is ready, traffic resumes. In production, you should run multiple replicas of the ingress controller and use a PodDisruptionBudget to minimize downtime.

**3. Can an Ingress route to Services in different namespaces?**

No. An Ingress resource is namespace-scoped -- it lives in a single namespace and can only route to Services in that same namespace. If you have a Service `api-server-service` in namespace `backend` and your Ingress is in namespace `default`, the Ingress cannot reference it. This is a common limitation. Workarounds include:

- Using an ExternalName Service to alias a cross-namespace service.
- Using an Ingress controller that supports cross-namespace routing (some controllers have annotations for this).
- Using a Gateway API (the successor to Ingress) which supports cross-namespace references.

**4. How would you add HTTPS/TLS to the Ingress configuration?**

Create a Kubernetes Secret containing the TLS certificate and private key, then reference it in the Ingress spec:

```bash
kubectl create secret tls app-tls --cert=cert.pem --key=key.pem
```

Then add a `tls` section to the Ingress:

```yaml
spec:
  tls:
    - hosts:
        - app.local
      secretName: app-tls
  rules:
    - host: app.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-app-service
                port:
                  number: 80
```

The ingress controller terminates TLS using the certificate from the Secret, decrypts the traffic, and forwards plain HTTP to the backend Services. For automatic certificate provisioning, integrate with cert-manager, which can request and renew certificates from Let's Encrypt automatically.

### Common Mistakes to Avoid

- **Creating one LoadBalancer per microservice.** This is the most expensive mistake. In a cloud environment with 30 microservices, you would pay for 30 load balancers. Use Ingress to consolidate behind a single external IP.
- **Forgetting that the ingress controller itself needs a LoadBalancer Service.** The ingress controller pod is typically exposed via a LoadBalancer Service. You still need one cloud load balancer -- but it serves all your Ingress resources.
- **Not waiting for the ingress controller to be ready.** If you create Ingress resources before the controller pod is running, the resources are accepted but nothing happens. Always verify `kubectl get pods -n ingress-nginx` shows Running pods before testing.
- **Confusing the Ingress resource with the ingress controller.** The Ingress resource is just a set of routing rules (a configuration object). The ingress controller is the actual software (nginx, Traefik, HAProxy) that reads those rules and handles traffic. You need both.
- **Not specifying `ingressClassName`.** If your cluster has multiple ingress controllers, omitting `ingressClassName` means no controller may claim the Ingress. Always specify it explicitly.
- **Path ordering matters.** Ingress rules are evaluated in order. A catch-all `/` path should be listed after more specific paths like `/api`. If `/` comes first, it may match everything before `/api` is evaluated, depending on the controller.

## Key Takeaway

LoadBalancer Services are simple but expensive -- one external IP per service, no HTTP-aware routing. Ingress consolidates multiple services behind a single load balancer with path-based and host-based routing, TLS termination, and lower cost. The tradeoff is added complexity: you must install and maintain an ingress controller. The standard production pattern is one ingress controller (exposed as a LoadBalancer Service) serving all your Ingress resources. For non-HTTP protocols (TCP, UDP, databases), LoadBalancer Services remain the right choice.
