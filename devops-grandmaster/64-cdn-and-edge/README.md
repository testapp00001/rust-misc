# 64 - CDN and Edge

## Problem

Your application runs in a single region (us-east-1). A user in Tokyo requests your homepage. The HTTP request travels 10,000 km across the Pacific, hits your load balancer, and returns a response. Round-trip latency is 150-200ms on a good day. Now multiply that by millions of requests. Your static assets (images, CSS, JavaScript) are served from the same origin server, consuming bandwidth and compute that could be used for dynamic content. During traffic spikes, your origin gets overwhelmed serving the same files over and over.

A Content Delivery Network (CDN) solves this by caching your content at edge locations distributed globally. When the Tokyo user requests your homepage, the CDN serves it from a Tokyo edge node in 5-10ms. Only cache misses go back to your origin. This reduces latency, cuts origin load, and provides DDoS protection as a side effect.

## Naive Way

Serve everything directly from your application server and rely on browser caching.

```python
# app.py - Flask serving static files directly
from flask import Flask, send_from_directory, make_response

app = Flask(__name__)

@app.route("/static/<path:filename>")
def serve_static(filename):
    response = make_response(send_from_directory("static", filename))
    # Set browser cache headers
    response.headers["Cache-Control"] = "public, max-age=3600"
    return response

@app.route("/")
def index():
    return """
    <html>
    <head>
        <link rel="stylesheet" href="/static/style.css">
    </head>
    <body>
        <h1>Hello World</h1>
        <img src="/static/logo.png">
        <script src="/static/app.js"></script>
    </body>
    </html>
    """
```

```bash
# nginx config serving static files
server {
    listen 80;
    server_name example.com;

    location /static/ {
        alias /var/www/static/;
        expires 1h;
        add_header Cache-Control "public";
    }

    location / {
        proxy_pass http://app:5000;
    }
}
```

This serves everything from a single location. Users far from the server get high latency. The origin handles every request, even for unchanged static files. No geographic distribution. No edge computing. The browser cache helps for repeat visits but does nothing for first-time visitors.

## Right Way

Use a CDN like CloudFront, Cloudflare, or Fastly to cache and serve content from edge locations.

**CloudFront distribution with S3 origin for static assets:**

```yaml
# CloudFormation - CloudFront distribution
AWSTemplateFormatVersion: "2010-09-09"
Resources:
  StaticBucket:
    Type: AWS::S3::Bucket
    Properties:
      BucketName: my-app-static-assets
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true

  CloudFrontDistribution:
    Type: AWS::CloudFront::Distribution
    Properties:
      DistributionConfig:
        Enabled: true
        DefaultRootObject: index.html
        Origins:
          # Static assets from S3
          - Id: S3Origin
            DomainName: !GetAtt StaticBucket.RegionalDomainName
            S3OriginConfig:
              OriginAccessIdentity: !Sub "origin-access-identity/cloudfront/${CloudFrontOAI}"
          # Dynamic content from application server
          - Id: AppOrigin
            DomainName: api.example.com
            CustomOriginConfig:
              HTTPPort: 80
              HTTPSPort: 443
              OriginProtocolPolicy: https-only
        DefaultCacheBehavior:
          TargetOriginId: S3Origin
          ViewerProtocolPolicy: redirect-to-https
          CachePolicyId: 658327ea-f89d-4fab-a63d-7e88639e58f6  # CachingOptimized
          Compress: true
        CacheBehaviors:
          # API requests go to origin, no caching
          - PathPattern: /api/*
            TargetOriginId: AppOrigin
            ViewerProtocolPolicy: redirect-to-https
            CachePolicyId: 4135ea2d-6df8-44a3-9df3-4b5a84be39ad  # CachingDisabled
            OriginRequestPolicyId: 216adef6-5c7f-47e4-b989-5492eafa07d3  # AllViewer
          # Static assets cached aggressively
          - PathPattern: /static/*
            TargetOriginId: S3Origin
            ViewerProtocolPolicy: redirect-to-https
            CachePolicyId: 658327ea-f89d-4fab-a63d-7e88639e58f6
            Compress: true
        Aliases:
          - www.example.com
        ViewerCertificate:
          AcmCertificateArn: arn:aws:acm:us-east-1:123456789:certificate/abc-123
          SslSupportMethod: sni-only

  CloudFrontOAI:
    Type: AWS::CloudFront::CloudFrontOriginAccessIdentity
    Properties:
      CloudFrontOriginAccessIdentityConfig:
        Comment: OAI for static assets
```

**Cache headers from your application:**

