# Exercise 02: Implement Rolling Updates for a Kubernetes Deployment

**Type:** Guided
**Duration:** 45-60 minutes

## Objective

Configure and execute a rolling update strategy for a Kubernetes Deployment.
You will control the update velocity, verify zero-downtime behavior, observe
rollout status, and perform a manual rollback.

## Prerequisites

- A running Kubernetes cluster with `kubectl` configured
- `curl` or a similar HTTP client for testing

## Instructions

### Step 1 -- Create the Initial Deployment

Create a file named `deployment-v1.yaml` that defines:

- A `Deployment` named `web-app` in namespace `change-mgmt`
- 4 replicas
- Container image: `nginx:1.24`
- A label `app: web-app` and `version: v1`
- A readiness probe on port 80, path `/`, with `initialDelaySeconds: 5` and
  `periodSeconds: 5`
- A resource request of 64Mi memory and 50m CPU
- A resource limit of 128Mi memory and 100m CPU

Apply the deployment and a ClusterIP Service that targets pods with label
`app: web-app` on port 80.

### Step 2 -- Configure the Rolling Update Strategy

Modify the deployment YAML to include a rolling update strategy with:

- `maxUnavailable: 1`
- `maxSurge: 1`

Explain in writing what these two parameters control and why you chose these
values for a 4-replica deployment.

### Step 3 -- Perform the Update

1. Update the deployment image from `nginx:1.24` to `nginx:1.25` using
   `kubectl set image` or by editing the YAML.
2. In a separate terminal, run a continuous `curl` loop against the Service
   endpoint (from inside the cluster or via port-forward).
3. Record the output of `kubectl rollout status` during the update.
4. Verify that no `curl` requests received a connection error during the
   update.

### Step 4 -- Inspect the Rollout History

After the update completes:

1. Run `kubectl rollout history deployment/web-app -n change-mgmt`
2. Run `kubectl get rs -n change-mgmt` and identify the old and new
   ReplicaSets
3. Explain how Kubernetes tracks revision history

### Step 5 -- Perform a Rollback

1. Roll back to revision 1 using `kubectl rollout undo`
2. Verify the image has reverted to `nginx:1.24`
3. Check that the rollback was also zero-downtime

### Step 6 -- Experiment with Failure

1. Update the image to a non-existent tag: `nginx:does-not-exist`
2. Observe the rollout behavior with `kubectl rollout status`
3. Use `kubectl describe deployment` to see the conditions
4. Perform a rollback to restore the working version
5. Document what happened and how Kubernetes handled the failed image pull

## Success Criteria

- [ ] Deployment with 4 replicas, readiness probes, and resource limits is running
- [ ] Rolling update strategy with `maxUnavailable: 1` and `maxSurge: 1` is configured
- [ ] Update from nginx:1.24 to nginx:1.25 completes with zero downtime
- [ ] Rollout history shows at least 2 revisions
- [ ] Rollback to revision 1 completes successfully and image is nginx:1.24
- [ ] Failed update (bad image tag) is observed and documented
- [ ] Written explanation of `maxUnavailable` and `maxSurge` is provided

## Hints

<details>
<summary>Hint 1 -- Namespace Creation</summary>

Remember to create the namespace first:

```bash
kubectl create namespace change-mgmt
```

</details>

<details>
<summary>Hint 2 -- Continuous Curl Loop</summary>

Use port-forwarding to test from your local machine:

```bash
kubectl port-forward svc/web-app 8080:80 -n change-mgmt
```

Then in another terminal:

```bash
while true; do curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8080; sleep 0.5; done
```

</details>

<details>
<summary>Hint 3 -- Rollout Status</summary>

Watch the rollout in real time:

```bash
kubectl rollout status deployment/web-app -n change-mgmt --watch
```

</details>

<details>
<summary>Hint 4 -- Failed Image Behavior</summary>

When a pod fails to start, Kubernetes will keep the old pods running and retry
creating new pods. The deployment enters a degraded state but existing traffic
is not affected. Use `kubectl describe rs` on the new ReplicaSet to see the
events.

</details>

## Deliverables

1. `deployment-v1.yaml` -- initial deployment manifest
2. `deployment-v2.yaml` -- deployment with rolling update strategy and v2 image
3. `exercise-02-report.md` -- contains:
   - Your explanation of `maxUnavailable` and `maxSurge`
   - Screenshot or text output of `kubectl rollout status` during the v2 update
   - Rollout history output
   - Description of the failed update behavior
   - Evidence of zero-downtime during both update and rollback
