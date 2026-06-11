# Solution 03: Nomad Job Specification

## Part A: Job Specification

```hcl
job "pipeline" {
  datacenters = ["dc1", "dc2"]
  type        = "service"

  update {
    max_parallel      = 1
    min_healthy_time  = "30s"
    healthy_deadline  = "5m"
    progress_deadline = "10m"
    auto_revert       = true
    canary            = 1
  }

  reschedule {
    attempts       = 3
    interval       = "30m"
    delay          = "5s"
    delay_function = "exponential"
    max_delay      = "1m"
  }

  group "ingest" {
    count = 3

    spread {
      attribute = "${node.datacenter}"
      weight    = 100
    }

    network {
      port "http" {
        static = 3000
      }
    }

    service {
      name = "ingest"
      port = "http"

      check {
        type     = "http"
        path     = "/health"
        interval = "10s"
        timeout  = "3s"
      }

      tags = ["v1", "pipeline"]
    }

    task "ingest" {
      driver = "docker"

      config {
        image = "myapp/ingest:latest"
        ports = ["http"]
      }

      resources {
        cpu    = 500  # MHz
        memory = 256  # MB
      }

      logs {
        max_files     = 10
        max_file_size = 15
      }
    }
  }

  group "processor" {
    count = 5

    network {
      port "http" {
        static = 4000
      }
    }

    service {
      name = "processor"
      port = "http"

      check {
        type     = "http"
        path     = "/health"
        interval = "10s"
        timeout  = "3s"
      }

      tags = ["v1", "pipeline"]
    }

    task "processor" {
      driver = "docker"

      config {
        image = "myapp/processor:latest"
        ports = ["http"]
      }

      template {
        data = <<EOF
INGEST_HOST={{ range service "ingest" }}{{ .Address }}:{{ .Port }}{{ end }}
EOF
        destination = "local/config.env"
        env         = true
      }

      resources {
        cpu    = 2000  # MHz
        memory = 1024  # MB
      }

      logs {
        max_files     = 10
        max_file_size = 15
      }
    }
  }
}
```

### Why This Works

- **Spread strategy** on the `ingest` group distributes instances evenly across `dc1` and `dc2`, ensuring no single datacenter hosts all ingest tasks.
- **Update block** with `auto_revert = true` automatically rolls back a failed deployment. `canary = 1` deploys one instance first to validate before updating the rest.
- **Reschedule block** with exponential backoff (`5s, 10s, 20s, 40s...`) prevents thundering herd when nodes fail. Limits to 3 attempts per 30 minutes.
- **Template stanza** in the `processor` group uses Consul Template syntax to dynamically inject the `ingest` service address. This is updated automatically when ingest instances change.
- **Service registration** with health checks enables Consul-based discovery. Other services can find `ingest` and `processor` via Consul DNS or HTTP API.

### Common Mistakes to Avoid

- Using `{{ .Address }}:{{ .Port }}` without the `range` function. The `range` iterates over all healthy instances. Without it, you only get one instance (and the template fails if none are healthy).
- Not specifying `env = true` on the template. Without it, the template writes to a file but does not inject the value as an environment variable.
- Forgetting the `reschedule` block. Without it, failed tasks stay failed until manual intervention.
- Setting `canary` without `auto_revert`. Canary deployments only help if you can automatically roll back on failure.

---

## Part B: Deployment and Operations Commands

```bash
# 1. Validate the job file
nomad job validate pipeline.nomad

# 2. Submit (run) the job
nomad job run pipeline.nomad

# 3. Check the job status
nomad job status pipeline

# 4. Scale the processor group to 8 instances
nomad job scale pipeline processor 8

# 5. View logs for a specific allocation
nomad alloc logs <alloc_id>

# 6. Stop the job
nomad job stop pipeline
```

### Common Mistakes to Avoid

- Running `nomad job run` without `validate` first. Syntax errors in HCL can lead to unexpected behavior.
- Using `nomad job scale` without specifying the group name. The command requires job name, group name, and count.
- Forgetting that `nomad job stop` does not immediately kill tasks. It sends a signal and respects `kill_timeout`.

---

## Part C: Comparison with Docker Swarm

### 1. Count vs. Replicas

Nomad's `group "ingest" { count = 3 }` is equivalent to Swarm's `deploy.replicas: 3`. Both declare the desired number of instances. The difference is that Nomad's `count` is at the task group level (which can contain multiple tasks), while Swarm's `replicas` is at the service level (one container per service).

### 2. Service Registration vs. Built-in DNS

Nomad requires an explicit `service` block to register with Consul for discovery. Other services use Consul Template (`{{ range service "ingest" }}`) to discover instances. Swarm automatically registers services in its built-in DNS -- any container on the overlay network can resolve `ingest` by name without any configuration.

**Key difference:** Nomad's approach is more powerful (supports tags, metadata, multiple health check types, multi-datacenter queries) but requires Consul as an external dependency. Swarm's approach is simpler but less flexible.

### 3. Update Configuration

Nomad's `update` block is more granular than Swarm's `deploy.update_config`:

| Capability | Nomad | Swarm |
|-----------|-------|-------|
| Canary deployment | `canary = 1` (first-class) | Not built-in (requires external tooling) |
| Auto-revert | `auto_revert = true` | `failure_action: rollback` |
| Health deadline | `healthy_deadline = "5m"` | `monitor: 60s` |
| Progress deadline | `progress_deadline = "10m"` | No equivalent |

Nomad adds canary deployments and progress deadlines, giving more control over the update lifecycle.

### Common Mistakes to Avoid

- Assuming Nomad and Swarm are interchangeable. Nomad is a scheduler; Swarm is an orchestrator. They overlap but have different capabilities.
- Not recognizing that Nomad's service discovery requires Consul. This is an operational dependency that Swarm does not have.

---

## Key Takeaway

Nomad job specifications are more explicit and verbose than Docker Swarm stack definitions. You must define networking, service registration, health checks, and resource allocation at the task group level. This verbosity gives you finer control (canary deployments, spread strategies, Consul Template integration) at the cost of more configuration. The real strength of Nomad is its ability to schedule any workload type, not just containers.
