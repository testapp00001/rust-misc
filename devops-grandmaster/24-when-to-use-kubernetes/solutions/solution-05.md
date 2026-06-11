# Solution 05: Docker Compose to Kubernetes Migration Plan

## Part A: Convert Docker Compose to Kubernetes Manifests

### Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: taskflow
  labels:
    app: taskflow
```

### Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: taskflow-secrets
  namespace: taskflow
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:password@taskflow-db:5432/myapp"
  REDIS_URL: "redis://taskflow-redis:6379"
  POSTGRES_USER: "user"
  POSTGRES_PASSWORD: "password"
  POSTGRES_DB: "myapp"
```

**Note:** In production, use a secrets management solution (AWS Secrets Manager, HashiCorp Vault, or Sealed Secrets) instead of storing credentials directly in Kubernetes Secrets.

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: taskflow-config
  namespace: taskflow
data:
  API_URL: "http://taskflow-api:4000"
  WORKER_CONCURRENCY: "5"
```

### Frontend Deployment and Service

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: taskflow-frontend
  namespace: taskflow
  labels:
    app: taskflow
    component: frontend
spec:
  replicas: 2
  selector:
    matchLabels:
      app: taskflow
      component: frontend
  template:
    metadata:
      labels:
        app: taskflow
        component: frontend
    spec:
      containers:
        - name: frontend
          image: myapp/frontend:latest
          ports:
            - containerPort: 3000
          env:
            - name: API_URL
              valueFrom:
                configMapKeyRef:
                  name: taskflow-config
                  key: API_URL
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 250m
              memory: 256Mi
          livenessProbe:
            httpGet:
              path: /
              port: 3000
            initialDelaySeconds: 10
            periodSeconds: 15
          readinessProbe:
            httpGet:
              path: /
              port: 3000
            initialDelaySeconds: 5
            periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: taskflow-frontend
  namespace: taskflow
  labels:
    app: taskflow
    component: frontend
spec:
  type: ClusterIP
  selector:
    app: taskflow
    component: frontend
  ports:
    - port: 3000
      targetPort: 3000
      protocol: TCP
```

### API Deployment and Service

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: taskflow-api
  namespace: taskflow
  labels:
    app: taskflow
    component: api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: taskflow
      component: api
  template:
    metadata:
      labels:
        app: taskflow
        component: api
    spec:
      containers:
        - name: api
          image: myapp/api:latest
          ports:
            - containerPort: 4000
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: DATABASE_URL
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: REDIS_URL
            - name: WORKER_CONCURRENCY
              valueFrom:
                configMapKeyRef:
                  name: taskflow-config
                  key: WORKER_CONCURRENCY
          resources:
            requests:
              cpu: 250m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            httpGet:
              path: /health
              port: 4000
            initialDelaySeconds: 15
            periodSeconds: 20
          readinessProbe:
            httpGet:
              path: /health
              port: 4000
            initialDelaySeconds: 10
            periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: taskflow-api
  namespace: taskflow
  labels:
    app: taskflow
    component: api
spec:
  type: ClusterIP
  selector:
    app: taskflow
    component: api
  ports:
    - port: 4000
      targetPort: 4000
      protocol: TCP
```

### Worker Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: taskflow-worker
  namespace: taskflow
  labels:
    app: taskflow
    component: worker
spec:
  replicas: 2
  selector:
    matchLabels:
      app: taskflow
      component: worker
  template:
    metadata:
      labels:
        app: taskflow
        component: worker
    spec:
      containers:
        - name: worker
          image: myapp/api:latest
          command: ["node", "worker.js"]
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: DATABASE_URL
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: REDIS_URL
          resources:
            requests:
              cpu: 250m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            exec:
              command:
                - node
                - -e
                - "process.exit(0)"
            initialDelaySeconds: 15
            periodSeconds: 30
```

### PostgreSQL StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: taskflow-db
  namespace: taskflow
  labels:
    app: taskflow
    component: database
