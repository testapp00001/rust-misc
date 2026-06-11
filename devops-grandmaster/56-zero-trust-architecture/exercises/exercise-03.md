# Exercise 03: Design Authorization Policies for a Microservices App

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design and implement authorization policies for a microservices application using Istio AuthorizationPolicy and OPA (Open Policy Agent). Enforce the principle of least privilege so that each service can only communicate with the specific services it needs, and only on the required endpoints.

## Tasks

### Part A: Map the Service Communication Requirements

You are given the following microservices architecture for an e-commerce platform:

```
                    [ Internet ]
                         |
                    [ Ingress Gateway ]
                         |
              +----------+----------+
              |                     |
        [ Frontend ]          [ Admin Portal ]
         (React)                 (React)
              |                     |
              +----+       +-------+
                   |       |
                [ API Gateway ]
                (Rust/Actix)
                   |
          +--------+--------+
          |        |        |
    [ Product ] [ Order ] [ User ]
      Svc       Svc       Svc
          |        |        |
          +---+----+---+---+
              |        |
         [ Payment ] [ Inventory ]
            Svc         Svc
              |            |
         [ Payment ]   [ Warehouse ]
           Provider       DB
```

Write out the complete communication matrix. For each service pair, determine:
- Is communication allowed? (Yes/No)
- What endpoints are permitted?
- What HTTP methods are allowed?

| Source | Destination | Allowed? | Endpoints | Methods |
|--------|-------------|----------|-----------|---------|
| Ingress | Frontend | ? | ? | ? |
| Ingress | Admin Portal | ? | ? | ? |
| Frontend | API Gateway | ? | ? | ? |
| Admin Portal | API Gateway | ? | ? | ? |
| API Gateway | Product Svc | ? | ? | ? |
| API Gateway | Order Svc | ? | ? | ? |
| API Gateway | User Svc | ? | ? | ? |
| Order Svc | Payment Svc | ? | ? | ? |
| Order Svc | Inventory Svc | ? | ? | ? |
| Product Svc | Inventory Svc | ? | ? | ? |
| Payment Svc | Payment Provider | ? | ? | ? |
| Inventory Svc | Warehouse DB | ? | ? | ? |
| ... | ... | ... | ... | ... |

<details><summary>Hint</summary>Apply least privilege: each service should only access what it absolutely needs. The Frontend should NOT directly access Payment Svc. The Admin Portal might need broader read access but limited write access. No service should have unrestricted access to all others.</details>

### Part B: Implement Istio AuthorizationPolicies

Write Istio AuthorizationPolicy resources that enforce your communication matrix. Create a default-deny policy first, then add specific allow rules.

**Start with default deny for the entire namespace:**

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

**Write allow policies for each service:**

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
            # TODO: Who can call the frontend?
      to:
        - operation:
            # TODO: What methods and paths are allowed?
```

Write policies for:
1. Frontend -- allow from ingress only, GET on static assets and /api/*
2. Admin Portal -- allow from ingress only, authenticated admin users
3. API Gateway -- allow from Frontend and Admin Portal only
4. Product Svc -- allow from API Gateway, GET requests only
5. Order Svc -- allow from API Gateway, POST/GET on /orders/*
6. Payment Svc -- allow from Order Svc only, POST on /charge and /refund
7. Inventory Svc -- allow from Order Svc and Product Svc, GET on /stock/*

<details><summary>Hint</summary>Use `source.principal` for mTLS identity (e.g., `cluster.local/ns/ecommerce/sa/frontend-sa`). Use `source.namespaces` when allowing traffic from a namespace. Always start with deny-all and add exceptions.</details>

### Part C: Write OPA Policies for Fine-Grained Authorization

Implement an OPA Rego policy that adds fine-grained authorization beyond what Istio can express. The policy should enforce:

1. Admin Portal users must have the `admin` role
2. Order creation is restricted to business hours (9 AM - 6 PM UTC)
3. Payment amounts over $10,000 require admin approval
4. Product price changes are logged and restricted to admin users

```rego
# policy.rego
package ecommerce.authz

import future.keywords.in

default allow = false

# Rule 1: Admin portal access requires admin role
allow {
    input.path[0] == "admin"
    input.user.role == "admin"
}

# Rule 2: Order creation restricted to business hours
# TODO: Implement time-based restriction
# allow {
#     input.method == "POST"
#     input.path[0] == "orders"
#     # How do you check the current hour?
# }

# Rule 3: High-value payment approval
# TODO: Implement amount-based restriction
# allow {
#     input.method == "POST"
#     input.path[0] == "payment"
#     input.body.amount <= 10000
# }

# Rule 4: Product price changes
# TODO: Implement admin-only price changes with logging
```

Write the input structure that the OPA sidecar would receive:

```json
{
    "input": {
        "method": "POST",
        "path": ["orders"],
        "user": {
            "id": "user-123",
            "role": "customer"
        },
        "source": {
            "principal": "cluster.local/ns/ecommerce/sa/api-gateway-sa"
        },
        "time": "2025-03-15T14:30:00Z",
        "body": {
            "items": [{"product_id": "SKU-001", "quantity": 2}],
            "total": 150.00
        }
    }
}
```

<details><summary>Hint</summary>Use `time.clock()` in Rego to extract the current hour. For the amount check, compare `input.body.amount` against the threshold. Consider using `trace(sprintf(...))` for logging decisions.</details>

### Part D: Test Your Policies

Write test cases in OPA's test format to verify your policies:

```rego
# policy_test.rego
package ecommerce.authz_test

import data.ecommerce.authz

# Test: admin can access admin portal
test_admin_portal_access {
    authz.allow with input as {
        "path": ["admin", "dashboard"],
        "user": {"role": "admin"}
    }
}

# Test: non-admin cannot access admin portal
test_non_admin_blocked {
    not authz.allow with input as {
        "path": ["admin", "dashboard"],
        "user": {"role": "customer"}
    }
}

# TODO: Write tests for:
# - Business hours order creation (allowed during hours)
# - After-hours order creation (blocked)
# - High-value payment (blocked without approval)
# - Low-value payment (allowed)
# - Product price change by admin (allowed)
# - Product price change by non-admin (blocked)
```

<details><summary>Hint</summary>Use `with input as {...}` to mock different request scenarios. Test both positive (allow) and negative (deny) cases. Consider edge cases like exactly $10,000 or exactly 9:00 AM.</details>

## Success Criteria

- [ ] Complete communication matrix with least privilege for all service pairs
- [ ] Default-deny Istio policy as the base
- [ ] At least 6 Istio AuthorizationPolicy resources with correct source/destination rules
- [ ] OPA policy implementing at least 3 of the 4 fine-grained rules
- [ ] At least 6 OPA test cases covering both allow and deny scenarios
- [ ] Policies use mTLS identity (service accounts) not IP addresses

## What You Should Understand After This Exercise

- How to apply least privilege in a microservices architecture
- The defense-in-depth approach: Istio for network-level authz, OPA for application-level authz
- Why default-deny is the correct starting point for authorization
- How service identity (via mTLS) enables fine-grained access control
- The difference between authentication (who are you?) and authorization (what can you do?)
