# Solution 04: Multi-Region Edge Architecture

## Part A -- Multi-Region Origin Design

### Architecture Diagram

```
Users (NA)              Users (EU)              Users (APAC)
    |                       |                       |
    v                       v                       v
[CDN Edge: NA PoPs]   [CDN Edge: EU PoPs]    [CDN Edge: APAC PoPs]
    |                       |                       |
    v                       v                       v
[Origin Shield:       [Origin Shield:         [Origin Shield:
 us-east-1]            eu-west-1]              ap-southeast-1]
    |                       |                       |
    v                       v                       v
[App + API            [App + API              [App + API
 us-east-1]            eu-west-1]              ap-southeast-1]
    |                       |                       |
    v                       v                       v
[DB Primary            [DB Read Replica        [DB Read Replica
 us-east-1]            eu-west-1]              ap-southeast-1]
    |                       ^                       ^
    |                       |                       |
    +------ replicates -----+------- replicates ----+
```

### Write Path

```
User -> CDN Edge -> Origin Shield -> us-east-1 App -> DB Primary (us-east-1)
                                              |
                                              +-> Replicates to eu-west-1 (1-5s lag)
                                              +-> Replicates to ap-southeast-1 (5-15s lag)
```

Writes are always routed to `us-east-1`. The application in other regions detects
write requests (POST, PUT, DELETE) and proxies them to the primary region, or the
CDN routes write paths to the primary origin group.

### Why This Design Works

Each region has a complete read path: edge -> shield -> local app -> local read
replica. This means cache misses in APAC resolve in ~20ms (local read replica) instead
of ~200ms (cross-region to Virginia). The write path is longer but writes are
infrequent compared to reads (typically 1:100 ratio for e-commerce).

## Part B -- Origin Shield Configuration

| Shield Region | Protects Origins In | Purpose |
|---------------|---------------------|---------|
| us-east-1 | North America, South America | Collapses cache misses from ~150 NA/SA PoPs into single origin requests. Primary shield also handles write-path requests. |
| eu-west-1 | Europe, Africa | Collapses cache misses from ~80 EU/AF PoPs. Prevents EU traffic from hitting the primary region on cache misses. Reduces cross-Atlantic latency from ~100ms to ~5ms for EU users. |
| ap-southeast-1 | Asia, Oceania | Collapses cache misses from ~100 AP/OC PoPs. Critical for APAC latency: without a local shield, every cache miss crosses the Pacific (~150ms round trip). |

### Terraform Configuration

```hcl
# Primary origin group (us-east-1)
origin {
  domain_name = "app-us-east-1.example.com"
  origin_id   = "origin-us-east-1"

  custom_origin_config {
    http_port              = 80
    https_port             = 443
    origin_protocol_policy = "https-only"
    origin_ssl_protocols   = ["TLSv1.2"]
  }

  origin_shield {
    enabled              = true
    origin_shield_region = "us-east-1"
  }
}

# Secondary origin group (eu-west-1)
origin {
  domain_name = "app-eu-west-1.example.com"
  origin_id   = "origin-eu-west-1"

  custom_origin_config {
    http_port              = 80
    https_port             = 443
    origin_protocol_policy = "https-only"
    origin_ssl_protocols   = ["TLSv1.2"]
  }

  origin_shield {
    enabled              = true
    origin_shield_region = "eu-west-1"
  }
}

# Tertiary origin group (ap-southeast-1)
origin {
  domain_name = "app-ap-southeast-1.example.com"
  origin_id   = "origin-ap-southeast-1"

  custom_origin_config {
    http_port              = 80
    https_port             = 443
    origin_protocol_policy = "https-only"
    origin_ssl_protocols   = ["TLSv1.2"]
  }

  origin_shield {
    enabled              = true
    origin_shield_region = "ap-southeast-1"
  }
}
```

### Shield Collapse Behavior

Without origin shields, a popular article cache-missing across 100 PoPs generates 100
origin requests. With shields, each shield region collapses those into 1 request per
region. For 3 shield regions, that is 3 origin requests total -- a 97% reduction.

## Part C -- Latency-Based Failover

### Health Check Endpoint