spec:
  serviceName: taskflow-db
  replicas: 1
  selector:
    matchLabels:
      app: taskflow
      component: database
  template:
    metadata:
      labels:
        app: taskflow
        component: database
    spec:
      containers:
        - name: postgres
          image: postgres:15
          ports:
            - containerPort: 5432
          env:
            - name: POSTGRES_USER
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: POSTGRES_USER
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: POSTGRES_PASSWORD
            - name: POSTGRES_DB
              valueFrom:
                secretKeyRef:
                  name: taskflow-secrets
                  key: POSTGRES_DB
            - name: PGDATA
              value: /var/lib/postgresql/data/pgdata
          resources:
            requests:
              cpu: 250m
              memory: 512Mi
            limits:
              cpu: 1000m
              memory: 1Gi
          volumeMounts:
            - name: pgdata
              mountPath: /var/lib/postgresql/data
          livenessProbe:
            exec:
              command:
                - pg_isready
                - -U
                - user
                - -d
                - myapp
            initialDelaySeconds: 30
            periodSeconds: 15
          readinessProbe:
            exec:
              command:
                - pg_isready
                - -U
                - user
                - -d
                - myapp
            initialDelaySeconds: 10
            periodSeconds: 10
  volumeClaimTemplates:
    - metadata:
        name: pgdata
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 20Gi
---
apiVersion: v1
kind: Service
metadata:
  name: taskflow-db
  namespace: taskflow
  labels:
    app: taskflow
    component: database
spec:
  type: ClusterIP
  selector:
    app: taskflow
    component: database
  ports:
    - port: 5432
      targetPort: 5432
      protocol: TCP
```

### Redis Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: taskflow-redis
  namespace: taskflow
  labels:
    app: taskflow
    component: redis
spec:
  replicas: 1
  selector:
    matchLabels:
      app: taskflow
      component: redis
  template:
    metadata:
      labels:
        app: taskflow
        component: redis
    spec:
      containers:
        - name: redis
          image: redis:7-alpine
          ports:
            - containerPort: 6379
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 250m
              memory: 256Mi
          livenessProbe:
            exec:
              command:
                - redis-cli
                - ping
            initialDelaySeconds: 10
            periodSeconds: 15
          readinessProbe:
            exec:
              command:
                - redis-cli
                - ping
            initialDelaySeconds: 5
            periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: taskflow-redis
  namespace: taskflow
  labels:
    app: taskflow
    component: redis
spec:
  type: ClusterIP
  selector:
    app: taskflow
    component: redis
  ports:
    - port: 6379
      targetPort: 6379
      protocol: TCP
```

---

## Part B: Migration Strategy

### Phase 1: Kubernetes Cluster Setup (Week 1)

**Goal:** Provision and configure the Kubernetes cluster alongside existing infrastructure.

**Steps:**
1. Provision EKS cluster (or GKE/AKS) with 3 worker nodes
2. Install and configure kubectl
3. Install nginx-ingress controller
4. Install cert-manager for SSL
5. Configure monitoring (Prometheus + Grafana)
6. Deploy all manifests to a staging namespace for validation

**Validation:**
- All Kubernetes nodes are healthy
- Ingress controller responds to health checks
- Monitoring dashboards show cluster metrics
- All manifests deploy successfully in staging

**Rollback:** Delete the Kubernetes cluster. No impact on existing Docker Compose infrastructure.

**Duration:** 3-5 days

---

### Phase 2: Deploy Stateless Services (Week 2)

**Goal:** Deploy frontend, API, and worker to Kubernetes while keeping the database on Docker Compose.

**Steps:**
1. Update DATABASE_URL in Kubernetes Secret to point to the existing database server (external IP)
2. Deploy frontend, API, and worker to Kubernetes production namespace
3. Configure Ingress with temporary domain (e.g., staging.taskflow.com)
4. Run integration tests against Kubernetes-deployed services
5. Verify all services can connect to the database on the existing server

**Validation:**
- All pods are running and healthy
- API responds correctly to requests
- Worker processes jobs from the queue
- Database connections from Kubernetes pods work
- No errors in application logs

**Rollback:** Delete Kubernetes deployments. Traffic continues to flow through Docker Compose.

**Duration:** 3-5 days

---

