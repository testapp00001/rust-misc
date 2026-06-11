# 49 - High Availability

**Previous:** [48 - Disaster Recovery](../48-disaster-recovery/README.md) | **Next:** [50 - Database Migration](../50-database-migration/README.md)

---

## Problem

Your application has 99.9% uptime -- sounds great until you calculate that 0.1% downtime is 8.76 hours per year. For an e-commerce platform making $10,000 per hour, that is $87,600 in lost revenue. For a SaaS platform with SLA commitments, that is broken contracts and customer churn.

High availability (HA) means designing systems that continue operating when individual components fail. No single server, database, network link, or availability zone failure should cause visible downtime to users.

The nines:
| Availability | Downtime per Year | Downtime per Month |
|--------------|-------------------|--------------------|
| 99% ("two nines") | 3.65 days | 7.31 hours |
| 99.9% ("three nines") | 8.76 hours | 43.83 minutes |
| 99.99% ("four nines") | 52.6 minutes | 4.38 minutes |
| 99.999% ("five nines") | 5.26 minutes | 26.3 seconds |

Most production systems target 99.99% (four nines). Achieving five nines is exponentially harder and rarely worth the cost.

---

## Naive Way

```yaml
# Single server, single database, hope for the best
version: '3'
services:
  app:
    image: myapp:latest
    ports:
      - "8080:8080"
    # If this container dies, everything is down
  db:
    image: postgres:15
    volumes:
      - pgdata:/var/lib/postgresql/data
    # Single point of failure
```

**Why this fails:**
- Single instance of every component is a single point of failure (SPOF)
- Any server crash, network blip, or maintenance window causes full outage
- No redundancy at any layer
- Scaling requires manual intervention
- Zero tolerance for any failure

---

## Right Way

### Active-Passive Architecture

The simplest HA pattern: two identical servers, one actively serving traffic, the other standing by ready to take over.

```
              +-------------+
              |   Load      |
              |  Balancer   |
              +------+------+
                     |
            Health checks every 5s
                     |
           +---------+---------+
           |                   |
    +------v------+    +------v------+
    |   Active     |    |  Passive    |
    |   Server     |    |  Server     |
    |  (serving)   |<-->|  (standby)  |
    |              |sync|             |
    +------+------+    +------+------+
           |                   |
    +------v------+    +------v------+
    |  Primary    |    |  Replica    |
    |  Database   |--->|  Database   |
    |             |    |  (standby)  |
    +-------------+    +-------------+
```

**Implementation with HAProxy and automatic failover:**

```
# /etc/haproxy/haproxy.cfg
global
    maxconn 4096

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httpchk GET /health

frontend http_front
    bind *:80
    default_backend app_servers

backend app_servers
    balance roundrobin
    option httpchk
    server active  10.0.1.10:8080 check inter 5s fall 3 rise 2
    server passive 10.0.1.11:8080 check inter 5s fall 3 rise 2 backup
    # "backup" means this server only receives traffic when "active" is down
```

**Keepalived for floating IP (VRRP):**

```bash
# /etc/keepalived/keepalived.conf - Active node
vrrp_script chk_haproxy {
    script "killall -0 haproxy"
    interval 2
    weight 2
}

vrrp_instance VI_1 {
    state MASTER
    interface eth0
    virtual_router_id 51
    priority 101
    advert_int 1

    authentication {
        auth_type PASS
        auth_pass secret123
    }

    virtual_ipaddress {
        10.0.1.100/24  # Floating VIP
    }

    track_script {
        chk_haproxy
    }
}
```

```bash
# /etc/keepalived/keepalived.conf - Passive node
vrrp_instance VI_1 {
    state BACKUP
    interface eth0
    virtual_router_id 51
    priority 100
    advert_int 1

    authentication {
        auth_type PASS
        auth_pass secret123
    }

    virtual_ipaddress {
        10.0.1.100/24
    }
}
```

