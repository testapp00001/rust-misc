# Solution 03: Maintenance Window Procedure

## Kubernetes Cluster Upgrade: v1.27 to v1.29

**Document Owner:** Platform Engineering Team
**Last Updated:** 2026-06-11
**Classification:** Internal -- Operations

---

## Part A -- Maintenance Window Scheduling

### When

**Day:** Sunday (lowest traffic day of the week)
**Time Window:** 02:00 -- 08:00 UTC
**Date:** First Sunday of the month following CAB approval

**Justification:** Production traffic is 50,000 concurrent users during peak
(10:00-22:00 UTC) and 5,000 during off-peak. Starting at 02:00 UTC provides:
- 8 hours of buffer before peak begins at 10:00 UTC
- The lowest concurrent user count (approximately 5,000)
- A Sunday reduces impact on business operations (no order processing, internal
  tool usage is minimal)

### Duration

**Planned duration:** 4 hours (02:00 -- 06:00 UTC)
**Buffer:** 2 hours (06:00 -- 08:00 UTC)
**Total window:** 6 hours

**Reasoning:**
- Control plane upgrade: ~30 minutes per control plane node (3 nodes = 90 min)
- Worker node pool upgrades: ~20 minutes per pool drain/upgrade cycle
  (3 pools = 60 min)
- Verification and smoke testing: ~30 minutes
- Contingency: ~60 minutes
- Total estimated: 4 hours, with 2 hours buffer before peak traffic

### Blackout Periods

Maintenance must NOT occur during:

1. **Black Friday / Cyber Monday weekend** (last Friday to Monday of November)
   -- E-commerce peak, 5x normal traffic
2. **Year-end freeze** (December 20 -- January 3) -- Holiday shopping, annual
   financial close
3. **Quarterly earnings releases** (specific dates announced by Finance) --
   Regulatory sensitivity, public scrutiny
4. **Major product launches** (as announced by Product team) -- High visibility,
   marketing spend in flight
5. **During any active Sev1/Sev2 incident** -- Resources must focus on
   resolution, not maintenance

### Frequency

This type of major version upgrade can occur at most once per quarter because:
- Kubernetes minor versions are released every ~4 months
- Each upgrade requires staging validation (2 weeks minimum)
- The organization needs a stable period after each upgrade to collect
  operational data
- Patch-level upgrades (e.g., 1.27.3 to 1.27.5) can occur monthly during
  regular maintenance windows

---

## Part B -- Communication Plan

### Audience Matrix

| Audience | Role | Interest |
|----------|------|----------|
| Platform Engineering | Executor | Execute the upgrade |
| Development Teams | Stakeholder | Their services run on the cluster |
| SRE / On-call | Support | First responder if issues arise |
| Product Management | Informed | Awareness of potential impact |
| Customer Support | Informed | Handle customer inquiries |
| VP Engineering | Approver | Authorizes the change |
| External Customers | End user | Experience the service |

### Notification Timeline

| Time Before | Audience | Channel | Content |
|-------------|----------|---------|---------|
| 2 weeks | Platform Eng, Dev Teams, SRE | Email + Slack #platform | Detailed RFC with schedule and impact |
| 1 week | All internal teams | Email + Slack #announcements | Reminder with link to RFC |
| 24 hours | All internal teams | Slack #announcements + Status page | Final reminder with checklist |
| 1 hour | SRE, On-call, Platform Eng | Slack #incident-response | "Maintenance starting in 1 hour" |
| Start | External customers | Status page banner | "Scheduled maintenance in progress" |
| Completion | All audiences | Email + Status page + Slack | "Maintenance completed successfully" |

### Communication Channels

| Channel | Audience | Use |
|---------|----------|-----|
| Slack #platform | Platform Eng, Dev Teams | Detailed technical discussion |
| Slack #announcements | All internal | High-level notifications |
| Email | All internal + management | Formal notifications with full details |
| Status page (e.g., Statuspage.io) | External customers | Public-facing status |
| In-app banner | External customers | Visible during maintenance window |
| PagerDuty | On-call engineers | Escalation if issues arise |