```python
# cache_headers.py
from flask import make_response
import hashlib

def serve_with_cache(response, content, cache_type="static"):
    """Set appropriate cache headers based on content type."""
    resp = make_response(content)

    if cache_type == "static":
        # Immutable assets with content hash in filename
        resp.headers["Cache-Control"] = "public, max-age=31536000, immutable"
    elif cache_type == "dynamic":
        # Short cache with stale-while-revalidate
        resp.headers["Cache-Control"] = "public, max-age=60, stale-while-revalidate=300"
    elif cache_type == "private":
        # User-specific content, cache in browser only
        resp.headers["Cache-Control"] = "private, max-age=0, no-cache"
    elif cache_type == "no-cache":
        # Never cache
        resp.headers["Cache-Control"] = "no-store"

    # ETag for conditional requests
    etag = hashlib.md5(content).hexdigest()
    resp.headers["ETag"] = f'"{etag}"'

    return resp
```

```bash
# Upload static assets with content-hash filenames
# style.a1b2c3d4.css -> cached forever, new hash = new file
aws s3 sync ./dist s3://my-app-static-assets/static/ \
  --cache-control "public, max-age=31536000, immutable"

# Invalidate CDN cache when needed
aws cloudfront create-invalidation \
  --distribution-id E1234567890 \
  --paths "/index.html" "/api/config"
```

**Cloudflare alternative with edge workers:**

```javascript
// cloudflare-worker.js - Edge compute for dynamic content
addEventListener("fetch", (event) => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  const cache = caches.default;
  const cacheKey = new Request(request.url, request);
  let response = await cache.match(cacheKey);

  if (!response) {
    // Cache miss - fetch from origin
    response = await fetch(request);

    // Only cache successful responses
    if (response.status === 200) {
      const headers = new Headers(response.headers);
      headers.set("Cache-Control", "public, max-age=3600");
      headers.set("X-Cache-Status", "MISS");

      response = new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers: headers,
      });

      // Store in edge cache
      event.waitUntil(cache.put(cacheKey, response.clone()));
    }
  } else {
    // Cache hit
    const headers = new Headers(response.headers);
    headers.set("X-Cache-Status", "HIT");
    response = new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: headers,
    });
  }

  return response;
}
```

## Production Way

Production CDN setups involve cache key design, origin shielding, edge computing, cache purging strategies, and cost optimization.

**Cache key design for personalized content:**

```python
# Vary cache by specific headers, not cookies
@app.route("/api/products")
def get_products():
    response = jsonify(products)
    # Vary by Accept-Language for i18n, but cache per language
    response.headers["Vary"] = "Accept-Language"
    response.headers["Cache-Control"] = "public, max-age=300"
    return response

# For user-specific content, use a two-tier approach
@app.route("/api/dashboard")
def dashboard():
    # Fetch shared data from cache
    shared_data = cache.get("dashboard-shared") or fetch_shared_data()
    cache.set("dashboard-shared", shared_data, timeout=60)

    # Combine with user-specific data (not cached at CDN)
    user_data = fetch_user_data(current_user.id)
    return jsonify({**shared_data, **user_data})
```

**Origin shielding to reduce origin load:**

```yaml
# CloudFront with origin shield
# Reduces origin requests by adding a regional cache layer
DistributionConfig:
  Origins:
    - Id: AppOrigin
      DomainName: api.example.com
      OriginShield:
        Enabled: true
        OriginShieldRegion: us-east-1  # Shield region closest to origin
```

**Edge-side includes (ESI) and edge compute:**

```javascript
// Cloudflare Worker - personalize at the edge
async function handleRequest(request) {
  const url = new URL(request.url);

  // Get cached page from edge
  const cachedResponse = await fetch(request.url + "?raw=1");

  // Read user token from cookie
  const token = getCookie(request, "auth_token");

  if (!token) {
    return cachedResponse;
  }

  // Fetch user-specific data at the edge
  const userData = await fetch(`https://api.example.com/user/me`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  const user = await userData.json();

  // Inject personalization into cached HTML
  let html = await cachedResponse.text();
  html = html.replace("{{user_name}}", user.name);
  html = html.replace("{{avatar_url}}", user.avatar_url);

  return new Response(html, {
    headers: {
      "Content-Type": "text/html",
      "Cache-Control": "private, max-age=0",
    },
  });
}

function getCookie(request, name) {
  const cookie = request.headers.get("Cookie") || "";
  const match = cookie.match(new RegExp(`${name}=([^;]+)`));
  return match ? match[1] : null;
}
```

**Cache purging strategy:**

```bash
# Purge by URL (surgical)
aws cloudfront create-invalidation \
  --distribution-id E1234567890 \
  --paths "/products/123" "/products/456"

# Purge by path prefix (broader)
aws cloudfront create-invalidation \
  --distribution-id E1234567890 \
  --paths "/products/*"

