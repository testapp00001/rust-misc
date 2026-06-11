# Exercise 01: Bridge vs Host vs Overlay Networks

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Explain the differences between Docker's bridge, host, and overlay network
drivers. After this exercise you should be able to choose the correct network
driver for a given scenario and justify your choice.

## Background

Docker provides several network drivers. Each driver creates a different kind
of virtual network with different isolation, performance, and scope
characteristics. Choosing the wrong driver leads to broken connectivity,
security holes, or wasted resources.

---

## Tasks

### Part A: Fill in the Comparison Table

Complete the table below. For each cell, write a short answer (one sentence
or a few keywords).

| Property | bridge | host | overlay |
|----------|--------|------|---------|
| Default network created by Docker? | | | |
| Containers get their own IP? | | | |
| Containers can reach each other by name (DNS)? | | | |
| Works across multiple hosts? | | | |
| Isolates container from host network? | | | |
| Performance overhead compared to native? | | | |
| Requires Docker Swarm? | | | |

<details>
<summary>Hint</summary>

Think about what each driver *shares* with the host and what it *isolates*.
- bridge: creates a virtual switch inside the host
- host: removes the network namespace entirely
- overlay: wraps packets in VXLAN tunnels between hosts

</details>

### Part B: Match the Scenario to the Driver

For each scenario below, choose the best network driver (bridge, host, or
overlay) and explain why in one or two sentences.

1. A development environment running a web app and a database on a laptop.
2. A high-frequency trading application that needs the lowest possible network latency.
3. A microservices application running across three physical servers in a data center.
4. A CI/CD pipeline job that downloads artifacts but should not be able to reach any other container or the internet.
5. A monitoring agent that needs to see all network traffic on the host.

<details>
<summary>Hint</summary>

- Scenario 4 might not use bridge, host, or overlay. Consider the `none` driver.
- Scenario 5 is a special case where isolation would actually prevent the container from doing its job.

</details>

### Part C: The Default Bridge Trap

A developer runs two containers without specifying a network:

```bash
docker run -d --name web nginx:alpine
docker run -d --name api python:3.12-slim sleep infinity
```

Then tries to connect from the api container to the web container:

```bash
docker exec api curl http://web:80
```

It fails. Explain:

1. Which network are both containers on?
2. Why does the `curl http://web:80` command fail?
3. What would you need to change to make it work using container names?

<details>
<summary>Hint</summary>

There are two problems here. One is about the network driver behavior. The
other is about a missing flag on the `python` container.

</details>

### Part D: When Would You Use Host Mode?

The `host` network driver removes network isolation entirely. List at least
**three** specific situations where this trade-off is acceptable or necessary.
For each, explain what the container gains and what it loses.

<details>
<summary>Hint</summary>

Think about:
- Applications that open many ports dynamically
- Monitoring and observability tools
- Performance-critical workloads
- What "no port mapping needed" actually means for operations

</details>

---

## Success Criteria

- [ ] You can describe the key differences between bridge, host, and overlay networks
- [ ] You can match a scenario to the correct network driver with justification
- [ ] You understand why the default bridge does not support DNS resolution by container name
- [ ] You can explain when the performance trade-off of host networking is worth the loss of isolation
- [ ] You understand that `none` networking exists and when to use it

## What You Should Understand After This Exercise

Docker network drivers are not interchangeable. Each one makes a specific
trade-off between isolation, performance, and scope. The default bridge
exists for backward compatibility, not for production use. Custom bridge
networks should be your default choice for single-host multi-container
applications.
