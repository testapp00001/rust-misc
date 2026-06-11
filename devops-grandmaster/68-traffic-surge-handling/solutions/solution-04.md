# Solution 04: Graceful Degradation Under Extreme Load

## Part A: Degradation Levels

### Level Definitions

| Level | Enter Threshold | Exit Threshold | Duration |
|-------|----------------|----------------|----------|
| NORMAL | Default state | Default state | - |
| ELEVATED | CPU > 70% OR error_rate > 2% OR queue_depth > 10K OR p99 > 500ms | CPU < 60% AND error_rate < 1% AND queue_depth < 5K AND p99 < 300ms | 30s hysteresis |
| HIGH | CPU > 85% OR error_rate > 5% OR queue_depth > 30K OR p99 > 1s | CPU < 75% AND error_rate < 3% AND queue_depth < 15K AND p99 < 700ms | 30s hysteresis |
| CRITICAL | CPU > 95% OR error_rate > 15% OR queue_depth > 80K OR p99 > 3s | CPU < 85% AND error_rate < 10% AND queue_depth < 50K AND p99 < 2s | 30s hysteresis |

### Feature Map by Level

| Feature | NORMAL | ELEVATED | HIGH | CRITICAL |
|---------|--------|----------|------|----------|
| Product listing | Full (dynamic) | Full (dynamic) | Cached (1hr TTL) | CDN only (stale-ok) |
| Search | Full | Full | Cached results (5min) | Disabled (browse only) |
| Add to cart | Full | Full | Full | localStorage only |
| Checkout | Full | Full | Full (queue) | Disabled |
| Recommendations | Full | Disabled | Disabled | Disabled |
| User reviews | Full | Cached (1hr) | CDN only | Disabled |
| Inventory check | Full | Cached (30s) | Cached (5min) | "Available" (optimistic) |

### User-Visible Impact

| Level | User Experience |
|-------|----------------|
| NORMAL | Full functionality, fast response times |
| ELEVATED | No recommendations, slightly slower search |
| HIGH | No recommendations, cached search results, no real-time reviews |
| CRITICAL | Browse-only mode. Products visible (cached). Cart works locally. Cannot checkout. Message: "High traffic -- your cart is saved, please try checkout in a few minutes." |

### Recovery

When load decreases below exit thresholds for 30 seconds (hysteresis), the system automatically re-enables features in reverse order: CRITICAL -> HIGH -> ELEVATED -> NORMAL. Each transition is logged for monitoring.

### Why This Works

- **Exit thresholds are lower than enter thresholds** (hysteresis) prevents oscillation
- **Critical features stay on** until CRITICAL level (product listing, cart)
- **High-cost features are disabled first** (recommendations, search)
- **CDN absorbs load** at HIGH and CRITICAL levels
- **User sees reduced functionality, not errors**

---

## Part B: Degradation Decision Logic