### Message Templates

**Initial Announcement (2 weeks before):**

```
Subject: [Scheduled Maintenance] Kubernetes Cluster Upgrade -- July 6, 2026

Team,

We will be upgrading the production Kubernetes cluster from v1.27 to v1.29 on
Sunday, July 6, 2026 from 02:00 to 08:00 UTC.

Impact:
- Brief pod disruptions during node draining (typically < 30 seconds per pod)
- No expected downtime for services with multiple replicas
- Single-replica services may experience brief unavailability

What you need to do:
- Ensure your services have at least 2 replicas in production
- Verify your readiness probes are configured correctly
- Review the upgrade RFC: [link]

Rollback plan: We can roll back to v1.27 within 2 hours if issues arise.

Questions? Reach out in #platform or contact the Platform Engineering team.

RFC: [link to full RFC document]
```

**24-Hour Reminder:**

```
Subject: [Reminder] Kubernetes Upgrade Tomorrow at 02:00 UTC

Reminder: The production Kubernetes cluster upgrade (v1.27 -> v1.29) is
scheduled for tomorrow, Sunday July 6, 2026 at 02:00 UTC.

Pre-maintenance checklist:
  [ ] Services have adequate replicas (2+)
  [ ] Readiness probes configured
  [ ] On-call engineer confirmed and available
  [ ] No active incidents

Status page: [link]
RFC: [link]
```

**Maintenance Started:**

```
Subject: [In Progress] Kubernetes Upgrade -- Started 02:00 UTC

The Kubernetes cluster upgrade has begun. Expected completion: 06:00 UTC.

Current status: Upgrading control plane nodes
Next milestone: Control plane upgrade complete (est. 03:30 UTC)

Live updates in Slack: #platform
Status page: [link]
```

**Maintenance Completed:**

```
Subject: [Complete] Kubernetes Upgrade -- Successful

The Kubernetes cluster upgrade from v1.27 to v1.29 has been completed
successfully.

Timeline:
- Started: 02:00 UTC
- Control plane upgraded: 03:15 UTC
- Worker pools upgraded: 05:00 UTC
- Verification complete: 05:30 UTC
- Completed: 05:30 UTC

All health checks are green. No issues detected.

Post-implementation review scheduled for Monday July 7 at 10:00 UTC.

Status page: [link]
```

### Escalation Contacts

| Role | Primary | Secondary | Contact |
|------|---------|-----------|---------|
| Upgrade Lead | Platform Eng Lead | Senior Platform Eng | Slack DM + PagerDuty |
| Cluster Admin | On-call SRE | Backup SRE | PagerDuty |
| Decision Authority | VP Engineering | Director of Platform | Phone |
| Communications | Engineering Manager | Tech Lead | Slack DM |
| Vendor Support | Cloud Provider TAM | Support ticket | Phone + Portal |

---

## Part C -- Pre-Maintenance Checklist

| # | Category | Check | Criterion | Owner | If Failed |
|---|----------|-------|-----------|-------|-----------|
| 1 | Cluster Health | All nodes Ready | `kubectl get nodes` shows all nodes as `Ready` | Platform Eng | Abort |
| 2 | Cluster Health | Pod health | No pods in `CrashLoopBackOff`, `ImagePullBackOff`, or `Pending` > 5 min | SRE | Delay (fix first) |
| 3 | Cluster Health | Resource utilization | CPU < 70%, Memory < 80% across all nodes | Platform Eng | Delay (scale up) |
| 4 | Backup | etcd snapshot | `etcdctl snapshot save` completes successfully, file size > 10MB | Platform Eng | Abort |
| 5 | Backup | Persistent volume backups | All PVs have snapshots taken within last 24 hours | Platform Eng | Abort |
| 6 | Backup | Database dumps | Production database dump completed and verified (checksum match) | DBA | Abort |
| 7 | Artifacts | Staging upgrade successful | Staging cluster is running v1.29 with all services healthy | Platform Eng | Abort |
| 8 | Artifacts | Rollback plan tested | Rollback procedure tested in staging and completed in < 30 minutes | Platform Eng | Abort |
| 9 | Team | On-call engineer confirmed | Primary and backup on-call engineers are available and have cluster access | SRE | Delay (find backup) |
| 10 | Team | Access verified | All engineers have valid `kubectl` credentials and VPN access | Platform Eng | Delay (fix access) |
| 11 | Dependencies | No active incidents | No Sev1 or Sev2 incidents are open | SRE | Abort |
| 12 | Dependencies | Cloud provider healthy | Cloud provider status page shows no incidents for our region | Platform Eng | Abort |
| 13 | Dependencies | Third-party services operational | Key dependencies (DNS, CDN, auth provider) are healthy | SRE | Delay (until clear) |