```python
from flask import Flask, Response
import psycopg2
import redis
import time

app = Flask(__name__)

@app.route('/health')
def health_check():
    checks = {}

    # Check 1: Database read replica connectivity
    try:
        conn = psycopg2.connect(DATABASE_URL, connect_timeout=2)
        cursor = conn.cursor()

        # Check replication lag
        cursor.execute("SELECT EXTRACT(EPOCH FROM (NOW() - pg_last_xact_replay_timestamp()))")
        lag_seconds = cursor.fetchone()[0]

        if lag_seconds > 30:  # More than 30 seconds behind
            checks['database'] = {'status': 'degraded', 'lag_seconds': lag_seconds}
            return Response(
                '{"status": "unhealthy", "checks": ' + str(checks) + '}',
                status=503,
                content_type='application/json'
            )

        checks['database'] = {'status': 'healthy', 'lag_seconds': lag_seconds}
        cursor.close()
        conn.close()

    except Exception as e:
        checks['database'] = {'status': 'unhealthy', 'error': str(e)}
        return Response(
            '{"status": "unhealthy", "checks": ' + str(checks) + '}',
            status=503,
            content_type='application/json'
        )

    # Check 2: Application readiness
    try:
        r = redis.Redis(host=REDIS_HOST, socket_timeout=2)
        r.ping()
        checks['cache'] = {'status': 'healthy'}
    except Exception as e:
        checks['cache'] = {'status': 'degraded', 'error': str(e)}
        # Cache failure is degraded, not unhealthy -- app can still serve from DB

    return Response(
        '{"status": "healthy", "checks": ' + str(checks) + '}',
        status=200,
        content_type='application/json'
    )
```

### CloudFront Origin Group with Failover

```hcl
# Origin group with failover
origin_group {
  origin_id = "multi-region-failover"

  failover_criteria {
    status_codes = [500, 502, 503, 504]
  }

  member {
    origin_id = "origin-us-east-1"
  }

  member {
    origin_id = "origin-eu-west-1"
  }

  member {
    origin_id = "origin-ap-southeast-1"
  }
}
```

CloudFront retries members in order. If `us-east-1` returns 503, it tries `eu-west-1`,
then `ap-southeast-1`. The failover happens transparently to the user.

### Route 53 Health Checks (Alternative)

If you use Route 53 for DNS-based routing instead of CloudFront origin groups:

```hcl
resource "aws_route53_health_check" "us_east" {
  fqdn              = "app-us-east-1.example.com"
  port               = 443
  type               = "HTTPS"
  resource_path      = "/health"
  failure_threshold  = 2
  request_interval   = 10
  measure_latency    = true
}

resource "aws_route53_record" "app" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "app.example.com"
  type    = "A"

  alias {
    name    = aws_cloudfront_distribution.app.domain_name
    zone_id = aws_cloudfront_distribution.app.hosted_zone_id
  }

  latency_routing_policy {
    region = "us-east-1"
  }
  health_check_id = aws_route53_health_check.us_east.id
}
```

## Part D -- Cache Coherence Across Regions

### The Problem

When a product price is updated in `us-east-1`:
1. The CDN caches the old price at edge locations globally.
2. Read replicas in `eu-west-1` and `ap-southeast-1` have not yet received the update
   (replication lag).
3. A user in Tokyo requests the product page.

### Solution: Coordinated Invalidation with Lag Compensation

```
1. Price updated in us-east-1 DB primary
2. Event published to EventBridge (global)
3. Invalidator service receives event in each region
4. CloudFront purge issued globally (covers all edge locations)
5. Wait for replication lag (15 seconds) before allowing the next request
   to serve from the read replica
```

### Implementation

```python
import boto3
import time

class GlobalInvalidator:
    def __init__(self, distribution_id):
        self.cloudfront = boto3.client('cloudfront')
        self.distribution_id = distribution_id

    def invalidate_globally(self, paths: list, replication_lag_seconds: int = 15):
        """
        Invalidate CDN caches globally and account for replication lag.
        """
        # Step 1: Issue CloudFront invalidation (applies to all edge locations)
        self.cloudfront.create_invalidation(
            DistributionId=self.distribution_id,
            InvalidationBatch={
                'Paths': {
                    'Quantity': len(paths),
                    'Items': paths
                },
                'CallerReference': str(time.time_ns())
            }
        )

        # Step 2: Set a "recently invalidated" flag in the global cache (Redis)
        # This flag tells the origin handler to query the primary DB
        # for recently updated data instead of the local read replica
        for path in paths:
            redis_client.setex(
                f"invalidated:{path}",
                replication_lag_seconds + 5,  # TTL slightly longer than lag
                "true"
            )
```

### Origin Handler with Read-Your-Writes

```python
@app.route('/products/<product_id>')
def get_product(product_id):
    # Check if this product was recently invalidated
    was_invalidated = redis_client.get(f"invalidated:/products/{product_id}")

    if was_invalidated:
        # Query the primary database to avoid reading stale replica data
        product = query_primary_db(product_id)
    else:
        # Query the local read replica (faster, lower latency)
        product = query_local_replica(product_id)

    return Response(
        product.to_json(),
        headers={
            'Cache-Control': 'public, max-age=15',
            'X-Data-Source': 'primary' if was_invalidated else 'replica'
        }
    )
```

### Cache-Control Strategy

```
Cache-Control: public, max-age=15, stale-while-revalidate=60
```

- `max-age=15`: 15 seconds covers the replication lag window.
- `stale-while-revalidate=60`: During the 15-75 second window, serve stale content
  while revalidating in the background. This prevents latency spikes during invalidation.

## Part E -- Edge Function for Geographic Routing

### CloudFront Function

