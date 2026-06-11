# Exercise 01: Volume Types Explained

**Type:** Conceptual
**Estimated Time:** 20 minutes

## Objective

Understand the three Docker storage mechanisms -- **named volumes**, **bind
mounts**, and **tmpfs mounts** -- well enough to explain when each is the right
choice.

## Background

Docker containers are ephemeral by default. When a container is removed, any
data written to its writable layer is lost. Docker provides three mechanisms to
persist or share data beyond the container lifecycle.

## Instructions

### Part A -- Definitions

For each storage type, write a one-paragraph explanation that covers:

1. Where the data physically lives (host filesystem, Docker-managed area, memory).
2. How the data behaves when the container is stopped and removed.
3. A concrete real-world use case where this type is the best fit.

Fill in the table below:

```
| Feature              | Named Volume | Bind Mount | tmpfs Mount |
|----------------------|--------------|------------|-------------|
| Data location        |              |            |             |
| Survives container   |              |            |             |
| removal?             |              |            |             |
| Can be shared across |              |            |             |
| containers?          |              |            |             |
| Editable from host?  |              |            |             |
| Works on all OS?     |              |            |             |
| Best use case        |              |            |             |
```

### Part B -- Command Translation

Translate each `--mount` flag syntax into the equivalent `-v` (shorthand)
syntax, and vice versa.

1. `docker run --mount type=volume,src=mydata,dst=/app/data nginx`
2. `docker run -v /home/user/site:/usr/share/nginx/html:ro nginx`
3. `docker run --mount type=tmpfs,dst=/tmp,size=100m nginx`
4. `docker run -v mydata:/data postgres`

### Part C -- Scenario Matching

Read each scenario and decide which storage type is most appropriate. Explain
your reasoning in one or two sentences.

1. A PostgreSQL container that must retain data across container upgrades.
2. A development container that needs to reflect source code changes made on
   the host in real time.
3. A CI/CD job container that stores sensitive temporary credentials that
   should never touch disk.
4. A Node.js application that needs its `node_modules` directory to persist
   across `docker compose down` and `docker compose up` cycles.
5. A log aggregation container that reads log files written by other processes
   on the host.

## Success Criteria

- [ ] Your table correctly identifies the key differences for all three types.
- [ ] All four command translations are syntactically correct and produce the
      same runtime behavior as the original.
- [ ] Each scenario is matched to the correct storage type with a valid
      justification.

## Hints

<details>
<summary>Hint 1 -- Where volumes live</summary>

Named volumes are stored under Docker's data root, typically
`/var/lib/docker/volumes/`. You do not interact with them directly on the host
filesystem. Bind mounts point to an explicit path on the host that you choose.

</details>

<details>
<summary>Hint 2 -- Shorthand syntax rules</summary>

The `-v` shorthand uses a colon-separated format: `source:destination:options`.
When the source looks like a path (starts with `/` or `./`), Docker treats it
as a bind mount. When it does not look like a path, Docker treats it as a
volume name.

</details>

<details>
<summary>Hint 3 -- tmpfs characteristics</summary>

A tmpfs mount stores data in the host's memory (RAM). It is never written to
the host's filesystem, so it is ideal for sensitive data that must not persist.
However, it cannot be shared between containers and is lost when the container
stops.

</details>
