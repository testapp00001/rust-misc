# Module 64: CDN and Edge -- Exercises

## Overview

These exercises build your understanding of Content Delivery Networks from foundational
architecture through production-grade multi-region edge deployments with origin failover
and security.

## Exercise Map

| # | Title | Type | Core Topics |
|---|-------|------|-------------|
| 01 | CDN Architecture and Edge Computing | Conceptual | Points of presence, request routing, edge compute models |
| 02 | Configure CloudFront for a Web Application | Guided | Distribution, origins, behaviors, OAC, custom error pages |
| 03 | Cache Invalidation Strategies | Independent | Purge, TTL management, versioned URLs, stale-while-revalidate |
| 04 | Multi-Region Edge Architecture | Challenge | Geo-routing, origin shields, latency-based failover |
| 05 | CDN with Origin Failover and Security | Integration | Origin groups, WAF integration, signed URLs, DDoS protection |

## How to Use These Exercises

1. Start with Exercise 01 even if you are experienced -- it establishes shared vocabulary.
2. Each exercise builds on concepts from the previous one.
3. Attempt each exercise on your own before reviewing hints.
4. Solutions are in the `../solutions/` directory.

## Prerequisites

- An AWS account with permissions to create CloudFront distributions, S3 buckets, and
  WAF rules (Exercise 02-05).
- Basic understanding of HTTP, DNS, and TLS.
- Familiarity with AWS CLI or Terraform for infrastructure provisioning.
- A domain name you control (optional, for custom domain exercises).
