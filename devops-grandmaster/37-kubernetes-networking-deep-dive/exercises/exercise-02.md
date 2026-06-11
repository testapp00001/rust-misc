# Exercise 02: Default-Deny NetworkPolicies with Selective Allow

**Type:** Guided
**Time:** 35 minutes
**Objective:** Implement a zero-trust network policy for a namespace by starting
with deny-all and selectively allowing only the traffic each service needs.

---

## Background

Your team runs a web application in the `webapp` namespace with three tiers:

- **frontend** -- Serves the user-facing UI on port 3000
- **api** -- Handles business logic on port 8080
- **database** -- PostgreSQL on port 5432

Currently, every Pod can talk to every other Pod. After a security audit, you
must implement zero-trust networking: deny all traffic by default, then
explicitly allow only the required flows.

The required traffic flows are:

1. External traffic (from the ingress controller namespace `ingress-nginx`)
   reaches `frontend` on port 3000.
2. `frontend` reaches `api` on port 8080.
3. `api` reaches `database` on port 5432.
4. All Pods can resolve DNS (CoreDNS is in `kube-system`).
5. No other traffic is allowed.

---

## Tasks

### Task 1: Create the Namespace and Deployments

Create the `webapp` namespace and deploy the three services.

```bash
kubectl create namespace webapp

# Deploy frontend
kubectl create deployment frontend --image=nginx -n webapp
kubectl expose deployment frontend --port=3000 --target-port=80 -n webapp

# Deploy api
kubectl create deployment api --image=nginx -n webapp
kubectl expose deployment api --port=8080 --target-port=80 -n webapp

# Deploy database
kubectl create deployment database --image=nginx -n webapp
kubectl expose deployment database --port=5432 --target-port=80 -n webapp

# Label the deployments
kubectl label deployment frontend app=frontend -n webapp
kubectl label deployment api app=api -n webapp
kubectl label deployment database app=database -n webapp

# Label Pods (deployments don't propagate labels to existing Pods on label change,
# so delete and let them recreate, or label existing Pods)
kubectl label pods -n webapp -l app=frontend app=frontend --overwrite
kubectl label pods -n webapp -l app=api app=api --overwrite
kubectl label pods -n webapp -l app=database app=database --overwrite
```

### Task 2: Verify Open Connectivity

Before applying policies, confirm that any Pod can reach any other Pod.

```bash
# From frontend, reach database directly (this should work -- it shouldn't!)
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432

# From database, reach frontend (also should work -- it shouldn't!)
kubectl exec -n webapp deploy/database -- curl -s --max-time 3 http://frontend.webapp.svc.cluster.local:3000
```

### Task 3: Apply Default-Deny

Create a NetworkPolicy that denies all ingress AND egress traffic in the
`webapp` namespace. Save it as `deny-all.yaml`.

Requirements:
- Select ALL Pods in the namespace (use `podSelector: {}`)
- Deny all ingress traffic
- Deny all egress traffic

Apply it and verify that connectivity is blocked:

```bash
kubectl apply -f deny-all.yaml

# These should all timeout now
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://api.webapp.svc.cluster.local:8080
kubectl exec -n webapp deploy/api -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432
```

### Task 4: Allow DNS for All Pods

Create a NetworkPolicy that allows all Pods in `webapp` to reach CoreDNS in
`kube-system`. Save it as `allow-dns.yaml`.

Requirements:
- Select ALL Pods in the namespace
- Allow egress to the `kube-system` namespace
- Allow UDP and TCP on port 53

Apply it:

```bash
kubectl apply -f allow-dns.yaml

# Verify DNS works (but HTTP still blocked)
kubectl exec -n webapp deploy/frontend -- nslookup api.webapp.svc.cluster.local
```

### Task 5: Allow Frontend Ingress from Ingress Controller

Create a NetworkPolicy that allows traffic from the `ingress-nginx` namespace
to `frontend` Pods. Save it as `allow-ingress-to-frontend.yaml`.

Requirements:
- Select only Pods with label `app=frontend`
- Allow ingress only from namespace `ingress-nginx`
- Allow only TCP port 3000

```bash
kubectl apply -f allow-ingress-to-frontend.yaml
```

### Task 6: Allow Frontend to API

Create a NetworkPolicy that allows `frontend` to reach `api`. Save it as
`allow-frontend-to-api.yaml`.

Requirements:
- Select Pods with label `app=api`
- Allow ingress only from Pods with label `app=frontend`
- Allow only TCP port 8080

```bash
kubectl apply -f allow-frontend-to-api.yaml

# Verify frontend can reach api
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://api.webapp.svc.cluster.local:8080
```

### Task 7: Allow API to Database