### Database High Availability with Patroni

```yaml
# patroni.yml - PostgreSQL HA with automatic failover
scope: pg-cluster
name: node1

restapi:
  listen: 0.0.0.0:8008
  connect_address: 10.0.1.10:8008

etcd3:
  hosts: 10.0.1.50:2379,10.0.1.51:2379,10.0.1.52:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576
    postgresql:
      use_pg_rewind: true
      parameters:
        max_connections: 200
        wal_level: replica
        hot_standby: "on"
        max_wal_senders: 5
        max_replication_slots: 5

postgresql:
  listen: 0.0.0.0:5432
  connect_address: 10.0.1.10:5432
  data_dir: /var/lib/postgresql/data
  authentication:
    replication:
      username: replicator
      password: rep-pass
    superuser:
      username: postgres
      password: pg-pass
```

---

## Production Way

### Active-Active Architecture

For maximum availability and performance, run identical stacks in multiple regions, all actively serving traffic.

```
                  +-------------------+
                  |    Global Load    |
                  |    Balancer       |
                  |  (CloudFront /    |
                  |   Route53 / GSLB) |
                  +--------+----------+
                           |
              +------------+------------+
              |                         |
    +---------v----------+  +----------v---------+
    |    Region A         |  |    Region B         |
    |                     |  |                     |
    | +-----+ +-----+    |  | +-----+ +-----+    |
    | |App 1| |App 2|    |  | |App 3| |App 4|    |
    | +--+--+ +--+--+    |  | +--+--+ +--+--+    |
    |    |       |        |  |    |       |        |
    | +--v-------v--+    |  | +--v-------v--+    |
    | |   Database   |<--+--->|   Database   |    |
    | |  (Primary)   | sync  |  (Primary)   |    |
    | +--------------+  |  | +--------------+    |
    +--------------------+  +--------------------+
```

### Consensus with etcd/Raft

For state that must be consistent across nodes, use a consensus protocol. Raft guarantees that a majority of nodes agree on a value before it is committed.

```go
// raft-example.go - Simplified Raft consensus for leader election
package main

import (
    "fmt"
    "log"
    "math/rand"
    "net/http"
    "strings"
    "sync"
    "time"
)

type RaftNode struct {
    id        string
    state     string // "follower", "candidate", "leader"
    term      int
    leader    string
    peers     []string
    mu        sync.RWMutex
    heartbeat chan bool
}

func NewRaftNode(id string, peers []string) *RaftNode {
    return &RaftNode{
        id:        id,
        state:     "follower",
        term:      0,
        peers:     peers,
        heartbeat: make(chan bool, 1),
    }
}

func (n *RaftNode) Run() {
    for {
        switch n.state {
        case "follower":
            select {
            case <-n.heartbeat:
                // Received heartbeat from leader, stay follower
            case <-time.After(randomTimeout(150, 300)):
                // No heartbeat received, become candidate
                n.mu.Lock()
                n.state = "candidate"
                n.mu.Unlock()
            }
        case "candidate":
            n.startElection()
        case "leader":
            n.sendHeartbeats()
            time.Sleep(50 * time.Millisecond)
        }
    }
}

func (n *RaftNode) startElection() {
    n.mu.Lock()
    n.term++
    n.state = "candidate"
    votes := 1 // Vote for self
    term := n.term
    n.mu.Unlock()

    for _, peer := range n.peers {
        go func(peer string) {
            resp, err := http.Post(
                fmt.Sprintf("http://%s/vote", peer),
                "application/json",
                strings.NewReader(fmt.Sprintf(`{"term":%d,"candidate":"%s"}`, term, n.id)),
            )
            if err == nil && resp.StatusCode == 200 {
                votes++
            }
        }(peer)
    }

    time.Sleep(100 * time.Millisecond) // Wait for votes

    majority := (len(n.peers)+1)/2 + 1
    if votes >= majority {
        n.mu.Lock()
        n.state = "leader"
        n.leader = n.id
        n.mu.Unlock()
        log.Printf("Node %s became leader for term %d", n.id, term)
    }
}

func randomTimeout(min, max int) time.Duration {
    return time.Duration(min+rand.Intn(max-min)) * time.Millisecond
}

func (n *RaftNode) sendHeartbeats() {
    for _, peer := range n.peers {
        go func(peer string) {
            http.Post(
                fmt.Sprintf("http://%s/heartbeat", peer),
                "application/json",
                strings.NewReader(fmt.Sprintf(`{"leader":"%s","term":%d}`, n.id, n.term)),
            )
        }(peer)
    }
}
```

