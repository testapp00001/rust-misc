# Solution 03: Backup and Restore Strategy

## Step 1 -- Prepare the Source Data

```bash
docker volume create appdata

docker run --rm -v appdata:/data alpine sh -c "
  mkdir -p /data/config /data/uploads
  echo '{ \"db_host\": \"postgres\", \"db_port\": 5432 }' > /data/config/settings.json
  echo 'This is upload A.' > /data/uploads/file-a.txt
  echo 'This is upload B.' > /data/uploads/file-b.txt
  echo 'This is upload C.' > /data/uploads/file-c.txt
  echo 'Data initialized.'
"
```

Verify:

```bash
docker run --rm -v appdata:/data alpine ls -R /data
# /data:
# config
# uploads
#
# /data/config:
# settings.json
#
# /data/uploads:
# file-a.txt
# file-b.txt
# file-c.txt
```

## Step 2 -- Back Up the Volume

The completed command:

```bash
mkdir -p ./backups

docker run --rm \
  -v appdata:/data:ro \
  -v $(pwd)/backups:/backups \
  alpine tar czf /backups/appdata-backup.tar.gz -C /data .
```

### Breakdown

- `-v appdata:/data:ro` -- mounts the named volume at `/data` as read-only.
  The backup process cannot accidentally modify or delete source files.
- `-v $(pwd)/backups:/backups` -- mounts the host's `./backups` directory so
  the tar archive is written directly to the host.
- `alpine` -- a minimal image with `tar` built in.
- `tar czf /backups/appdata-backup.tar.gz -C /data .` -- creates a gzipped
  tar archive. `-C /data .` changes into `/data` before archiving so that
  paths in the archive are relative (e.g., `./config/settings.json`) rather
  than absolute (e.g., `/data/config/settings.json`). Relative paths make
  restore into any target directory straightforward.

## Step 3 -- Inspect the Backup

```bash
ls -lh ./backups/
# -rw-r--r-- 1 root root ... appdata-backup.tar.gz

tar tzf ./backups/appdata-backup.tar.gz
# ./
# ./config/
# ./config/settings.json
# ./uploads/
# ./uploads/file-a.txt
# ./uploads/file-b.txt
# ./uploads/file-c.txt
```

## Step 4 -- Restore into a New Volume

Create the target volume:

```bash
docker volume create appdata-restored
```

The completed restore command:

```bash
docker run --rm \
  -v appdata-restored:/data \
  -v $(pwd)/backups:/backups:ro \
  alpine sh -c 'tar xzf /backups/appdata-backup.tar.gz -C /data'
```

### Breakdown

- `-v appdata-restored:/data` -- mounts the empty target volume.
- `-v $(pwd)/backups:/backups:ro` -- mounts the backup directory as read-only
  so the restore process cannot overwrite the archive.
- `tar xzf /backups/appdata-backup.tar.gz -C /data` -- extracts the archive
  into `/data`, which is the target volume.

## Step 5 -- Verify the Restore

```bash
docker run --rm -v appdata-restored:/data alpine ls -R /data
# Same structure as the original.

docker run --rm -v appdata-restored:/data alpine cat /data/config/settings.json
# { "db_host": "postgres", "db_port": 5432 }

docker run --rm -v appdata-restored:/data alpine cat /data/uploads/file-b.txt
# This is upload B.
```

All files and contents match the original.

## Step 6 -- Clean Up

```bash
docker volume rm appdata appdata-restored
rm -rf ./backups
```

## Why It Works

The backup pattern uses a **temporary container** that mounts two things:

1. The source volume (as read-only) -- to read the data.
2. A host directory -- to write the archive.

The container runs `tar` to bundle the volume contents into a single `.gz`
file on the host, then exits and is automatically removed (`--rm`). No
permanent infrastructure is needed.

The restore pattern reverses the direction: mount the target volume and the
backup directory, then extract the archive into the volume.

This approach is **portable** -- the tar archive is a standard format that can
be moved to any host, stored in S3, emailed, etc. It is **version-agnostic**
-- the archive contains files, not disk blocks, so it works regardless of the
underlying filesystem or Docker storage driver.

## Common Mistakes

1. **Forgetting `:ro` on the source volume during backup:** Without read-only,
   a buggy `tar` command (e.g., wrong flags) could corrupt the source data.
   Always mount the source as read-only during backup.

2. **Using absolute paths in the archive:** If you run `tar czf backup.tar.gz
   /data` without `-C`, the archive contains absolute paths like
   `/data/config/settings.json`. When you extract, `tar` tries to recreate
   `/data/` on the root filesystem, which either fails or writes to the wrong
   place. Always use `-C` to create relative paths.

3. **Not creating the host backup directory:** If `./backups` does not exist,
   Docker creates it as root-owned, which may cause permission issues later.
   Create it explicitly with `mkdir -p` before running the backup container.

4. **Running backup while the application is writing data:** For file-based
   data, `tar` captures a snapshot at a point in time. If the application is
   actively writing files during the backup, you may get a partially-written
   file. For databases, always use `pg_dump` instead of tar (see Exercise 04).

5. **Confusing volume backup with image backup:** A volume backup (`tar` of
   the volume contents) is different from a Docker image backup (`docker save`).
   They are not interchangeable. Volume backups capture runtime data; image
   backups capture the application binary and its dependencies.
