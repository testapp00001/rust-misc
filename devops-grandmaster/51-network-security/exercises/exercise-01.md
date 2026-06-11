# Exercise 01: Anatomy of a Flat Network Breach

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective
Given a flat network diagram where all hosts share a single subnet with no segmentation, identify every way an attacker could move laterally after compromising one host. You will learn to see a network the way an attacker does and understand exactly why network segmentation is a foundational security control.

## The Scenario

You are given the following flat network. Every host sits on the same `10.0.0.0/24` subnet. There are no firewalls between hosts, no VLANs, and no access control lists.

```
                        Internet
                           |
                     [Firewall/Router]
                           |
                   10.0.0.0/24 (flat)
         +--------+--------+--------+--------+--------+
         |        |        |        |        |        |
     Web-01   Web-02   App-01   DB-01   CI/CD   JumpBox
     .10      .11      .20      .30     .40     .50

     Services running:
     Web-01:  Nginx (80/443), SSH (22)
     Web-02:  Nginx (80/443), SSH (22)
     App-01:  Node.js (3000), SSH (22)
     DB-01:   PostgreSQL (5432), SSH (22)
     CI/CD:   Jenkins (8080), SSH (22), Docker API (2375)
     JumpBox: SSH (22)
```

## Tasks

### Part A: Identify Lateral Movement Paths
The attacker has compromised **Web-01** (10.0.0.10) through an Nginx vulnerability. List every host and service the attacker can directly reach from Web-01, and describe the technique for each.

Create a table with columns: Target Host, Target Service/Port, Attack Technique, Impact if Exploited.

<details>
<summary>Hint</summary>
Since this is a flat /24 subnet, Web-01 can send traffic to any IP in 10.0.0.0/24 without passing through any filtering device. Think about what services are exposed and what common vulnerabilities or misconfigurations exist on each.
</details>

### Part B: Map the Attack Chain
Using your findings from Part A, draw an attack chain (sequence of compromises) that starts at Web-01 and ends with full control of the database. Show each hop, the technique used, and what credentials or access the attacker gains at each step.

<details>
<summary>Hint</summary>
Think about what data each compromised host exposes. SSH keys stored on disk, database credentials in environment variables, and CI/CD pipelines often contain secrets that unlock the next hop.
</details>

### Part C: Identify the Segmentation Failures
For each lateral movement path you identified, state what network segmentation control would have blocked it. Use the following categories:
- **VLAN/subnet isolation** -- separate broadcast domains
- **Firewall rules** -- stateful packet filtering between segments
- **Micro-segmentation** -- host-based or identity-based policies
- **Zero-trust** -- mutual TLS, identity verification per connection

<details>
<summary>Hint</summary>
Not every control is appropriate for every path. A database should never be on the same subnet as a web server, but SSH between app servers and web servers might be legitimate -- it just needs to be restricted by firewall rules and identity.
</details>

### Part D: Design the Segmented Architecture
Redraw the network with proper segmentation. You should have at least three segments (tiers). Label each segment with its CIDR range, the firewall rules between them, and which hosts belong to each segment.

<details>
<summary>Hint</summary>
A classic three-tier architecture places web servers, application servers, and databases in separate subnets. The CI/CD system and jump box should also be in a restricted management segment. Think about which segments need to talk to which, and deny everything else.
</details>

## Success Criteria
- [ ] You identified at least 5 distinct lateral movement paths from Web-01
- [ ] Your attack chain reaches DB-01 in no more than 3 hops
- [ ] Each lateral movement path has a specific segmentation control that blocks it
- [ ] Your redesigned architecture has at least 3 network segments with explicit allow rules between them
- [ ] No segment can reach another segment without passing through a firewall rule

## What You Should Understand After This Exercise
After completing this exercise, you should viscerally understand why a flat network is an attacker's paradise. Every host is reachable from every other host, meaning a single compromise gives an attacker a direct path to your most sensitive assets. Segmentation forces the attacker through choke points where you can inspect, log, and block traffic. The principle of least privilege applies to networking just as it does to software permissions -- no host should be able to reach another host unless there is an explicit, documented business need.
