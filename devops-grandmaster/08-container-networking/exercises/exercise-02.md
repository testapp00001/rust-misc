# Exercise 02: Create Custom Networks and Connect Containers

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create custom Docker networks, connect containers to them, verify DNS
resolution between containers, and practice connecting a single container
to multiple networks.

---

## Tasks

### Part A: Create a Custom Bridge Network

Create a custom bridge network called `app-net` with a specific subnet.

Requirements:
- Network name: `app-net`
- Subnet: `10.10.0.0/24`
- Gateway: `10.10.0.1`

Then verify the network was created correctly.

```bash
# Your commands here
```

<details>
<summary>Hint</summary>

Use `docker network create` with `--subnet` and `--gateway` flags. Then
use `docker network inspect` to verify.

</details>

### Part B: Connect Containers and Test DNS

Start two containers on the `app-net` network:

1. A container named `web` running `nginx:alpine`
2. A container named `api` running `python:3.12-slim` (use `sleep infinity` to keep it running)

Then perform these tests from inside the `api` container:

1. Ping the `web` container by name
2. Resolve the `web` container's hostname using `nslookup` or `getent hosts`
3. Make an HTTP request to `web:80` using `curl`

Record the commands and their output.

<details>
<summary>Hint</summary>

Use `docker run -d --name <name> --network app-net <image>` to start each
container. Use `docker exec api <command>` to run commands inside the api
container. You may need to install `curl` and `dnsutils` inside the
python container:
```bash
docker exec api apt-get update && apt-get install -y curl dnsutils
```

</details>

### Part C: Verify DNS Does NOT Work Across Networks

Create a second network called `db-net` and start a third container on it:

```bash
docker run -d --name db --network db-net postgres:16 -e POSTGRES_PASSWORD=secret
```

Now test connectivity from the `api` container to the `db` container:

1. Try to ping `db` from the `api` container
2. Try to resolve `db` from the `api` container

Explain why this fails.

<details>
<summary>Hint</summary>

Containers on different bridge networks are completely isolated. The
embedded DNS server at 127.0.0.11 only knows about containers on the
same network.

</details>

### Part D: Multi-Network Container

Connect the `api` container to `db-net` so it can reach both `web` and `db`:

1. Use `docker network connect` to add `api` to `db-net`
2. Verify `api` can now ping `db`
3. Verify `api` can still ping `web`
4. Inspect the `api` container to see it now has two IP addresses

Record the commands and explain what happened at the network level.

<details>
<summary>Hint</summary>

`docker network connect` adds a new virtual network interface to the
container. The container now participates in two separate networks
simultaneously. Use `docker inspect api` and look at the `Networks`
section.

</details>

### Part E: Clean Up

Remove all containers and networks you created. Verify everything is gone.

```bash
# Your cleanup commands here
```

<details>
<summary>Hint</summary>

Use `docker rm -f` to force-remove running containers, then
`docker network rm` to remove the networks. Alternatively, use
`docker network prune` to remove all unused networks.

</details>

---

## Success Criteria

- [ ] You created a custom bridge network with a specific subnet
- [ ] Containers on the same custom network can resolve each other by name
- [ ] Containers on different networks cannot reach each other
- [ ] You connected a container to two networks and verified dual connectivity
- [ ] You can explain why the embedded DNS server is network-scoped
- [ ] You cleaned up all resources

## What You Should Understand After This Exercise

Custom bridge networks give you DNS resolution and network isolation. A
container can participate in multiple networks, which is the foundation
for building architectures where only specific containers can reach
specific backends (for example, an API gateway that can reach both a
frontend network and a backend network).
