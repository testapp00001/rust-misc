# Solution 02: Idempotent API Design

## Part A: Idempotent vs. Non-Idempotent Operations

**1. `GET /users/123` -- Naturally idempotent**
Reading a resource does not change state. Calling it 100 times returns the same result every time (assuming no concurrent writes). No idempotency key needed.

**2. `PUT /users/123` (with full body) -- Naturally idempotent**
PUT replaces the entire resource with the provided body. Calling it 100 times with the same body produces the same result. The resource ends up in the same state regardless of how many times the request is sent.

**3. `POST /payments` (charge a customer) -- Non-idempotent (conditionally idempotent)**
Each call creates a new payment and charges the customer. Calling it 100 times charges the customer 100 times. Becomes conditionally idempotent with an idempotency key.

**4. `DELETE /users/123` -- Naturally idempotent**
Deleting a resource that does not exist is a no-op. Calling DELETE 100 times has the same effect as calling it once (the resource is deleted after the first call, subsequent calls are harmless).

**5. `POST /users/123/notifications` (send an email) -- Non-idempotent (conditionally idempotent)**
Each call sends a new email. Calling it 100 times sends 100 emails. Becomes conditionally idempotent with an idempotency key.

**6. `PATCH /users/123` (increment login_count by 1) -- Non-idempotent**
Each call increments the counter. Calling it 100 times increments 100 times. This is inherently non-idempotent because the operation is relative (add 1), not absolute (set to value). Cannot be made idempotent with a key -- you must redesign the operation.

### Common Mistakes to Avoid

- Saying DELETE is non-idempotent because "the second call returns 404." The effect on the server is the same (resource is deleted). The response may differ, but the state is identical.
- Saying PATCH is always idempotent. PATCH with absolute values (`set name to "John"`) is idempotent. PATCH with relative values (`increment by 1`) is not.
- Forgetting that `POST` is the HTTP method most likely to be non-idempotent. Always add idempotency keys to POST endpoints that create resources or trigger side effects.

---

## Part B: Idempotent Payment Endpoint

```python
import uuid
import hashlib
from datetime import datetime, timedelta


class PaymentService:
    def __init__(self, db):
        self.db = db

    def create_payment(self, request):
        # 1. Extract idempotency key from header
        idempotency_key = request.headers.get("X-Idempotency-Key")
        if not idempotency_key:
            return {"error": "X-Idempotency-Key header is required"}, 400

        # 2. Validate key format (UUID recommended)
        try:
            uuid.UUID(idempotency_key)
        except ValueError:
            return {"error": "X-Idempotency-Key must be a valid UUID"}, 400

        # 3. Check if this key was already processed
        existing = self.db.query(
            "SELECT * FROM payments WHERE idempotency_key = ?",
            idempotency_key
        )

        if existing:
            # 4. Return previous result (no duplicate charge)
            return {
                "payment_id": existing["payment_id"],
                "status": existing["status"],
                "amount": existing["amount"],
                "idempotent_replay": True
            }, 200

        # 5. Process the payment
        try:
            payment_id = str(uuid.uuid4())
            amount = request.json["amount"]
            customer_id = request.json["customer_id"]

            # Call external payment gateway
            result = self._charge_customer(customer_id, amount)

            # 6. Store result with idempotency key
            self.db.execute(
                """INSERT INTO payments
                   (idempotency_key, payment_id, customer_id, amount, status, created_at)
                   VALUES (?, ?, ?, ?, ?, ?)""",
                idempotency_key, payment_id, customer_id, amount,
                "completed", datetime.utcnow()
            )

            return {
                "payment_id": payment_id,
                "status": "completed",
                "amount": amount,
                "idempotent_replay": False
            }, 201

        except Exception as e:
            # Store failure to prevent retries from charging again
            self.db.execute(
                """INSERT INTO payments
                   (idempotency_key, payment_id, customer_id, amount, status, error, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?)""",
                idempotency_key, str(uuid.uuid4()),
                request.json.get("customer_id"),
                request.json.get("amount"),
                "failed", str(e), datetime.utcnow()
            )
            return {"error": str(e)}, 500

    def _charge_customer(self, customer_id, amount):
        # External payment gateway call
        # This is the non-idempotent part
        import requests
        response = requests.post(
            "https://payment-gateway.example.com/charge",
            json={"customer_id": customer_id, "amount": amount},
            timeout=10
        )
        response.raise_for_status()
        return response.json()
```

### Why This Works

- **Client-generated key:** The client creates a UUID and sends it with every retry. The same key is used across retries, so the server recognizes it as the same request.
- **Database lookup:** Before processing, the server checks if the key exists. If it does, the previous result is returned without re-processing.
- **Unique constraint:** The `idempotency_key` column has a unique constraint. If two concurrent requests arrive with the same key, only one inserts; the other gets a constraint violation and can retry the lookup.
- **Failure storage:** Even failures are stored. This prevents a retry from charging the customer after a previous attempt failed at the gateway.

### Common Mistakes to Avoid

- Not storing failures. If you only store successes, a retry after a gateway failure will charge the customer again.
- Using a hash of the request body as the key. Different requests with the same parameters should have different keys (the client determines intent, not the parameters).
- Not handling concurrent requests with the same key. Use a database unique constraint or `INSERT ... ON CONFLICT DO NOTHING`.

---

## Part C: Idempotency Key Design

**1. Who generates the key?**
The client generates the key. The client needs the same key across retries, and only the client knows which requests are retries of the same operation. The server should not generate the key because it cannot distinguish a new request from a retry.

**2. TTL of an idempotency key:**
Keys should expire after 24-48 hours. This balances two concerns: (a) long enough for all retries to complete (even with exponential backoff and network issues), (b) short enough to prevent unbounded database growth. Some systems use 7 days for financial transactions.

**3. Same key, different parameters:**
Reject the request with a 422 (Unprocessable Entity) or 409 (Conflict). The idempotency key represents the client's intent. If the parameters differ, it is either a bug (client reusing keys incorrectly) or an attack. Do not silently use the old parameters.

**4. Endpoint scoping:**
Keys should be scoped to the endpoint (e.g., `POST /payments` and `POST /refunds` have separate key spaces). A key for a payment should not prevent a refund with the same key. Global keys would cause unexpected conflicts across unrelated operations.

### Common Mistakes to Avoid

- Letting the server generate keys. The server cannot distinguish retries from new requests.
- Never expiring keys. The database grows without bound.
- Using global keys. Cross-endpoint conflicts are confusing and hard to debug.

---

## Key Takeaway

Idempotency transforms non-idempotent operations into safe-to-retry operations using a client-generated key. The server checks the key before processing, returns cached results for duplicates, and stores both successes and failures. Keys should be UUIDs, scoped to endpoints, and expired after 24-48 hours. Without idempotency, retries in distributed systems cause duplicate side effects (double charges, duplicate emails).
