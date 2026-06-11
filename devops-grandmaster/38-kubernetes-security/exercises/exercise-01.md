# Exercise 01: Pod Security Standards Explained

**Type:** Conceptual
**Estimated time:** 20 minutes

## Objective

Understand the three Kubernetes Pod Security Standards (privileged, baseline,
restricted) and explain when each should be used.

## Background

Kubernetes replaced the deprecated PodSecurityPolicy with Pod Security
Admission (PSA), which enforces three fixed profiles at the namespace level.
Every namespace can be labeled with a `pod-security.kubernetes.io/enforce`
label that rejects pods violating the chosen standard.

## Instructions

### Part A -- Definitions

Answer the following questions in your own words:

1. What is the purpose of Pod Security Standards?
2. What are the three enforcement levels and what does each one allow?
3. How does the `enforce`, `warn`, and `audit` mode differ on a namespace?
4. Why was PodSecurityPolicy replaced by PSA?

### Part B -- Classification

For each scenario below, state which Pod Security Standard (privileged,
baseline, or restricted) is the **minimum** standard that permits the
described pod configuration. Explain your reasoning.

| # | Scenario |
|---|----------|
| 1 | A pod that runs as root with `hostNetwork: true` |
| 2 | A pod that uses a non-root UID and drops all capabilities |
| 3 | A pod that uses `hostPID: true` but runs as non-root |
| 4 | A pod that binds to a host port (e.g. 80) |
| 5 | A pod that uses the `RuntimeDefault` seccomp profile and runs as UID 1000 |

### Part C -- Namespace Labels

Write the `kubectl` command to label a namespace called `production` so that:
- The **restricted** standard is enforced.
- Violations also trigger warnings.
- Violations are logged for auditing.

### Part D -- Quick Validation

Create a namespace, apply the labels from Part C, and try to create a pod
that runs as root. Observe what happens and record the output.

```bash
kubectl create namespace test-psa
# Apply your labels here
# Try to create a root-running pod here
```

## Success Criteria

- [ ] You can describe all three standards and their differences.
- [ ] You correctly classify all five scenarios.
- [ ] Your `kubectl label` command uses the correct label keys and values.
- [ ] You can explain the difference between enforce, warn, and audit modes.
- [ ] You observed a pod being rejected in Part D.

## Hints

<details>
<summary>Hint 1 -- Standard levels</summary>
The three standards are ordered from most permissive to most restrictive:
privileged > baseline > restricted. Each subsequent level includes all the
restrictions of the previous one plus additional constraints.
</details>

<details>
<summary>Hint 2 -- Label format</summary>
The PSA labels follow the pattern:
`pod-security.kubernetes.io/<MODE>: <LEVEL>`
where MODE is one of enforce, warn, or audit.
</details>

<details>
<summary>Hint 3 -- RuntimeDefault seccomp</summary>
A pod using the `RuntimeDefault` seccomp profile and a non-root UID meets
the restricted standard. The restricted standard *requires* a seccomp
profile of type `RuntimeDefault` or `Localhost`.
</details>
