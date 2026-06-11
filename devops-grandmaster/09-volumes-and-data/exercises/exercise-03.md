# Exercise 03: Backup and Restore Strategy

**Type:** Independent
**Estimated Time:** 45 minutes

## Objective

Implement a reproducible **backup and restore** workflow for data stored in a
Docker named volume. You will back up a volume's contents to a tar archive on
the host and restore them into a fresh volume.

## Instructions

### Step 1 -- Prepare the Source Data

Create a volume and populate it with files that simulate application data:

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
```

### Step 2 -- Back Up the Volume to a Tar Archive

Your task: write a single `docker run` command that:

1. Mounts the `appdata` volume as read-only.
2. Mounts a bind mount for the host backup directory (create `./backups` first).
3. Uses `tar` inside the container to archive the volume contents to the host.

Write your command below (fill in the blanks):

```bash
mkdir -p ./backups

docker run --rm \
  -v appdata:_____:ro \
  -v $(pwd)/backups:_____ \
  _____ tar czf _____ -C / _____
```

### Step 3 -- Inspect the Backup

From the host, confirm the backup file exists and list its contents:

```bash
ls -lh ./backups/
tar tzf ./backups/appdata-backup.tar.gz
```

### Step 4 -- Restore into a New Volume

Create a second volume and restore the backup into it:

```bash
docker volume create appdata-restored
```

Your task: write a `docker run` command that:

1. Mounts the `appdata-restored` volume.
2. Mounts the backup file as a read-only bind mount.
3. Extracts the tar archive into the volume.

```bash
docker run --rm \
  -v appdata-restored:/data \
  -v $(pwd)/backups:_____:ro \
  _____ sh -c '_____'
```

### Step 5 -- Verify the Restore

```bash
docker run --rm -v appdata-restored:/data alpine ls -R /data
docker run --rm -v appdata-restored:/data alpine cat /data/config/settings.json
docker run --rm -v appdata-restored:/data alpine cat /data/uploads/file-b.txt
```

All files and their contents must match the original.

### Step 6 -- Clean Up

```bash
docker volume rm appdata appdata-restored
rm -rf ./backups
```

## Success Criteria

- [ ] The backup tar archive is created on the host and contains all files
      from the volume.
- [ ] The restored volume contains identical file contents and directory
      structure.
- [ ] You used `--mount` or `-v` with read-only flags where appropriate to
      avoid accidental writes.
- [ ] All resources are cleaned up.

## Hints

<details>
<summary>Hint 1 -- tar flags</summary>

- `c` = create archive
- `z` = compress with gzip
- `f` = file name follows
- `C` = change to directory before archiving (use `-C /data .` to archive
  relative paths)

</details>

<details>
<summary>Hint 2 -- Restore command pattern</summary>

The restore command extracts the tar archive into `/data`:

```
tar xzf /backups/appdata-backup.tar.gz -C /data
```

You can wrap this in `sh -c '...'` to run it inside the container.

</details>

<details>
<summary>Hint 3 -- Read-only safety</summary>

When backing up, mount the source volume with `:ro` so the backup process
cannot accidentally modify or delete the original data. When restoring, mount
the backup directory with `:ro` so the restore process cannot accidentally
overwrite the archive.

</details>
