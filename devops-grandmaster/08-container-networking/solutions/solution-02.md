# Solution 02: Create Custom Networks and Connect Containers

## Part A: Create a Custom Bridge Network

```bash
docker network create --subnet 10.10.0.0/24 --gateway 10.10.0.1 app-net
```

Verify:

```bash
docker network inspect app-net
```

Expected output (key fields):

```json
{
    "Name": "app-net",
    "Driver": "bridge",
    "IPAM": {
        "Config": [
            {
                "Subnet": "10.10.0.0/24",
                "Gateway": "10.10.0.1"
            }
        ]
    }
}
```

### Why It Works

`docker network create` creates a new Linux bridge on the host and
configures Docker's IPAM (IP Address Management) to assign addresses from
the specified subnet. The gateway is the bridge interface's IP address on
the host.

### Common Mistakes

- Forgetting `--gateway` -- Docker will auto-assign one, but you lose
  control over the addressing scheme.
- Using a subnet that conflicts with an existing network on the host
  (e.g., your LAN uses `10.10.0.0/24`). Use `ip route` to check.

---

## Part B: Connect Containers and Test DNS

Start the containers:

```bash
docker run -d --name web --network app-net nginx:alpine
docker run -d --name api --network app-net python:3.12-slim sleep infinity
```

Install tools inside the api container:

```bash
docker exec api apt-get update && docker exec api apt-get install -y curl dnsutils
```

Test 1: Ping by name:

```bash
docker exec api ping -c 3 web
```

Expected output:

```
PING web (10.10.0.2): 56 data bytes
64 bytes from 10.10.0.2: seq=0 ttl=64 time=0.089 ms
64 bytes from 10.10.0.2: seq=1 ttl=64 time=0.058 ms
64 bytes from 10.10.0.2: seq=2 ttl=64 time=0.063 ms
```

Test 2: DNS resolution:

```bash
docker exec api nslookup web
```

Or:

```bash
docker exec api getent hosts web
```

Expected: resolves to `10.10.0.2` (or similar IP from the subnet).

Test 3: HTTP request:

```bash
docker exec api curl -s http://web:80
```

Expected: Returns the Nginx default HTML page.

### Why It Works

Custom bridge networks run an embedded DNS server at `127.0.0.11` inside
each container. When the `api` container tries to resolve `web`, the DNS
query goes to this embedded server, which looks up the container name in
Docker's service discovery table and returns the container's IP address.

The default bridge does NOT have this DNS server, which is why container
name resolution only works on custom networks.

---

## Part C: Verify DNS Does NOT Work Across Networks

Create db-net and start the database:

```bash
docker network create db-net
docker run -d --name db --network db-net -e POSTGRES_PASSWORD=secret postgres:16
```

Test connectivity from api to db:

```bash
docker exec api ping -c 1 -W 2 db
```

Expected output:

```
ping: bad address 'db'
```

DNS resolution also fails:

```bash
docker exec api getent hosts db
# No output
```

### Why It Works (Why It Fails)

The embedded DNS server at `127.0.0.11` only knows about containers on the
same network. The `api` container is on `app-net`, and the DNS server on
`app-net` has no knowledge of containers on `db-net`. The two networks are
completely isolated at Layer 3.

You can verify this by checking the DNS server inside each container:

```bash
# Inside api (on app-net)
docker exec api cat /etc/resolv.conf
# nameserver 127.0.0.11

# The DNS server at 127.0.0.11 only knows about app-net containers
```

---

## Part D: Multi-Network Container

Connect api to db-net:

```bash
docker network connect db-net api
```

Test connectivity:

```bash
# Can reach db
docker exec api ping -c 1 db
# PING db (172.19.0.2): 56 data bytes
# 64 bytes from 172.19.0.2: seq=0 ttl=64 time=0.078 ms

# Can still reach web
docker exec api ping -c 1 web
# PING web (10.10.0.2): 56 data bytes
# 64 bytes from 10.10.0.2: seq=0 ttl=64 time=0.065 ms
```

Inspect the api container:

```bash
docker inspect api --format '{{json .NetworkSettings.Networks}}' | python3 -m json.tool
```

Expected output (abbreviated):

```json
{
    "app-net": {
        "IPAddress": "10.10.0.2",
        "Gateway": "10.10.0.1"
    },
    "db-net": {
        "IPAddress": "172.19.0.3",
        "Gateway": "172.19.0.1"
    }
}
```

### Why It Works

`docker network connect` adds a new virtual Ethernet interface (veth pair)
to the container. The container now has:

- An interface on `app-net` (10.10.0.x) with access to `web`
- An interface on `db-net` (172.19.0.x) with access to `db`

The container's kernel routes traffic to the correct interface based on
the destination subnet. The embedded DNS server is also updated to know
about containers on both networks.

This is the foundation of multi-tier architecture: the `api` container
acts as a bridge between the application tier (`app-net`) and the data
tier (`db-net`), while `web` and `db` remain isolated from each other.

---

## Part E: Clean Up

```bash
# Stop and remove all containers
docker rm -f web api db

# Remove the networks
docker network rm app-net db-net

# Verify everything is gone
docker network ls
docker ps -a
```

Alternative (removes all unused networks at once):

```bash
docker network prune -f
```

### Common Mistakes

- Running `docker network rm` before removing containers. Docker will
  refuse to remove a network that has connected containers. Remove
  containers first.
- Using `docker network prune` when you have other unused networks you
  want to keep. It removes ALL unused networks, not just the ones you
  created in this exercise.
