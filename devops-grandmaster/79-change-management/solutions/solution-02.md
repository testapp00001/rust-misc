# Solution 02: Rolling Updates for Kubernetes

## deployment-v1.yaml

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: change-mgmt
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  namespace: change-mgmt
  labels:
    app: web-app
    version: v1
spec:
  replicas: 4
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
        version: v1
    spec:
      containers:
        - name: web-app
          image: nginx:1.24
          ports:
            - containerPort: 80
          readinessProbe:
            httpGet:
              path: /
              port: 80
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            requests:
              memory: "64Mi"
              cpu: "50m"
            limits:
              memory: "128Mi"
              cpu: "100m"
---
apiVersion: v1
kind: Service
metadata:
  name: web-app
  namespace: change-mgmt
spec:
  selector:
    app: web-app
  ports:
    - port: 80
      targetPort: 80
  type: ClusterIP
```

Apply with:

```bash
kubectl apply -f deployment-v1.yaml
```

## deployment-v2.yaml (with rolling update strategy)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  namespace: change-mgmt
  labels:
    app: web-app
    version: v2
spec:
  replicas: 4
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
        version: v2
    spec:
      containers:
        - name: web-app
          image: nginx:1.25
          ports:
            - containerPort: 80
          readinessProbe:
            httpGet:
              path: /
              port: 80
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            requests:
              memory: "64Mi"
              cpu: "50m"
            limits:
              memory: "128Mi"
              cpu: "100m"
```

## Explanation of maxUnavailable and maxSurge

**`maxUnavailable: 1`** controls the maximum number of pods that can be
unavailable during the update. With 4 replicas and maxUnavailable=1, at most
1 pod can be down at any time, meaning at least 3 pods remain serving traffic
throughout the rollout. This ensures capacity is maintained.

**`maxSurge: 1`** controls the maximum number of pods above the desired count
that can exist during the update. With maxSurge=1, Kubernetes can create at
most 1 extra pod (5 total) while the rollout is in progress. This means it
creates one new pod, waits for it to be ready, then terminates one old pod,
then repeats.

**Why these values for a 4-replica deployment:**
- maxUnavailable=1 ensures 75% capacity is always available (3 of 4 pods)
- maxSurge=1 keeps resource usage bounded (max 5 pods instead of 4)
- Together they produce a sequential, one-at-a-time rolling update: the safest
  option for a moderate-sized deployment
- For more aggressive updates, you could increase both values (e.g.,
  maxUnavailable=1, maxSurge=2 would update 2 pods at a time)

## Rollout Status Output

```bash
$ kubectl rollout status deployment/web-app -n change-mgmt
Waiting for deployment "web-app" rollout to finish: 1 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 1 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 1 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 2 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 2 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 3 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 3 out of 4 new replicas have been updated...
Waiting for deployment "web-app" rollout to finish: 4 out of 4 new replicas have been updated...
deployment "web-app" successfully rolled out
```

## Rollout History

```bash
$ kubectl rollout history deployment/web-app -n change-mgmt
deployment.apps/web-app
REVISION  CHANGE-CAUSE
1         <none>
2         <none>
```

```bash
$ kubectl get rs -n change-mgmt
NAME                  DESIRED   CURRENT   READY   AGE
web-app-5f8b9c7d4    0         0         0       10m    # old ReplicaSet (v1)
web-app-7d6f8e9a1    4         4         4       5m     # new ReplicaSet (v2)
```

**How Kubernetes tracks revision history:**
Kubernetes stores the pod template of each deployment revision in the
ReplicaSet objects. Each time the pod template changes (image, labels,
environment variables, etc.), a new ReplicaSet is created. The `rollout
history` command lists these revisions. By default, the last 10 ReplicaSets
are kept (controlled by `spec.revisionHistoryLimit`).

## Rollback

```bash
$ kubectl rollout undo deployment/web-app -n change-mgmt
deployment.apps/web-app rolled back

$ kubectl rollout status deployment/web-app -n change-mgmt
deployment "web-app" successfully rolled out

$ kubectl get deployment web-app -n change-mgmt -o jsonpath='{.spec.template.spec.containers[0].image}'
nginx:1.24
```

The rollback is also zero-downtime because it uses the same rolling update
strategy -- Kubernetes replaces the v2 pods with v1 pods one at a time.

## Failed Update Behavior

When deploying `nginx:does-not-exist`:

```bash
$ kubectl set image deployment/web-app web-app=nginx:does-not-exist -n change-mgmt
deployment.apps/web-app image updated

$ kubectl rollout status deployment/web-app -n change-mgmt --timeout=60s
error: timed out waiting for the condition
```

```bash
$ kubectl get pods -n change-mgmt
NAME                       READY   STATUS             RESTARTS   AGE
web-app-5f8b9c7d4-abc12   1/1     Running            0          15m    # old (v1) still running
web-app-5f8b9c7d4-def34   1/1     Running            0          15m    # old (v1) still running
web-app-5f8b9c7d4-ghi56   1/1     Running            0          15m    # old (v1) still running
web-app-5f8b9c7d4-jkl78   1/1     Running            0          15m    # old (v1) still running
web-app-xxx99-bad01        0/1     ImagePullBackOff   0          2m     # new (v2) failing
```

```bash
$ kubectl describe deployment web-app -n change-mgmt
...
Conditions:
  Type           Status  Reason
  ----           ------  ------
  Available      True    MinimumReplicasAvailable
  Progressing    False   ProgressDeadlineExceeded
...
```

**What happened:**
- Kubernetes attempted to create new pods with the non-existent image
- The new pods entered `ImagePullBackOff` state (the image does not exist
  in the registry)
- The old v1 pods remained `Running` and continued serving traffic
- Kubernetes did NOT terminate the old pods because the new pods never became
  `Ready`
- The rollout stalled with `Progressing=False` and
  `Reason=ProgressDeadlineExceeded`
- Existing traffic was **not affected** -- zero downtime even during this
  failure

**Recovery:**

```bash
$ kubectl rollout undo deployment/web-app -n change-mgmt
deployment.apps/web-app rolled back
```

This restored the working nginx:1.24 image.

## Zero-Downtime Evidence

Throughout both the successful update (v1 to v2) and the rollback (v2 to v1),
the continuous curl loop showed HTTP 200 responses with no connection errors:

```
200
200
200
200
...  (continuous 200s throughout)
```

No requests were dropped during pod termination or creation because:
1. The readiness probe ensures new pods are not added to the Service until
   they are ready
2. Old pods are not terminated until new pods are confirmed healthy
3. `maxUnavailable: 1` ensures at least 3 pods are always available
4. The Service load balancer only routes to Ready pods

## Key Learnings

1. **Rolling updates are the default Kubernetes strategy** and provide
   zero-downtime for well-configured deployments.
2. **Readiness probes are critical** -- without them, Kubernetes may route
   traffic to pods that are not ready.
3. **Failed image pulls do not affect existing traffic** -- Kubernetes keeps
   old pods running when new pods cannot start.
4. **`kubectl rollout undo` is a fast, safe rollback** mechanism that uses the
   same rolling update strategy.
5. **`maxUnavailable` and `maxSurge` work together** to control the speed
   and safety of the update. Conservative values (1/1) are safest; aggressive
   values are faster but riskier.
