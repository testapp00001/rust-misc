# Solution 03: Cache Invalidation Strategies

## Part A -- TTL-Based Invalidation

| Content Type | Update Frequency | Recommended TTL | Justification |
|-------------|------------------|-----------------|---------------|
| Breaking news articles | Every few minutes | 60 seconds | Short TTL balances freshness with origin load. Combined with stale-while-revalidate of 300s, users see near-instant updates while background revalidation keeps the cache warm. |
| Opinion columns | Once per day | 3600 seconds (1 hour) | Columns update once daily. A 1-hour TTL means most users get a cached response, and updates propagate within an hour of publication. Event-driven invalidation on publish provides instant propagation for the editor. |
| Static assets (JS/CSS/images) | On deploy | 31536000 seconds (1 year) | Static assets use content-hashed filenames (e.g., `app.a1b2c3.js`). The URL changes on every deploy, so the cached version is always correct. A 1-year TTL maximizes cache hit ratio with zero invalidation needed. |
| API responses (user-specific) | Per request | 0 (no cache) | User-specific data (profile, cart, recommendations) must not be cached at the CDN layer. Use `Cache-Control: private, no-store` to prevent caching. Edge compute can add personalization to cached base content. |
| Homepage | Every 5 minutes | 300 seconds | The homepage aggregates multiple content types (featured articles, trending, ads). A 5-minute TTL keeps it reasonably fresh without overwhelming the origin. Purge on critical homepage updates. |

### Why This TTL Strategy Works

The key principle is **content-addressed caching for immutable assets** and
**time-based caching for mutable content**. Immutable assets (hashed filenames) can
have infinite TTLs because the URL is the cache key and it changes when content
changes. Mutable content needs TTLs that match the acceptable staleness window.

## Part B -- Purge-Based Invalidation

### Invalidation API Implementation

```python
import boto3
import time
from collections import defaultdict
from datetime import datetime, timedelta

class InvalidationService:
    def __init__(self, distribution_id, max_paths_per_month=1000):
        self.client = boto3.client('cloudfront')
        self.distribution_id = distribution_id
        self.max_paths_per_month = max_paths_per_month
        self.usage_tracker = defaultdict(int)  # month -> count

    def create_invalidation(self, paths: list, invalidation_type: str = "exact"):
        # Rate limiting: check monthly usage
        current_month = datetime.utcnow().strftime("%Y-%m")
        total_paths = self.usage_tracker[current_month]

        if total_paths + len(paths) > self.max_paths_per_month:
            raise RateLimitExceeded(
                f"Monthly limit of {self.max_paths_per_month} paths reached. "
                f"Current usage: {total_paths}. Consider using versioned URLs instead."
            )

        # Normalize paths based on type
        normalized_paths = self._normalize_paths(paths, invalidation_type)

        # Create the invalidation
        response = self.client.create_invalidation(
            DistributionId=self.distribution_id,
            InvalidationBatch={
                'Paths': {
                    'Quantity': len(normalized_paths),
                    'Items': normalized_paths
                },
                'CallerReference': str(time.time_ns())
            }
        )

        # Track usage
        self.usage_tracker[current_month] += len(normalized_paths)

        return {
            'invalidation_id': response['Invalidation']['Id'],
            'status': response['Invalidation']['Status'],
            'paths': normalized_paths,
            'remaining_budget': self.max_paths_per_month - self.usage_tracker[current_month]
        }

    def _normalize_paths(self, paths, invalidation_type):
        if invalidation_type == "wildcard":
            # Wildcard counts as 1 path but invalidates all matches
            return [f"{path}*" if not path.endswith('*') else path for path in paths]
        elif invalidation_type == "prefix":
            return [f"{path}*" if not path.endswith('*') else path for path in paths]
        else:  # exact
            return paths

class RateLimitExceeded(Exception):
    pass
```

### Why Wildcards Are Cost-Effective

CloudFront charges per invalidation path, not per object invalidated. A wildcard
invalidation `/articles/*` counts as 1 path but invalidates thousands of objects.
This makes prefix-based invalidation extremely cost-effective for bulk updates (e.g.,
regenerating all article thumbnails).

## Part C -- Versioned URL Strategy

### Implementation