# Purge by cache tag (most efficient)
# Your application sets: Cache-Tag: product-123, product-listing
aws cloudfront create-invalidation \
  --distribution-id E1234567890 \
  --paths "/*" \
  --cli-input-json '{
    "InvalidationBatch": {
      "Paths": {
        "Quantity": 1,
        "Items": ["/*"]
      },
      "CallerReference": "tag-purge-'$(date +%s)'"
    }
  }'
```

**Cost optimization:**

```python
# Monitor cache hit ratio
# Target: > 90% for static assets, > 70% for API responses
# CloudWatch metric: CacheHitRate

import boto3

cloudwatch = boto3.client("cloudwatch")

def check_cache_health(distribution_id):
    response = cloudwatch.get_metric_statistics(
        Namespace="AWS/CloudFront",
        MetricName="CacheHitRate",
        Dimensions=[
            {"Name": "DistributionId", "Value": distribution_id},
        ],
        StartTime=datetime.utcnow() - timedelta(hours=1),
        EndTime=datetime.utcnow(),
        Period=300,
        Statistics=["Average"],
    )

    for datapoint in response["Datapoints"]:
        if datapoint["Average"] < 70:
            send_alert(f"Cache hit rate dropped to {datapoint['Average']}%")
```

## Hands-On Lab

**Exercise: Set up CloudFront with S3 for static site hosting**

1. Create an S3 bucket and upload content:

```bash
# Create bucket
aws s3 mb s3://cdn-lab-assets-$(whoami)

# Create sample static files
mkdir -p static-site
cat > static-site/index.html <<EOF
<!DOCTYPE html>
<html>
<head><title>CDN Lab</title>
<link rel="stylesheet" href="/style.css">
</head>
<body>
<h1>CDN Edge Test</h1>
<p>Server time: <!-- replaced by edge --></p>
<img src="/logo.png">
</body>
</html>
EOF

cat > static-site/style.css <<EOF
body { font-family: sans-serif; margin: 40px; }
h1 { color: #2563eb; }
EOF

# Upload with cache headers
aws s3 sync static-site/ s3://cdn-lab-assets-$(whoami)/ \
  --cache-control "public, max-age=3600"
```

2. Use Cloudflare (free tier) as a CDN alternative:

```bash
# Sign up at cloudflare.com (free plan)
# Add your domain
# Update nameservers at your registrar
# Enable proxy (orange cloud) for your A record
```

3. Test cache behavior with curl:

```bash
# First request - cache MISS
curl -I https://your-site.example.com/style.css
# Look for: cf-cache-status: MISS or X-Cache: Miss from cloudfront

# Second request - cache HIT
curl -I https://your-site.example.com/style.css
# Look for: cf-cache-status: HIT or X-Cache: Hit from cloudfront

# Test from different locations using online tools
# https://www.whatsmydns.net/ - check DNS propagation
# https://tools.keycdn.com/performance - test from multiple locations
```

4. Set up a simple Cloudflare Worker for edge compute:

```javascript
// worker.js
export default {
  async fetch(request) {
    const url = new URL(request.url);

    if (url.pathname === "/api/time") {
      // Generate at the edge, not the origin
      return new Response(
        JSON.stringify({
          timestamp: new Date().toISOString(),
          edge_location: request.cf?.colo || "unknown",
          country: request.cf?.country || "unknown",
        }),
        {
          headers: { "Content-Type": "application/json" },
        }
      );
    }

    // For other paths, fetch from origin with caching
    const cache = caches.default;
    let response = await cache.match(request);

    if (!response) {
      response = await fetch(request);
      if (response.ok) {
        const cloned = response.clone();
        const headers = new Headers(cloned.headers);
        headers.set("Cache-Control", "public, max-age=3600");
        event.waitUntil(cache.put(request, new Response(cloned.body, { headers })));
      }
    }

    return response;
  },
};
```

5. Measure the difference:

```bash
# Time requests to origin vs CDN
# Origin (direct to server)
time curl -s -o /dev/null https://origin.example.com/style.css

# CDN (cached at edge)
time curl -s -o /dev/null https://www.example.com/style.css

# Compare latency from different regions using:
# curl -w "time_total: %{time_total}\n" -o /dev/null -s URL
```

## Limitation

CDNs solve the problem of serving static content and caching at the edge. But once your architecture involves dozens of microservices communicating with each other, the challenge shifts from user-to-service communication to service-to-service communication. Services need mutual TLS, traffic splitting, retries, circuit breaking, and observability. Managing this at the application level in every service is unsustainable. You need a dedicated infrastructure layer for service-to-service networking: a service mesh.

## Next Topic

[65 - Service Mesh](../65-service-mesh/README.md) - Istio, Linkerd, mTLS, and traffic management for service-to-service communication.
