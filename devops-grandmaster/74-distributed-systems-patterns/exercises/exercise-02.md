# Exercise 02: Idempotent API Design

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

You will design idempotent API endpoints for a payment system. This exercise practices the idempotency key pattern, which prevents duplicate operations when clients retry requests.

## Scenario

You are building the payment API for an e-commerce platform. Clients call `POST /payments` to charge a customer. The network is unreliable -- clients sometimes do not receive a response and retry the request. Without idempotency, a retry would double-charge the customer.

## Tasks

### Part A: Identify Idempotent vs. Non-Idempotent Operations

For each operation below, classify it as naturally idempotent, non-idempotent, or conditionally idempotent. Explain why.

1. `GET /users/123`
2. `PUT /users/123` (with full body)
3. `POST /payments` (charge a customer)
4. `DELETE /users/123`
5. `POST /users/123/notifications` (send an email)
6. `PATCH /users/123` (increment login_count by 1)

<details>
<summary>Hint</summary>
Naturally idempotent: calling it N times has the same effect as calling it once. Non-idempotent: each call has a different effect. Conditionally idempotent: idempotent only if you add an idempotency key.
</details>

### Part B: Implement Idempotent Payment Endpoint

Write a Python function `create_payment` that:

1. Accepts an idempotency key as a header (`X-Idempotency-Key`).
2. Checks if the key already exists in the database.
3. If it exists, returns the previous result (no duplicate charge).
4. If it does not exist, processes the payment and stores the result with the key.
5. Rejects requests without an idempotency key.

```python
def create_payment(request):
    # Your implementation here
    pass
```

<details>
<summary>Hint</summary>
Store the idempotency key and the result in a database table. On each request, check the table first. Use a unique constraint on the key column to handle race conditions.
</details>

### Part C: Idempotency Key Design

Answer the following design questions:

1. Who generates the idempotency key -- the client or the server?
2. What is the TTL of an idempotency key? Should it expire?
3. What happens if two different requests use the same idempotency key but different parameters?
4. Should the idempotency key be scoped to the endpoint or global?

<details>
<summary>Hint</summary>
The client generates the key (it needs to be the same across retries). Keys should expire (24-48 hours is common). Same key with different parameters should be rejected. Keys should be scoped to the endpoint.
</details>

## Success Criteria

- [ ] All 6 operations are correctly classified with explanations.
- [ ] The `create_payment` function checks for existing keys and returns cached results.
- [ ] Race conditions are handled (unique constraint or atomic check-and-set).
- [ ] Design questions are answered with justified decisions.

## What You Should Understand After This Exercise

Idempotency is not a property of the operation itself -- it is a property of how the operation is implemented. Adding an idempotency key transforms a non-idempotent operation (charging a customer) into a safe-to-retry operation. The key must be client-generated, scoped to the endpoint, and stored with a TTL. Without idempotency, retries in distributed systems cause duplicate side effects.