Create a NetworkPolicy that allows `api` to reach `database`. Save it as
`allow-api-to-database.yaml`.

Requirements:
- Select Pods with label `app=database`
- Allow ingress only from Pods with label `app=api`
- Allow only TCP port 5432

```bash
kubectl apply -f allow-api-to-database.yaml

# Verify api can reach database
kubectl exec -n webapp deploy/api -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432
```

### Task 8: Verify the Complete Policy Set

Run these tests. All must pass:

```bash
# 1. Frontend CANNOT reach database directly (should timeout)
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432

# 2. Database CANNOT reach frontend (should timeout)
kubectl exec -n webapp deploy/database -- curl -s --max-time 3 http://frontend.webapp.svc.cluster.local:3000

# 3. Database CANNOT reach api (should timeout)
kubectl exec -n webapp deploy/database -- curl -s --max-time 3 http://api.webapp.svc.cluster.local:8080

# 4. Frontend CAN reach api (should succeed)
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://api.webapp.svc.cluster.local:8080

# 5. API CAN reach database (should succeed)
kubectl exec -n webapp deploy/api -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432

# 6. DNS works for all Pods
kubectl exec -n webapp deploy/frontend -- nslookup kubernetes.default
kubectl exec -n webapp deploy/api -- nslookup kubernetes.default
kubectl exec -n webapp deploy/database -- nslookup kubernetes.default
```

---

## Success Criteria

- [ ] `deny-all.yaml` blocks ALL traffic (ingress and egress) in the namespace.
- [ ] `allow-dns.yaml` permits DNS resolution for all Pods without allowing
      any other egress.
- [ ] `allow-ingress-to-frontend.yaml` permits only ingress-nginx to reach
      frontend on port 3000.
- [ ] `allow-frontend-to-api.yaml` permits only frontend to reach api on port
      8080.
- [ ] `allow-api-to-database.yaml` permits only api to reach database on port
      5432.
- [ ] Direct frontend-to-database traffic is blocked.
- [ ] Database cannot initiate connections to any other service.
- [ ] All six verification tests from Task 8 pass.

---

## Hints

<details>
<summary>Hint 1: Default-Deny Pattern</summary>

The default-deny pattern uses `podSelector: {}` (selects all Pods) with empty
`ingress` and `egress` arrays in the `policyTypes`. When a Pod is selected by
a NetworkPolicy, all traffic not explicitly allowed by any policy selecting that
Pod is denied.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

The key: `podSelector: {}` with no rules means "select everything, allow
nothing."

</details>

<details>
<summary>Hint 2: DNS Egress is Critical</summary>

When you apply a default-deny egress policy, DNS resolution breaks immediately.
CoreDNS listens on UDP port 53 (and TCP port 53). You must allow egress to
kube-system on port 53 before anything else.

Without DNS, Pods cannot resolve Service names. Even if you allow API-to-database
traffic on port 5432, the API Pod cannot find the database IP without DNS.

Use `namespaceSelector` with `kubernetes.io/metadata.name: kube-system` to
target the kube-system namespace. This label is automatically set by Kubernetes
on every namespace.

</details>

<details>
<summary>Hint 3: Egress Policies for Source Pods</summary>

The policies in Tasks 5-7 control **ingress** on the destination Pods. But
remember that the default-deny in Task 3 also blocks **egress** from all Pods.

You need egress policies on the **source** Pods to allow them to send traffic:

- Frontend needs egress to api on port 8080.
- API needs egress to database on port 5432.

Without egress policies, the source Pods will have their outbound packets
dropped before they even reach the destination.

If your tests fail with "connection timed out" on the source side, you are
missing egress rules.

</details>

<details>
<summary>Hint 4: Label Propagation</summary>

`kubectl label deployment` does not automatically relabel existing Pods. The
Deployment's Pod template gets the label, but existing Pods keep their old
labels. New Pods created after the label change will get the new labels.

To label existing Pods immediately:

```bash
kubectl label pods -n webapp -l app=frontend tier=frontend
```

Or delete the Pods and let the Deployment recreate them:

```bash
kubectl delete pods -n webapp -l app=frontend
```

Check your labels with:

```bash
kubectl get pods -n webapp --show-labels
```

</details>

<details>
<summary>Hint 5: Testing NetworkPolicy Order</summary>

Apply policies in this order:

1. deny-all (blocks everything)
2. allow-dns (restores DNS)
3. allow-ingress-to-frontend (external access to frontend)
4. allow-frontend-to-api (frontend -> api)
5. allow-api-to-database (api -> database)
6. egress policies for source Pods (frontend egress, api egress)

If you apply allow rules before deny-all, they have no effect because there
is no policy selecting the Pods yet. The deny-all is what activates the
NetworkPolicy engine for the namespace.

</details>
