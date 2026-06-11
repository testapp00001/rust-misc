# Exercise 03: TLS Termination with cert-manager

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

Configure TLS termination on an Ingress resource using both manual certificate management and automated certificate provisioning with cert-manager. You will create TLS Secrets manually, then install cert-manager and configure it to automatically request and renew certificates from Let's Encrypt.

## Background

Production web applications must serve traffic over HTTPS. Without TLS, traffic between clients and your cluster is visible to anyone on the network. Kubernetes Ingress supports TLS termination: the Ingress controller decrypts HTTPS traffic and forwards plain HTTP to backend Pods. Managing certificates manually is error-prone and does not scale. cert-manager solves this by automating certificate issuance and renewal using the ACME protocol (Let's Encrypt).

## Tasks

### Part A: Manual TLS with a Self-Signed Certificate

Generate a self-signed certificate and configure TLS on an Ingress resource.

**Step 1:** Generate a self-signed certificate:

```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key -out tls.crt \
  -subj "/CN=myapp.local" \
  -addext "subjectAltName=DNS:myapp.local"
```

**Step 2:** Create a TLS Secret:

```bash
kubectl create secret tls myapp-tls --cert=tls.crt --key=tls.key
```

Verify the Secret:

```bash
kubectl get secret myapp-tls -o yaml
```

**Step 3:** Deploy a backend service (reuse from Exercise 02 or create a simple one):

```yaml
# Save as backend.yaml
```

Create a Deployment named `web` with 2 replicas using `nginx:1.25` with label `app: web`. Serve custom content:

```yaml
command: ["/bin/sh", "-c"]
args: ["echo '<h1>Secure Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
```

Create a ClusterIP Service named `web-svc` on port 80.

**Step 4:** Create an Ingress with TLS:

```yaml
# Save as tls-ingress.yaml
```

Create an Ingress with:
- Name: `tls-ingress`
- `ingressClassName: nginx`
- TLS section referencing `myapp-tls` for host `myapp.local`
- Rule: host `myapp.local`, path `/` to `web-svc:80`
- Annotation: `nginx.ingress.kubernetes.io/ssl-redirect: "true"`

Apply and verify:

```bash
kubectl apply -f tls-ingress.yaml
kubectl get ingress tls-ingress
kubectl describe ingress tls-ingress
```

**Step 5:** Test HTTPS access:

```bash
curl -k https://myapp.local/
```

The `-k` flag tells curl to accept the self-signed certificate. You should see the "Secure Frontend" response.

<details>
<summary>Hint -- TLS Secret Structure</summary>
A TLS Secret must have type `kubernetes.io/tls` and contain two keys: `tls.crt` (the certificate chain) and `tls.key` (the private key). The `kubectl create secret tls` command creates this structure automatically from your certificate files.
</details>

### Part B: Install cert-manager

Install cert-manager for automated certificate management.

**Step 1:** Install cert-manager:

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml
```

Wait for all cert-manager Pods to be ready:

```bash
kubectl get pods -n cert-manager -w
```

You should see three Pods: `cert-manager`, `cert-manager-cainjector`, and `cert-manager-webhook`, all in `Running` state.

**Step 2:** Verify the installation:

```bash
kubectl get crds | grep cert-manager
```

You should see CRDs like `certificates.cert-manager.io`, `clusterissuers.cert-manager.io`, and `issuers.cert-manager.io`.

<details>
<summary>Hint -- CRD Readiness</summary>
If the cert-manager Pods fail to start, it may be because the CRDs are not yet registered. Wait a minute and try again. On some clusters, you may need to apply the CRDs separately before installing cert-manager.
</details>

### Part C: Create a ClusterIssuer

Create a ClusterIssuer that will request certificates from Let's Encrypt.

```yaml
# Save as cluster-issuer.yaml
```

Create a ClusterIssuer with:
- Name: `letsencrypt-staging`
- ACME server: `https://acme-staging-v02.api.letsencrypt.org/directory`
- Email: your email address
- HTTP-01 solver using the `nginx` Ingress class

Apply:

```bash
kubectl apply -f cluster-issuer.yaml
kubectl get clusterissuer
kubectl describe clusterissuer letsencrypt-staging
```

The ClusterIssuer should show `Ready: True`.

<details>
<summary>Hint -- Staging vs Production</summary>
Use the staging server (`acme-staging-v02.api.letsencrypt.org`) for testing. It issues certificates that are not trusted by browsers but have no rate limits. Once everything works, switch to the production server (`acme-v02.api.letsencrypt.org`). The staging server prevents you from hitting Let's Encrypt's production rate limits during development.
</details>

### Part D: Automatic TLS with cert-manager

Create a new Ingress resource that uses cert-manager to automatically provision a TLS certificate.

**Step 1:** Delete the manual TLS Ingress from Part A:

```bash
kubectl delete ingress tls-ingress
```

**Step 2:** Create a new Ingress with the cert-manager annotation:

```yaml
# Save as auto-tls-ingress.yaml
```

Create an Ingress with:
- Name: `auto-tls-ingress`
- `ingressClassName: nginx`
- Annotation: `cert-manager.io/cluster-issuer: "letsencrypt-staging"`
- TLS section with `secretName: auto-tls-secret` (cert-manager will create this)
- Rule: host `myapp.local`, path `/` to `web-svc:80`

Apply:

```bash
kubectl apply -f auto-tls-ingress.yaml
```

**Step 3:** Watch cert-manager provision the certificate:

```bash
# Check the Certificate resource
kubectl get certificates
kubectl describe certificate auto-tls-secret

# Check the CertificateRequest
kubectl get certificaterequests

# Check the Order
kubectl get orders

# Check the Challenge (HTTP-01)
kubectl get challenges
```

Answer these questions:
1. What resources does cert-manager create in response to the annotation?
2. Why does the HTTP-01 challenge require the Ingress to be working?
3. How long does it take for the certificate to be issued?

<details>
<summary>Hint -- cert-manager Resource Chain</summary>
cert-manager creates a chain of resources: Certificate -> CertificateRequest -> Order -> Challenge. The HTTP-01 challenge works by creating a temporary Ingress rule that serves a token at `/.well-known/acme-challenge/<token>`. Let's Encrypt verifies ownership by requesting this token. This is why the Ingress must be accessible from the internet for the challenge to succeed.
</details>

### Part E: Verify and Reflect

If the certificate was issued successfully, test HTTPS:

```bash
curl -k https://myapp.local/
```

If the certificate was not issued (common with local clusters that are not publicly accessible), answer these questions:

1. Why does the HTTP-01 challenge fail on a local minikube cluster?
2. What alternative challenge types does cert-manager support?
3. How would you use cert-manager with a DNS-01 challenge?

<details>
<summary>Hint -- Local Cluster Limitations</summary>
The HTTP-01 challenge requires Let's Encrypt to reach your Ingress controller from the internet. On a local minikube cluster, the external IP is not publicly accessible. For local testing, you can use a self-signed certificate (Part A) or the DNS-01 challenge (which does not require inbound connectivity). For production, the cluster must have a publicly accessible IP and DNS records pointing to it.
</details>

## Success Criteria

- [ ] A self-signed TLS certificate is created and used in an Ingress resource
- [ ] `curl -k https://myapp.local/` returns the expected content
- [ ] cert-manager is installed and all Pods are running
- [ ] A ClusterIssuer is created and shows `Ready: True`
- [ ] An Ingress with the `cert-manager.io/cluster-issuer` annotation triggers certificate provisioning
- [ ] You understand the cert-manager resource chain: Certificate -> CertificateRequest -> Order -> Challenge
- [ ] You can explain why HTTP-01 challenges require public accessibility

## What You Should Understand After This Exercise

After completing this exercise, you should understand how TLS termination works with Kubernetes Ingress. The Ingress controller terminates TLS using certificates stored in Kubernetes Secrets. Manual certificate management works but does not scale. cert-manager automates the entire lifecycle: requesting certificates from Let's Encrypt, completing ACME challenges, storing certificates as Secrets, and renewing them before expiration. The annotation `cert-manager.io/cluster-issuer` on an Ingress resource triggers this automation. For local development, self-signed certificates are sufficient. For production, cert-manager with Let's Encrypt is the standard approach.
