# Exercise 05: Orchestrator Migration Plan

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

You will design a complete migration plan from Docker Swarm to Kubernetes, covering service mapping, networking, secrets, storage, and rollout strategy. This exercise integrates all concepts from the module and requires you to think about real-world operational concerns.

## Scenario

Your company runs a production application on Docker Swarm with the following stack:

```yaml
# Current docker-compose.yml (Swarm stack)
version: "3.8"
services:
  api:
    image: myapp/api:v3
    ports: ["8080:8080"]
    deploy:
      replicas: 4
      update_config:
        parallelism: 1
        delay: 30s
      restart_policy:
        condition: on-failure
    secrets:
      - db_password
    networks:
      - frontend
      - backend

  worker:
    image: myapp/worker:v3
    deploy:
      replicas: 6
    networks:
      - backend

  postgres:
    image: postgres:15
    volumes:
      - pg_data:/var/lib/postgresql/data
    secrets:
      - db_password
    networks:
      - backend

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    networks:
      - backend

networks:
  frontend:
    driver: overlay
  backend:
    driver: overlay
    internal: true

volumes:
  pg_data:
  redis_data:

secrets:
  db_password:
    file: ./secrets/db_password.txt
```

The team has decided to migrate to Kubernetes. You have 3 months.

## Tasks

### Part A: Kubernetes Manifest Translation

Translate each Swarm service into the appropriate Kubernetes resource. For each service, create:

- A `Deployment` (or `StatefulSet` where appropriate).
- A `Service` for internal communication.
- A `Service` with `type: LoadBalancer` or `Ingress` for external access (api only).
- `Secret` and `PersistentVolumeClaim` resources where needed.

Write the complete YAML manifests.

<details>
<summary>Hint</summary>
Postgres should be a StatefulSet (stable network identity, persistent storage). Secrets in K8s are base64-encoded. Use `ClusterIP` for internal services. The overlay network concept maps to Kubernetes Services and CNI networking.
</details>

### Part B: Migration Rollout Strategy

Design a phased migration plan with the following constraints:

- Zero downtime for the API.
- Database migration must happen first (data gravity).
- Rollback capability at every phase.

Write a plan with at least 4 phases, each with:
1. What changes.
2. How to verify success.
3. How to rollback.

<details>
<summary>Hint</summary>
Phase 1: Set up Kubernetes cluster and networking. Phase 2: Migrate stateful services (postgres, redis) with data migration. Phase 3: Migrate stateless services (worker, api) behind a load balancer. Phase 4: Cut over DNS and decommission Swarm.
</details>

### Part C: Service Discovery Mapping

Explain how each service discovers the others in Swarm vs. Kubernetes:

1. How does the `api` service find `postgres` in Swarm?
2. How does the `api` service find `postgres` in Kubernetes?
3. What DNS names change?
4. How do health checks differ between the two platforms?

<details>
<summary>Hint</summary>
Swarm uses built-in DNS on the overlay network (service name resolves to VIPs). Kubernetes uses CoreDNS (service name resolves to ClusterIP). Swarm health checks are in the Docker healthcheck block; Kubernetes uses liveness/readiness probes.
</details>

## Success Criteria

- [ ] Kubernetes manifests are complete and correct for all 4 services.
- [ ] Postgres uses a StatefulSet with PersistentVolumeClaim.
- [ ] Secrets are properly defined (not in plaintext in the manifest).
- [ ] The migration plan has at least 4 phases with rollback steps.
- [ ] Service discovery differences are clearly explained with specific DNS names.
- [ ] You can articulate why the database should migrate first (data gravity).

## What You Should Understand After This Exercise

Migrating between orchestrators is not a lift-and-shift operation. Each platform has different concepts for networking (overlay vs. CNI), service discovery (Swarm DNS vs. CoreDNS), secrets (Docker secrets vs. K8s Secrets), and storage (Docker volumes vs. PV/PVC). The migration must be phased to minimize risk, with the database migrated first because everything depends on it, and each phase must have a clear rollback path.
