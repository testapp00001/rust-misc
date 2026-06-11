# Solution 04: Design a Containerization Strategy

## Part A: Service Decomposition Analysis

### Dependency Matrix

| Module | Patient | Billing | Lab | Scheduling | Pharmacy | Reporting |
|--------|---------|---------|-----|------------|----------|-----------|
| Patient | - | R | R | S | R | R |
| Billing | W | - | R | S | R | R |
| Lab | W | W | - | S | R | R |
| Scheduling | S | S | S | - | R | R |
| Pharmacy | W | W | W | R | - | R |
| Reporting | R | R | R | R | R | - |

Key:
- **R** = reads data from
- **W** = writes data to
- **S** = shares a database table with

### Analysis

**Patient Records** and **Scheduling** share tables extensively -- patient
appointments are stored in shared tables. Separating them into different
containers requires either:
- A shared database (defeats the purpose of microservices)
- API calls between services (adds latency and complexity)
- Data duplication (consistency challenges)

**Recommendation:** Keep Patient Records and Scheduling in the same
container initially. They are tightly coupled and separating them adds
complexity without clear benefit.

**Billing** is moderately coupled to Patient Records (reads patient data
for invoices) but has its own data model. It can be separated once an
API boundary is defined.

**Lab Results** and **Pharmacy** are more independent -- they primarily
read from other services and have their own data.

**Reporting** reads from everyone. It should be a separate service that
consumes data through APIs or a read replica.

### Recommended Containerization Order

```
Phase 1 (tightly coupled, stay together):
  └── Patient Records + Scheduling (one container)

Phase 2 (extract when API boundary is clear):
  └── Billing (separate container)
  └── Lab Results (separate container)

Phase 3 (most independent):
  └── Pharmacy (separate container)
  └── Reporting (separate container, reads via APIs)
```

## Part B: Data Strategy

### 1. Should each service get its own database?

**No, not initially.** In the monolith, all modules share a single PostgreSQL
database with shared tables. Splitting this into 6 databases requires:
- Defining clear data ownership boundaries
- Migrating shared tables
- Implementing cross-service data access patterns
- Handling distributed transactions

**Recommended approach:**
- Phase 1: All containers connect to the same PostgreSQL database
- Phase 2: Introduce separate schemas per service
- Phase 3: Extract databases for fully independent services

### 2. How do you handle shared data?

The Patient Records table is read by Billing, Lab, Pharmacy, and Reporting.
Options:

| Approach | Complexity | Consistency | Performance |
|----------|-----------|-------------|-------------|
| Shared database | Low | Strong | Good |
| API calls | Medium | Eventual | Network latency |
| Event sourcing | High | Eventual | Good |
| Data replication | High | Eventual | Good (local reads) |

**Recommended:** Start with shared database, migrate to API calls as
services mature. The API approach allows Patient Records to own its data
and expose it through a controlled interface.

### 3. Patient data storage

Patient data **must not** be stored inside the container. Containers are
ephemeral -- when a container restarts, its filesystem is wiped.

```
Container (ephemeral)
├── Application code        ← Lives in the image
├── Runtime dependencies    ← Lives in the image
└── /data/patients          ← MUST be an external volume or managed database

External Storage (persistent)
├── PostgreSQL on dedicated server
├── Encrypted volume mounts
└── Backed up independently
```

**Critical rule:** Never store patient data in a container image or
container filesystem. Use external volumes or a managed database service.

## Part C: Compliance and Security

### Requirement 1: Patient data encrypted at rest

**Implementation:**
- Use PostgreSQL's built-in encryption: `pgcrypto` extension for column-level encryption
- Use LUKS or dm-crypt for disk-level encryption on the database server
- Container volumes can use encrypted storage backends (AWS EBS encryption,
  LUKS-encrypted volumes)