```python
import time
import logging
from enum import Enum
from dataclasses import dataclass

logger = logging.getLogger(__name__)

class DegradationLevel(Enum):
    NORMAL = 0
    ELEVATED = 1
    HIGH = 2
    CRITICAL = 3

@dataclass
class SystemMetrics:
    cpu_utilization: float      # 0.0 - 1.0
    error_rate: float           # 0.0 - 1.0
    queue_depth: int
    p99_latency_ms: float

class DegradationManager:
    """Degradation level manager with hysteresis."""

    # Enter thresholds (more sensitive)
    ENTER_THRESHOLDS = {
        DegradationLevel.ELEVATED: {"cpu": 0.70, "error_rate": 0.02, "queue": 10000, "p99": 500},
        DegradationLevel.HIGH:     {"cpu": 0.85, "error_rate": 0.05, "queue": 30000, "p99": 1000},
        DegradationLevel.CRITICAL: {"cpu": 0.95, "error_rate": 0.15, "queue": 80000, "p99": 3000},
    }

    # Exit thresholds (less sensitive -- must recover significantly)
    EXIT_THRESHOLDS = {
        DegradationLevel.ELEVATED: {"cpu": 0.60, "error_rate": 0.01, "queue": 5000, "p99": 300},
        DegradationLevel.HIGH:     {"cpu": 0.75, "error_rate": 0.03, "queue": 15000, "p99": 700},
        DegradationLevel.CRITICAL: {"cpu": 0.85, "error_rate": 0.10, "queue": 50000, "p99": 2000},
    }

    # Features enabled at each level
    FEATURE_MAP = {
        DegradationLevel.NORMAL: {
            "product_listing", "search", "add_to_cart", "checkout",
            "recommendations", "user_reviews", "inventory_check"
        },
        DegradationLevel.ELEVATED: {
            "product_listing", "search", "add_to_cart", "checkout",
            "user_reviews", "inventory_check"
        },
        DegradationLevel.HIGH: {
            "product_listing", "add_to_cart", "checkout", "inventory_check"
        },
        DegradationLevel.CRITICAL: {
            "product_listing", "add_to_cart"
        },
    }

    def __init__(self, hysteresis_seconds: int = 30):
        self.current_level = DegradationLevel.NORMAL
        self.last_change_time = time.time()
        self.hysteresis_seconds = hysteresis_seconds

    def evaluate(self, metrics: SystemMetrics) -> DegradationLevel:
        """Evaluate current metrics and return appropriate degradation level."""
        target_level = self._determine_target_level(metrics)

        if target_level != self.current_level:
            # Apply hysteresis
            if time.time() - self.last_change_time >= self.hysteresis_seconds:
                previous = self.current_level
                self.current_level = target_level
                self.last_change_time = time.time()
                logger.warning(
                    f"Degradation level changed: {previous.name} -> {target_level.name} "
                    f"(cpu={metrics.cpu_utilization:.1%}, errors={metrics.error_rate:.1%}, "
                    f"queue={metrics.queue_depth}, p99={metrics.p99_latency_ms}ms)"
                )

        return self.current_level

    def _determine_target_level(self, metrics: SystemMetrics) -> DegradationLevel:
        """Determine target level based on metrics (before hysteresis)."""
        # Check from highest to lowest severity
        for level in [DegradationLevel.CRITICAL, DegradationLevel.HIGH, DegradationLevel.ELEVATED]:
            if self.current_level < level:
                # Entering a higher level
                t = self.ENTER_THRESHOLDS[level]
                if self._exceeds(metrics, t):
                    return level
            elif self.current_level >= level:
                # Exiting to a lower level
                t = self.EXIT_THRESHOLDS[level]
                if not self._exceeds(metrics, t):
                    continue  # Can exit this level
                else:
                    return level  # Must stay at this level or higher

        return DegradationLevel.NORMAL

    def _exceeds(self, metrics: SystemMetrics, thresholds: dict) -> bool:
        """Check if any metric exceeds the threshold."""
        return (
            metrics.cpu_utilization >= thresholds["cpu"] or
            metrics.error_rate >= thresholds["error_rate"] or
            metrics.queue_depth >= thresholds["queue"] or
            metrics.p99_latency_ms >= thresholds["p99"]
        )

    def get_enabled_features(self) -> set:
        """Return the set of currently enabled features."""
        return self.FEATURE_MAP[self.current_level]

    def is_feature_enabled(self, feature: str) -> bool:
        """Check if a specific feature is currently enabled."""
        return feature in self.get_enabled_features()
```

### Why This Works

- **Separate enter/exit thresholds** prevent oscillation: entering ELEVATED requires CPU > 70%, but exiting requires CPU < 60%
- **30-second hysteresis** means the system must sustain the new load level before changing features
- **Feature map** makes degradation decisions explicit and auditable
- **Logging** captures every level transition for post-incident analysis
- **Check from highest severity first** ensures the system always applies the most conservative degradation

---

## Part C: CDN Fallback Strategy

### Nginx Cache Configuration

```nginx
# /etc/nginx/conf.d/cdn-fallback.conf
proxy_cache_path /var/cache/nginx/products levels=1:2
    keys_zone=products:100m max_size=10g inactive=24h;
proxy_cache_path /var/cache/nginx/static levels=1:2
    keys_zone=static:50m max_size=5g inactive=7d;

server {
    listen 80;

    # Product pages: aggressive caching with stale-if-error
    location /products/ {
        proxy_cache products;
        proxy_cache_valid 200 1h;
        proxy_cache_valid 404 1m;
        proxy_cache_use_stale error timeout updating http_500 http_502 http_503 http_504;
        proxy_cache_background_update on;
        proxy_cache_lock on;
        proxy_cache_lock_timeout 5s;

        # Serve stale for up to 24 hours if origin is down
        add_header Cache-Control "public, max-age=3600, stale-if-error=86400, stale-while-revalidate=300";
        add_header X-Cache-Status $upstream_cache_status;

        proxy_pass http://origin;
    }

    # Static assets: long cache, immutable
    location /static/ {
        proxy_cache static;
        proxy_cache_valid 200 7d;
        add_header Cache-Control "public, max-age=604800, immutable";
        proxy_pass http://origin;
    }

    # API endpoints: short cache with stale-if-error
    location /api/products {
        proxy_cache products;
        proxy_cache_valid 200 5m;
        proxy_cache_use_stale error timeout updating http_503;
        add_header Cache-Control "public, max-age=300, stale-if-error=3600";
        proxy_pass http://origin;
    }

    # Cart: no caching, but graceful fallback
    location /api/cart {
        # No cache -- dynamic, user-specific
        add_header Cache-Control "private, no-store";

        # If origin returns 503, serve fallback page
        error_page 503 = @cart_fallback;
        proxy_pass http://origin;
    }

    location @cart_fallback {
        default_type application/json;
        return 503 '{"error": "high_traffic", "message": "Cart is temporarily unavailable. Your items are saved locally.", "retry_after": 30}';
    }

    # Checkout: queue-based, not cacheable
    location /api/checkout {
        proxy_pass http://origin;
        # No caching, no fallback -- must be processed by origin
    }

    # Static high-traffic page for CRITICAL degradation
    location / {
        # If origin is completely down, serve static page
        proxy_cache static;
        proxy_cache_valid 200 5m;
        proxy_cache_use_stale error timeout updating http_503;
        add_header Cache-Control "public, max-age=300, stale-if-error=86400";

        proxy_pass http://origin;

        error_page 503 = @maintenance_page;
    }

    location @maintenance_page {
        root /var/www/fallback;
        try_files /maintenance.html =503;
    }
}
```