### Kubernetes HA with Pod Disruption Budgets

```yaml
# Ensure at least 2 replicas always running during voluntary disruptions
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: app-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: myapp
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    metadata:
      labels:
        app: myapp
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            - labelSelector:
                matchExpressions:
                  - key: app
                    operator: In
                    values: ["myapp"]
              topologyKey: topology.kubernetes.io/zone
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: topology.kubernetes.io/zone
          whenUnsatisfiable: DoNotSchedule
          labelSelector:
            matchLabels:
              app: myapp
```

### Health Check Pattern

Every HA system needs proper health checks that verify the full request path:

```rust
// health.rs - Comprehensive health check endpoint
use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::PgPool;
use redis::Client as RedisClient;
use std::time::Instant;

#[derive(serde::Serialize)]
struct HealthStatus {
    status: String,
    version: String,
    checks: Vec<ComponentCheck>,
}

#[derive(serde::Serialize)]
struct ComponentCheck {
    name: String,
    status: String,
    latency_ms: u64,
}

async fn health_check(
    db: web::Data<PgPool>,
    redis: web::Data<RedisClient>,
) -> HttpResponse {
    let mut checks = Vec::new();
    let mut healthy = true;

    // Check database connectivity
    let start = Instant::now();
    let db_status = match sqlx::query("SELECT 1").execute(db.get_ref()).await {
        Ok(_) => "ok",
        Err(_) => { healthy = false; "error" }
    };
    checks.push(ComponentCheck {
        name: "postgres".to_string(),
        status: db_status.to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    });

    // Check Redis connectivity
    let start = Instant::now();
    let mut conn = redis.get_ref().get_connection().unwrap();
    let redis_status = match redis::cmd("PING").query::<String>(&mut conn) {
        Ok(_) => "ok",
        Err(_) => { healthy = false; "error" }
    };
    checks.push(ComponentCheck {
        name: "redis".to_string(),
        status: redis_status.to_string(),
        latency_ms: start.elapsed().as_millis() as u64,
    });

    let status = if healthy { "healthy" } else { "unhealthy" };
    let code = if healthy { 200 } else { 503 };

    HttpResponse::build(actix_web::http::StatusCode::from_u16(code).unwrap())
        .json(HealthStatus {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks,
        })
}
```

---

## Hands-On Lab

### Lab: Build a Highly Available PostgreSQL Cluster with Patroni

**Duration:** 120 minutes

**Prerequisites:**
- 3 VMs or Docker containers (for etcd cluster)
- 2 VMs or Docker containers (for PostgreSQL nodes)
- Basic PostgreSQL knowledge

**Step 1: Set Up etcd Cluster (Consensus Layer)**

```bash
# On each of 3 etcd nodes
docker run -d --name etcd1 \
    --network host \
    quay.io/coreos/etcd:latest \
    etcd --name etcd1 \
    --initial-advertise-peer-urls http://10.0.1.50:2380 \
    --listen-peer-urls http://10.0.1.50:2380 \
    --listen-client-urls http://10.0.1.50:2379,http://localhost:2379 \
    --advertise-client-urls http://10.0.1.50:2379 \
    --initial-cluster-token pg-cluster \
    --initial-cluster etcd1=http://10.0.1.50:2380,etcd2=http://10.0.1.51:2380,etcd3=http://10.0.1.52:2380 \
    --initial-cluster-state new

# Verify cluster health
etcdctl cluster-health
etcdctl member list
```