```sql
-- Column-level encryption in PostgreSQL
CREATE EXTENSION pgcrypto;

INSERT INTO patients (name, ssn_encrypted)
VALUES ('John Doe', pgp_sym_encrypt('123-45-6789', 'encryption_key'));
```

### Requirement 2: All access to patient records logged

**Implementation:**
- Application-level logging: Every database query on patient data is logged
- Audit trail table: Who accessed what, when
- Sidecar container: A logging sidecar collects and forwards logs to a
  centralized logging system

```yaml
# Logging sidecar concept (Docker Compose)
services:
  patient-records:
    image: medrecord/patient-api:1.0
    volumes:
      - log-volume:/var/log/app

  log-collector:
    image: fluent/fluentd
    volumes:
      - log-volume:/var/log/app:ro
    # Forwards logs to centralized system
```

### Requirement 3: No patient data in container images

**Implementation:**
- Multi-stage builds: Build stage has test data, production stage does not
- `.dockerignore`: Exclude test fixtures, sample data, seed files
- CI pipeline: Scan images for PII before pushing to registry

```dockerfile
# Multi-stage build
FROM python:3.11 AS builder
COPY . /app
RUN pip install -r /app/requirements.txt
# Run tests here (with test data)

FROM python:3.11-slim AS production
COPY --from=builder /app/src /app/src
COPY --from=builder /usr/local/lib/python3.11 /usr/local/lib/python3.11
# No test data, no fixtures, no sample databases
```

### Requirement 4: Containers run as non-root

**Implementation:**

```dockerfile
FROM python:3.11-slim

RUN groupadd -r medrecord && useradd -r -g medrecord medrecord

WORKDIR /app
COPY --chown=medrecord:medrecord . .

USER medrecord

CMD ["python3", "app.py"]
```

### Requirement 5: Network traffic between services encrypted

**Implementation:**
- mTLS (mutual TLS) between services
- Service mesh (Istio, Linkerd) handles encryption automatically
- Or use TLS at the application level

```
[Patient API] --mTLS--> [Billing API]
      │                      │
      └──mTLS--> [Lab API] <-┘
```

### Requirement 6: Container images scanned for vulnerabilities

**Implementation:**
- CI pipeline step: Scan every image before pushing to registry
- Tools: Trivy, Snyk, Anchore
- Policy: Block deployment if critical vulnerabilities found

```yaml
# CI pipeline step
- name: Scan image for vulnerabilities
  run: |
    trivy image --severity HIGH,CRITICAL medrecord/patient-api:latest
    if [ $? -ne 0 ]; then
      echo "Critical vulnerabilities found. Blocking deployment."
      exit 1
    fi
```

### Requirement 7: Secrets not in images or environment variables

**Implementation:**
- HashiCorp Vault: Secrets injected at runtime via sidecar or agent
- Kubernetes Secrets (if using K8s): Encrypted at rest, mounted as volumes
- AWS Secrets Manager / GCP Secret Manager: Retrieved at startup

```
Container startup flow:
  1. Container starts
  2. Vault agent sidecar authenticates
  3. Secrets mounted as files in /run/secrets/
  4. Application reads secrets from files
  5. Secrets never in image, env vars, or logs
```

**Why environment variables are problematic:**
- Visible in `docker inspect`
- Leaked in crash dumps and log files
- Passed through to child processes
- Visible in `/proc/<pid>/environ`

## Part D: Migration Plan

### Strangler Fig Pattern -- Phased Migration

