# Exercise 01: Conceptual — Reverse Proxy vs Forward Proxy vs Load Balancer

## Type
Conceptual — no code required. Write your answers in a text file or on paper.

## Objective

You must be able to clearly distinguish between three networking components that are frequently confused: forward proxy, reverse proxy, and load balancer. If you cannot explain the difference, you will misconfigure your infrastructure.

## Instructions

### Part A: Definitions

For each component, fill in the blanks:

1. **Forward Proxy**
   - Sits on the ______ side.
   - The ______ configures it.
   - It hides ______ from ______.
   - Common use cases: ______, ______, ______.

2. **Reverse Proxy**
   - Sits on the ______ side.
   - The ______ configures it.
   - It hides ______ from ______.
   - Common use cases: ______, ______, ______.

3. **Load Balancer**
   - Distributes ______ across ______.
   - Uses algorithms such as ______, ______, ______.
   - Can operate at Layer ______ (TCP) or Layer ______ (HTTP).

### Part B: Scenario Classification

For each scenario below, identify which component (forward proxy, reverse proxy, or load balancer) is the best fit. Explain your reasoning in one sentence.

1. A company wants to block employees from accessing social media websites during work hours.

2. An application has three identical API servers. Incoming requests should be spread across them so no single server is overwhelmed.

3. A team wants to serve `app.example.com`, `api.example.com`, and `admin.example.com` all through a single public IP address on port 443, with automatic SSL certificates.

4. A developer in a restricted network needs to route their outbound traffic through an intermediary server to reach a public npm registry.

5. A platform needs to handle 50,000 requests per second to its checkout service, which runs 10 identical containers.

6. A security team wants to add authentication and WAF rules in front of all backend services without modifying the services themselves.

### Part C: Architecture Diagram

Draw (ASCII or describe) a network diagram that shows all three components working together in a realistic production setup. Your diagram must include:

- At least 2 clients (one internal, one external)
- A forward proxy for the internal client
- A reverse proxy with SSL termination for the external client
- A load balancer distributing to 3 backend instances
- Clear labels showing which direction traffic flows

### Part D: Overlap Question

A reverse proxy can perform load balancing. A load balancer can perform SSL termination. Explain why we still distinguish between these components despite their overlapping capabilities. When would you deploy a dedicated load balancer *behind* a reverse proxy rather than combining both functions?

## Success Criteria

- [ ] All blanks in Part A are correctly filled.
- [ ] All 6 scenarios in Part B are correctly classified with valid reasoning.
- [ ] Part C diagram shows correct traffic flow direction for each component.
- [ ] Part D demonstrates understanding that separating concerns improves scalability, fault isolation, and operational flexibility.

## Hints

<details>
<summary>Hint 1 — Part A direction</summary>
Think about the word "forward" as "outbound" (client to internet) and "reverse" as "inbound" (internet to servers).
</details>

<details>
<summary>Hint 2 — Part B scenario 3</summary>
This scenario involves multiple concerns. Which component handles SSL? Which handles routing by hostname? Can one component do both?
</details>

<details>
<summary>Hint 3 — Part C traffic flow</summary>
Client requests flow left to right. Responses flow right to left. Draw the request path first, then the return path.
</details>

<details>
<summary>Hint 4 — Part D</summary>
Consider what happens when the proxy/LB component itself becomes a bottleneck or fails. How does separation help?
</details>