### Phase 3: Traffic Migration (Week 3)

**Goal:** Gradually shift traffic from Docker Compose to Kubernetes.

**Steps:**
1. Configure DNS with low TTL (60 seconds) before migration
2. Deploy a load balancer or reverse proxy that can split traffic
3. Start with 10% traffic to Kubernetes, 90% to Docker Compose
4. Monitor error rates, response times, and database connection counts
5. Gradually increase: 10% -> 25% -> 50% -> 75% -> 100%
6. At each step, validate metrics before proceeding

**Validation:**
- Error rate does not increase
- Response times are comparable or better
- Database connections are stable
- No data inconsistencies between the two environments

**Rollback:** Route 100% traffic back to Docker Compose. DNS TTL allows quick propagation.

**Duration:** 5-7 days (gradual rollout)

---

### Phase 4: Database Migration (Week 4)

**Goal:** Migrate the database from Docker Compose to a managed service (RDS).

**Steps:**
1. Provision RDS PostgreSQL instance (same version: PostgreSQL 15)
2. Configure RDS security group to allow connections from Kubernetes worker nodes
3. Create a database dump from the existing PostgreSQL:
   ```bash
   pg_dump -h <current-db-host> -U user -d myapp -F c -f taskflow_backup.dump
   ```
4. Restore the dump to RDS:
   ```bash
   pg_restore -h <rds-endpoint> -U user -d myapp -Fc taskflow_backup.dump
   ```
5. Verify data integrity (row counts, checksums)
6. Set up logical replication from old DB to RDS for zero-downtime cutover:
   ```sql
   -- On old database
   ALTER SYSTEM SET wal_level = logical;
   -- Create replication slot
   SELECT pg_create_logical_replication_slot('taskflow_slot', 'pgoutput');
   -- Create publication
   CREATE PUBLICATION taskflow_pub FOR ALL TABLES;
   
   -- On RDS
   CREATE SUBSCRIPTION taskflow_sub
   CONNECTION 'host=<old-db> port=5432 dbname=myapp user=user password=password'
   PUBLICATION taskflow_pub;
   ```
7. Wait for replication to catch up (monitor lag)
8. During a maintenance window (low traffic period):
   - Stop writes to old database (put application in read-only mode)
   - Verify replication lag is zero
   - Update Kubernetes Secret to point DATABASE_URL to RDS
   - Restart API and worker pods
   - Verify application works with RDS
   - Resume writes

**Validation:**
- Row counts match between old and new database
- Application connects to RDS successfully
- No data loss during cutover
- Application performance is comparable

**Rollback:** If replication fails or data integrity issues occur:
1. Update Kubernetes Secret to point back to old database
2. Restart API and worker pods
3. Verify application works
4. Investigate the issue before retrying

**Duration:** 3-5 days (including replication setup and monitoring)

---

### Phase 5: Decommission Docker Compose (Week 6-8)

**Goal:** Shut down the Docker Compose infrastructure after validating Kubernetes stability.

**Steps:**
1. Keep Docker Compose infrastructure running for 2 weeks after 100% traffic migration
2. Monitor Kubernetes for any issues (error rates, performance, stability)
3. After 2 weeks of stable operation:
   - Stop Docker Compose services
   - Take a final backup of the old database
   - Keep the old server available (but stopped) for 1 additional month
4. After 1 month, terminate the old server

**Validation:**
- 2 weeks of stable Kubernetes operation
- No need to roll back to Docker Compose
- All monitoring and alerting is functional

**Rollback:** If issues arise during the 2-week validation, restart Docker Compose infrastructure and route traffic back.

**Duration:** 2-4 weeks (mostly waiting and monitoring)

---

## Part C: Database Migration Plan

### Recommendation: Use RDS (Managed Database)

**Why not PostgreSQL in Kubernetes (StatefulSet)?**

Running PostgreSQL in Kubernetes is possible but adds significant complexity:

