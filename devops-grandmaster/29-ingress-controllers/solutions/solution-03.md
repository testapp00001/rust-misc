# Solution 03: TLS Termination with cert-manager

---

## Part A: Manual TLS with a Self-Signed Certificate

### Generate the certificate:

```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key -out tls.crt \
  -subj "/CN=myapp.local" \
  -addext "subjectAltName=DNS:myapp.local"
```

### Create the TLS Secret:

```bash
kubectl create secret tls myapp-tls --cert=tls.crt --key=tls.key
```

**Secret structure:**

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: myapp-tls
type: kubernetes.io/tls
data:
  tls.crt: <base64-encoded-certificate>
  tls.key: <base64-encoded-private-key>
```

### Backend Deployment:

```yaml
# backend.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
spec:
  replicas: 2
  selector:
    matchLabels:
      app: web
  template:
    metadata:
      labels:
        app: web
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args: ["echo '<h1>Secure Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: web-svc
spec:
  selector:
    app: web
  ports:
    - port: 80
      targetPort: 80
```

### Ingress with TLS:

```yaml
# tls-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: tls-ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - myapp.local
      secretName: myapp-tls
  rules:
    - host: myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-svc
                port:
                  number: 80
```

**Why this works:** The `tls` section tells the Ingress controller to terminate TLS for host `myapp.local` using the certificate stored in Secret `myapp-tls`. The `ssl-redirect` annotation automatically redirects HTTP requests to HTTPS. The controller decrypts HTTPS traffic and forwards plain HTTP to `web-svc`.

### Test:

```bash
curl -k https://myapp.local/
# Returns: <h1>Secure Frontend</h1>
```

The `-k` flag accepts the self-signed certificate. In production, you would use a certificate from a trusted CA.

---

## Part B: Install cert-manager

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml
```

**Verify:**

```bash
kubectl get pods -n cert-manager
```

Expected: three Pods running: `cert-manager`, `cert-manager-cainjector`, `cert-manager-webhook`.

```bash
kubectl get crds | grep cert-manager
```

Expected CRDs:
- `certificates.cert-manager.io`
- `certificaterequests.cert-manager.io`
- `challenges.acme.cert-manager.io`
- `clusterissuers.cert-manager.io`
- `issuers.cert-manager.io`
- `orders.acme.cert-manager.io`

---

## Part C: Create a ClusterIssuer

```yaml
# cluster-issuer.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-staging
spec:
  acme:
    server: https://acme-staging-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-staging-key
    solvers:
      - http01:
          ingress:
            class: nginx
```

**Verify:**

```bash
kubectl get clusterissuer letsencrypt-staging
kubectl describe clusterissuer letsencrypt-staging
```

The status should show `Ready: True`.

**Why staging first:** The Let's Encrypt production server has strict rate limits (50 certificates per registered domain per week). The staging server has no rate limits but issues untrusted certificates. Always test with staging first, then switch to production.

---

## Part D: Automatic TLS with cert-manager

```yaml
# auto-tls-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: auto-tls-ingress
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-staging"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - myapp.local
      secretName: auto-tls-secret
  rules:
    - host: myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-svc
                port:
                  number: 80
```

**Resource chain created by cert-manager:**

```bash
kubectl get certificates
kubectl get certificaterequests
kubectl get orders
kubectl get challenges
```

1. **Certificate** -- cert-manager detects the annotation and creates a Certificate resource in the same namespace.
2. **CertificateRequest** -- cert-manager creates a CertificateRequest to track the ACME order.
3. **Order** -- cert-manager creates an Order with the ACME server (Let's Encrypt).
4. **Challenge** -- Let's Encrypt responds with an HTTP-01 challenge. cert-manager creates a temporary Ingress rule to serve the challenge token at `/.well-known/acme-challenge/<token>`.

**Why HTTP-01 requires public accessibility:** The HTTP-01 challenge works by Let's Encrypt making an HTTP request to your Ingress controller. If the controller is not reachable from the internet (e.g., on a local minikube cluster), the challenge fails.

---

## Part E: Verify and Reflect

**If the certificate was issued:**

```bash
kubectl get certificate auto-tls-secret
# Should show READY = True

curl -k https://myapp.local/
# Should work with the staging certificate
```

**If the certificate was not issued (local cluster):**

**1. Why does HTTP-01 fail on a local minikube cluster?**

The HTTP-01 challenge requires Let's Encrypt to connect to your Ingress controller from the internet. A minikube cluster runs locally and its external IP (typically 192.168.x.x or the minikube tunnel IP) is not reachable from the internet. Let's Encrypt cannot verify domain ownership.

**2. What alternative challenge types does cert-manager support?**

- **DNS-01:** Proves domain ownership by creating a DNS TXT record. Does not require inbound connectivity. Works with local clusters if you control the DNS zone. Supported DNS providers include Route53, Cloudflare, Google Cloud DNS, and others.
- **HTTP-01:** Requires inbound HTTP connectivity (the default).
- **TLS-ALPN-01:** Uses a TLS handshake on port 443. Less common.

**3. How would you use cert-manager with a DNS-01 challenge?**

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-dns
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-dns-key
    solvers:
      - dns01:
          cloudflare:
            email: your-email@example.com
            apiTokenSecretRef:
              name: cloudflare-api-token
              key: api-token
```

The DNS-01 solver creates a TXT record like `_acme-challenge.myapp.local` in your DNS zone. Let's Encrypt checks for this record to verify ownership. This works even if the cluster is not publicly accessible.

---

## Common Mistakes

1. **Using the production Let's Encrypt server for testing.** The production server has rate limits. Hit them during testing and you will be locked out for a week. Always start with the staging server.

2. **Forgetting the `tls` section in the Ingress.** The annotation `cert-manager.io/cluster-issuer` triggers certificate provisioning, but the `tls` section tells the Ingress controller to actually use the certificate. Both are required.

3. **Expecting cert-manager to work on a local cluster without DNS.** The HTTP-01 challenge requires public accessibility. For local development, use self-signed certificates or set up a DNS-01 solver with a provider like nip.io or sslip.io.

4. **Not waiting for the Certificate to be ready.** The certificate provisioning process takes time (30 seconds to several minutes). Check `kubectl get certificate` until `READY` is `True` before testing.

5. **Confusing ClusterIssuer with Issuer.** A ClusterIssuer is cluster-scoped and can be referenced from any namespace. An Issuer is namespace-scoped and can only be referenced from the same namespace. Use ClusterIssuer for shared infrastructure.
