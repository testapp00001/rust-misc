# Exercise 02: Designing a Queue-Based Ingestion Layer

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Design a queue-based architecture that separates request ingestion from request processing, allowing your system to absorb traffic surges without dropping requests.

## Starting Point

You have a synchronous order processing system:

```
Client -> API Server -> Database -> Response
```

The API server handles 5,000 RPS. During a flash sale, traffic hits 50,000 RPS. The database can sustain 3,000 writes/second. The API server becomes unresponsive when CPU exceeds 90%.

Current deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-api
spec:
  replicas: 5
  selector:
    matchLabels:
      app: order-api
  template:
    spec:
      containers:
        - name: api
          image: order-api:latest
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 1
              memory: 1Gi
```

## Tasks

### Part A: Redesign with a Queue

Redesign the architecture to use a queue between ingestion and processing. Draw the new architecture as ASCII art and explain the role of each component.

Your new architecture must:
- Accept orders at 50,000 RPS (ingestion)
- Process orders at 3,000/second (database limit)
- Return an order ID to the client immediately
- Allow clients to check order status later

<details>
<summary>Hint 1</summary>

The ingestion layer should be lightweight -- validate the request, generate an order ID, push to the queue, and return immediately. The processing layer reads from the queue at a sustainable rate. Use Redis or RabbitMQ as the queue.

</details>

<details>
<summary>Hint 2</summary>

The client flow becomes: POST /orders -> ingestion validates and queues -> returns `{order_id, status: "queued"}` -> client polls GET /orders/{id}/status. The worker processes the queue and updates the status.

</details>

### Part B: Write the Ingestion Service

Write the Kubernetes deployment and service YAML for the ingestion tier. It should:
- Be lightweight (low CPU/memory)
- Scale to 20+ replicas independently of the processing tier
- Connect to a Redis queue
- Implement backpressure (reject when queue is full)

<details>
<summary>Hint 3</summary>

The ingestion service needs minimal resources because it does not do heavy processing. Use `LLLEN` on the Redis list to check queue depth before enqueueing. Return 503 when the queue exceeds a threshold.

</details>

### Part C: Write the Worker Service

Write the Kubernetes deployment YAML for the worker (processing) tier. It should:
- Consume from the Redis queue
- Process at a controlled rate (not faster than the database can handle)
- Be independently scalable via HPA based on queue depth
- Handle failures with retry logic

<details>
<summary>Hint 4</summary>

Use a semaphore or rate limiter in the worker to cap processing rate. Use KEDA with a Redis list length trigger to auto-scale workers based on queue depth. Use `BLPOP` for blocking pop with timeout.

</details>

## Success Criteria

- [ ] Architecture diagram shows clear separation between ingestion, queue, and processing tiers
- [ ] Ingestion deployment is lightweight and horizontally scalable
- [ ] Worker deployment has controlled processing rate that respects database limits
- [ ] Backpressure mechanism rejects requests when queue is full (503 response)
- [ ] Client receives immediate response with order ID and can poll for status

## What You Should Understand After This Exercise

Queue-based architecture decouples request acceptance from request processing. The ingestion tier handles the surge (lightweight, horizontally scalable), the queue absorbs the burst (buffering), and the processing tier drains at a sustainable rate. This transforms an impossible 50,000 RPS problem into a manageable one: 50,000 RPS ingestion + 3,000/sec processing + a growing queue that eventually drains.