```python
import hashlib
from datetime import datetime

class VersionedURLGenerator:
    def __init__(self, db_connection):
        self.db = db_connection

    def get_article_url(self, article_id: str) -> str:
        # Fetch the current version from the database
        version = self.db.query(
            "SELECT version FROM articles WHERE id = %s",
            (article_id,)
        ).version

        return f"/articles/{article_id}?v={version}"

    def update_article(self, article_id: str, content: dict):
        # Update the article and increment version atomically
        self.db.execute("""
            UPDATE articles
            SET content = %s, version = version + 1, updated_at = NOW()
            WHERE id = %s
        """, (content, article_id))

    def get_asset_url(self, file_path: str, file_content: bytes) -> str:
        # Content-hash based versioning for static assets
        content_hash = hashlib.md5(file_content).hexdigest()[:8]
        # Transform: /assets/app.js -> /assets/app.a1b2c3d4.js
        parts = file_path.rsplit('.', 1)
        return f"{parts[0]}.{content_hash}.{parts[1]}"
```

### Template Usage

```html
<!-- In your template engine (e.g., Jinja2, Handlebars) -->
<script src="{{ versioned_url('/assets/app.js') }}"></script>
<link rel="stylesheet" href="{{ versioned_url('/assets/style.css') }}">

<!-- For article pages -->
<a href="{{ article_url(article.id) }}">{{ article.title }}</a>
```

### Why This Eliminates Cache Purging

With versioned URLs, the URL itself is the cache key. When the content changes:

1. The version increments (e.g., `?v=2` becomes `?v=3`).
2. The new URL has never been seen by the CDN, so it is a cache miss.
3. The CDN fetches and caches the new version.
4. The old version remains cached until its TTL expires (harmless -- nobody requests it).

This approach is **pull-based**: the CDN only fetches new content when a user requests
the new URL. There is no purge, no race condition, and no thundering herd. The tradeoff
is that old cached versions consume storage until TTL expiry, but this is negligible
cost compared to purge API calls.

## Part D -- Stale-While-Revalidate

### Cache-Control Configuration

Origin response header:

```
Cache-Control: public, max-age=60, stale-while-revalidate=300, stale-if-error=86400
```

- `max-age=60`: Serve from cache for 60 seconds without revalidating.
- `stale-while-revalidate=300`: Between 60 and 360 seconds, serve stale content
  immediately but trigger a background revalidation with the origin.
- `stale-if-error=86400`: If the origin returns an error during revalidation, continue
  serving stale content for up to 24 hours.

### Origin-Side Revalidation Handler

```python
from flask import Flask, request, Response
import hashlib
from datetime import datetime

app = Flask(__name__)

# Simulated article store
articles = {}

@app.route('/articles/<article_id>')
def get_article(article_id):
    article = articles.get(article_id)
    if not article:
        return Response(status=404)

    # Generate ETag from content hash
    etag = hashlib.md5(article['content'].encode()).hexdigest()[:16]
    last_modified = article['updated_at'].strftime('%a, %d %b %Y %H:%M:%S GMT')

    # Handle conditional requests
    if_none_match = request.headers.get('If-None-Match')
    if if_none_match == etag:
        return Response(status=304, headers={
            'ETag': etag,
            'Cache-Control': 'public, max-age=60, stale-while-revalidate=300'
        })

    if_modified_since = request.headers.get('If-Modified-Since')
    if if_modified_since == last_modified:
        return Response(status=304, headers={
            'ETag': etag,
            'Last-Modified': last_modified,
            'Cache-Control': 'public, max-age=60, stale-while-revalidate=300'
        })

    # Return full response with cache headers
    return Response(
        article['content'],
        headers={
            'ETag': etag,
            'Last-Modified': last_modified,
            'Cache-Control': 'public, max-age=60, stale-while-revalidate=300'
        }
    )
```

### How Stale-While-Revalidate Prevents Thundering Herd

Without stale-while-revalidate, when the cache expires, every concurrent request
triggers an origin fetch (thundering herd). With stale-while-revalidate:

1. The first request after TTL expiry gets the stale response immediately.
2. That request also triggers a single background revalidation.
3. All subsequent requests continue getting the stale response.
4. Once revalidation completes, the next request gets the fresh response.

This collapses what would be hundreds of origin requests into exactly one.

## Part E -- Event-Driven Invalidation

### Event Schema

