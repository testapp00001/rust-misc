# Exercise 02: Configure CloudFront for a Web Application (Guided)

## Objective

Configure an AWS CloudFront distribution that serves a static website from S3, with
proper origin access control, cache behaviors, custom error pages, and HTTP/2 support.

## Background

Your team has a static web application (React SPA) hosted in an S3 bucket. You need
to front it with CloudFront to reduce latency for global users, enforce HTTPS, and
restrict direct S3 access. The application has both cacheable assets (HTML, JS, CSS,
images) and dynamic API calls that should not be cached.

## Instructions

### Part A -- S3 Bucket and Origin Access

1. Create an S3 bucket named `my-app-assets-<your-id>` with static website hosting
   disabled (we will use CloudFront, not S3 website hosting).
2. Upload sample files:
   - `index.html` (the SPA entry point)
   - `assets/app.js`
   - `assets/style.css`
   - `images/logo.png`
3. Create an Origin Access Control (OAC) that allows CloudFront to read from the
   bucket.
4. Write the S3 bucket policy that grants the OAC `s3:GetObject` permission.

### Part B -- CloudFront Distribution

Create a CloudFront distribution with the following configuration:

```
Distribution Settings:
  - Price Class: PriceClass_100 (US, Canada, Europe) or PriceClass_All (global)
  - HTTP Version: HTTP/2 and HTTP/3
  - Default Root Object: index.html
  - Viewer Protocol Policy: Redirect HTTP to HTTPS

Origins:
  - S3 origin (using OAC from Part A)

Behaviors:
  - Default behavior (*): cache with TTL of 86400s (1 day)
  - /assets/*: cache with TTL of 604800s (7 days), include immutable directive
  - /api/*: no caching, forward all headers, methods, and query strings
```

### Part C -- Custom Error Pages

Configure custom error responses so that the SPA routing works correctly:

```
Error Code: 403
Response Page: /index.html
HTTP Response Code: 200
TTL: 300

Error Code: 404
Response Page: /index.html
HTTP Response Code: 200
TTL: 300
```

Explain why this is necessary for a single-page application.

### Part D -- Cache Policy and Response Headers

Create a cache policy or use a managed policy that:

1. Includes `Accept` and `Accept-Encoding` as cache key headers.
2. Enables Brotli compression.
3. Sets a default TTL of 86400 seconds.

Create a response headers policy that adds:

1. `Strict-Transport-Security` (HSTS) with max-age of 31536000.
2. `X-Content-Type-Options: nosniff`.
3. `Content-Security-Policy` appropriate for a static site.

### Part E -- Terraform Representation

Write the Terraform configuration that codifies the entire distribution from Parts A-D.
Use `aws_cloudfront_distribution`, `aws_cloudfront_origin_access_control`, and
`aws_s3_bucket_policy` resources.

## Success Criteria

- [ ] S3 bucket is only accessible through CloudFront (direct access returns 403).
- [ ] Static assets are served with correct cache headers (check with `curl -I`).
- [ ] SPA routing works: navigating directly to `/about` returns the SPA, not a 403.
- [ ] HTTP requests are redirected to HTTPS.
- [ ] Security headers are present in all responses.
- [ ] Terraform plan shows the expected resources without errors.

## Hints

<details>
<summary>Hint 1 -- OAC vs OAI</summary>
Origin Access Control (OAC) is the successor to Origin Access Identity (OAI). OAC
supports all S3 features including SSE-KMS encrypted objects, request signing with
SigV4, and does not require a separate CloudFront user. Always prefer OAC for new
distributions.
</details>

<details>
<summary>Hint 2 -- SPA routing with custom errors</summary>
Single-page applications use client-side routing. When a user navigates to
`/about`, there is no `about.html` in S3 -- the SPA's JavaScript handles rendering.
The custom error page configuration tells CloudFront: "When S3 returns 403 or 404,
serve `/index.html` with a 200 status instead." This lets the SPA's router handle
the path.
</details>

<details>
<summary>Hint 3 -- Cache key headers</summary>
Including `Accept-Encoding` in the cache key ensures that Brotli-compressed and
Gzip-compressed responses are cached separately. Without this, a Brotli-capable
client might receive a Gzip-cached response or vice versa.
</details>

<details>
<summary>Hint 4 -- Terraform dynamic blocks for error pages</summary>
```hcl
dynamic "custom_error_response" {
  for_each = [403, 404]
  content {
    error_code            = custom_error_response.value
    response_page_path    = "/index.html"
    response_code         = 200
    error_caching_min_ttl = 300
  }
}
```
</details>
