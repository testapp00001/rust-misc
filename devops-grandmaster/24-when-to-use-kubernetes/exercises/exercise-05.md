# Exercise 05: Docker Compose to Kubernetes Migration Plan

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Create a detailed, phased migration plan for moving an application from Docker Compose to Kubernetes. You will convert Docker Compose configurations to Kubernetes manifests, plan the migration strategy, identify risks, and define rollback procedures.

## Scenario / Starting Point

Your team has decided that Kubernetes is the right choice for your application after completing the cost and complexity analysis. You currently run the application on Docker Compose and need to migrate without downtime.

**Current Docker Compose Configuration:**

```yaml
# docker-compose.yml (current production setup)
version: "3.8"

services:
  frontend:
    image: myapp/frontend:latest
    ports:
      - "3000:3000"
    environment:
      - API_URL=http://api:4000
    depends_on:
      - api
    restart: always

  api:
    image: myapp/api:latest
    ports:
      - "4000:4000"
    environment:
      - DATABASE_URL=postgresql://user:password@db:5432/myapp
      - REDIS_URL=redis://redis:6379
      - WORKER_CONCURRENCY=5
    depends_on:
      - db
      - redis
    restart: always
    deploy:
      replicas: 3

  worker:
    image: myapp/api:latest
    command: ["node", "worker.js"]
    environment:
      - DATABASE_URL=postgresql://user:password@db:5432/myapp
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
    restart: always
    deploy:
      replicas: 2

  db:
    image: postgres:15
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=myapp
    restart: always

  redis:
    image: redis:7-alpine
    restart: always

volumes:
  pgdata:
```

**Current Infrastructure:**
- 1 server: 8 vCPU, 32GB RAM, 500GB SSD
- Nginx reverse proxy for SSL termination
- Let's Encrypt certificates
- Daily database backups to S3
- Monitoring with basic health checks

## Tasks

### Part A: Convert Docker Compose to Kubernetes Manifests

Convert each service in the docker-compose.yml to appropriate Kubernetes resources. You need:

1. **Namespace** -- Create a namespace for the application
2. **Deployments** -- For frontend, api, and worker
3. **Services** -- ClusterIP for internal communication, NodePort or LoadBalancer for frontend
4. **ConfigMap** -- For non-sensitive environment variables
5. **Secret** -- For database credentials and sensitive data
6. **PersistentVolumeClaim** -- For PostgreSQL data
7. **StatefulSet** -- Decide if PostgreSQL should be a StatefulSet or use an external managed database

For each resource, write the YAML manifest. Include:
- Resource requests and limits
- Health checks (liveness and readiness probes)
- Appropriate labels and selectors

<details>
<summary>Hint 1</summary>

Start with the simplest resources first: Namespace, ConfigMap, Secret. Then create Deployments for stateless services (frontend, api, worker). Finally, handle PostgreSQL -- consider whether you want to run it in Kubernetes (StatefulSet) or use a managed database (RDS). Running databases in Kubernetes is possible but adds complexity.

</details>

<details>
<summary>Hint 2</summary>

For the Docker Compose `depends_on` equivalent in Kubernetes, use init containers that check for service availability, or rely on application-level retry logic. Kubernetes does not have a direct equivalent of `depends_on`.

</details>

### Part B: Migration Strategy

Design a phased migration plan with the following requirements:

1. **Zero downtime** -- Users should not experience any service interruption
2. **Rollback capability** -- Ability to revert to Docker Compose at any phase
3. **Data integrity** -- Database migration must not lose any data
4. **Testing** -- Each phase must be validated before proceeding

Define the phases:

```
Phase 1: [name]
  Goal: ?
  Steps: ?
  Validation: ?
  Rollback: ?
  Duration: ?

Phase 2: [name]
  ...
```

<details>
<summary>Hint 3</summary>

