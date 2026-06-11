# Exercise 02: Core Infrastructure with Kubernetes

**Type:** Guided
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Implement the Kubernetes infrastructure for an unbreakable system. You
will create production-grade manifests that provide zero-downtime
deployments, automatic scaling, self-healing, and disruption tolerance.
This is the foundation that everything else builds on.

## Scenario

You are deploying a payment processing API called `payment-api` to a
Kubernetes cluster. The API:

- Serves REST requests on port 8080
- Has a `/healthz` endpoint for liveness checks (returns 200 if the
  process can handle requests)
- Has a `/readyz` endpoint for readiness checks (returns 200 only when
  the database connection pool is initialized and warm)
- Needs to connect to a PostgreSQL database via a `DATABASE_URL`
  environment variable
- Requires a TLS certificate for HTTPS

You must create a namespace, deployment, service, ingress, horizontal
pod autoscaler, pod disruption budget, network policy, and resource
quota -- all production-grade.

## Tasks

### Part A: Create the Namespace and Resource Quota

Create a namespace called `payment-system` with a resource quota that
limits:

- Maximum 20 pods
- Maximum 10 CPU cores
- Maximum 20Gi memory

Write the YAML manifests.

<details>
<summary>Hint</summary>

Use `ResourceQuota` in the namespace. Apply it with `kubectl apply -f`.
The quota applies to all pods in the namespace, so new pods will be
rejected if they would exceed the limits.

</details>

### Part B: Create the Deployment

Write a Deployment manifest for `payment-api` that:

1. Runs 3 replicas
2. Uses a rolling update strategy with `maxUnavailable: 0` and
   `maxSurge: 1` (zero downtime)
3. Sets resource requests (250m CPU, 256Mi memory) and limits (500m
   CPU, 512Mi memory)
4. Configures a liveness probe on `/healthz` with a 10-second initial
   delay and 3-second period
5. Configures a readiness probe on `/readyz` with a 5-second initial
   delay and 5-second period
6. Injects `DATABASE_URL` from a Secret called `payment-db-credentials`
7. Runs as a non-root user (UID 1000) with a read-only root filesystem
8. Uses a `topologySpreadConstraints` to distribute pods across
   availability zones

<details>
<summary>Hint</summary>

For zero-downtime rolling updates, `maxUnavailable: 0` ensures old pods
are not terminated until new pods are ready. The readiness probe is
critical -- Kubernetes will not send traffic to a pod until it passes.
Use `securityContext` for the non-root and read-only filesystem settings.

</details>

### Part C: Create the Service and Ingress

Write a ClusterIP Service that routes traffic to the deployment pods on
port 8080. Then write an Ingress manifest that:

1. Routes `api.payments.example.com` to the service
2. Terminates TLS with a certificate from `cert-manager`
3. Enables HTTPS redirect
4. Sets rate limiting to 100 requests per second per IP

<details>
<summary>Hint</summary>

The Service selects pods by label (`app: payment-api`). The Ingress
uses an annotation for cert-manager (`cert-manager.io/cluster-issuer`)
and nginx-ingress annotations for rate limiting
(`nginx.ingress.kubernetes.io/limit-rps`).

</details>

### Part D: Create the Horizontal Pod Autoscaler

Write an HPA manifest that:

1. Scales the deployment between 3 and 20 replicas
2. Targets 70% average CPU utilization
3. Targets 80% average memory utilization
4. Scales up quickly (60-second stabilization window) but scales down
   slowly (300-second stabilization window)

<details>
<summary>Hint</summary>

Use `behavior` in the HPA spec to control scale-up and scale-down
rates differently. The stabilization window prevents flapping -- rapid
scale-up followed by immediate scale-down.

</details>

### Part E: Create the Pod Disruption Budget and Network Policy

Write a PDB that ensures at least 2 pods are always available during
voluntary disruptions (node drains, upgrades). Then write a Network
Policy that:

1. Allows ingress traffic only from pods in the same namespace and
   from the ingress controller namespace
2. Allows egress only to the database (on port 5432) and to DNS
   (port 53)
3. Denies all other traffic by default

<details>
<summary>Hint</summary>

The PDB uses `minAvailable: 2` (or `maxUnavailable: 1`). The Network
Policy uses `policyTypes: [Ingress, Egress]` with specific `from` and
`to` rules. Without a Network Policy, all traffic is allowed by default
in Kubernetes.

</details>

## Success Criteria

- [ ] All manifests are valid YAML and pass `kubectl apply --dry-run=client`
- [ ] The Deployment uses `maxUnavailable: 0` for zero-downtime updates
- [ ] The HPA scales up faster than it scales down
- [ ] The PDB allows at most 1 pod to be disrupted at a time
- [ ] The Network Policy restricts ingress and egress to only necessary sources/destinations
- [ ] The pod runs as non-root with a read-only filesystem
- [ ] Resource requests and limits are set (no "unbounded" pods)

## What You Should Understand After This Exercise

Production Kubernetes is not just `kubectl create deployment`. Every
manifest you wrote serves a specific reliability purpose: the rolling
update strategy prevents downtime during deploys, the HPA handles
traffic spikes, the PDB protects against node drains, the Network
Policy limits blast radius, and the security context reduces the
impact of a container compromise. Missing any one of these creates a
gap that will eventually cause an outage.
