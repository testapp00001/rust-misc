# Solution 03: Design Authorization Policies for a Microservices App

## Part A: Communication Matrix

| Source | Destination | Allowed? | Endpoints | Methods |
|--------|-------------|----------|-----------|---------|
| Ingress Gateway | Frontend | Yes | /* | GET |
| Ingress Gateway | Admin Portal | Yes | /* | GET (requires admin auth) |
| Frontend | API Gateway | Yes | /api/v1/products/*, /api/v1/orders/*, /api/v1/cart/* | GET, POST |
| Admin Portal | API Gateway | Yes | /api/v1/admin/* | GET, POST, PUT, DELETE |
| API Gateway | Product Svc | Yes | /products/* | GET, POST, PUT |
| API Gateway | Order Svc | Yes | /orders/* | GET, POST |
| API Gateway | User Svc | Yes | /users/*, /auth/* | GET, POST |
| Order Svc | Payment Svc | Yes | /charge, /refund | POST |
| Order Svc | Inventory Svc | Yes | /stock/reserve, /stock/release | POST |
| Product Svc | Inventory Svc | Yes | /stock/* | GET |
| Payment Svc | Payment Provider | Yes | /v1/charges, /v1/refunds | POST (external) |
| Inventory Svc | Warehouse DB | Yes | /query | POST (SQL only) |
| Frontend | Payment Svc | **No** | -- | -- |
| Frontend | Database | **No** | -- | -- |
| Admin Portal | Payment Svc | **No** | -- | -- |
| Product Svc | Order Svc | **No** | -- | -- |
| Product Svc | User Svc | **No** | -- | -- |
| Payment Svc | User Svc | **No** | -- | -- |
| Any | Any | **No** (default) | -- | -- |

### Why This Works

The communication matrix follows the principle of least privilege. Each service can only communicate with the specific services it needs to perform its function:

- **Frontend** can only talk to the API Gateway, not directly to backend services. This prevents a compromised frontend from reaching sensitive services.
- **API Gateway** acts as a single entry point and can reach all backend services, but each service has restricted endpoints.
- **Order Svc** needs Payment Svc (for charging) and Inventory Svc (for reserving stock), but cannot access User Svc or Product Svc directly.
- **Payment Svc** can only communicate with the external Payment Provider. It has no access to user data, product data, or any other internal service.
- **Inventory Svc** can only talk to the Warehouse DB. It has no access to user data or payment processing.

The default-deny rule is critical. Any service pair not explicitly listed is blocked. This means if a new service is added, it has zero access until policies are explicitly configured.

### Common Mistakes

- **Allowing Frontend to access backend services directly.** The Frontend should only talk to the API Gateway. Direct access to Payment Svc or User Svc from the browser is a security risk.
- **Granting overly broad permissions to the API Gateway.** The API Gateway should not have unrestricted access. Each backend service should have its own AuthorizationPolicy limiting what the API Gateway can do.
- **Forgetting about DNS and health checks.** Services need DNS resolution and health check endpoints. Include these in your policies or use Istio's built-in handling.
- **Not considering internal service-to-service calls.** The matrix must account for all communication paths, not just external-facing ones.

---

## Part B: Istio AuthorizationPolicies

### Default Deny

```yaml
# 00-default-deny.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: deny-all
  namespace: ecommerce
spec:
  {}  # Empty spec = deny all traffic in namespace
```

### Frontend Policy

```yaml
# 01-frontend-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: frontend-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: frontend
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/istio-system/sa/istio-ingressgateway-service-account"]
      to:
        - operation:
            methods: ["GET"]
            paths: ["/*"]
```

### Admin Portal Policy

```yaml
# 02-admin-portal-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: admin-portal-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: admin-portal
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/istio-system/sa/istio-ingressgateway-service-account"]
            requestPrincipals: ["*"]  # Requires authenticated user
      to:
        - operation:
            methods: ["GET"]
            paths: ["/*"]
```

### API Gateway Policy

```yaml
# 03-api-gateway-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: api-gateway-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: api-gateway
  action: ALLOW
  rules:
    - from:
        - source:
            principals:
              - "cluster.local/ns/ecommerce/sa/frontend-sa"
              - "cluster.local/ns/ecommerce/sa/admin-portal-sa"
      to:
        - operation:
            methods: ["GET", "POST", "PUT", "DELETE"]
            paths: ["/api/v1/*"]
```

### Product Service Policy

```yaml
# 04-product-svc-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: product-svc-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: product-svc
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/ecommerce/sa/api-gateway-sa"]
      to:
        - operation:
            methods: ["GET", "POST", "PUT"]
            paths: ["/products/*"]
```

### Order Service Policy

```yaml
# 05-order-svc-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: order-svc-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: order-svc
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/ecommerce/sa/api-gateway-sa"]
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/orders/*"]
```

### Payment Service Policy

```yaml
# 06-payment-svc-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: payment-svc-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: payment-svc
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/ecommerce/sa/order-svc-sa"]
      to:
        - operation:
            methods: ["POST"]
            paths: ["/charge", "/refund"]
```

### Inventory Service Policy

```yaml
# 07-inventory-svc-policy.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: inventory-svc-allow
  namespace: ecommerce
spec:
  selector:
    matchLabels:
      app: inventory-svc
  action: ALLOW
  rules:
    - from:
        - source:
            principals:
              - "cluster.local/ns/ecommerce/sa/order-svc-sa"
              - "cluster.local/ns/ecommerce/sa/product-svc-sa"
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/stock/*"]
```

### Why This Works

The policies implement defense in depth through Istio's AuthorizationPolicy:

1. **Default deny (00)**: Establishes the baseline. No traffic is allowed unless explicitly permitted.
2. **Identity-based access (01-07)**: Each policy uses `source.principals` which references mTLS service account identities. This is cryptographically verified -- a service cannot spoof another service's identity.
3. **Path and method restrictions**: Each policy limits not just who can call a service, but what endpoints and HTTP methods are allowed. Even if an attacker compromises the API Gateway, they cannot use it to call POST /charge on the Payment Service (the policy only allows the Order Svc to do that).
4. **Selector-based scoping**: Each policy targets specific pods via `matchLabels`, so policies do not accidentally affect other services.

The policies are applied in order: if any ALLOW policy matches, the request is permitted. If no ALLOW policy matches, the default-deny policy blocks the request.

### Common Mistakes

- **Using IP addresses instead of service identities.** IPs can be spoofed and change with pod restarts. Always use `source.principals` with mTLS identities.
- **Forgetting the default-deny policy.** Without it, all traffic is allowed by default. The default-deny must exist as the baseline.
- **Allowing wildcard methods or paths.** Use specific methods and paths. `methods: ["*"]` is almost never the right choice.
- **Not accounting for Istio's policy ordering.** If a DENY policy matches, it takes precedence over ALLOW policies. Use DENY for explicit blocks (e.g., block admin endpoints from non-admin sources).

---

## Part C: OPA Policies for Fine-Grained Authorization

```rego
# policy.rego
package ecommerce.authz

import future.keywords.in
import future.keywords.if

default allow = false

# Rule 1: Admin portal access requires admin role
allow if {
    input.path[0] == "admin"
    input.user.role == "admin"
}

# Rule 2: Order creation restricted to business hours (9 AM - 6 PM UTC)
allow if {
    input.method == "POST"
    input.path[0] == "orders"
    input.user.role == "customer"
    hour := time.clock(time.now_ns() / 1000000000)[0]
    hour >= 9
    hour < 18
}

# Rule 3: Payment validation -- amount limits
allow if {
    input.method == "POST"
    input.path[0] == "payment"
    input.body.amount <= 10000
}

allow if {
    input.method == "POST"
    input.path[0] == "payment"
    input.body.amount > 10000
    input.user.role == "admin"
    input.body.approval_token != ""
}

# Rule 4: Product price changes -- admin only with audit logging
allow if {
    input.method == "PUT"
    input.path[0] == "products"
    input.path[2] == "price"  # /products/{id}/price
    input.user.role == "admin"
}

# Helper: Extract audit event for price changes
audit_event := {
    "action": "price_change",
    "user": input.user.id,
    "product_id": input.path[1],
    "new_price": input.body.price,
    "timestamp": time.now_ns(),
} if {
    input.method == "PUT"
    input.path[0] == "products"
    input.path[2] == "price"
}

# Default deny with reason
deny_reason := "admin_role_required" if {
    input.path[0] == "admin"
    input.user.role != "admin"
}

deny_reason := "outside_business_hours" if {
    input.method == "POST"
    input.path[0] == "orders"
    hour := time.clock(time.now_ns() / 1000000000)[0]
    hour < 9
}

deny_reason := "outside_business_hours" if {
    input.method == "POST"
    input.path[0] == "orders"
    hour := time.clock(time.now_ns() / 1000000000)[0]
    hour >= 18
}

deny_reason := "amount_exceeds_limit" if {
    input.method == "POST"
    input.path[0] == "payment"
    input.body.amount > 10000
    input.user.role != "admin"
}

deny_reason := "admin_approval_required" if {
    input.method == "POST"
    input.path[0] == "payment"
    input.body.amount > 10000
    input.user.role == "admin"
    input.body.approval_token == ""
}
```

### Why This Works

OPA provides fine-grained authorization that goes beyond what Istio AuthorizationPolicies can express:

1. **Time-based restrictions**: OPA can evaluate the current time to enforce business hours. Istio policies cannot do this natively.
2. **Data-based decisions**: OPA can inspect request bodies to enforce amount limits. Istio operates at the HTTP layer (method, path, headers) but cannot inspect request bodies.
3. **Multi-factor authorization**: The high-value payment rule requires both admin role AND an approval token. This implements the "four eyes" principle.
4. **Audit trail**: OPA can generate audit events as part of the policy evaluation, providing a built-in audit trail for compliance.
5. **Deny reasons**: Providing structured deny reasons helps with debugging and user-facing error messages.

The input structure represents the context that the OPA sidecar receives from Istio via the `envoy.ext_authz` filter. Each request is enriched with user identity, source service identity, and request details.

### Common Mistakes

- **Using `time.now_ns()` incorrectly.** OPA's time functions expect nanosecond precision. Divide by 1000000000 for seconds.
- **Not handling edge cases.** What happens at exactly 18:00? Is that allowed or blocked? The policy uses `< 18` (blocked at 18:00).
- **Storing secrets in Rego policies.** OPA policies should not contain API keys, passwords, or other secrets. Use external data sources for sensitive configuration.
- **Not testing policies before deployment.** Always run `opa test` before deploying policies to production.

---

## Part D: Test Cases

```rego
# policy_test.rego
package ecommerce.authz_test

import data.ecommerce.authz

# ===== ADMIN PORTAL TESTS =====

test_admin_can_access_admin_portal {
    authz.allow with input as {
        "path": ["admin", "dashboard"],
        "user": {"id": "admin-1", "role": "admin"}
    }
}

test_customer_cannot_access_admin_portal {
    not authz.allow with input as {
        "path": ["admin", "dashboard"],
        "user": {"id": "user-1", "role": "customer"}
    }
}

test_unauthenticated_cannot_access_admin_portal {
    not authz.allow with input as {
        "path": ["admin", "dashboard"],
        "user": {"id": "anon", "role": "anonymous"}
    }
}

# ===== BUSINESS HOURS TESTS =====

test_order_allowed_during_business_hours {
    # Note: This test depends on the current time
    # In production, mock time.now_ns() for deterministic tests
    authz.allow with input as {
        "method": "POST",
        "path": ["orders"],
        "user": {"id": "user-1", "role": "customer"}
    }
    with time.now_ns as 1710508800000000000  # 2024-03-15 12:00:00 UTC
}

# ===== PAYMENT TESTS =====

test_low_value_payment_allowed {
    authz.allow with input as {
        "method": "POST",
        "path": ["payment"],
        "user": {"id": "user-1", "role": "customer"},
        "body": {"amount": 500}
    }
}

test_high_value_payment_blocked_for_customer {
    not authz.allow with input as {
        "method": "POST",
        "path": ["payment"],
        "user": {"id": "user-1", "role": "customer"},
        "body": {"amount": 15000}
    }
}

test_high_value_payment_allowed_for_admin_with_approval {
    authz.allow with input as {
        "method": "POST",
        "path": ["payment"],
        "user": {"id": "admin-1", "role": "admin"},
        "body": {"amount": 15000, "approval_token": "APPROVAL-XYZ"}
    }
}

test_high_value_payment_blocked_for_admin_without_approval {
    not authz.allow with input as {
        "method": "POST",
        "path": ["payment"],
        "user": {"id": "admin-1", "role": "admin"},
        "body": {"amount": 15000, "approval_token": ""}
    }
}

test_exact_threshold_payment_allowed {
    authz.allow with input as {
        "method": "POST",
        "path": ["payment"],
        "user": {"id": "user-1", "role": "customer"},
        "body": {"amount": 10000}
    }
}

# ===== PRODUCT PRICE TESTS =====

test_admin_can_change_price {
    authz.allow with input as {
        "method": "PUT",
        "path": ["products", "SKU-001", "price"],
        "user": {"id": "admin-1", "role": "admin"},
        "body": {"price": 29.99}
    }
}

test_customer_cannot_change_price {
    not authz.allow with input as {
        "method": "PUT",
        "path": ["products", "SKU-001", "price"],
        "user": {"id": "user-1", "role": "customer"},
        "body": {"price": 0.01}
    }
}

# ===== DENY REASON TESTS =====

test_deny_reason_for_non_admin {
    result := authz.deny_reason with input as {
        "path": ["admin", "dashboard"],
        "user": {"id": "user-1", "role": "customer"}
    }
    result == "admin_role_required"
}

test_deny_reason_for_high_value_payment {
    result := authz.deny_reason with input as {
        "method": "POST",
        "path": ["payment"],
        "user": {"id": "user-1", "role": "customer"},
        "body": {"amount": 15000}
    }
    result == "amount_exceeds_limit"
}
```

### Why This Works

Comprehensive testing of OPA policies requires:

1. **Positive tests**: Verify that allowed actions succeed (admin accessing admin portal, low-value payments, admin price changes).
2. **Negative tests**: Verify that blocked actions fail (customer accessing admin portal, high-value payments without approval).
3. **Edge case tests**: Verify boundary conditions (exactly $10,000, exactly 9:00 AM, admin without approval token).
4. **Deny reason tests**: Verify that the correct denial reason is returned for debugging and auditing.

The `with input as {...}` syntax allows mocking different request scenarios without modifying the policy. This enables deterministic testing regardless of the actual current time.

### Common Mistakes

- **Not testing negative cases.** Only testing that allowed actions succeed does not verify that blocked actions are actually blocked.
- **Ignoring edge cases.** The boundary between $10,000 and $10,001 is a common source of bugs. Test both sides.
- **Not mocking time.** Tests that depend on the current time will pass at some times and fail at others. Mock `time.now_ns()` for deterministic tests.
- **Testing too few scenarios.** A policy might have 10 rules but only 3 tests. Each rule should have at least 2 tests (positive and negative).

## Key Takeaway

Authorization in a zero trust architecture operates at two levels: Istio AuthorizationPolicies for network-level access control (who can talk to whom), and OPA for application-level authorization (what can they do). Neither alone is sufficient. Istio ensures that only authorized services can reach a given endpoint. OPA ensures that the request itself complies with business rules. Together, they implement the principle of least privilege at both the network and application layers.