**Decision rules:**
- **Abort:** Postpone to next available window (1 week minimum)
- **Delay:** Fix the issue within the current window if possible, otherwise abort
- **Proceed:** All checks pass

---

## Part D -- Execution Runbook

### Prerequisites
- All pre-maintenance checks passed
- On-call engineer confirmed and available
- Communication sent ("Maintenance started")

---

**Step 1: Take etcd Snapshot**
- **Action:** `ETCDCTL_API=3 etcdctl snapshot save /backups/etcd-pre-upgrade-$(date +%Y%m%d).db`
- **Expected:** Snapshot file created, `Snapshot saved at /backups/etcd-pre-upgrade-*.db`
- **Duration:** 2-5 minutes
- **If failed:** Abort maintenance. Investigate etcd health.
- **Checkpoint:** Verify file exists and size is reasonable (> 10MB)

---

**Step 2: Verify Pre-Upgrade Cluster State**
- **Action:** `kubectl get nodes -o wide && kubectl get pods -A | grep -v Running | grep -v Completed`
- **Expected:** All nodes Ready, no abnormal pods
- **Duration:** 2 minutes
- **If failed:** Resolve issues before proceeding or abort
- **Checkpoint:** GO/NO-GO -- all nodes Ready, no abnormal pods

---

**Step 3: Upgrade Control Plane (first node)**
- **Action:**
  ```bash
  # SSH to first control plane node
  ssh cp-node-1
  # Upgrade kubeadm
  apt-get update && apt-get install -y kubeadm=1.29.0-*
  # Apply upgrade
  kubeadm upgrade apply v1.29.0
  # Drain
  kubectl drain cp-node-1 --ignore-daemonsets
  # Upgrade kubelet
  apt-get install -y kubelet=1.29.0-* kubectl=1.29.0-*
  systemctl daemon-reload && systemctl restart kubelet
  # Uncordon
  kubectl uncordon cp-node-1
  ```
- **Expected:** Node shows `v1.29` in `kubectl get nodes`, status `Ready`
- **Duration:** 15-20 minutes
- **If failed:** Run `kubeadm upgrade apply` again. If persistent, restore etcd from snapshot.
- **Checkpoint:** Verify node version and Ready status

---

**Step 4: Upgrade Remaining Control Plane Nodes**
- **Action:** Repeat Step 3 for cp-node-2 and cp-node-3, one at a time
- **Expected:** All control plane nodes show v1.29
- **Duration:** 30-40 minutes (total for both)
- **If failed:** Same as Step 3. Upgrade is still reversible at this point.
- **Checkpoint:** `kubectl get nodes` shows all CP nodes at v1.29

---

**Step 5: Upgrade Worker Node Pool 1**
- **Action:**
  ```bash
  # For each node in pool 1:
  kubectl drain worker-pool1-node-N --ignore-daemonsets --delete-emptydir-data
  ssh worker-pool1-node-N
  apt-get update && apt-get install -y kubeadm=1.29.0-*
  kubeadm upgrade node
  apt-get install -y kubelet=1.29.0-* kubectl=1.29.0-*
  systemctl daemon-reload && systemctl restart kubelet
  # Verify node reports new version
  exit
  kubectl uncordon worker-pool1-node-N
  # Wait for pods to reschedule before draining next node
  sleep 60
  ```
