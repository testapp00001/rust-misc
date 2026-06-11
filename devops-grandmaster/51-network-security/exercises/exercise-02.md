# Exercise 02: Build a Firewall Ruleset from Requirements

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective
Given a set of services and their communication requirements, write iptables and UFW firewall rules that allow only the necessary traffic while blocking everything else. You will practice the principle of least privilege at the network layer and learn to express security policies as concrete, testable rules.

## The Scenario

You manage a single Linux server (`10.0.1.50`) that runs the following services:

| Service | Port | Protocol | Purpose |
|---------|------|----------|---------|
| SSH | 22 | TCP | Remote administration (admin subnet only) |
| Nginx | 80, 443 | TCP | Public web server |
| PostgreSQL | 5432 | TCP | Database (app servers only) |
| Node.js App | 3000 | TCP | Backend API (Nginx reverse proxy only) |
| Prometheus | 9090 | TCP | Metrics (monitoring server only) |

Network context:
- Admin subnet: `10.0.0.0/24`
- App server subnet: `10.0.2.0/24`
- Monitoring server: `10.0.3.10`
- Public traffic arrives on `eth0`
- Internal traffic arrives on `eth1`

## Tasks

### Part A: Write the Traffic Policy Matrix
Before writing any firewall rules, create a table that documents the complete traffic policy. For each service, specify: source, destination port, protocol, action (ALLOW/DENY), and the default policy for unmatched traffic.

<details>
<summary>Hint</summary>
Start with a "deny all" default policy, then enumerate only the traffic you need to allow. Every "ALLOW" rule must have a specific source -- never allow from `0.0.0.0/0` unless the service is genuinely public-facing.
</details>

### Part B: Implement iptables Rules
Write the complete iptables ruleset that implements your policy from Part A. Use the filter table with INPUT, OUTPUT, and FORWARD chains. Include rules for:
1. Allowing established/related connections
2. Allowing loopback traffic
3. Each service-specific rule from your policy
4. Default DROP policy on INPUT

```bash
# Write your iptables rules here
# Example format:
# iptables -A INPUT -s 10.0.0.0/24 -p tcp --dport 22 -j ACCEPT
```

<details>
<summary>Hint</summary>
Rule order matters in iptables. Place the established/related rule early for performance. Place specific allows before any broad rules. The default DROP policy goes last but should be set explicitly with `iptables -P INPUT DROP`.
</details>

### Part C: Convert to UFW Commands
Rewrite your iptables ruleset as equivalent UFW commands. UFW is a higher-level interface that is easier to read and maintain. Show the complete command sequence.

<details>
<summary>Hint</summary>
UFW uses `ufw allow from <source> to <dest> port <port>` syntax. You can also use application profiles defined in `/etc/ufw/applications.d/`. Remember to enable UFW and set the default incoming policy to deny.
</details>

### Part D: Create a Persistent Firewall Script
Write a bash script that:
1. Flushes all existing rules
2. Applies the iptables ruleset from Part B
3. Saves the rules for persistence across reboots
4. Logs all dropped packets to `/var/log/firewall-dropped.log`
5. Includes a `--dry-run` flag that shows the rules without applying them

<details>
<summary>Hint</summary>
Use `iptables -F` to flush, `iptables-save > /etc/iptables/rules.v4` for persistence on Debian/Ubuntu, and create a custom LOG chain for dropped packets. The dry-run mode can use `iptables -L -n -v` to display the rules.
</details>

### Part E: Test Your Ruleset
Write a set of test commands (using `nc`, `curl`, or `nmap`) that verify each rule in your firewall. For each rule, show one command that should succeed and one that should be blocked.

<details>
<summary>Hint</summary>
From the admin subnet, `ssh admin@10.0.1.50` should work. From the public internet, it should timeout or be refused. Use `nc -zv 10.0.1.50 5432` from the app subnet to test database connectivity. Document what "success" and "failure" look like for each test.
</details>

## Success Criteria
- [ ] Your traffic policy matrix has no "ALLOW from 0.0.0.0/0" rules for non-public services
- [ ] iptables rules include established/related, loopback, and default DROP
- [ ] UFW commands produce an equivalent security posture
- [ ] The persistent script includes dry-run capability and logging
- [ ] Every rule has a corresponding test command showing allowed and denied traffic

## What You Should Understand After This Exercise
Firewalls are not magic -- they are the concrete expression of a traffic policy. The hard part is writing the policy, not the rules. Once you have a clear matrix of "who talks to what on which port," translating that into iptables or UFW is mechanical. Always start with deny-all, then add only what is needed. Test every rule from both allowed and denied sources. Persist your rules so they survive reboots. And log dropped traffic so you can debug connectivity issues without weakening your security posture.