```javascript
function handler(event) {
    var request = event.request;
    var headers = request.headers;

    // Country-to-region mapping
    var regionMap = {
        // North America
        'US': 'us-east-1', 'CA': 'us-east-1', 'MX': 'us-east-1',
        'BR': 'us-east-1', 'AR': 'us-east-1', 'CO': 'us-east-1',

        // Europe
        'GB': 'eu-west-1', 'DE': 'eu-west-1', 'FR': 'eu-west-1',
        'IT': 'eu-west-1', 'ES': 'eu-west-1', 'NL': 'eu-west-1',
        'SE': 'eu-west-1', 'PL': 'eu-west-1',

        // Asia-Pacific
        'JP': 'ap-southeast-1', 'AU': 'ap-southeast-1', 'SG': 'ap-southeast-1',
        'KR': 'ap-southeast-1', 'IN': 'ap-southeast-1', 'TH': 'ap-southeast-1',
        'ID': 'ap-southeast-1', 'NZ': 'ap-southeast-1'
    };

    // Get viewer country (set by CloudFront based on IP geolocation)
    var country = 'US';  // default
    if (headers['cloudfront-viewer-country']) {
        country = headers['cloudfront-viewer-country'].value;
    }

    // Determine preferred origin region
    var preferredRegion = regionMap[country] || 'us-east-1';

    // Set the routing header for the origin
    request.headers['x-origin-region'] = { value: preferredRegion };
    request.headers['x-viewer-country'] = { value: country };

    // Observability: add routing info to response headers (for debugging)
    // Note: In production, use CloudFront real-time logs instead
    return request;
}
```

### Lambda@Edge Alternative (More Capable)

```javascript
'use strict';

exports.handler = async (event) => {
    const request = event.Records[0].cf.request;
    const headers = request.headers;

    const regionMap = {
        'US': 'us-east-1', 'CA': 'us-east-1',
        'GB': 'eu-west-1', 'DE': 'eu-west-1', 'FR': 'eu-west-1',
        'JP': 'ap-southeast-1', 'AU': 'ap-southeast-1', 'SG': 'ap-southeast-1'
    };

    const country = headers['cloudfront-viewer-country']
        ? headers['cloudfront-viewer-country'][0].value
        : 'US';

    const preferredRegion = regionMap[country] || 'us-east-1';

    // Override the origin based on region
    // This requires the distribution to have origins configured per region
    const originMap = {
        'us-east-1': {
            domain: 'app-us-east-1.example.com',
            id: 'origin-us-east-1'
        },
        'eu-west-1': {
            domain: 'app-eu-west-1.example.com',
            id: 'origin-eu-west-1'
        },
        'ap-southeast-1': {
            domain: 'app-ap-southeast-1.example.com',
            id: 'origin-ap-southeast-1'
        }
    };

    const origin = originMap[preferredRegion];
    request.origin = {
        custom: {
            domainName: origin.domain,
            port: 443,
            protocol: 'https',
            path: '',
            sslProtocols: ['TLSv1.2'],
            readTimeout: 30,
            keepaliveTimeout: 5
        }
    };
    request.headers['host'] = [{ key: 'Host', value: origin.domain }];

    // Log the routing decision
    console.log(JSON.stringify({
        action: 'geo-routing',
        country: country,
        preferredRegion: preferredRegion,
        targetOrigin: origin.domain,
        uri: request.uri,
        timestamp: new Date().toISOString()
    }));

    return request;
};
```

## Common Mistakes to Avoid

1. **Not accounting for replication lag in cache TTLs**: If your CDN TTL is shorter
   than the replication lag, users may hit a cache miss and read stale data from a
   lagging replica. Set TTLs at least 2x the maximum expected lag.

2. **Using the same origin group for reads and writes**: Write requests must go to the
   primary region. If you use the same origin group with failover, a failover to a
   secondary region means writes hit a read replica and fail silently or return errors.

3. **CloudFront invalidations do not target specific regions**: Unlike what many people
   assume, you cannot invalidate cache only in a specific region. All edge locations
   are invalidated globally. This is a feature, not a limitation -- it ensures
   consistency.

4. **Forgetting health check cost**: Each Route 53 health check costs $0.50/month.
   With 3 origins and multiple health check types (HTTP, HTTPS, TCP), costs add up.
   Use CloudWatch alarms as health check sources to reduce costs.

5. **Edge function latency adds up**: If your edge function makes external API calls
   (e.g., to check user preferences), each call adds latency. Prefer reading from
   headers, cookies, or a local key-value store (CloudFront Functions cache) over
   network calls.

## Key Takeaway

Multi-region CDN architecture is not just about deploying origin servers in multiple
regions. It requires coordinated cache invalidation that accounts for database
replication lag, origin shields to collapse cache misses, health-check-driven failover
for resilience, and edge functions for intelligent routing. The complexity is worth it:
sub-100ms p95 latency globally transforms the user experience for a global audience.
