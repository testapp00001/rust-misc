# Exercise 04: Incident Response Simulation

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Walk through the full incident response lifecycle for a complex Kubernetes cluster failure.
You will detect, assess, mobilize, investigate, resolve, and communicate -- putting together
all the skills from Exercises 01-03 in a realistic, time-pressured scenario.

## Prerequisites

- Completion of Exercises 01-03.
- Working knowledge of Kubernetes (pods, deployments, services, namespaces).
- Familiarity with `kubectl` commands for debugging.
- Understanding of incident severity classification and communication templates.

## Scenario

It is 03:47 UTC on a Saturday morning. You are the on-call engineer for **Acme Corp**.
Your PagerDuty app wakes you up with the following alert:

```
┌─────────────────────────────────────────────────────────────────────┐
│  PAGERDUTY ALERT                                                    │
│  ─────────────────────────────────────────────────────────────────  │
│  Severity:       critical                                           │
│  Source:         prometheus-prod-us-east-1                          │
│  Service:        platform-core                                      │
│  Timestamp:      2026-06-11T03:47:00Z                               │
│                                                                     │
│  Title:        KubePodCrashLooping - production/*                   │
│  Description:  Multiple pods in namespace "production" are          │
│                crash-looping. Affected deployments:                 │
│                  - api-gateway (12/12 pods CrashLoopBackOff)        │
│                  - order-service (8/8 pods CrashLoopBackOff)        │
│                  - user-service (6/6 pods CrashLoopBackOff)         │
│                                                                     │
│  Dashboard:    https://grafana.internal/d/k8s-prod                  │
│  Runbook:      https://runbooks.internal/k8s/crashloop              │
└─────────────────────────────────────────────────────────────────────┘
```

When you check the Grafana dashboard, you see:

```
┌─────────────────────────────────────────────────────────────────────┐
│  GRAFANA: K8s Cluster Health - us-east-1                            │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                     │
│  API Gateway Error Rate:  ████████████████████████░░░  87% (5xx)    │
│  Order Service Latency:   ████████████████████████████  p99: 30s    │
│  User Service Latency:    ██████████████████████████░░  p99: 18s    │
│  Node CPU:                ████████████████████████████  98%         │
│  Node Memory:             ████████████████████████░░░░  82%         │
│  Pod Restarts (5min):     ████████████████████████████  142         │
│                                                                     │
│  Recent Events:                                                     │
│  03:42  Warning  FailedScheduling  0/5 nodes available              │
│  03:43  Warning  OOMKilling        container "app" in pod           │
│         "api-gateway-7f8b9-xk4lp" exceeded memory limit            │
│  03:44  Warning  OOMKilling        container "app" in pod           │
│         "order-service-5d4c2-m8rjs" exceeded memory limit           │
│  03:45  Warning  BackOff           restarting failed container       │
└─────────────────────────────────────────────────────────────────────┘
```

The cluster has 5 nodes, each with 8 vCPUs and 32 GB RAM. The production namespace normally
runs about 40 pods.

## Tasks

### Part A: Detect and Assess (T+0 to T+5 minutes)

You have just received the alert. Perform the following:

1. **Classify the severity.** Using the framework from Exercise 01, determine the severity
   level. Write your classification and justification.

2. **Run initial diagnostic commands.** Write out the exact `kubectl` commands you would run
   to gather information about the crash-looping pods. You need to:
   - Check pod status across the namespace.
   - View logs from one of the crash-looping pods.
   - Describe a pod to see events and resource requests.
   - Check node resource utilization.

3. **Form an initial hypothesis.** Based on the Grafana data and the alert, what is your
   best guess for the root cause? Write it as a single statement.

<details><summary>Hint</summary>
OOMKilling events point to memory pressure. But why are multiple deployments hitting memory
limits simultaneously? Consider what changed recently -- was there a deployment, a config
change, or a resource limit modification? The fact that all three services are affected
suggests a cluster-level issue, not a single bad deployment.
</details>

### Part B: Mobilize and Communicate (T+5 to T+15 minutes)

1. **Write the paging message.** You need to page your secondary (Bob) and the platform
   team lead (Diana). Write a concise Slack message for `#incident-response` that includes:
   - Incident summary
   - Severity classification
   - Current impact
   - What you need from the responders

2. **Write the customer-facing communication.** Using the templates from Exercise 03, write
   the initial status page update and Slack message for `#incidents`.

3. **Define the Incident Commander handoff.** If you are the first responder and Diana (team
   lead) will become Incident Commander, what information do you need to brief her on? Write
   a structured handoff note.

