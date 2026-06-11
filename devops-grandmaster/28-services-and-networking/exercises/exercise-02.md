# Exercise 02: Create Services and Test Connectivity

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create Kubernetes Services of different types and verify that traffic flows correctly between pods and services. This exercise gives you hands-on experience with Service creation, selector matching, port configuration, and basic connectivity testing.

## Background

You have a team running a simple nginx web application. You need to expose it internally via ClusterIP, then externally via NodePort, and verify that traffic is load-balanced across replicas. This is the most common first task when working with Kubernetes Services.

## Tasks

### Part A: Deploy the Backend

Create a Deployment with 3 replicas of nginx that will serve as your backend throughout this exercise.

```yaml
# Save as backend-deployment.yaml
```

Requirements:
- Deployment name: `web-backend`
- 3 replicas
- Container image: `nginx:1.25`
- Label: `app: web-backend`
- Container port: 80

Apply it and verify all 3 pods are running.

<details>
<summary>Hint -- Deployment Structure</summary>
The Deployment needs `spec.selector.matchLabels` and `spec.template.metadata.labels` to both include `app: web-backend`. The container should expose port 80. Use `kubectl get pods -l app=web-backend` to verify all replicas are running.
</details>

### Part B: Create a ClusterIP Service

Create a ClusterIP Service that exposes the `web-backend` Deployment internally.

```yaml
# Save as web-clusterip.yaml
```

Requirements:
- Service name: `web-clusterip`
- Type: ClusterIP
- Selector: `app: web-backend`
- Port 80 mapping to targetPort 80

Apply the Service and verify it was created:

```bash
kubectl get service web-clusterip
kubectl get endpoints web-clusterip
```

Test connectivity from within the cluster by running a temporary pod:

```bash
kubectl run test-curl --image=curlimages/curl:8.4.0 --rm -it -- sh
# Inside the pod:
curl http://web-clusterip:80
```

Record the output. Can you reach the nginx welcome page?

<details>
<summary>Hint -- Endpoints</summary>
After creating the Service, check that the Endpoints object lists all 3 pod IPs. Run `kubectl get endpoints web-clusterip` and confirm you see 3 IP:port entries. If you see `<none>`, your selector labels do not match the pod labels.
</details>

### Part C: Create a NodePort Service

Create a NodePort Service that exposes the same Deployment externally.

```yaml
# Save as web-nodeport.yaml
```

Requirements:
- Service name: `web-nodeport`
- Type: NodePort
- Selector: `app: web-backend`
- Port 80 mapping to targetPort 80
- Let Kubernetes auto-assign the nodePort (do not specify one)

Apply the Service and check the assigned nodePort:

```bash
kubectl get service web-nodeport
```

Record the nodePort number. Get the node IP and test external access:

```bash
kubectl get nodes -o wide
curl http://<node-ip>:<nodePort>
```

<details>
<summary>Hint -- Finding Node IP</summary>
Use `kubectl get nodes -o wide` to find the INTERNAL-IP column. If you are using minikube, run `minikube ip` instead. If you are using kind, the node IP is typically 127.0.0.1 but you may need to use `docker port` to find the mapped port.
</details>

### Part D: Verify Load Balancing

Run the following command from a temporary pod to see traffic being distributed across replicas:

```bash
kubectl run lb-test --image=curlimages/curl:8.4.0 --rm -it -- sh
# Inside the pod, run this multiple times:
for i in $(seq 1 6); do curl -s http://web-clusterip | head -1; done
exit
```

Then check which pod received each request:

```bash
kubectl logs -l app=web-backend --tail=1
```

Answer these questions:
1. Are requests distributed across all 3 pods?
2. Is there a pattern to the distribution (round-robin, random, etc.)?
3. What happens to the Endpoints if you scale the Deployment to 1 replica?

<details>
<summary>Hint -- Checking Distribution</summary>
Each nginx pod logs its access to stdout. Use `kubectl logs web-backend-<pod-suffix>` for each pod individually to see which ones received requests. By default, iptables-based kube-proxy uses a probability-based selection that approximates round-robin.
</details>

## Success Criteria

- [ ] The Deployment has 3 running pods with label `app: web-backend`
- [ ] The ClusterIP Service is accessible from within the cluster via `curl http://web-clusterip:80`
- [ ] The NodePort Service is accessible from outside the cluster via `<node-ip>:<nodePort>`
- [ ] The Endpoints object lists all 3 pod IPs
- [ ] You observed traffic being distributed across multiple pods

## What You Should Understand After This Exercise

After completing this exercise, you should be able to create Services that match pod selectors, understand the relationship between `port`, `targetPort`, and `nodePort`, and verify that load balancing is working by checking endpoints and pod logs. The key insight is that the Service selector must match pod labels exactly, and the Endpoints object is the live record of which pods are receiving traffic.
