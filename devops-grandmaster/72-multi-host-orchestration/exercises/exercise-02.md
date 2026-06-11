# Exercise 02: Docker Swarm Stack Deployment

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

You will write a complete Docker Swarm stack definition that deploys a multi-service application across multiple hosts. This exercise practices overlay networking, placement constraints, resource limits, health checks, and secrets management.

## Scenario

You are deploying a microservices application called "shop" with the following services:

- **api** -- The API gateway, must be accessible on port 8080 from outside, 3 replicas, runs on worker nodes only.
- **catalog** -- Internal service that talks to a database, 2 replicas, requires SSD-labeled nodes.
- **redis** -- Cache layer, 1 replica, must persist data to a volume.
- **postgres** -- Database, 1 replica, must use a Docker secret for the password.

All services communicate over an internal overlay network called `shop-backend`. The `api` service also needs access to an external-facing network called `shop-frontend`.

## Tasks

### Part A: Write the Stack Definition

Write a `docker-compose.yml` file that defines this stack. Include:

- Service definitions with `deploy` blocks (replicas, placement constraints, resource limits, restart policy, update config).
- Two overlay networks: `shop-frontend` (attachable) and `shop-backend` (internal).
- A named volume for redis data.
- A Docker secret for the postgres password.
- Health checks for the `api` and `catalog` services.

<details>
<summary>Hint</summary>
Use `node.labels.ssd == true` for the catalog placement constraint and `node.role == worker` for the api constraint. The `deploy.resources` block uses `limits` and `reservations` with `cpus` and `memory`.
</details>

### Part B: Deployment Commands

Write the bash commands to:

1. Label a node as having SSD storage.
2. Deploy the stack.
3. Scale the `api` service to 5 replicas.
4. Verify all services are running.
5. View logs for the `catalog` service.

<details>
<summary>Hint</summary>
Use `docker node update --label-add` for labeling. The stack deploy command takes `-c` for the compose file and a stack name.
</details>

### Part C: Rolling Update Scenario

Write the command to update the `api` service image from `shop/api:v1` to `shop/api:v2` with a rolling update strategy: update 1 instance at a time, wait 30 seconds between updates, and automatically rollback if health checks fail within 60 seconds.

<details>
<summary>Hint</summary>
You can set update configuration in the `deploy.update_config` block or pass flags to `docker service update`. The `failure_action` can be set to `rollback`.
</details>

## Success Criteria

- [ ] Your compose file defines all 4 services with correct deploy blocks.
- [ ] Overlay networks are configured with the right drivers and visibility.
- [ ] The postgres service references a Docker secret.
- [ ] Health checks are defined for api and catalog with reasonable intervals and timeouts.
- [ ] Rolling update configuration includes parallelism, delay, failure_action, and monitor.

## What You Should Understand After This Exercise

Docker Swarm extends Docker Compose with production-grade features: placement constraints ensure services land on the right nodes, overlay networks enable cross-host communication, and the `deploy` block controls scaling, updates, and recovery behavior. The stack definition is the single source of truth for your multi-host deployment.