```
Phase 0: Preparation (Weeks 1-2)
══════════════════════════════════
  Milestone: Monolith runs in a container
  
  1. Create Dockerfile for the monolith (lift-and-shift)
  2. Set up container registry
  3. Set up CI/CD pipeline for container builds
  4. Deploy containerized monolith to staging
  5. Validate: containerized monolith passes all tests
  
  Risk: Subtle behavior differences between bare metal and container
  Mitigation: Run both in parallel for 1 week, compare behavior

Phase 1: API Gateway (Weeks 3-4)
══════════════════════════════════
  Milestone: All traffic goes through API gateway
  
  1. Deploy API gateway (nginx, Kong, or Traefik)
  2. Route all traffic: Gateway -> Monolith container
  3. No behavior change -- gateway is transparent
  4. Validate: All endpoints work through gateway
  
  Risk: Gateway adds latency
  Mitigation: Measure latency, tune configuration

Phase 2: Extract Reporting (Weeks 5-8)
═══════════════════════════════════════
  Milestone: Reporting runs as independent container
  
  Why first: Reporting only READS from other modules (lowest coupling)
  
  1. Create Reporting container with API client
  2. Route reporting endpoints: Gateway -> Reporting container
  3. All other traffic: Gateway -> Monolith
  4. Monitor for errors
  
  Risk: API calls slower than direct DB queries
  Mitigation: Add caching layer for reporting queries

Phase 3: Extract Pharmacy (Weeks 9-12)
═══════════════════════════════════════
  Milestone: Pharmacy runs as independent container
  
  1. Create Pharmacy container
  2. Define API boundary for patient data access
  3. Route pharmacy endpoints: Gateway -> Pharmacy container
  4. Monitor for errors
  
  Risk: Pharmacy needs real-time patient data
  Mitigation: Implement event-driven updates or synchronous API calls

Phase 4: Extract Lab Results (Weeks 13-18)
══════════════════════════════════════════
  Milestone: Lab Results runs as independent container
  
  1. Create Lab Results container
  2. Migrate lab-specific tables to Lab service
  3. Route lab endpoints: Gateway -> Lab container
  4. Monitor for errors
  
  Risk: Lab data used by multiple other services
  Mitigation: Lab service exposes read API for other services

Phase 5: Extract Billing (Weeks 19-22)
═══════════════════════════════════════
  Milestone: Billing runs as independent container
  
  1. Create Billing container
  2. Billing reads patient data via Patient API
  3. Route billing endpoints: Gateway -> Billing container
  4. Monitor for errors
  
  Risk: Billing accuracy depends on patient data consistency
  Mitigation: Implement reconciliation job to verify data consistency

Phase 6: Final Decomposition (Weeks 23-26)
══════════════════════════════════════════
  Milestone: Patient Records + Scheduling split into separate containers
  
  1. Define API boundary between Patient and Scheduling
  2. Migrate shared tables to respective services
  3. Route all remaining traffic to new containers
  4. Decommission monolith container
  
  Risk: Highest coupling -- most likely to cause issues
  Mitigation: Extensive integration testing, canary deployment

Ongoing: Security Hardening (Throughout)
══════════════════════════════════════════
  - Image scanning in CI (Phase 0)
  - Non-root containers (Phase 0)
  - mTLS between services (Phase 1)
  - Secrets management (Phase 1)
  - Audit logging (Phase 2)
```

### Timeline Summary

```
Week:  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26
       ├──────┤
       Phase 0
             ├────┤
             Phase 1
                   ├──────────┤
                   Phase 2
                            ├──────────┤
                            Phase 3
                                     ├────────────────┤
                                     Phase 4
                                                        ├──────────┤
                                                        Phase 5
                                                                   ├──────────────┤
                                                                   Phase 6
```

### Common Mistakes to Avoid

- **Big bang migration.** Do not try to rewrite everything at once. The
  Strangler Fig pattern lets you migrate incrementally with rollback at
  every step.
- **Starting with the most coupled modules.** Extract the easiest (most
  independent) services first to build confidence and tooling.
- **Ignoring the database.** The hardest part of microservices is data
  decomposition. Plan this carefully before extracting services.
- **Forgetting about monitoring.** You cannot manage what you cannot measure.
  Set up logging and monitoring before extracting the first service.
- **Not having a rollback plan.** Every phase must be reversible. If the
  extracted service has issues, route traffic back to the monolith.