A common migration strategy is:
1. Set up Kubernetes cluster alongside existing infrastructure
2. Deploy stateless services to Kubernetes, keep database on Docker Compose
3. Migrate traffic gradually (canary or blue-green)
4. Migrate database last (most risky step)
5. Decompose Docker Compose infrastructure

Each phase should have a clear success criteria and rollback procedure.

</details>

### Part C: Database Migration Plan

PostgreSQL is the most critical and risky part of the migration. Create a detailed plan for migrating the database that addresses:

1. Should you run PostgreSQL in Kubernetes (StatefulSet) or use a managed service (RDS)?
2. How will you migrate the data with minimal downtime?
3. How will you handle the connection string change across all services?
4. What is your rollback procedure if the database migration fails?

Write a step-by-step plan with specific commands and procedures.

<details>
<summary>Hint 4</summary>

For production databases, managed services (RDS, Cloud SQL) are almost always the better choice. They handle backups, patching, failover, and replication. If you must run PostgreSQL in Kubernetes, use a StatefulSet with PersistentVolumes and understand that you are taking on significant operational burden.

For data migration, consider: pg_dump/pg_restore for small databases, logical replication for larger ones, or a read-replica approach where you replicate to the new database, switch traffic, then promote the replica.

</details>

### Part D: Networking and Ingress

Design the networking layer for Kubernetes:

1. How will external traffic reach the frontend?
2. How will internal services communicate (api -> db, api -> redis)?
3. How will you handle SSL/TLS termination?
4. How will you manage DNS cutover from the current server to Kubernetes?

Create the necessary Kubernetes networking resources:
- Ingress resource with TLS configuration
- Service definitions for internal communication
- Network policies (if applicable)

<details>
<summary>Hint 5</summary>

Use an Ingress controller (like nginx-ingress or traefik) for external traffic. SSL can be managed by cert-manager with Let's Encrypt. Internal services communicate via ClusterIP services using DNS names. For DNS cutover, lower TTL before migration, then switch the DNS record to the Kubernetes load balancer IP.

</details`

### Part E: Rollback and Disaster Recovery

Define rollback procedures for each migration phase and a disaster recovery plan:

1. **Phase rollback** -- How to revert each phase if it fails
2. **Full rollback** -- How to revert the entire migration to Docker Compose
3. **Data recovery** -- How to restore the database if corruption occurs during migration
4. **Monitoring** -- What metrics to watch during and after migration

Create a runbook with specific steps and commands.

<details>
<summary>Hint 6</summary>

Keep the Docker Compose infrastructure running in parallel until the migration is fully validated (at least 1-2 weeks). This gives you a safety net. Monitor error rates, response times, and database replication lag during migration. Set up alerts for anomalies.

</details>

## Success Criteria

- [ ] Part A includes valid Kubernetes YAML for all resources (Namespace, Deployments, Services, ConfigMap, Secret, PVC/StatefulSet).
- [ ] Resource requests and limits are set appropriately for each workload.
- [ ] Health checks (liveness and readiness probes) are configured for all services.
- [ ] The migration strategy in Part B has clear phases with validation criteria and rollback procedures.
- [ ] Part C addresses the database migration with a realistic, low-downtime approach.
- [ ] Part D includes working Ingress and Service configurations with TLS.
- [ ] Part E has specific, actionable rollback procedures (not vague instructions).
- [ ] The overall plan demonstrates understanding that database migration is the riskiest part.
- [ ] You can explain why running PostgreSQL in Kubernetes is harder than using a managed database.

## What You Should Understand After This Exercise

Migrating from Docker Compose to Kubernetes is not a single event -- it is a phased process that requires careful planning, especially for stateful components like databases. The key principles are: migrate stateless services first, keep the old infrastructure running in parallel, validate each phase before proceeding, and have a rollback plan at every step. The database migration is the highest-risk component and should be planned with the most care. After this exercise, you should be able to plan a real-world migration that minimizes risk and downtime.
