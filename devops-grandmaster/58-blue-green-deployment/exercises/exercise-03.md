# Exercise 03: Design a Kubernetes Blue-Green Deployment

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design and write the Kubernetes manifests for a blue-green deployment using
a Service selector to control traffic routing. This exercise trains you to
implement blue-green natively in Kubernetes without external tools.

## Scenario

You are deploying `order-service` to a Kubernetes cluster. The service has
these requirements:

- 3 replicas in each environment (blue and green)
- Health checks via `/health` endpoint on port 8080
- Resource limits: 100m CPU request, 500m CPU limit, 128Mi memory request,
  256Mi memory limit
- Traffic routing via a Kubernetes Service that selects pods by version label

## Tasks

### Part A: Write the Blue Deployment

Create the Deployment manifest for the blue environment. It should:
- Use the label `version: blue` on pods
- Run `order-service:v1`
- Include readiness and liveness probes
- Set resource requests and limits

<details>
<summary>Hint</summary>

The readiness probe determines when a pod is ready to receive traffic.
The liveness probe determines if a pod should be restarted. For blue-green,
the readiness probe is critical -- you do not want traffic routed to a pod
that is not ready.

```yaml
readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 15
  periodSeconds: 20
```

</details>

### Part B: Write the Green Deployment

Create the Deployment manifest for the green environment. It should be
identical to blue except for the version label and image tag.

<details>
<summary>Hint</summary>

The green deployment is a copy of the blue deployment with:
- `version: green` label instead of `version: blue`
- `order-service:v2` image instead of `order-service:v1`

Both deployments exist simultaneously in the cluster.

</details>

### Part C: Write the Service

Create a Kubernetes Service that routes traffic to the active environment.
Write two versions of the Service: one pointing to blue and one pointing
to green.

<details>
<summary>Hint</summary>

The Service uses a `selector` to match pods. To switch traffic, change the
selector from `version: blue` to `version: green`. Apply the change with
`kubectl apply -f service-green.yaml`.

```yaml
selector:
  app: order-service
  version: blue  # Change to "green" to switch
```

</details>

### Part D: Write the Deployment Script

Create a bash script `switch-traffic.sh` that:
1. Accepts an argument: `blue` or `green`
2. Patches the Service to route to the specified environment
3. Verifies that the target pods are ready before switching
4. Outputs which environment is now active

<details>
<summary>Hint</summary>

Use `kubectl patch` to update the Service selector:

```bash
kubectl patch service order-service -p '{"spec":{"selector":{"version":"'$TARGET_ENV'"}}}'
```

Before switching, verify pods are ready:

```bash
kubectl wait --for=condition=ready pod -l version=$TARGET_ENV -l app=order-service --timeout=120s
```

</details>

## Success Criteria

- [ ] Blue deployment manifest includes readiness and liveness probes
- [ ] Green deployment is identical except for version label and image
- [ ] Service selector controls which environment receives traffic
- [ ] Deployment script patches the Service to switch environments
- [ ] Deployment script verifies pod readiness before switching

## What You Should Understand After This Exercise

In Kubernetes, blue-green deployment is implemented by running two
Deployments simultaneously and using a Service selector to control which
one receives traffic. The switch is a simple label change on the Service.
This is simpler than the Docker Compose approach because Kubernetes handles
health checking and traffic routing natively.