1. **Persistent Volumes:** Kubernetes PersistentVolumes can fail. If the PV becomes unavailable, your database is down.
2. **Backups:** You must configure and manage your own backup solution. RDS provides automated backups with point-in-time recovery.
3. **Failover:** RDS Multi-AZ provides automatic failover. In Kubernetes, you must configure Patroni or similar tools yourself.
4. **Patching:** RDS handles PostgreSQL version patches automatically. In Kubernetes, you must manage this yourself.
5. **Scaling:** RDS supports vertical scaling with minimal downtime. In Kubernetes, you must manually resize the StatefulSet and PVC.

**The only reason to run PostgreSQL in Kubernetes is if you need full control over the database configuration or if you are running on-premises without a managed database option.**

### Data Migration Procedure

**Step 1: Provision RDS**
```bash
aws rds create-db-instance \
  --db-instance-identifier taskflow-db \
  --db-instance-class db.t3.medium \
  --engine postgres \
  --engine-version 15 \
  --master-username user \
  --master-user-password <password> \
  --allocated-storage 20 \
  --vpc-security-group-ids <sg-id> \
  --db-subnet-group-name <subnet-group> \
  --backup-retention-period 7 \
  --multi-az
```

**Step 2: Dump and Restore**
```bash
# Dump from existing database
pg_dump -h <current-host> -U user -d myapp -F c -f taskflow.dump

# Restore to RDS
pg_restore -h <rds-endpoint> -U user -d myapp -Fc --no-owner taskflow.dump
```

**Step 3: Verify Data Integrity**
```sql
-- Compare row counts
SELECT 'users' as table, COUNT(*) FROM users
UNION ALL
SELECT 'tasks', COUNT(*) FROM tasks
UNION ALL
SELECT 'projects', COUNT(*) FROM projects;
```

**Step 4: Connection String Update**
```yaml
# Update the Kubernetes Secret
apiVersion: v1
kind: Secret
metadata:
  name: taskflow-secrets
  namespace: taskflow
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:password@taskflow-db.xxxxx.us-east-1.rds.amazonaws.com:5432/myapp"
```

**Step 5: Rolling Restart**
```bash
# Restart API pods to pick up new secret
kubectl rollout restart deployment/taskflow-api -n taskflow

# Restart worker pods
kubectl rollout restart deployment/taskflow-worker -n taskflow
```

### Rollback Procedure for Database Migration

If the database migration fails at any point:

1. **Before cutover:** Delete the RDS instance. No impact on production.
2. **During cutover (if replication fails):** Update Kubernetes Secret to point back to old database. Restart pods.
3. **After cutover (if data issues found):** Restore from the pre-migration dump to the old database, update Kubernetes Secret, restart pods.

---

## Part D: Networking and Ingress

### Ingress Configuration

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: taskflow-ingress
  namespace: taskflow
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - taskflow.com
        - www.taskflow.com
      secretName: taskflow-tls
  rules:
    - host: taskflow.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: taskflow-frontend
                port:
                  number: 3000
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: taskflow-api
                port:
                  number: 4000
    - host: www.taskflow.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: taskflow-frontend
                port:
                  number: 3000
```

### Cert-Manager ClusterIssuer

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@taskflow.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
      - http01:
          ingress:
            class: nginx
```

### Internal Service Communication

Services communicate via Kubernetes DNS:
- Frontend connects to API: `http://taskflow-api:4000`
- API connects to Database: `postgresql://user:password@taskflow-db:5432/myapp`
- API connects to Redis: `redis://taskflow-redis:6379`
- Worker connects to Database: `postgresql://user:password@taskflow-db:5432/myapp`
- Worker connects to Redis: `redis://taskflow-redis:6379`

### DNS Cutover Procedure

1. **Pre-migration:** Lower DNS TTL to 60 seconds (24-48 hours before cutover)
2. **During migration:**
   - Create a CNAME record pointing to the Kubernetes Ingress load balancer
   - Or update the A record to point to the load balancer IP
3. **Post-migration:** Monitor DNS propagation and verify traffic flows to Kubernetes
4. **Rollback:** Update DNS to point back to the Docker Compose server

---

## Part E: Rollback and Disaster Recovery

### Phase Rollback Procedures

**Phase 1 Rollback (Cluster Setup):**
```bash
# Delete the EKS cluster
aws eks delete-cluster --name taskflow-cluster
# No impact on existing infrastructure
```

