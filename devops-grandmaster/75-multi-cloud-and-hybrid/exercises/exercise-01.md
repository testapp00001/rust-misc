# Exercise 01: Multi-Cloud vs Hybrid Cloud Trade-Offs

**Type:** Conceptual
**Difficulty:** Beginner
**Estimated Time:** 60-90 minutes

## Objective

Develop a clear mental model of when to use multi-cloud, hybrid cloud, or a
single-cloud strategy. You will evaluate real-world scenarios and justify
architectural decisions based on trade-offs in cost, complexity, compliance,
and resilience.

## Background

Organizations adopt multi-cloud or hybrid strategies for many reasons:
avoiding vendor lock-in, meeting data sovereignty requirements, leveraging
best-of-breed services, or maintaining on-premises investments. Each approach
carries distinct costs and benefits that are frequently misunderstood.

Key definitions:

- **Multi-cloud**: Using two or more public cloud providers (e.g., AWS + GCP).
- **Hybrid cloud**: Combining a public cloud with on-premises or private cloud
  infrastructure.
- **Single-cloud**: Committing fully to one provider's ecosystem.

## Instructions

### Part A: Scenario Analysis (30 minutes)

For each scenario below, recommend whether the organization should adopt a
**single-cloud**, **multi-cloud**, or **hybrid** strategy. Write 3-5 sentences
justifying your choice. Consider cost, operational complexity, talent
requirements, and risk.

**Scenario 1 -- FinTech Startup**
A 20-person startup building a payment processing platform. They need PCI-DSS
compliance, low latency to US and EU customers, and must minimize operational
overhead. They have a small team with strong AWS experience.

**Scenario 2 -- Global Media Company**
A large media company that produces content in 40 countries. They have existing
on-premises rendering farms, must comply with data residency laws in each
country, and want to use best-of-breed AI/ML services from multiple providers.

**Scenario 3 -- Government Agency**
A federal agency with strict data sovereignty requirements. Some workloads are
classified and must remain on air-gapped on-premises infrastructure. Unclassified
workloads can move to an authorized cloud. They need unified identity and policy
management.

**Scenario 4 -- E-Commerce Platform**
A mid-size e-commerce company that currently runs on AWS. They are negotiating
a new cloud contract and want leverage. Their application uses many AWS-native
services (Lambda, DynamoDB, SQS). They need 99.99% availability.

**Scenario 5 -- Healthcare Provider**
A hospital network that stores patient records on-premises (HIPAA requirement
interpretation) but wants to run analytics and ML workloads in the cloud. They
need a unified security posture across both environments.

### Part B: Trade-Off Matrix (20 minutes)

Create a trade-off matrix comparing single-cloud, multi-cloud, and hybrid
strategies across these dimensions. Rate each as Low / Medium / High cost or
complexity.

| Dimension | Single-Cloud | Multi-Cloud | Hybrid |
|-----------|-------------|-------------|--------|
| Operational complexity | ? | ? | ? |
| Vendor lock-in risk | ? | ? | ? |
| Talent/skill requirements | ? | ? | ? |
| Network complexity | ? | ? | ? |
| Cost optimization potential | ? | ? | ? |
| Disaster recovery capability | ? | ? | ? |
| Compliance flexibility | ? | ? | ? |
| Time to market | ? | ? | ? |

### Part C: Anti-Patterns Essay (20 minutes)

Write a short essay (200-400 words) addressing ONE of the following topics:

1. **Accidental multi-cloud**: How organizations end up with multi-cloud by
   accident (shadow IT, mergers) and why this is different from strategic
   multi-cloud.
2. **Cloud repatriation**: Why some companies move workloads back on-premises
   and what this teaches us about hybrid strategy.
3. **The abstraction tax**: The hidden cost of building cloud-agnostic
   abstractions and when it is worth the investment.

## Success Criteria

- [ ] All five scenarios have a clear recommendation with justified reasoning.
- [ ] The trade-off matrix is complete and reflects realistic assessments.
- [ ] The essay demonstrates nuanced understanding, not just surface-level
      pros and cons.
- [ ] You can articulate at least three conditions where multi-cloud is NOT
      the right choice.

## Hints

<details>
<summary>Hint 1: When is single-cloud the right answer?</summary>

Single-cloud is often the best choice for small teams, startups, or
organizations that can fully commit to one provider's ecosystem. The
operational overhead of multi-cloud is real and ongoing. If your team has
deep expertise in one provider and that provider meets your compliance and
availability needs, adding a second cloud increases complexity without
proportional benefit.

</details>

<details>
<summary>Hint 2: Strategic vs accidental multi-cloud</summary>

Strategic multi-cloud means each cloud is chosen for a specific workload
based on technical or business requirements. Accidental multi-cloud means
different teams adopted different clouds without coordination, resulting in
duplicated effort, inconsistent security, and no leverage in negotiations.
The architecture and outcomes are fundamentally different.

</details>

<details>
<summary>Hint 3: The hybrid cloud middle ground</summary>

Hybrid cloud is often the right answer when an organization has significant
existing on-premises investment or regulatory requirements that mandate
on-premises data storage. The key challenge is building a unified control
plane that spans both environments without creating two separate operational
models.

</details>
