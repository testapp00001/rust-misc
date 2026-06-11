# Exercise 02: Consul Service Registration

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

You will register services with HashiCorp Consul, configure health checks, query the service catalog, and use Consul's DNS interface for discovery.

## Scenario

You are deploying a microservices application with three services:

- **api** -- The API gateway on port 8080, tagged as `v2` and `production`.
- **payment** -- Payment processing on port 9090, tagged as `v1` and `production`.
- **notification** -- Email/SMS service on port 3000, tagged as `v1` and `staging`.

Each service has a `/health` endpoint. You need to register them with Consul so they can discover each other.

## Tasks

### Part A: Service Definitions

Write the Consul service definition JSON files for all three services. Each file should include:

- Service name, ID, port, and tags
- An HTTP health check with appropriate interval and timeout
- Metadata (version, environment)
- `deregister_critical_service_after` set to 30 minutes

<details>
<summary>Hint</summary>
Service definitions go in `/etc/consul.d/<service>.json`. The health check uses `"http"` type with `"interval"` and `"timeout"` fields.
</details>

### Part B: Service Registration via HTTP API

Write the curl commands to:

1. Register the `api` service dynamically via the Consul HTTP API (not via config file).
2. Deregister the `api` service.
3. List all healthy instances of `payment`.
4. Use blocking queries to long-poll for changes to the `notification` service.

<details>
<summary>Hint</summary>
The registration endpoint is `PUT /v1/agent/service/register`. Use `?passing=true` to filter healthy instances. Blocking queries use `?index=<last_index>&wait=5m`.
</details>

### Part C: DNS-Based Discovery

Write the `dig` commands to:

1. Resolve all instances of the `api` service using Consul's DNS interface (port 8600).
2. Get SRV records (including port and node information) for `payment`.
3. Filter by tag: find only `production` instances of `notification`.

<details>
<summary>Hint</summary>
Consul DNS format: `<service>.service.consul`. For tags: `<tag>.<service>.service.consul`. SRV records use `+vc` or `SRV` query type.
</details>

## Success Criteria

- [ ] Service definitions include all required fields (name, ID, port, tags, health check, metadata).
- [ ] HTTP API commands correctly register, deregister, and query services.
- [ ] DNS queries return correct results for service resolution and SRV records.
- [ ] You can explain the difference between config-file registration and HTTP API registration.

## What You Should Understand After This Exercise

Consul provides two registration methods: static (config files in `/etc/consul.d/`) for stable services and dynamic (HTTP API) for services that register themselves at startup. Health checks ensure only healthy instances are returned by queries. Consul's dual interface (HTTP API on port 8500 and DNS on port 8600) allows both programmatic and standard DNS-based discovery.
