# Solution 03: Create a CronJob for Database Backups

## Complete Solution

### backup-script-configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: backup-script
  namespace: backups
data:
  backup.sh: |
    #!/bin/bash
    set -e

    # Configuration
    DB_HOST="${DB_HOST:-postgres.default.svc.cluster.local}"
    DB_PORT="${DB_PORT:-5432}"
    DB_NAME="${DB_NAME:-mydb}"
    DB_USER="${DB_USER:-postgres}"
    BACKUP_DIR="/backups"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_FILE="${BACKUP_DIR}/${DB_NAME}_${TIMESTAMP}.sql.gz"
    RETENTION_DAYS="${RETENTION_DAYS:-7}"

    # Create backup directory if it doesn't exist
    mkdir -p "${BACKUP_DIR}"

    echo "Starting backup of ${DB_NAME} at $(date)"
    echo "Backup file: ${BACKUP_FILE}"

    # Create the backup
    pg_dump -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" | gzip > "${BACKUP_FILE}"

    # Check if backup was successful
    if [ $? -eq 0 ]; then
      echo "Backup completed successfully: ${BACKUP_FILE}"
      echo "Backup size: $(du -h ${BACKUP_FILE} | cut -f1)"
    else
      echo "Backup failed!"
      exit 1
    fi

    # List current backups
    echo "Current backups:"
    ls -lh "${BACKUP_DIR}"/*.sql.gz 2>/dev/null || echo "No backups found"

    # Clean up old backups
    echo "Cleaning up backups older than ${RETENTION_DAYS} days..."
    DELETED_COUNT=$(find "${BACKUP_DIR}" -name "*.sql.gz" -mtime +${RETENTION_DAYS} -delete -print | wc -l)
    echo "Deleted ${DELETED_COUNT} old backup(s)"

    # Verify backup integrity (optional)
    echo "Verifying backup integrity..."
    if gzip -t "${BACKUP_FILE}"; then
      echo "Backup integrity check passed"
    else
      echo "WARNING: Backup integrity check failed!"
      exit 1
    fi

    echo "Backup process completed at $(date)"
```

### backup-pvc.yaml

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: backup-storage
  namespace: backups
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  # Optional: specify storage class
  # storageClassName: standard
```

### db-secret.yaml

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
  namespace: backups
type: Opaque
data:
  # Base64 encoded values
  # echo -n "password123" | base64
  password: cGFzc3dvcmQxMjM=
  # echo -n "postgres" | base64
  username: cG9zdGdyZXM=
```

### postgres-backup-cronjob.yaml

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: backups
  labels:
    app: postgres-backup
spec:
  # Run daily at 2:00 AM UTC
  schedule: "0 2 * * *"

  # Don't run if previous job is still running
  concurrencyPolicy: Forbid

  # Keep history for debugging
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 5

  # Optional: timezone (Kubernetes 1.27+)
  # timeZone: "America/New_York"

  # Deadline for starting a missed job
  startingDeadlineSeconds: 600  # 10 minutes

  jobTemplate:
    metadata:
      labels:
        app: postgres-backup
    spec:
      # Timeout after 1 hour
      activeDeadlineSeconds: 3600

      # Retry up to 3 times on failure
      backoffLimit: 3

      # Clean up failed jobs after 7 days
      ttlSecondsAfterFinished: 604800

      template:
        metadata:
          labels:
            app: postgres-backup
        spec:
          # Don't restart pods; let Job handle retries
          restartPolicy: OnFailure

          # Use init container to wait for database
          initContainers:
            - name: wait-for-db
              image: busybox:1.36
              command:
                - sh
                - -c
                - |
                  echo "Waiting for database to be ready..."
                  until nc -z ${DB_HOST} ${DB_PORT}; do
                    echo "Database not ready, waiting..."
                    sleep 5
                  done
                  echo "Database is ready!"

          containers:
            - name: backup
              image: postgres:15
              command: ["/bin/bash", "/scripts/backup.sh"]

              env:
                - name: DB_HOST
                  value: "postgres.default.svc.cluster.local"
                - name: DB_PORT
                  value: "5432"
                - name: DB_NAME
                  value: "mydb"
                - name: DB_USER
                  value: "postgres"
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: db-credentials
                      key: password
                - name: RETENTION_DAYS
                  value: "7"

              resources:
                requests:
                  cpu: 100m
                  memory: 128Mi
                limits:
                  cpu: 500m
                  memory: 512Mi

              volumeMounts:
                - name: backup-storage
                  mountPath: /backups
                - name: backup-script
                  mountPath: /scripts

          volumes:
            - name: backup-storage
              persistentVolumeClaim:
                claimName: backup-storage
            - name: backup-script
              configMap:
                name: backup-script
                defaultMode: 0755  # Make script executable
```

## Why This Solution Works

### 1. Proper Schedule Configuration

The `schedule: "0 2 * * *"` uses standard cron syntax:
- `0`: Minute 0
- `2`: Hour 2 (2:00 AM)
- `*`: Every day of month
- `*`: Every month
- `*`: Every day of week

This ensures backups run at a consistent time when database load is typically lowest.

### 2. Concurrency Policy Prevents Overlapping Runs

`concurrencyPolicy: Forbid` ensures:
- If a backup is still running at 2:00 AM the next day, the new backup is skipped
- Prevents resource conflicts and potential corruption
- Safer than `Allow` (which could run multiple backups simultaneously)

### 3. History Limits for Debugging

```yaml
successfulJobsHistoryLimit: 3  # Keep last 3 successful backups
failedJobsHistoryLimit: 5      # Keep last 5 failed attempts
```

This provides:
- Enough history to verify backups are working
- Not too many old jobs cluttering the namespace
- More failed job history for debugging issues

### 4. Backoff Limit for Resilience

`backoffLimit: 3` means:
- If a backup fails, it will retry up to 3 times
- Each retry uses exponential backoff (10s, 20s, 40s)
- After 3 failures, the job is marked as failed
- Prevents infinite retries on persistent failures

### 5. Resource Limits Prevent Node Issues

```yaml
resources:
  requests:
    cpu: 100m      # Guaranteed CPU
    memory: 128Mi  # Guaranteed memory
  limits:
    cpu: 500m      # Maximum CPU
    memory: 512Mi  # Maximum memory
```

This prevents:
- Backup jobs consuming all node resources
- OOM kills during large database backups
- CPU starvation for other workloads

### 6. Init Container for Database Readiness

The `wait-for-db` init container:
- Waits for the database to be ready before starting backup
- Prevents immediate failures if database is temporarily unavailable
- Uses simple `nc` (netcat) to check TCP connectivity

### 7. Secret for Secure Credentials

Database credentials are stored in a Secret:
- Not hardcoded in the script
- Mounted as environment variable
- Base64 encoded (note: not encrypted; use external secret management in production)

### 8. Persistent Storage for Backups

The PVC provides:
- Durable storage that survives pod restarts
- Consistent mount path for backup files
- Easy backup retrieval and management

## Verification Steps

### 1. Create a Test Job

```bash
# Create a job from the CronJob template
kubectl create job --from=cronjob/postgres-backup postgres-backup-test -n backups

# Watch the job
kubectl get jobs -n backups -w
```

### 2. Check Job Status

```bash
# Get job details
kubectl describe job postgres-backup-test -n backups

# Check pod status
kubectl get pods -n backups -l job-name=postgres-backup-test
```

### 3. View Backup Logs

```bash
# Get pod logs
kubectl logs -n backups job/postgres-backup-test

# Expected output:
# Starting backup of mydb at Thu Jun 11 02:00:01 UTC 2026
# Backup file: /backups/mydb_20260611_020001.sql.gz
# Backup completed successfully: /backups/mydb_20260611_020001.sql.gz
# Backup size: 15M
# Current backups:
# -rw-r--r-- 1 root root 15M Jun 11 02:00 /backups/mydb_20260611_020001.sql.gz
# Cleaning up backups older than 7 days...
# Deleted 0 old backup(s)
# Verifying backup integrity...
# Backup integrity check passed
# Backup process completed at Thu Jun 11 02:00:15 UTC 2026
```

### 4. Verify Backup File

```bash
# Check PVC is bound
kubectl get pvc backup-storage -n backups

# List backups (exec into a pod with PVC mounted)
kubectl run -it --rm debug --image=busybox --restart=Never -n backups \
  --overrides='{"spec":{"containers":[{"name":"debug","image":"busybox","command":["sh"],"volumeMounts":[{"name":"backup-storage","mountPath":"/backups"}]}],"volumes":[{"name":"backup-storage","persistentVolumeClaim":{"claimName":"backup-storage"}}]}}' \
  -- ls -lh /backups/
```

### 5. Check CronJob Schedule

```bash
# View CronJob details
kubectl describe cronjob postgres-backup -n backups

# Check next scheduled run
kubectl get cronjob postgres-backup -n backups
```

## Common Mistakes to Avoid

### 1. Not Making Script Executable

**Mistake:** Forgetting `defaultMode: 0755` on ConfigMap volume.

**Problem:** `permission denied` when trying to run the script.

**Solution:** Always set executable permissions:
```yaml
volumes:
  - name: backup-script
    configMap:
      name: backup-script
      defaultMode: 0755
```

### 2. Using Default RestartPolicy

**Mistake:** Using `restartPolicy: Always` (default).

**Problem:** Pods restart indefinitely on failure, wasting resources.

**Solution:** Use `restartPolicy: OnFailure` or `restartPolicy: Never` and let the Job handle retries.

### 3. Not Setting activeDeadlineSeconds

**Mistake:** No deadline for job completion.

**Problem:** Jobs can hang indefinitely if they get stuck.

**Solution:** Always set a reasonable deadline:
```yaml
activeDeadlineSeconds: 3600  # 1 hour
```

### 4. Storing Passwords in Plain Text

**Mistake:** Hardcoding database password in script or ConfigMap.

**Problem:** Security risk; password visible in version control.

**Solution:** Use Kubernetes Secrets (or external secret management):
```yaml
env:
  - name: PGPASSWORD
    valueFrom:
      secretKeyRef:
        name: db-credentials
        key: password
```

### 5. Not Testing Before Production

**Mistake:** Deploying CronJob without testing.

**Problem:** Backup fails in production; data loss occurs.

**Solution:** Always test with a manual job first:
```bash
kubectl create job --from=cronjob/postgres-backup test-backup -n backups
```

### 6. Ignoring Retention Policy

**Mistake:** Not cleaning up old backups.

**Problem:** Disk fills up over time; storage costs increase.

**Solution:** Implement retention in backup script:
```bash
find "${BACKUP_DIR}" -name "*.sql.gz" -mtime +${RETENTION_DAYS} -delete
```

## Production Considerations

### 1. External Secret Management

Use external secret managers instead of Kubernetes Secrets:

- **AWS Secrets Manager**
- **HashiCorp Vault**
- **Azure Key Vault**
- **Google Secret Manager**

Example with External Secrets Operator:
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: db-credentials
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: db-credentials
  data:
    - secretKey: password
      remoteRef:
        key: database/creds
        property: password
```

### 2. Backup Encryption

Encrypt backups before storing:

```bash
# Generate encryption key
openssl rand -base64 32 > /backups/encryption.key

# Encrypt backup
gpg --batch --yes --passphrase-file /backups/encryption.key \
  --symmetric --output ${BACKUP_FILE}.gpg ${BACKUP_FILE}

# Remove unencrypted backup
rm ${BACKUP_FILE}
```

### 3. Offsite Backup Storage

Copy backups to offsite storage:

```bash
# Upload to S3
aws s3 cp ${BACKUP_FILE} s3://my-backups/postgres/

# Upload to GCS
gsutil cp ${BACKUP_FILE} gs://my-backups/postgres/

# Upload to Azure
az storage blob upload --file ${BACKUP_FILE} --container-name backups
```

### 4. Backup Verification

Automatically verify backup integrity:

```bash
# Test backup can be restored
pg_restore --list ${BACKUP_FILE} > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "Backup verification passed"
else
  echo "Backup verification failed!"
  exit 1
fi
```

### 5. Monitoring and Alerting

Monitor backup jobs and alert on failures:

```yaml
# Prometheus alert rules
groups:
  - name: backup-alerts
    rules:
      - alert: BackupJobFailed
        expr: kube_job_status_failed{job_name=~".*backup.*"} > 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Backup job failed"
          description: "Backup job {{ $labels.job_name }} has failed"
```

### 6. Database-Specific Considerations

For PostgreSQL specifically:

- Use `pg_dump` for single database
- Use `pg_dumpall` for all databases
- Consider `pg_basebackup` for physical backups
- Use `--format=custom` for flexible restore options

```bash
# Custom format (recommended)
pg_dump -Fc -f ${BACKUP_FILE} ${DB_NAME}

# Restore from custom format
pg_restore -d ${DB_NAME} ${BACKUP_FILE}
```
