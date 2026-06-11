# Solution 04: Debug a Misbehaving Container

## Step 1: Verify the Problem

```bash
docker ps
```

```
CONTAINER ID   IMAGE        COMMAND                  STATUS        PORTS                  NAMES
a1b2c3d4e5f6   httpd:2.4    "httpd-foreground"       Up 30 seconds 0.0.0.0:8080->80/tcp   broken-web
```

The container IS running. So the issue is not that it crashed.

```bash
docker ps -a
```

Confirms `broken-web` exists and is `Up`.

## Step 2: Check the Logs

```bash
docker logs broken-web
```

```
AH00558: httpd: Could not reliably determine the server's fully qualified domain name, using 172.17.0.2. Set the 'ServerName' directive globally to suppress this message
[Wed Jun 11 04:00:00.000000 2026] [mpm_event:notice] [pid 1:tid 1] AH00489: Apache/2.4.57 (Unix) configured -- resuming normal operations
[Wed Jun 11 04:00:00.000000 2026] [core:notice] [pid 1:tid 1] AH00094: Command line: 'httpd -D FOREGROUND'
```

The logs look normal. Apache started successfully. The "Could not reliably
determine" warning is harmless -- it appears on every Apache startup.

So the problem is NOT inside the container. Apache is running fine.

## Step 3: Inspect the Container Configuration

```bash
docker inspect -f '{{json .HostConfig.PortBindings}}' broken-web
```

```json
{"80/tcp":[{"HostIp":"","HostPort":"8080"}]}
```

Port mapping looks correct: host 8080 -> container 80.

```bash
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' broken-web
```

```
172.17.0.2
```

```bash
docker inspect -f '{{.State.Status}}' broken-web
```

```
running
```

Everything looks correct in the configuration. The container is running,
the port is mapped, and Apache is healthy.

## Step 4: Exec Into the Container

```bash
docker exec -it broken-web bash
```

Inside the container:

```bash
# Check Apache is running
ps aux
```

```
USER       PID   %CPU  %MEM   VSZ    RSS   TTY   STAT   START   TIME   COMMAND
root       1     0.0   0.1    ...    ...   ?     Ss     ...     0:00   httpd -DFOREGROUND
daemon     7     0.0   0.0    ...    ...   ?     S      ...     0:00   httpd -DFOREGROUND
daemon     8     0.0   0.0    ...    ...   ?     S      ...     0:00   httpd -DFOREGROUND
```

```bash
# Check what port Apache is listening on
apt-get update && apt-get install -y net-tools
netstat -tlnp
```

```
Active Internet connections (only servers)
Proto Recv-Q Send-Q Local Address    Foreign Address   State    PID/Program
tcp        0      0 0.0.0.0:80       0.0.0.0:*         LISTEN   1/httpd
```

Apache is listening on port 80 inside the container. This matches the port mapping.

```bash
# Test from inside
curl -s localhost:80
```

```html
<html><body><h1>It works!</h1></body></html>
```

Apache responds correctly from inside the container.

## Step 5: Test from the Host

```bash
curl http://localhost:8080
```

```
<html><body><h1>It works!</h1></body></html>
```

**Wait -- it works!** If your test actually succeeds, then the container
is NOT misbehaving. The colleague's problem might have been:

1. **A transient issue.** The container was still starting when they tried.
2. **Browser caching.** The browser cached a previous error response.
3. **Wrong URL.** They typed the wrong port or hostname.

## The Root Cause

In this exercise, the container was actually working correctly. The "bug"
was a red herring designed to teach you an important lesson:

**Always verify the problem exists before trying to fix it.**

The correct debugging workflow is:
1. Confirm the symptom (does it actually fail?)
2. Check the obvious (is the container running?)
3. Check the logs (what does the application say?)
4. Check the configuration (are ports mapped correctly?)
5. Test from inside the container (is the app working internally?)
6. Test from the host (is the port forwarding working?)

If you had immediately started changing things without verifying, you might
have introduced new problems.

## What If It Had Been Broken?

Here are the most common real failures and how to diagnose each:

### Failure 1: Container exited immediately

```bash
docker ps -a
# STATUS: Exited (1) 5 seconds ago
```

Diagnosis: Check `docker logs` for the crash reason.

### Failure 2: Wrong port mapping

```bash
docker inspect -f '{{json .HostConfig.PortBindings}}' broken-web
# {"80/tcp":[{"HostIp":"","HostPort":"9090"}]}
# HostPort is 9090, not 8080!
```

Diagnosis: The colleague mapped to port 9090 but tried to access port 8080.

### Failure 3: Port already in use

```bash
docker run -d --name broken-web -p 8080:80 httpd:2.4
# Error: Bind for 0.0.0.0:8080 failed: port is already allocated
```

Diagnosis: Another process is using port 8080. Check with `ss -tlnp | grep 8080`.

### Failure 4: Application not listening on expected port

If Apache was configured to listen on port 8080 inside the container but
the port mapping expects port 80:

```bash
docker exec broken-web netstat -tlnp
# tcp  0  0  0.0.0.0:8080  0.0.0.0:*  LISTEN  1/httpd
```

Diagnosis: The internal port does not match the port mapping. Fix the
mapping: `-p 8080:8080` instead of `-p 8080:80`.

## Debugging Checklist

When a container misbehaves, follow this order:

1. `docker ps -a` -- Is the container running? Did it exit?
2. `docker logs <name>` -- What does the application say?
3. `docker inspect <name>` -- What is the configuration?
4. `docker exec -it <name> sh` -- What is happening inside?
5. Test from the host -- Is the port mapping working?

**Never skip step 1.** Many debugging sessions waste time because the
engineer assumed the container was running when it was not.

## Common Mistakes to Avoid

- **Assuming the container is running.** Always check `docker ps` first.
- **Ignoring exit codes.** Exit code 0 means success; anything else means failure.
  Check `docker inspect -f '{{.State.ExitCode}}' <name>`.
- **Not checking logs before inspecting.** Logs are the fastest way to find
  application-level errors.
- **Changing things without understanding.** Fix the root cause, not the symptom.
- **Forgetting to test from inside the container.** If the app works inside but
  not outside, the problem is in port mapping or networking, not the application.
