# Exercise 03: Trace a Request Across Multiple Microservices

## Type: Independent

## Objective

Build three Python microservices that communicate via HTTP and propagate trace context between them, so that a single request produces a distributed trace visible in Jaeger.

## Architecture

```
[Client] --> [Frontend Service :5001]
                  |
                  +--> [Product Service :5002]
                  |         |
                  |         +--> [Database Service :5003]
                  |
                  +--> [Cart Service :5004]
                            |
                            +--> [Database Service :5003]
```

## Tasks

### Task 1: Create the Database Service (port 5003)

Create `service_db.py` — a Flask app that simulates a database:

```python
# Endpoints:
# GET /products       -> returns a list of products (simulate 50ms DB query)
# GET /cart/<user_id> -> returns cart items for user (simulate 30ms DB query)
```

- Use OpenTelemetry to instrument this service with service name `"database-service"`.
- Add a manual span named `"db-query"` for each endpoint that records the query as an attribute `db.statement`.
- Export traces to Jaeger on `localhost:6831`.

### Task 2: Create the Product Service (port 5002)

Create `service_products.py` — a Flask app that:

1. Receives requests at `GET /products`.
2. Makes an HTTP call to `http://localhost:5003/products` using the `requests` library.
3. Returns the products to the caller.

- Use OpenTelemetry to instrument this service with service name `"product-service"`.
- Instrument the `requests` library using `RequestsInstrumentor` so trace context is automatically propagated in outgoing HTTP calls.
- Add a manual span named `"process-products"` that wraps the call to the database service.

### Task 3: Create the Cart Service (port 5004)

Create `service_cart.py` — a Flask app that:

1. Receives requests at `GET /cart/<user_id>`.
2. Makes an HTTP call to `http://localhost:5003/cart/<user_id>` using `requests`.
3. Returns the cart data to the caller.

- Use OpenTelemetry to instrument this service with service name `"cart-service"`.
- Instrument the `requests` library for context propagation.
- Add a manual span named `"process-cart"` that wraps the call to the database service.

### Task 4: Create the Frontend Service (port 5001)

Create `service_frontend.py` — a Flask app that:

1. Receives requests at `GET /dashboard/<user_id>`.
2. Makes concurrent HTTP calls to both the Product Service and Cart Service.
3. Combines the results and returns a JSON response.

- Use OpenTelemetry to instrument this service with service name `"frontend-service"`.
- Use `requests` with trace context propagation for outgoing calls.
- Add a manual span named `"build-dashboard"` that contains child spans for each downstream call.

### Task 5: Run and Trace

1. Start all four services in separate terminals.
2. Send a request:
   ```bash
   curl http://localhost:5001/dashboard/user123
   ```
3. Open the Jaeger UI and find the trace.

### Questions to Answer

1. How many spans are in the complete trace? List each one.
2. Do all spans share the same `traceId`? Why is this critical?
3. What does the waterfall view look like? Which spans run in parallel?
4. If the Database Service takes 200ms instead of 50ms, how does that affect the total request time? Which spans are affected?

## Success Criteria

- [ ] All four services start without errors
- [ ] A request to `/dashboard/user123` returns combined product and cart data
- [ ] The Jaeger UI shows a single trace spanning all four services
- [ ] All spans share the same `traceId`
- [ ] The waterfall view shows Product Service and Cart Service calls running in parallel (or sequentially, depending on your implementation)
- [ ] Each service has its own manual spans with relevant attributes

## Hints

<details>
<summary>Hint 1: RequestsInstrumentor for context propagation</summary>

The key to making trace context propagate automatically across HTTP calls:

```python
from opentelemetry.instrumentation.requests import RequestsInstrumentor

# Call this AFTER setting the TracerProvider
RequestsInstrumentor().instrument()
```

Once instrumented, any `requests.get()` or `requests.post()` call automatically injects `traceparent` and `tracestate` headers into outgoing requests. The receiving service's Flask instrumentor extracts them and continues the trace.

</details>

<details>
<summary>Hint 2: Each service needs its own TracerProvider</summary>

Each service is a separate Python process. Each process needs its own TracerProvider configuration. The service name in the Resource distinguishes them in Jaeger:

```python
resource = Resource.create({"service.name": "product-service"})
```

You can reuse the same `tracing.py` helper from Exercise 02 by parameterizing the service name.

</details>

<details>
<summary>Hint 3: Sequential vs concurrent downstream calls</summary>

If you call the Product Service and Cart Service sequentially, the waterfall will show them stacked. If you want them to run in parallel (which is more realistic), use `concurrent.futures.ThreadPoolExecutor`:

```python
from concurrent.futures import ThreadPoolExecutor, as_completed

with ThreadPoolExecutor(max_workers=2) as executor:
    products_future = executor.submit(requests.get, "http://localhost:5002/products")
    cart_future = executor.submit(requests.get, "http://localhost:5003/cart/user123")
    products = products_future.result().json()
    cart = cart_future.result().json()
```

Note: The Flask instrumentor handles the thread-local context correctly for this pattern.

</details>

<details>
<summary>Hint 4: Starting multiple services</summary>

Run each in a separate terminal:

```bash
# Terminal 1
python service_db.py

# Terminal 2
python service_products.py

# Terminal 3
python service_cart.py

# Terminal 4
python service_frontend.py
```

Make sure each service uses a different port. Start from the leaf services (DB) first, then work up to the frontend.

</details>