**Phase 2 Rollback (Stateless Services):**
```bash
# Delete Kubernetes deployments
kubectl delete namespace taskflow
# Traffic continues to Docker Compose (no DNS changes made)
```

**Phase 3 Rollback (Traffic Migration):**
```bash
# Update DNS to point back to Docker Compose server
# With 60-second TTL, propagation takes < 5 minutes
aws route53 change-resource-record-sets --hosted-zone-id <zone-id> --change-batch '{
  "Changes": [{
    "Action": "UPSERT",
    "ResourceRecordSet": {
      "Name": "taskflow.com",
      "Type": "A",
      "TTL": 60,
      "ResourceRecords": [{"Value": "<old-server-ip>"}]
    }
  }]
}'
```

**Phase 4 Rollback (Database Migration):**
```bash
# Update Kubernetes Secret to point to old database
kubectl create secret generic taskflow-secrets \
  --from-literal=DATABASE_URL="postgresql://user:password@<old-db-host>:5432/myapp" \
  -n taskflow --dry-run=client -o yaml | kubectl apply -f -

# Restart pods
kubectl rollout restart deployment -n taskflow

# Delete RDS instance (if no longer needed)
aws rds delete-db-instance --db-instance-identifier taskflow-db --skip-final-snapshot
```

### Disaster Recovery

**Database Corruption During Migration:**
1. Stop all writes (put application in maintenance mode)
2. Restore from pre-migration dump:
   ```bash
   pg_restore -h <rds-endpoint> -U user -d myapp -Fc --clean taskflow.dump
   ```
3. Verify data integrity
4. Resume operations

**Kubernetes Cluster Failure:**
1. Route DNS back to Docker Compose infrastructure (keep it running during validation period)
2. Investigate cluster failure
3. If unrecoverable, provision a new cluster and redeploy

**Complete Kubernetes Failure (worst case):**
1. The Docker Compose infrastructure is kept running for 2+ weeks after migration
2. Route DNS back to Docker Compose
3. Update DATABASE_URL in Docker Compose to point to RDS (if database was migrated)
4. Resume operations on Docker Compose
5. Investigate and fix Kubernetes issues before re-attempting migration

### Monitoring During Migration

**Key Metrics to Watch:**
- Error rate (5xx responses)
- Response time (p50, p95, p99)
- Database connection count
- Database replication lag (during Phase 4)
- Pod restart count
- Node CPU/memory utilization

**Alerts to Configure:**
- Error rate > 1% for 5 minutes
- Response time p95 > 500ms for 5 minutes
- Database connection count > 80% of max
- Replication lag > 30 seconds
- Pod restart count > 3 in 10 minutes

---

## Common Mistakes to Avoid

- **Migrating the database first.** The database is the riskiest component. Migrate stateless services first, validate, then tackle the database.
- **Not keeping Docker Compose running.** Always have a rollback path. Keep the old infrastructure running for at least 2 weeks after migration.
- **Skipping the gradual traffic rollout.** A sudden cutover has no safety net. Gradual rollout (10% -> 25% -> 50% -> 100%) catches issues early.
- **Running PostgreSQL in Kubernetes without a strong reason.** Managed databases (RDS, Cloud SQL) handle backups, failover, and patching. Only run databases in Kubernetes if you have no other option.
- **Not lowering DNS TTL before migration.** If DNS TTL is 3600 seconds (1 hour), rollback takes up to 1 hour to propagate. Lower TTL to 60 seconds before migration.
- **Ignoring the `depends_on` equivalent.** Kubernetes does not have `depends_on`. Use init containers or application-level retry logic to handle service startup order.

## Key Takeaway

Migrating from Docker Compose to Kubernetes is a phased process that requires careful planning, especially for stateful components like databases. The key principles are: migrate stateless services first, keep the old infrastructure running in parallel, validate each phase before proceeding, and have a rollback plan at every step. The database migration is the highest-risk component and should be planned with the most care. Use managed databases (RDS) instead of running databases in Kubernetes unless you have a specific reason not to.