```json
{
  "source": "cms.article-service",
  "detail-type": "ArticleUpdated",
  "detail": {
    "article_id": "breaking-news-123",
    "event_type": "updated",
    "timestamp": "2025-01-15T10:30:00Z",
    "idempotency_key": "article-breaking-news-123-1705312200",
    "affected_paths": [
      "/articles/breaking-news-123",
      "/",
      "/category/news"
    ],
    "invalidation_strategy": "purge",
    "priority": "high",
    "metadata": {
      "editor_id": "editor-456",
      "content_hash": "a1b2c3d4e5f6",
      "previous_version": 2,
      "new_version": 3
    }
  }
}
```

### Invalidator Service

```python
import boto3
import json
import hashlib
from datetime import datetime

class ArticleInvalidator:
    def __init__(self, distribution_id, dlq_url):
        self.cloudfront = boto3.client('cloudfront')
        self.sqs = boto3.client('sqs')
        self.distribution_id = distribution_id
        self.dlq_url = dlq_url
        self.processed_keys = set()  # In production, use Redis with TTL

    def handle_event(self, event: dict):
        detail = event['detail']
        idempotency_key = detail['idempotency_key']

        # Idempotency check: skip if already processed
        if idempotency_key in self.processed_keys:
            return {'status': 'skipped', 'reason': 'duplicate'}

        try:
            strategy = detail.get('invalidation_strategy', 'purge')
            paths = detail['affected_paths']

            if strategy == 'purge':
                result = self._purge_paths(paths)
            elif strategy == 'version':
                result = self._bump_version(detail)
            else:
                result = self._purge_paths(paths)

            # Mark as processed
            self.processed_keys.add(idempotency_key)

            return {
                'status': 'success',
                'invalidation_id': result.get('invalidation_id'),
                'strategy': strategy,
                'paths': paths
            }

        except Exception as e:
            # Send to dead-letter queue for retry
            self.sqs.send_message(
                QueueUrl=self.dlq_url,
                MessageBody=json.dumps({
                    'original_event': event,
                    'error': str(e),
                    'failed_at': datetime.utcnow().isoformat(),
                    'retry_count': 0
                }),
                MessageGroupId=detail['article_id'],
                MessageDeduplicationId=idempotency_key
            )
            return {'status': 'failed', 'error': str(e)}

    def _purge_paths(self, paths: list) -> dict:
        response = self.cloudfront.create_invalidation(
            DistributionId=self.distribution_id,
            InvalidationBatch={
                'Paths': {
                    'Quantity': len(paths),
                    'Items': paths
                },
                'CallerReference': str(datetime.utcnow().timestamp())
            }
        )
        return {
            'invalidation_id': response['Invalidation']['Id'],
            'status': response['Invalidation']['Status']
        }

    def _bump_version(self, detail: dict) -> dict:
        # Update the version in the database
        # This triggers versioned URL generation on next page render
        # No CDN purge needed -- the URL itself changes
        return {'strategy': 'version_bump', 'new_version': detail['metadata']['new_version']}
```

### Dead-Letter Queue Processing

The DLQ should have a retry policy:

1. Failed invalidations are retried up to 3 times with exponential backoff.
2. After 3 failures, the event is sent to a manual-review queue.
3. An alarm fires when the manual-review queue has messages, alerting the operations
   team.

## Common Mistakes to Avoid

1. **Purging `/*` to invalidate everything**: This works but counts as 1 path.
   However, it forces the CDN to refetch every object from the origin, causing a
   thundering herd. Use targeted invalidation or versioned URLs instead.

2. **Not using idempotency keys in event-driven invalidation**: Without idempotency,
   a retried event causes duplicate purge API calls, wasting the monthly free tier.

3. **Setting TTLs too low "just to be safe"**: A 5-second TTL on a high-traffic site
   means the origin receives a request every 5 seconds per edge PoP. With 400 PoPs,
   that is 80 origin requests per second for a single object.

4. **Forgetting `stale-if-error`**: Without it, an origin outage means the CDN stops
   serving content once the cache expires. With `stale-if-error`, the CDN continues
   serving stale content, maintaining availability during origin failures.

5. **Mixing purge and versioned URL strategies inconsistently**: If some paths use
   purging and others use versioned URLs, you need two invalidation pipelines. Pick
   one strategy per content type and be consistent.

## Key Takeaway

Cache invalidation is not a single strategy but a portfolio of approaches, each
optimized for different content lifecycle patterns. TTLs handle time-based freshness,
purges handle urgent updates, versioned URLs eliminate purging entirely for immutable
content, and stale-while-revalidate balances freshness with origin protection. The
event-driven pipeline ties these strategies together into an automated, reliable system.
