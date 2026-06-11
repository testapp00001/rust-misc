# Exercise 02: DNS Resolution Flow

**Type:** Guided
**Time:** 30 min
**Difficulty:** Easy-Medium

## Objective

Trace the complete DNS resolution process from client query to final answer, understanding each step and the role of recursive and authoritative servers.

## Scenario

A user types `app.example.com` into their browser. The following DNS infrastructure exists:

```
DNS Infrastructure:

Client (192.168.1.100)
  └── Stub Resolver (system DNS: 8.8.8.8)

Recursive Resolver (8.8.8.8 - Google Public DNS)

Root Servers (13 root server groups, a.root-servers.net through m.root-servers.net)

TLD Servers (.com)
  └── Managed by Verisign

Authoritative NS for example.com (ns1.example.com, ns2.example.com)
  └── ns1.example.com = 198.51.100.1
  └── ns2.example.com = 198.51.100.2

Zone data:
  app.example.com.  300  IN  A  203.0.113.50
  example.com.      300  IN  NS  ns1.example.com.
  example.com.      300  IN  NS  ns2.example.com.
```

## Tasks

### Part A: Trace the Resolution

Describe each step of the DNS resolution process for `app.example.com`. Number your steps and indicate which server is being queried at each point.

```
Step 1: Client asks ________________
Step 2: ________________
Step 3: ________________
...
```

<details>
<summary>Hint</summary>
The resolution is iterative: the recursive resolver starts at the root, then asks the TLD servers, then asks the authoritative servers. Each level refers the resolver to the next level down.
</details>

### Part B: Use dig to Trace Resolution

Run the following `dig` command and analyze the output. For each section (QUESTION, ANSWER, AUTHORITY, ADDITIONAL), explain what it tells you.

```bash
dig +trace app.example.com
```

Given this simulated output, explain what each line means:

```
; <<>> DiG 9.18 <<>> +trace app.example.com
;; global options: +cmd
.                       518400  IN  NS  a.root-servers.net.
.                       518400  IN  NS  b.root-servers.net.
;; Received 239 bytes from 8.8.8.8#53(8.8.8.8) in 12 ms

com.                    172800  IN  NS  a.gtld-servers.net.
com.                    172800  IN  NS  b.gtld-servers.net.
;; Received 897 bytes from 198.41.0.4#53(a.root-servers.net) in 24 ms

example.com.            172800  IN  NS  ns1.example.com.
example.com.            172800  IN  NS  ns2.example.com.
;; Received 215 bytes from 192.5.6.30#53(a.gtld-servers.net) in 30 ms

app.example.com.        300     IN  A   203.0.113.50
;; Received 56 bytes from 198.51.100.1#53(ns1.example.com) in 8 ms
```

<details>
<summary>Hint</summary>
Each block shows the response from a different server in the chain. The `Received` line tells you which server answered and how many bytes were in the response.
</details>

### Part C: Recursive vs Iterative Queries

Explain the difference between recursive and iterative DNS queries. For each scenario below, identify whether it involves a recursive or iterative query:

1. Client asks its configured DNS server to resolve `example.com`
2. Recursive resolver asks root server for `.com` NS records
3. Recursive resolver asks TLD server for `example.com` NS records
4. Client receives the final A record answer

<details>
<summary>Hint</summary>
A recursive query means "please resolve this fully and give me the answer." An iterative query means "give me the best answer you have, or tell me who to ask next."
</details>

## Success Criteria

- [ ] You can trace a DNS query from client through root, TLD, and authoritative servers
- [ ] You can read and interpret `dig +trace` output
- [ ] You understand the difference between recursive and iterative resolution
- [ ] You know the role of the recursive resolver as the intermediary

## What You Should Understand After This Exercise

DNS resolution is a hierarchical, iterative process. The client asks its resolver, which then walks the tree from root servers to TLD servers to authoritative servers. Understanding this flow is essential for diagnosing DNS issues, because a failure at any level can break resolution entirely.