**Step 2: Install and Configure Patroni on Node 1**

```yaml
# /etc/patroni/node1.yml
scope: ha-pg
name: node1

restapi:
  listen: 0.0.0.0:8008
  connect_address: 10.0.1.10:8008

etcd3:
  hosts: 10.0.1.50:2379,10.0.1.51:2379,10.0.1.52:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576
    postgresql:
      use_pg_rewind: true

postgresql:
  listen: 0.0.0.0:5432
  connect_address: 10.0.1.10:5432
  data_dir: /var/lib/postgresql/15/main
  authentication:
    replication:
      username: replicator
      password: rep-pass
    superuser:
      username: postgres
      password: pg-pass
```

**Step 3: Start Patroni and Verify Leader Election**

```bash
# Start Patroni on both nodes
patroni /etc/patroni/node1.yml &
patroni /etc/patroni/node2.yml &

# Check cluster status
patronictl -c /etc/patroni/node1.yml list

# Expected output:
# + Cluster: ha-pg -----+---------+----+-----------+
# | Member  | Host      | Role    | TL | Lag in MB |
# +---------+-----------+---------+----+-----------+
# | node1   | 10.0.1.10 | Leader  |  1 |           |
# | node2   | 10.0.1.11 | Replica |  1 |         0 |
# +---------+-----------+---------+----+-----------+
```

**Step 4: Test Automatic Failover**

```bash
# Write test data to leader
psql -h 10.0.1.10 -U postgres -c "
CREATE TABLE ha_test (id serial PRIMARY KEY, data text, ts timestamp DEFAULT now());
INSERT INTO ha_test (data) VALUES ('before-failover');
"

# Stop the leader to simulate failure
patronictl -c /etc/patroni/node1.yml switchover --master node1 --candidate node2

# Wait and check - node2 should become leader
patronictl -c /etc/patroni/node2.yml list

# Verify data survived failover
psql -h 10.0.1.11 -U postgres -c "SELECT * FROM ha_test;"
# Should show "before-failover" row

# Insert more data on new leader
psql -h 10.0.1.11 -U postgres -c "INSERT INTO ha_test (data) VALUES ('after-failover');"
```

**Step 5: Measure Failover Time**

```bash
#!/bin/bash
echo "Starting failover measurement..."

# Record time before failover
START=$(date +%s%N)

# Trigger failover
patronictl -c /etc/patroni/node1.yml switchover --force

# Poll until new leader responds
while ! psql -h 10.0.1.11 -U postgres -c "SELECT 1" >/dev/null 2>&1; do
    sleep 0.1
done

END=$(date +%s%N)
ELAPSED=$(( (END - START) / 1000000 ))
echo "Failover completed in ${ELAPSED}ms"
```

**Deliverable:** Document your measured failover time and compare it to your RTO requirement.

---

## Limitation

High availability solves downtime, but it introduces a dangerous problem: **schema changes become extremely risky**.

When you have multiple active database nodes with synchronous replication, running `ALTER TABLE` on the primary propagates to all replicas. A poorly written migration can:
- Lock tables for minutes, causing request timeouts across all nodes
- Replicate a failed migration to every replica, leaving the entire cluster in a broken state
- Cause replication lag spikes that trigger false failovers
- Break the application on all nodes simultaneously if a column rename or type change is incompatible with the current code

In an active-active setup, schema changes are even more complex because both regions may have active writes to the same table. You need a strategy to evolve your schema without breaking anything.

---

## Next Topic

[50 - Database Migration](../50-database-migration/README.md) -- Learn how to perform schema changes on live, highly available databases without downtime using expand-and-contract migrations, online DDL tools, and backward-compatible change patterns.
