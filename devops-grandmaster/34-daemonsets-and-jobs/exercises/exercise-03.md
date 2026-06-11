# Exercise 03: Create a CronJob for Database Backups

## Objective

Create a CronJob that automatically backs up a PostgreSQL database every day at 2:00 AM, stores the backup in a PersistentVolume, and retains backup history.

## Background

Database backups are critical for disaster recovery. A CronJob ensures backups run reliably on schedule without manual intervention. This exercise simulates a real-world backup strategy.

## Instructions

### Step 1: Create the Namespace and Resources

```bash
kubectl create namespace backups
```

### Step 2: Create a ConfigMap for Backup Script

Create a ConfigMap containing a backup script that:
1. Connects to a PostgreSQL database
2. Creates a timestamped backup
3. Compresses the backup
4. Logs the backup status
5. Cleans up backups older than 7 days

```yaml
# backup-script-configmap.yaml
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

    # Clean up old backups
    echo "Cleaning up backups older than ${RETENTION_DAYS} days..."
    find "${BACKUP_DIR}" -name "*.sql.gz" -mtime +${RETENTION_DAYS} -delete

    echo "Backup process completed at $(date)"
```

Create the ConfigMap:

```bash
kubectl apply -f backup-script-configmap.yaml
```

### Step 3: Create a PersistentVolumeClaim

Create a PVC to store backups:

```yaml
# backup-pvc.yaml
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
```

### Step 4: Create the CronJob

Create a CronJob that:
1. Runs daily at 2:00 AM UTC
2. Uses the backup script from the ConfigMap
3. Mounts the PVC for backup storage
4. Has appropriate resource limits
5. Keeps a history of successful and failed jobs
6. Doesn't run if a previous job is still running

Your CronJob should:
- Name: `postgres-backup`
- Schedule: `0 2 * * *` (2:00 AM daily)
- Image: `postgres:15` (includes pg_dump)
- Successful jobs history limit: 3
- Failed jobs history limit: 5
- Concurrency policy: `Forbid`
- Restart policy: `OnFailure`
- Active deadline: 3600 seconds (1 hour)
- Backoff limit: 3

### Step 5: Create a Secret for Database Credentials

Create a Secret for the database password:

```yaml
# db-secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
  namespace: backups
type: Opaque
data:
  password: cGFzc3dvcmQxMjM=  # base64 encoded "password123"
```

### Step 6: Test the CronJob

1. Create a test Job from the CronJob template to verify it works
2. Check the job logs to ensure the backup script runs correctly
3. Verify the backup file is created in the PVC

## Deliverables

Create the following files:
1. `backup-script-configmap.yaml` - The backup script ConfigMap
2. `backup-pvc.yaml` - The PersistentVolumeClaim
3. `db-secret.yaml` - The database credentials Secret
4. `postgres-backup-cronjob.yaml` - The CronJob manifest

## Success Criteria

- [ ] CronJob is created with the correct schedule
- [ ] Backup script is properly mounted and executable
- [ ] PVC is mounted for backup storage
- [ ] Database credentials are provided via Secret
- [ ] Concurrency policy prevents overlapping runs
- [ ] Job history limits are configured
- [ ] Resource limits are set appropriately

## Hints

<details>
<summary>Hint 1: CronJob Template Structure</summary>

Basic CronJob structure:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: backups
spec:
  schedule: "0 2 * * *"
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 5
  jobTemplate:
    spec:
      activeDeadlineSeconds: 3600
      backoffLimit: 3
      template:
        spec:
          restartPolicy: OnFailure
          containers:
            - name: backup
              image: postgres:15
              # ... container spec
```
</details>

<details>
<summary>Hint 2: Mounting the ConfigMap Script</summary>

To make the script executable:

```yaml
volumeMounts:
  - name: backup-script
    mountPath: /scripts
volumes:
  - name: backup-script
    configMap:
      name: backup-script
      defaultMode: 0755  # Makes scripts executable
```

Then run it with: `command: ["/scripts/backup.sh"]`
</details>

<details>
<summary>Hint 3: Using Secrets for Credentials</summary>

Reference the Secret in your container:

```yaml
env:
  - name: PGPASSWORD
    valueFrom:
      secretKeyRef:
        name: db-credentials
        key: password
```

The `PGPASSWORD` environment variable is automatically used by PostgreSQL client tools.
</details>

<details>
<summary>Hint 4: Testing the CronJob</summary>

To test without waiting for the schedule:

```bash
# Create a job from the CronJob template
kubectl create job --from=cronjob/postgres-backup postgres-backup-test -n backups

# Watch the job
kubectl get jobs -n backups -w

# Check logs
kubectl logs -n backups job/postgres-backup-test
```
</details>

<details>
<summary>Hint 5: Concurrency Policies</summary>

- `Allow`: Multiple jobs can run simultaneously (default)
- `Forbid`: Skip new job if previous is still running
- `Replace`: Cancel running job and start new one

For backups, `Forbid` is usually the safest choice to prevent resource conflicts.
</details>

## Common Issues

1. **Permission denied**: Ensure the script has execute permissions (`defaultMode: 0755`)
2. **Connection refused**: Verify the database host and port are correct
3. **No space left**: Check PVC size and cleanup policy
4. **Job stuck**: Set `activeDeadlineSeconds` to prevent infinite retries