- **Expected:** All nodes in pool 1 at v1.29, all pods rescheduled and Running
- **Duration:** 20-30 minutes
- **If failed:** Uncordon the node and investigate. Pods will reschedule on other nodes.
- **Checkpoint:** `kubectl get pods -n <namespace>` -- all pods Running

---

**Step 6: Verify Pool 1 Health**
- **Action:**
  ```bash
  kubectl get nodes | grep pool1  # all Ready, v1.29
  kubectl get pods -A | grep pool1-node  # all Running
  kubectl top nodes  # resource usage normal
  ```
- **Expected:** Healthy nodes and pods
- **Duration:** 5 minutes
- **If failed:** If pods are not rescheduling, check resource quotas and pod disruption budgets
- **Checkpoint:** GO/NO-GO -- proceed to next pool or rollback

---

**Step 7: Upgrade Worker Node Pool 2 and Pool 3**
- **Action:** Repeat Steps 5-6 for each remaining pool
- **Expected:** All worker nodes at v1.29
- **Duration:** 40-60 minutes
- **If failed:** Same as Step 5. At this point, rollback requires downgrading
  the affected pool only.
- **Checkpoint:** `kubectl get nodes -o wide` -- all nodes v1.29, all Ready

---

**Step 8: Run Application Smoke Tests**
- **Action:**
  ```bash
  # Health endpoints
  for svc in api-gateway user-service order-service payment-service; do
    curl -sf "https://${svc}.internal/health" || echo "FAIL: $svc"
  done

  # Functional smoke test
  ./scripts/smoke-test.sh --env production
  ```
- **Expected:** All health endpoints return 200, smoke tests pass
- **Duration:** 10-15 minutes
- **If failed:** Investigate the failing service. If isolated, may continue
  with monitoring. If widespread, rollback.
- **Checkpoint:** GO/NO-GO -- all smoke tests pass

---

**Step 9: Monitor for 30 Minutes**
- **Action:** Watch Grafana dashboards for:
  - HTTP error rate (should remain < 0.1%)
  - P99 latency (should remain within baseline +/- 10%)
  - Pod restart count (should be 0)
  - Node resource utilization (should be normal)
- **Expected:** All metrics within normal ranges
- **Duration:** 30 minutes
- **If failed:** If metrics degrade, trigger rollback
- **Checkpoint:** Final GO/NO-GO -- maintenance complete or rollback

---

**Step 10: Close Maintenance Window**
- **Action:**
  ```bash
  # Send completion notification
  # Update status page to "All Systems Operational"
  # Close the RFC with status "Successful"
  # Schedule PIR for next business day
  ```
- **Expected:** All communications sent, status page updated
- **Duration:** 5 minutes

---

## Part E -- Rollback Criteria and Procedure

### Rollback Triggers

Any ONE of the following conditions triggers an immediate rollback:

1. **HTTP 5xx error rate exceeds 1% for 2 consecutive minutes**
   - Metric: `sum(rate(http_requests_total{status=~"5.."}[1m])) / sum(rate(http_requests_total[1m])) > 0.01`
   - Checked on Grafana dashboard every 30 seconds

2. **P99 latency exceeds 2x baseline for 3 consecutive minutes**
   - Metric: `histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) > (baseline * 2)`
   - Baseline: measured during the 30-minute window before maintenance

3. **More than 10 pod restarts across the cluster in 5 minutes**
   - Metric: `sum(increase(kube_pod_container_status_restarts_total[5m])) > 10`
   - Indicates pods are crashing after the upgrade

4. **Any node enters NotReady state and does not recover within 5 minutes**
   - Metric: `kube_node_status_condition{condition="Ready",status="true"} == 0`
   - A NotReady node means workloads cannot be scheduled there

5. **Smoke test failure rate exceeds 50%**
   - Metric: Smoke test script exit code
   - Indicates functional breakage, not just performance degradation

6. **Customer-reported outage confirmed by SRE**
   - Trigger: SRE confirms a customer-facing impact via PagerDuty
   - Subjective but necessary for issues not caught by automated monitoring

### Rollback Procedure

