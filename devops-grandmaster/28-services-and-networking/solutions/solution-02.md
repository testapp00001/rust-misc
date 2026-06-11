# Solution 02: Create Services and Test Connectivity

## Part A: Deploy the Backend

**backend-deployment.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-backend
  template:
    metadata:
      labels:
        app: web-backend
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
```

Apply and verify:

```bash
kubectl apply -f backend-deployment.yaml
kubectl get pods -l app=web-backend
```

You should see 3 pods in Running state. If any pod is in ImagePullBackOff, check that the image name is correct and that the node has network access to pull the image.

## Part B: Create a ClusterIP Service

**web-clusterip.yaml:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-clusterip
spec:
  type: ClusterIP
  selector:
    app: web-backend
  ports:
    - port: 80
      targetPort: 80
```

Apply and verify:

```bash
kubectl apply -f web-clusterip.yaml
kubectl get service web-clusterip
```

Check endpoints to confirm all 3 pod IPs are listed:

```bash
kubectl get endpoints web-clusterip
```

Expected output shows 3 IP:port entries, for example:

```
NAME           ENDPOINTS                                      AGE
web-clusterip  10.244.0.5:80,10.244.1.3:80,10.244.2.7:80     30s
```

Test connectivity:

```bash
kubectl run test-curl --image=curlimages/curl:8.4.0 --rm -it -- sh
# Inside the pod:
curl http://web-clusterip:80
```

You should see the nginx welcome page HTML. This confirms that the ClusterIP service is routing traffic to the backend pods.

## Part C: Create a NodePort Service

**web-nodeport.yaml:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-nodeport
spec:
  type: NodePort
  selector:
    app: web-backend
  ports:
    - port: 80
      targetPort: 80
```

Apply and check the auto-assigned nodePort:

```bash
kubectl apply -f web-nodeport.yaml
kubectl get service web-nodeport
```

The output will show the assigned nodePort in the range 30000-32767. For example:

```
NAME          TYPE       CLUSTER-IP      EXTERNAL-IP   PORT(S)        AGE
web-nodeport  NodePort   10.96.123.45    <none>        80:31234/TCP   10s
```

In this example, the nodePort is 31234. Get the node IP and test:

```bash
kubectl get nodes -o wide
curl http://<INTERNAL-IP>:31234
```

You should receive the nginx welcome page, confirming external access via NodePort.

## Part D: Verify Load Balancing

Run multiple requests from a temporary pod:

```bash
kubectl run lb-test --image=curlimages/curl:8.4.0 --rm -it -- sh
# Inside the pod:
for i in $(seq 1 6); do curl -s http://web-clusterip | head -1; done
exit
```

Check which pods received requests:

```bash
kubectl logs -l app=web-backend --tail=1
```

**Answers to the questions:**

1. **Are requests distributed across all 3 pods?** Yes. Kubernetes kube-proxy programs iptables (or IPVS) rules that distribute connections across all healthy pods matching the service selector. With 3 replicas and 6 requests, you should see traffic on all 3 pods.

2. **Is there a pattern to the distribution?** The default kube-proxy mode (iptables) uses probability-based random selection, not strict round-robin. With 3 pods, each connection has roughly a 1/3 chance of going to any given pod. Over a small number of requests (6), you may not see perfectly even distribution. Over thousands of requests, the distribution converges toward equal. IPVS mode supports true round-robin and other algorithms.

3. **What happens to Endpoints when you scale to 1 replica?** The Endpoints object is automatically updated. When you run `kubectl scale deployment web-backend --replicas=1`, the two excess pods are terminated. The endpoints controller detects this and removes their IPs from the Endpoints object. The service now routes all traffic to the single remaining pod. You can watch this happen with `kubectl get endpoints web-clusterip -w`.

### Common Mistakes to Avoid

- **Selector label mismatch.** The most common error is a typo in the Service selector that does not match the pod labels. If `app: web-backend` in the Service but `app: web-backned` in the Deployment, the Endpoints object will be empty and the service will route to nothing. Always verify with `kubectl get endpoints`.
- **Confusing `port` and `targetPort`.** The `port` is what clients connect to on the service IP. The `targetPort` is the port on the container. If your container listens on 8080, you need `targetPort: 8080`. Setting both to 80 only works because nginx listens on port 80 by default.
- **Forgetting that NodePort includes ClusterIP.** When you create a NodePort service, it automatically gets a ClusterIP too. You can access the service both via `http://web-nodeport:80` from inside the cluster and via `<node-ip>:<nodePort>` from outside.
- **Expecting strict round-robin.** iptables-based kube-proxy uses random probability selection, not round-robin. Do not build application logic that assumes requests are delivered in a fixed order.
- **Not checking endpoints first when troubleshooting.** If a service is not responding, check `kubectl get endpoints <service-name>` before anything else. An empty endpoints list means the selector does not match any pods.

## Key Takeaway

A Service uses a selector to match pods by label and creates an Endpoints object that tracks the current pod IPs. ClusterIP provides internal access, NodePort adds external access on high ports. The Endpoints object is automatically updated as pods are created and destroyed, which is how Kubernetes maintains stable networking despite ephemeral pods. The `port` field is the service port clients connect to, and `targetPort` is the container port that receives the traffic.
