# Exercise 02: Inspect the Control Plane

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Use kubectl to inspect the control plane components of a running cluster,
understand what each one does by examining its configuration, and verify
that all components are healthy.

## Scenario

You have just inherited a Kubernetes cluster from another team. You need
to understand what is running, verify the cluster is healthy, and document
the control plane configuration. You have `kubectl` access and nothing else.

## Prerequisites

A running Kubernetes cluster. Any of these will work:

```bash
# kind (recommended for this exercise)
kind create cluster --name lab-cluster

# minikube
minikube start

# k3s
curl -sfL https://get.k3s.io | sh -
```

Verify your cluster is ready:

```bash
kubectl cluster-info
kubectl get nodes
```

## Tasks

### Part A: Discover the Control Plane Pods

List all pods in the `kube-system` namespace. This is where the control
plane components run.

```bash
kubectl get pods -n kube-system
```

Answer these questions:

1. How many pods are running in `kube-system`?
2. Which pods are the four control plane components?
3. What other pods do you see, and what are they for?
4. Are all pods in the `Running` state? If not, which ones are not?

<details>
<summary>Hint</summary>

The four control plane pods have names that start with:
- `kube-apiserver`
- `kube-scheduler`
- `kube-controller-manager`
- `etcd`

Other pods you might see include CoreDNS (DNS), kube-proxy (networking),
and a CNI plugin (kindnet, calico, flannel, or cilium).

</details>

### Part B: Examine the API Server

The API server is the front door to the cluster. Inspect it:

```bash
kubectl describe pod kube-apiserver -n kube-system
```

Find and record:

1. What image is it running? (Look at the `Image:` field)
2. What port does it listen on? (Look at the `--secure-port` flag)
3. What authentication flags are configured?
4. What is the `--etcd-servers` flag set to?
5. How much CPU and memory is it requesting?

<details>
<summary>Hint</summary>

The `describe` output has a `Containers` section with `Args:` that lists
all command-line flags. Look for flags like:
- `--secure-port=6443`
- `--etcd-servers=https://127.0.0.1:2379`
- `--authorization-mode`
- `--client-ca-file`

Resource requests are in the `Limits` and `Requests` section.

</details>

### Part C: Examine etcd

etcd stores all cluster state. Inspect it:

```bash
kubectl describe pod etcd -n kube-system
```

Find and record:

1. What image is it running?
2. Where does it store data on disk? (Look for `--data-dir`)
3. What is the `--listen-client-urls` flag?
4. What ports does it expose?

<details>
<summary>Hint</summary>

etcd typically stores data in `/var/lib/etcd`. The client URL is usually
`https://127.0.0.1:2379`. Only the API server should talk to etcd --
look for flags that restrict access.

</details>

### Part D: Examine the Scheduler and Controller Manager

Inspect both components:

```bash
kubectl describe pod kube-scheduler -n kube-system
kubectl describe pod kube-controller-manager -n kube-system
```

For each, find:

1. What image is it running?
2. What is the `--leader-elect` flag, and why is it there?
3. What is the `--kubeconfig` flag pointing to?

<details>
<summary>Hint</summary>

`--leader-elect=true` means only one instance is active at a time, even
if multiple replicas exist. This is important for high availability --
we will cover this in Exercise 05.

The `--kubeconfig` flag tells the component how to authenticate to the
API server. It is usually `/etc/kubernetes/scheduler.conf` or
`/etc/kubernetes/controller-manager.conf`.

</details>

### Part E: Check Component Health

Use these commands to verify each component is healthy:

```bash
# API server health
kubectl get --raw='/readyz?verbose'

# Component status (may show "deprecated" in newer versions)
kubectl get componentstatuses

# Check if scheduler and controller manager are responding
kubectl get endpoints kube-scheduler -n kube-system
kubectl get endpoints kube-controller-manager -n kube-system
```

Record the output. Is everything healthy?

<details>
<summary>Hint</summary>

`kubectl get componentstatuses` (or `kubectl get cs`) may show
"Deprecated" warnings in Kubernetes 1.19+. This is normal. The
more reliable way to check health is the `/readyz` endpoint and
examining pod status.

If any component shows `Unhealthy`, check its logs:
```bash
kubectl logs kube-scheduler -n kube-system --tail=20
```

</details>

### Part F: Summary Document

Create a table summarizing everything you found:

| Component | Image | Port | Key Flag | Status |
|-----------|-------|------|----------|--------|
| API Server | ? | ? | ? | ? |
| etcd | ? | ? | ? | ? |
| Scheduler | ? | ? | ? | ? |
| Controller Manager | ? | ? | ? | ? |

## Success Criteria

- [ ] You can list all pods in `kube-system` and identify the control plane components
- [ ] You recorded the image, port, and one key flag for each component
- [ ] You understand what `--leader-elect` does and why it matters
- [ ] You verified all components are healthy
- [ ] You can explain what each component's `--kubeconfig` flag does
- [ ] Your summary table is complete with accurate data from the cluster

## What You Should Understand After This Exercise

The control plane runs as pods in the `kube-system` namespace, just like
any other workload. Each component has specific flags that configure its
behavior. The API server is the central hub -- it talks to etcd and
serves the kubeconfig files that other components use to authenticate.
The `--leader-elect` flag enables high availability by ensuring only
one instance of each component is active at a time.