**Time limit for full rollback:** 06:00 UTC (4 hours after start). After this
point, peak traffic begins in 4 hours and the rollback itself takes ~2 hours.
If we have not decided to rollback by 06:00, the decision shifts from "rollback
or continue" to "fix forward or escalate."

**Full rollback steps:**

1. **Announce rollback decision**
   ```
   Slack #platform: "ROLLBACK INITIATED. Reason: [trigger]. ETA to restore: 90 minutes."
   Slack #announcements: "Maintenance rollback in progress. Service may be briefly affected."
   Status page: Update to "Degraded Performance"
   ```

2. **Roll back worker node pools (reverse order)**
   ```bash
   # For each worker node, pool 3 first, then pool 2, then pool 1:
   kubectl drain worker-poolN-node-X --ignore-daemonsets --delete-emptydir-data
   ssh worker-poolN-node-X
   apt-get install -y kubeadm=1.27.* kubelet=1.27.* kubectl=1.27.*
   kubeadm upgrade node  # downgrade
   systemctl daemon-reload && systemctl restart kubelet
   exit
   kubectl uncordon worker-poolN-node-X
   ```

3. **Roll back control plane (reverse order)**
   ```bash
   # For each CP node (cp-node-3, then 2, then 1):
   ssh cp-node-N
   apt-get install -y kubeadm=1.27.* kubelet=1.27.* kubectl=1.27.*
   kubeadm upgrade apply v1.27.*  # may require --force
   systemctl daemon-reload && systemctl restart kubelet
   exit
   kubectl uncordon cp-node-N
   ```

4. **If downgrade fails:** Restore etcd from the pre-upgrade snapshot
   ```bash
   # On each control plane node:
   systemctl stop etcd
   ETCDCTL_API=3 etcdctl snapshot restore /backups/etcd-pre-upgrade-*.db \
     --data-dir=/var/lib/etcd-restore
   mv /var/lib/etcd /var/lib/etcd-old
   mv /var/lib/etcd-restore /var/lib/etcd
   systemctl start etcd
   ```

5. **Verify rollback**
   ```bash
   kubectl get nodes -o wide  # all nodes at v1.27
   kubectl get pods -A | grep -v Running  # no abnormal pods
   ./scripts/smoke-test.sh --env production  # smoke tests pass
   ```

6. **Monitor for 15 minutes** (same metrics as Step 9 of the runbook)

7. **Announce rollback complete**
   - Update status page to "All Systems Operational"
   - Send notification: "Rollback completed. System restored to v1.27."

### Partial Rollback

**Yes, a single node pool can be rolled back independently.**

If only Pool 2 shows issues (e.g., pods on Pool 2 nodes are crashing but Pool 1
and Pool 3 are healthy):

1. Drain and downgrade only the Pool 2 nodes back to v1.27
2. The cluster can run with mixed node versions (v1.27 workers + v1.29 workers)
   for a limited time
3. This is supported by Kubernetes skew policy (kubelet can be up to 3 minor
   versions behind the API server)
4. Investigate the Pool 2 issue (e.g., kernel incompatibility, specific hardware)
5. If the issue is pool-specific, fix it and re-attempt the upgrade for Pool 2
   only

**Decision process after rollback time limit (06:00 UTC):**
- The team lead and VP Engineering make a joint decision
- Options: fix forward (apply a targeted fix to the v1.29 cluster), continue
  rollback despite the time pressure, or extend the maintenance window
- The decision is based on: severity of the issue, confidence in a fix, and
  proximity to peak traffic
- If peak traffic is less than 2 hours away, rollback is the only safe option

---

## Appendix: Rollback Decision Flowchart

```
Issue Detected
  |
  v
Is it a Sev1 (service down)?
  YES --> Immediate rollback
  NO  --> Is the error rate > 1% for 2 min?
            YES --> Rollback
            NO  --> Is it a single service/pool?
                      YES --> Partial rollback for affected pool
                      NO  --> Continue monitoring
                              Is the trend improving?
                                YES --> Continue with monitoring
                                NO  --> Escalate to decision authority
```