<details><summary>Hint</summary>
The paging message should give responders enough context to start thinking about the problem
before they even join the call. Include links to dashboards, runbooks, and the incident
channel. The handoff note should cover: what happened, what you have done so far, what you
think the cause is, and what you need from the IC.
</details>

### Part C: Investigate and Resolve (T+15 to T+45 minutes)

You discover the root cause: a team member deployed a new version of `api-gateway` at 03:30
UTC that introduced a memory leak. The `api-gateway` pods consumed all available memory on
the nodes, causing OOM kills across all three services.

```
┌─────────────────────────────────────────────────────────────────────┐
│                      ROOT CAUSE TIMELINE                            │
│  ─────────────────────────────────────────────────────────────────  │
│  03:30  api-gateway v3.2.0 deployed (introduces memory leak)        │
│  03:35  api-gateway pods start consuming memory (200MB -> 2GB)      │
│  03:40  Node memory hits 98%, Kubernetes starts evicting pods       │
│  03:42  order-service and user-service pods evicted due to pressure │
│  03:43  OOMKilling events begin across all deployments              │
│  03:45  CrashLoopBackOff as pods cannot be rescheduled              │
│  03:47  PagerDuty alert fires                                       │
└─────────────────────────────────────────────────────────────────────┘
```

1. **Write the rollback commands.** You need to roll back `api-gateway` to the previous
   version. Write the exact `kubectl` commands, including:
   - Rolling back the deployment.
   - Verifying the rollback succeeded.
   - Checking that pods are healthy after the rollback.

2. **Write a mitigation script.** In addition to the rollback, write a bash script that:
   - Scales up the affected deployments to restore capacity.
   - Checks that all pods are in `Running` state.
   - Reports the error rate from the API gateway.

3. **Write the resolution communication.** Once the rollback is complete and error rates
   return to normal, write the Monitoring and Resolved stage communications.

<details><summary>Hint</summary>
The rollback command is `kubectl rollout undo deployment/api-gateway -n production`.
After rolling back, verify with `kubectl rollout status deployment/api-gateway -n production`.
For the mitigation script, use `kubectl get pods -n production` in a loop to check pod
status. Use `curl` or `kubectl exec` to check the API gateway health endpoint.
</details>

### Part D: Post-Incident Actions

1. **Write the post-incident review (PIR) agenda.** Include:
   - Incident timeline (from detection to resolution).
   - Root cause analysis.
   - What went well in the response.
   - What could be improved.
   - Action items with owners and deadlines.

2. **Propose preventive measures.** Based on this incident, suggest 3 technical changes that
   would prevent or detect this type of failure earlier. Consider:
   - Resource limit policies.
   - Deployment safeguards.
   - Monitoring improvements.

3. **Write the PIR communication.** Draft an email to the engineering leadership team
   summarizing the incident, its impact, and the planned improvements.

<details><summary>Hint</summary>
Good PIRs are blameless. Focus on the system failure (why was a memory-leaking deployment
allowed to reach production?) rather than the person who deployed it. Preventive measures
should address the systemic gap: ResourceQuota enforcement, deployment canary analysis,
and memory leak detection in CI could all have caught this.
</details>

### Part E: Retrospective Questions

Answer the following reflection questions:

1. At what point in the timeline could this incident have been prevented entirely?
2. Was the P2 severity classification correct, or should it have been P1? Justify your answer.
3. What information would you have needed sooner during the investigation?
4. How would you modify the on-call runbook to include this scenario?
5. If this incident happened again at 14:00 UTC on a Monday (peak traffic), how would your
   response differ?

<details><summary>Hint</summary>
Question 1 is about prevention, not detection. The best answer is "before the deployment"
-- canary analysis, resource limit testing, or a pre-deploy check could have caught the
memory leak. Question 2 depends on your company's severity matrix. With 87% error rate
affecting all customers, a P1 classification is defensible.
</details>

## Success Criteria

- [ ] Severity classification is justified using the matrix from Exercise 01.
- [ ] Diagnostic commands are correct and would produce useful output.
- [ ] Communications follow the templates from Exercise 03 and are appropriate for each audience.
- [ ] Rollback commands are accurate and include verification steps.
- [ ] The PIR agenda is blameless and focuses on systemic improvements.
- [ ] Preventive measures address the root cause, not just the symptom.

## What You Should Understand After This Exercise

Incident response is a coordinated effort that blends technical skill with communication and
leadership. The technical fix (rollback) is often the easiest part. The harder parts are
making fast decisions under pressure, communicating clearly to different audiences, and
following up with systemic improvements so the same failure does not happen again. Practice
makes this muscle memory.
