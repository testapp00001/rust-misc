# Exercise 04: Debug a Misbehaving Container

**Type:** Challenge
**Time:** 30 minutes
**Difficulty:** Medium-Hard

## Objective

Use Docker inspection and debugging commands to diagnose why a container is
not working correctly. This exercise simulates a real troubleshooting scenario
where you need to investigate a container without knowing the root cause upfront.

## Scenario

A colleague tells you: "I ran a web server container but something is wrong.
I cannot access it from my browser. Can you figure out what is going on?"

They ran this command:

```bash
docker run -d --name broken-web -p 8080:80 httpd:2.4
```

They claim that `http://localhost:8080` returns nothing.

## Tasks

### Step 1: Verify the Problem

Check if the container is actually running:

```bash
docker ps
```

Check if the container exists at all (it might have exited):

```bash
docker ps -a
```

<details>
<summary>Hint</summary>

If the container is not in `docker ps` but IS in `docker ps -a`, it has exited.
Check the STATUS column for the exit code.

</details>

### Step 2: Check the Logs

The first place to look when a container misbehaves is its logs:

```bash
docker logs broken-web
```

Look for error messages, warnings, or anything unusual.

<details>
<summary>Hint</summary>

The httpd image logs to stdout/stderr by default. If the container started
successfully, you should see Apache's "AH00558: httpd: Could not reliably
determine the server's fully qualified domain name" warning (this is normal)
and a notice that it is listening on port 80.

</details>

### Step 3: Inspect the Container Configuration

Examine the container's network and port mappings:

```bash
docker inspect broken-web
```

This outputs a lot of JSON. Focus on the `NetworkSettings` and `HostConfig` sections.

Extract specific information:

```bash
# What ports are mapped?
docker inspect -f '{{json .HostConfig.PortBindings}}' broken-web

# What is the container's IP?
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' broken-web

# What is the container's state?
docker inspect -f '{{.State.Status}}' broken-web

# What command did the container run?
docker inspect -f '{{.Config.Cmd}}' broken-web
```

<details>
<summary>Hint</summary>

Port bindings should show `"80/tcp":[{"HostIp":"","HostPort":"8080"}]`.
If HostPort is different from what your colleague expects, that is your clue.

</details>

### Step 4: Exec Into the Container

If the container is running, get inside and investigate:

```bash
docker exec -it broken-web bash
```

Inside the container, check:

```bash
# Is httpd actually running?
ps aux

# Is it listening on the expected port?
apt-get update && apt-get install -y net-tools
netstat -tlnp

# Can you reach it from inside?
curl -s localhost:80
```

<details>
<summary>Hint</summary>

If `curl localhost:80` works inside the container but not from the host,
the problem is in the port mapping, not the application.

</details>

### Step 5: Test from the Host

From your host machine (exit the container first), try different approaches:

```bash
# Try the obvious way
curl http://localhost:8080

# Try with verbose output to see connection details
curl -v http://localhost:8080

# Check if anything is listening on port 8080
ss -tlnp | grep 8080
```

<details>
<summary>Hint</summary>

If `curl -v` shows "Connection refused", nothing is listening on port 8080
on the host side. Check if the port mapping is correct.

</details>

### Step 6: Identify and Fix the Root Cause

Based on your investigation, identify what is wrong and fix it.

Possible issues to check:
1. The container exited immediately (check `docker ps -a`)
2. The port mapping is wrong (check `docker inspect`)
3. The application inside is not listening on the expected port
4. A firewall is blocking the connection

Once you identify the issue, fix it:

```bash
# Remove the broken container
docker rm -f broken-web

# Run a corrected version (you need to figure out what to change)
docker run -d --name broken-web -p 8080:80 httpd:2.4
```

Verify the fix:

```bash
curl http://localhost:8080
```

You should see the Apache "It works!" page.

### Step 7: Document Your Findings

Write down:
1. What was the symptom?
2. What commands did you use to investigate?
3. What was the root cause?
4. How did you fix it?

## Success Criteria

- [ ] You checked the container status with `docker ps` and `docker ps -a`
- [ ] You examined logs with `docker logs`
- [ ] You inspected the container configuration with `docker inspect`
- [ ] You used `docker exec` to investigate inside the container
- [ ] You identified the root cause of the problem
- [ ] You fixed the problem and verified the container works
- [ ] You can explain your debugging process step by step

## Debugging Checklist

When a container misbehaves, follow this order:

1. `docker ps -a` -- Is the container running? Did it exit?
2. `docker logs <name>` -- What does the application say?
3. `docker inspect <name>` -- What is the configuration?
4. `docker exec -it <name> sh` -- What is happening inside?
5. Test from the host -- Is the port mapping working?
