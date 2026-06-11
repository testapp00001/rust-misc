# Solution 01: Volume Types Explained

## Part A -- Completed Table

| Feature | Named Volume | Bind Mount | tmpfs Mount |
|---------|-------------|------------|-------------|
| **Data location** | Docker-managed directory under `/var/lib/docker/volumes/` | An explicit path on the host filesystem that you specify | Host memory (RAM); never written to disk |
| **Survives container removal?** | Yes -- the volume persists until you explicitly `docker volume rm` it | Yes -- the host directory is unaffected by container lifecycle | No -- data is lost when the container stops |
| **Can be shared across containers?** | Yes -- multiple containers can mount the same named volume | Yes -- multiple containers can mount the same host path | No -- each container gets its own isolated tmpfs |
| **Editable from host?** | Not directly -- you must use a container to access the files | Yes -- any process on the host can read and write the files | No -- exists only in memory |
| **Works on all OS?** | Yes -- Docker manages the storage abstractly | Mostly -- path syntax differs between Linux/macOS and Windows; file permission semantics vary | Linux only (not supported on Docker Desktop for Mac/Windows without a Linux VM) |
| **Best use case** | Database files, application state that must persist independently of the host | Source code during development, config files, log directories | Sensitive temporary data, secrets, credential files that must never touch disk |

### Detailed Explanations

**Named Volumes:**
A named volume is a Docker-managed directory. Docker creates it under its data
root (typically `/var/lib/docker/volumes/<name>/_data`). You never need to
worry about the underlying path or permissions -- Docker handles it. The volume
persists even after all containers using it are removed. You must explicitly
run `docker volume rm` to delete it. Named volumes are the recommended
approach for production data like database files because they are portable,
easy to back up (via `tar` from a helper container), and insulated from
host-directory structure changes.

**Bind Mounts:**
A bind mount maps a specific directory or file on the host filesystem into a
container. The path you specify must exist on the host. Changes made by the
container are immediately visible on the host and vice versa. This makes bind
mounts ideal for development workflows where you edit code on the host and
want the container to see changes in real time. The downside is that the
container is tightly coupled to the host's filesystem structure and
permissions, which reduces portability.

**tmpfs Mounts:**
A tmpfs mount stores data in the host's RAM. It is never written to the host's
filesystem, which makes it ideal for sensitive data like credentials or
temporary processing files. The data is lost when the container stops. tmpfs
mounts cannot be shared between containers and are only available on Linux
hosts (Docker Desktop runs a Linux VM, so they work there too, but the data
still lives in the VM's memory, not the macOS/Windows host's memory directly).

## Part B -- Command Translations

### 1. Mount syntax to shorthand

**Original:**
```bash
docker run --mount type=volume,src=mydata,dst=/app/data nginx
```

**Shorthand equivalent:**
```bash
docker run -v mydata:/app/data nginx
```

When the source (`mydata`) does not look like a file path (no leading `/` or
`./`), Docker interprets it as a named volume.

### 2. Shorthand to mount syntax

**Original:**
```bash
docker run -v /home/user/site:/usr/share/nginx/html:ro nginx
```

**Mount syntax equivalent:**
```bash
docker run --mount type=bind,src=/home/user/site,dst=/usr/share/nginx/html,readonly nginx
```

Note: `--mount` uses `readonly` as the flag name, not `:ro`.

### 3. Mount syntax to shorthand

**Original:**
```bash
docker run --mount type=tmpfs,dst=/tmp,size=100m nginx
```

**Shorthand equivalent:**
There is no `-v` shorthand for tmpfs mounts. You must use `--mount` or
`--tmpfs`. The closest shorthand is:

```bash
docker run --tmpfs /tmp:size=100m nginx
```

### 4. Shorthand to mount syntax

**Original:**
```bash
docker run -v mydata:/data postgres
```

**Mount syntax equivalent:**
```bash
docker run --mount type=volume,src=mydata,dst=/data postgres
```

## Part C -- Scenario Matching

### 1. PostgreSQL container retaining data across upgrades

**Best choice: Named volume**

A named volume persists independently of the container. When you upgrade
PostgreSQL (pull a new image, remove the old container, start a new one with
the same volume), the data in `/var/lib/postgresql/data` is untouched. A bind
mount would also persist, but named volumes are preferred for databases because
Docker manages permissions and the storage location is abstracted from the host.

### 2. Development container reflecting host source code changes

**Best choice: Bind mount**

A bind mount maps the host directory directly into the container. When you edit
a file on the host (e.g., in your IDE), the change is immediately visible
inside the container. This is the standard pattern for development with hot
reload. A named volume would require a separate sync mechanism.

### 3. CI/CD job container with sensitive temporary credentials

**Best choice: tmpfs mount**

A tmpfs mount stores data in RAM and never writes it to disk. If the CI runner
is compromised or the disk is inspected after the job, the credentials are
gone. A named volume or bind mount would leave the credentials on disk,
potentially accessible to other jobs or processes.

### 4. Node.js node_modules persisting across compose cycles

**Best choice: Named volume**

A named volume persists across `docker compose down` and `docker compose up`.
Using a named volume for `node_modules` avoids reinstalling dependencies every
time you restart the stack. A bind mount would work but would conflict with a
host `node_modules` directory if one exists, and the volume approach is faster
on macOS/Windows where bind mounts have performance overhead.

### 5. Log aggregation container reading host log files

**Best choice: Bind mount**

The container needs to read files written by other processes on the host.
A bind mount is the only option that gives the container access to a specific
host directory. A named volume is Docker-managed and invisible to host
processes. tmpfs is in-memory and not shared with host processes.

## Common Mistakes

1. **Confusing `-v` syntax:** When using `-v`, if the source starts with `/`
   or `./`, Docker treats it as a bind mount. If it does not, Docker treats
   it as a named volume. This implicit behavior surprises many beginners.

2. **Forgetting that bind mounts require the host path to exist:** Unlike
   named volumes, Docker does not create the host directory for you (with the
   exception of `docker compose` which may create it). If the path does not
   exist, Docker creates it as a directory, which may not be what you want
   (e.g., if you intended to mount a file).

3. **Using tmpfs on non-Linux hosts:** tmpfs mounts are a Linux kernel
   feature. On Docker Desktop for Mac/Windows, they work because Docker runs
   a Linux VM, but the data lives in the VM's memory, not directly on the
   host.

4. **Overusing bind mounts in production:** Bind mounts couple your container
   to the host's filesystem structure. If you deploy to a different host with
   a different directory layout, your compose file or run command breaks. Use
   named volumes for production data.