### Static Maintenance Page

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>High Traffic - Please Wait</title>
    <style>
        body { font-family: system-ui; text-align: center; padding: 60px 20px; }
        .container { max-width: 600px; margin: 0 auto; }
        h1 { color: #333; }
        p { color: #666; font-size: 18px; line-height: 1.6; }
        .status { background: #f0f0f0; padding: 20px; border-radius: 8px; margin: 20px 0; }
        .cart-info { background: #e8f5e9; padding: 15px; border-radius: 8px; }
        #countdown { font-size: 24px; font-weight: bold; color: #1976d2; }
    </style>
</head>
<body>
    <div class="container">
        <h1>We're Experiencing High Traffic</h1>
        <p>Our site is currently handling a large number of visitors.
           Product browsing is available below. Checkout will be available shortly.</p>
        <div class="status">
            <p>Estimated wait time: <span id="countdown">60</span> seconds</p>
            <p>This page will automatically refresh.</p>
        </div>
        <div class="cart-info">
            <p><strong>Your cart is safe.</strong> Items are saved in your browser
               and will sync when the system recovers.</p>
        </div>
        <script>
            // Auto-refresh after countdown
            let seconds = 60;
            setInterval(() => {
                seconds--;
                document.getElementById('countdown').textContent = seconds;
                if (seconds <= 0) location.reload();
            }, 1000);

            // Cart stored in localStorage
            const cart = JSON.parse(localStorage.getItem('cart') || '[]');
            if (cart.length > 0) {
                document.querySelector('.cart-info').innerHTML +=
                    '<p>Items in cart: ' + cart.length + '</p>';
            }
        </script>
    </div>
</body>
</html>
```

### Why This Works

- **`stale-if-error=86400`** serves cached content for up to 24 hours when the origin returns errors
- **`stale-while-revalidate=300`** serves stale content while refreshing in the background
- **`proxy_cache_use_stale`** covers multiple failure modes (timeout, 5xx, updating)
- **`proxy_cache_lock`** prevents cache stampedes (only one request populates the cache)
- **localStorage cart** preserves user state even when the server is down
- **Auto-refresh** recovers users automatically when the system stabilizes

---

## Common Mistakes to Avoid

- **No hysteresis.** Without enter/exit threshold separation, the system oscillates rapidly between levels, causing inconsistent user experience.

- **Disabling checkout too early.** Checkout is a critical feature. Only disable it at CRITICAL level, and even then, queue the request rather than rejecting it.

- **Not storing cart in localStorage.** If the server goes down and the cart is only in server-side sessions, users lose their carts. Client-side storage survives server failures.

- **Caching user-specific data.** Never cache personal data (cart contents, order history) at the CDN level. Cache only public data (product listings, search results).

- **Forgetting to warm the CDN.** If the CDN cache is cold when the surge hits, every request goes to the origin. Warm the CDN before the event.

## Key Takeaway

Graceful degradation is about prioritizing critical features and serving cached content when the origin is overloaded. The user should see a reduced experience (no recommendations, cached search) rather than an error page. Client-side storage (localStorage) preserves user state during server outages. The key design principle: serve something rather than nothing.

## Relevant README Sections
- [Backpressure Patterns](../README.md#backpressure-patterns)
- [Graceful Degradation](../README.md#graceful-degradation)
- [CDN Shielding](../README.md#cdn-shielding)
