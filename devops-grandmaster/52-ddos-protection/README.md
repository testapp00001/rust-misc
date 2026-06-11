# Module 52: DDoS Protection

## Rate Limiting, Traffic Analysis, and Mitigation

**Previous Module:** [51 - Network Security](../51-network-security/README.md)
**Next Module:** [53 - WAF Rules](../53-waf-rules/README.md)

---

## 1. The Problem

Your application is running, traffic is flowing, and everything looks healthy. Then suddenly, your servers are overwhelmed with millions of requests per second from thousands of IP addresses. Your CPU maxes out, your load balancer queues fill up, and legitimate users cannot reach your service. You are under a Distributed Denial of Service (DDoS) attack.

The core challenges:
- Attack traffic looks similar to legitimate traffic
- Attacks can come from thousands of sources simultaneously
- Bandwidth can be saturated before traffic reaches your infrastructure
- Application-layer attacks are hard to distinguish from real users
- Manual response is too slow when attacks last seconds

---

## 2. The Naive Way

```nginx
# nginx.conf - No rate limiting at all
http {
    server {
        listen 80;
        location / {
            proxy_pass http://backend;
        }
    }
}
```

```python
# No connection limits
# No rate limiting
# No traffic analysis
# Just hope for the best
@app.route('/api/data')
def get_data():
    return expensive_database_query()
```

Problems:
- No limits on request rate per client
- Expensive endpoints can be hammered indefinitely
- No way to distinguish legitimate from attack traffic
- Single IP can exhaust server resources
- No circuit breakers for downstream services

---

## 3. The Right Way

### 3.1 Understanding DDoS Attack Types

```
DDoS Attack Types:
    |
    +-- Volumetric Attacks (Layer 3/4)
    |       - UDP flood: Send massive UDP packets
    |       - ICMP flood: Ping flood
    |       - DNS amplification: Small query, large response
    |       Goal: Saturate bandwidth
    |
    +-- Protocol Attacks (Layer 3/4)
    |       - SYN flood: Half-open TCP connections
    |       - Ping of death: Malformed packets
    |       - Smurf attack: ICMP amplification
    |       Goal: Exhaust server resources
    |
    +-- Application Layer Attacks (Layer 7)
            - HTTP flood: Massive GET/POST requests
            - Slowloris: Slow, persistent connections
            - API abuse: Expensive endpoint hammering
            Goal: Exhaust application resources
```

### 3.2 Rate Limiting with Nginx

```nginx
# /etc/nginx/nginx.conf
http {
    # Define rate limiting zones
    # Zone 1: General API rate limit (10 requests/second per IP)
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;

    # Zone 2: Login endpoint (stricter - 1 request/second per IP)
    limit_req_zone $binary_remote_addr zone=login_limit:10m rate=1r/s;

    # Zone 3: Expensive operations (1 request/5 seconds per IP)
    limit_req_zone $binary_remote_addr zone=expensive_limit:10m rate=12r/m;

    # Connection limit per IP
    limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

    # Custom error pages for rate limiting
    limit_req_status 429;
    limit_conn_status 429;

    server {
        listen 80;

        # Global connection limit: 10 concurrent connections per IP
        limit_conn conn_limit 10;

        # General API rate limit with burst
        location /api/ {
            limit_req zone=api_limit burst=20 nodelay;
            proxy_pass http://backend;
        }

        # Strict rate limit for authentication
        location /api/auth/ {
            limit_req zone=login_limit burst=3 nodelay;
            proxy_pass http://backend;
        }

        # Rate limit for expensive operations
        location /api/reports/ {
            limit_req zone=expensive_limit burst=2 nodelay;
            proxy_pass http://backend;
        }

        # Static files - no rate limit needed
        location /static/ {
            root /var/www/html;
        }
    }
}
```

### 3.3 Rate Limiting with API Gateway (Kong Example)

```yaml
# Kong rate limiting configuration
_format_version: "3.0"

services:
  - name: api-service
    url: http://backend:8080
    routes:
      - name: api-route
        paths:
          - /api
    plugins:
      - name: rate-limiting
        config:
          minute: 100
          hour: 5000
          policy: redis
          redis_host: redis
          redis_port: 6379
          fault_tolerant: true
          hide_client_headers: false

      - name: rate-limiting
        config:
          minute: 5
          policy: local
          hide_client_headers: false
        # This applies to auth endpoints
        route: auth-route

      - name: ip-restriction
        config:
          deny:
            - 192.168.1.100  # Blocked IP
          allow:
            - 10.0.0.0/8    # Internal network
```

### 3.4 SYN Flood Protection

```bash
#!/bin/bash
# SYN flood protection with iptables

# Enable SYN cookies (kernel level protection)
sysctl -w net.ipv4.tcp_syncookies=1

# Reduce SYN-ACK retries
sysctl -w net.ipv4.tcp_synack_retries=2

# Increase SYN backlog
sysctl -w net.ipv4.tcp_max_syn_backlog=2048

# Enable TCP timestamps
sysctl -w net.ipv4.tcp_timestamps=1

# iptables rules for SYN flood protection
# Limit SYN packets per source IP
iptables -A INPUT -p tcp --syn -m limit --limit 10/s --limit-burst 20 -j ACCEPT
iptables -A INPUT -p tcp --syn -j DROP

# Limit new connections per source IP
iptables -A INPUT -p tcp --syn -m connlimit --connlimit-above 20 -j REJECT

# Drop invalid packets
iptables -A INPUT -m state --state INVALID -j DROP

# Drop XMAS and NULL scans
iptables -A INPUT -p tcp --tcp-flags ALL ALL -j DROP
iptables -A INPUT -p tcp --tcp-flags ALL NONE -j DROP

# Connection rate limiting
iptables -A INPUT -p tcp -m conntrack --ctstate NEW -m recent --set
iptables -A INPUT -p tcp -m conntrack --ctstate NEW -m recent --update --seconds 60 --hitcount 100 -j DROP

echo "SYN flood protection enabled"
```

### 3.5 Connection Limits

```nginx
# Nginx connection and request limits
http {
    # Limit concurrent connections per IP
    limit_conn_zone $binary_remote_addr zone=addr:10m;

    # Limit request rate per IP
    limit_req_zone $binary_remote_addr zone=one:10m rate=1r/s;

    server {
        listen 80;

        # Maximum 10 concurrent connections per IP
        limit_conn addr 10;

        # Rate limit with burst handling
        limit_req zone=one burst=5 nodelay;

        # Timeout for slow connections (Slowloris protection)
        client_body_timeout 10s;
        client_header_timeout 10s;

        # Limit body size
        client_max_body_size 1m;

        # Limit number of headers
        limit_req_fields 50;
        limit_req_field_size 4k;

        # Keep-alive timeout
        keepalive_timeout 15s;
    }
}
```

### 3.6 Geo-Blocking

```nginx
# Nginx geo-blocking
geo $blocked_country {
    default 0;
    # Block traffic from specific countries (example)
    1.0.0.0/8 1;   # Example: block specific IP ranges
    2.0.0.0/8 1;
}

# Use GeoIP2 module for country-based blocking
geoip2 /etc/nginx/geoip/GeoLite2-Country.mmdb {
    auto_reload 60m;
    $geoip2_metadata_country_build metadata build_epoch;
    $geoip2_data_country_code country iso_code;
    $geoip2_data_country_name country names en;
}

map $geoip2_data_country_code $is_blocked_country {
    default 0;
    "XX" 1;  # Replace with actual country codes to block
    "YY" 1;
}

server {
    listen 80;

    if ($is_blocked_country) {
        return 403;
    }
}
```

```hcl
# AWS WAF geo-blocking with Terraform
resource "aws_wafv2_web_acl" "main" {
  name  = "geo-blocking-acl"
  scope = "REGIONAL"

  default_action {
    allow {}
  }

  rule {
    name     = "block-countries"
    priority = 1

    action {
      block {}
    }

    statement {
      geo_match_statement {
        country_codes = ["XX", "YY"]  # Country codes to block
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "GeoBlockRule"
      sampled_requests_enabled   = true
    }
  }
}
```

### 3.7 Traffic Analysis

```python
#!/usr/bin/env python3
"""
Simple traffic analysis for DDoS detection.
Monitors request rates and alerts on anomalies.
"""

import time
import json
import redis
from collections import defaultdict
from datetime import datetime, timedelta

class TrafficAnalyzer:
    def __init__(self, redis_host='localhost', redis_port=6379):
        self.redis = redis.Redis(host=redis_host, port=redis_port, decode_responses=True)
        self.window_size = 60  # 1 minute window
        self.threshold_rps = 1000  # Requests per second threshold
        self.alert_cooldown = 300  # 5 minutes between alerts

    def record_request(self, client_ip: str, path: str, status_code: int):
        """Record a request for analysis."""
        now = int(time.time())
        pipe = self.redis.pipeline()

        # Track requests per second
        pipe.incr(f"rps:{now}")
        pipe.expire(f"rps:{now}", 120)

        # Track requests per IP
        pipe.incr(f"ip:{client_ip}:{now}")
        pipe.expire(f"ip:{client_ip}:{now}", 120)

        # Track requests per path
        pipe.incr(f"path:{path}:{now}")
        pipe.expire(f"path:{path}:{now}", 120)

        # Track error rates
        if status_code >= 400:
            pipe.incr(f"errors:{now}")
            pipe.expire(f"errors:{now}", 120)

        pipe.execute()

    def get_current_rps(self) -> int:
        """Get current requests per second."""
        now = int(time.time())
        return int(self.redis.get(f"rps:{now}") or 0)

    def get_top_ips(self, limit: int = 10) -> list:
        """Get top requesting IPs in the last minute."""
        now = int(time.time())
        ip_counts = defaultdict(int)

        for t in range(now - 60, now):
            keys = self.redis.keys(f"ip:*:{t}")
            for key in keys:
                ip = key.split(':')[1]
                count = int(self.redis.get(key) or 0)
                ip_counts[ip] += count

        return sorted(ip_counts.items(), key=lambda x: x[1], reverse=True)[:limit]

    def detect_anomaly(self) -> dict:
        """Detect traffic anomalies."""
        rps = self.get_current_rps()
        top_ips = self.get_top_ips(5)
        error_rate = self._get_error_rate()

        alerts = []

        if rps > self.threshold_rps:
            alerts.append({
                'type': 'high_rps',
                'message': f'High request rate: {rps} RPS (threshold: {self.threshold_rps})',
                'severity': 'critical'
            })

        for ip, count in top_ips:
            if count > self.threshold_rps * 0.5:
                alerts.append({
                    'type': 'ip_flood',
                    'message': f'IP {ip} sending {count} requests/minute',
                    'severity': 'warning'
                })

        if error_rate > 0.5:  # More than 50% errors
            alerts.append({
                'type': 'high_error_rate',
                'message': f'High error rate: {error_rate:.1%}',
                'severity': 'critical'
            })

        return {
            'timestamp': datetime.utcnow().isoformat(),
            'rps': rps,
            'top_ips': top_ips,
            'error_rate': error_rate,
            'alerts': alerts,
            'is_under_attack': len([a for a in alerts if a['severity'] == 'critical']) > 0
        }

    def _get_error_rate(self) -> float:
        """Get error rate in the last minute."""
        now = int(time.time())
        total = 0
        errors = 0

        for t in range(now - 60, now):
            total += int(self.redis.get(f"rps:{t}") or 0)
            errors += int(self.redis.get(f"errors:{t}") or 0)

        return errors / total if total > 0 else 0


# Flask middleware example
from flask import Flask, request, g

app = Flask(__name__)
analyzer = TrafficAnalyzer()

@app.before_request
def track_request():
    g.start_time = time.time()

@app.after_request
def analyze_request(response):
    analyzer.record_request(
        client_ip=request.remote_addr,
        path=request.path,
        status_code=response.status_code
    )

    # Check for anomalies
    analysis = analyzer.detect_anomaly()
    if analysis['is_under_attack']:
        app.logger.warning(f"DDoS detected: {analysis['alerts']}")

    return response

@app.route('/metrics')
def metrics():
    """Endpoint to check traffic metrics."""
    return analyzer.detect_anomaly()
```

### 3.8 Emergency Procedures

```bash
#!/bin/bash
# emergency-ddos-response.sh
# Emergency DDoS response playbook

set -e

echo "=== DDoS Emergency Response ==="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Step 1: Enable aggressive rate limiting
echo "[1/6] Enabling aggressive rate limiting..."
cat > /etc/nginx/conf.d/emergency-rate-limit.conf << 'EOF'
limit_req_zone $binary_remote_addr zone=emergency:10m rate=2r/s;
limit_conn_zone $binary_remote_addr zone=emergency_conn:10m;

server {
    listen 80;
    server_name _;

    limit_req zone=emergency burst=5 nodelay;
    limit_conn emergency_conn 5;

    # Block suspicious user agents
    if ($http_user_agent ~* (bot|crawl|spider|scanner)) {
        return 403;
    }

    # Block empty user agents
    if ($http_user_agent = "") {
        return 403;
    }

    location / {
        proxy_pass http://backend;
    }
}
EOF

nginx -s reload
echo "  Aggressive rate limiting enabled"

# Step 2: Enable SYN cookies
echo "[2/6] Enabling SYN cookies..."
sysctl -w net.ipv4.tcp_syncookies=1
sysctl -w net.ipv4.tcp_max_syn_backlog=2048
sysctl -w net.ipv4.tcp_synack_retries=2

# Step 3: Block top attacking IPs
echo "[3/6] Blocking top attacking IPs..."
TOP_IPS=$(tail -10000 /var/log/nginx/access.log | \
    awk '{print $1}' | sort | uniq -c | sort -rn | head -20 | \
    awk '{print $2}')

for ip in $TOP_IPS; do
    iptables -A INPUT -s "$ip" -j DROP
    echo "  Blocked: $ip"
done

# Step 4: Enable Cloudflare Under Attack Mode (if using Cloudflare)
echo "[4/6] Checking Cloudflare status..."
if command -v cf-cli &> /dev/null; then
    cf-cli security-level --level=under_attack
    echo "  Cloudflare Under Attack Mode enabled"
else
    echo "  Cloudflare CLI not found, skip this step"
fi

# Step 5: Alert the team
echo "[5/6] Sending alerts..."
curl -X POST "https://hooks.slack.com/services/YOUR/WEBHOOK/URL" \
    -H 'Content-Type: application/json' \
    -d "{
        \"text\": \"DDoS ALERT: Emergency response activated\",
        \"attachments\": [{
            \"color\": \"danger\",
            \"fields\": [
                {\"title\": \"Time\", \"value\": \"$(date -u)\", \"short\": true},
                {\"title\": \"Action\", \"value\": \"Emergency rate limiting enabled\", \"short\": true}
            ]
        }]
    }"

# Step 6: Document incident
echo "[6/6] Documenting incident..."
cat >> /var/log/ddos-incidents.log << EOF
---
Incident: DDoS Attack Detected
Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Response: Emergency rate limiting enabled
Blocked IPs: $(echo "$TOP_IPS" | wc -l)
Status: Mitigating
EOF

echo ""
echo "=== Emergency response complete ==="
echo "Monitor: tail -f /var/log/nginx/access.log"
echo "Metrics: curl http://localhost/metrics"
```

---

## 4. The Production Way

### 4.1 Multi-Layer DDoS Protection

```
                    Internet
                       |
    +------------------+------------------+
    |         Cloud DDoS Protection       |
    |    (AWS Shield / Cloudflare)        |
    |    - Volumetric attack mitigation   |
    |    - Protocol attack protection     |
    +------------------+------------------+
                       |
              [CDN / Edge Network]
              - Cache static content
              - Absorb traffic spikes
                       |
              [Application Load Balancer]
              - Connection limits
              - Slow request timeout
                       |
              [Rate Limiting Layer]
              - Per-IP rate limits
              - Per-endpoint limits
              - Token bucket algorithm
                       |
              [WAF Layer]
              - Application attack detection
              - Bot detection
              - Challenge pages
                       |
              [Application Servers]
              - Circuit breakers
              - Graceful degradation
```

### 4.2 AWS Shield + WAF Configuration

```hcl
# Terraform: AWS Shield Advanced + WAF

# Enable Shield Advanced
resource "aws_shield_protection" "alb" {
  name         = "production-alb-shield"
  resource_arn = aws_lb.main.arn
}

resource "aws_shield_protection" "cloudfront" {
  name         = "production-cf-shield"
  resource_arn = aws_cloudfront_distribution.main.arn
}

# WAF Web ACL with rate limiting
resource "aws_wafv2_web_acl" "ddos_protection" {
  name  = "ddos-protection"
  scope = "REGIONAL"

  default_action {
    allow {}
  }

  # Rule 1: Rate limiting
  rule {
    name     = "rate-limit"
    priority = 1

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 2000
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitRule"
      sampled_requests_enabled   = true
    }
  }

  # Rule 2: Block known bad IPs
  rule {
    name     = "block-bad-ips"
    priority = 2

    action {
      block {}
    }

    statement {
      ip_set_reference_statement {
        arn = aws_wafv2_ip_set.bad_ips.arn
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "BlockBadIPs"
      sampled_requests_enabled   = true
    }
  }

  # Rule 3: Challenge suspicious requests
  rule {
    name     = "challenge-suspicious"
    priority = 3

    action {
      challenge {}
    }

    statement {
      or_statement {
        statement {
          byte_match_statement {
            positional_constraint = "EXACTLY"
            search_string         = ""
            field_to_match {
              single_header {
                name = "user-agent"
              }
            }
            text_transformation {
              priority = 0
              type     = "NONE"
            }
          }
        }
        statement {
          sqli_match_statement {
            field_to_match {
              body {}
            }
            text_transformation {
              priority = 0
              type     = "URL_DECODE"
            }
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "ChallengeSuspicious"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "DDoSProtection"
    sampled_requests_enabled   = true
  }
}

resource "aws_wafv2_ip_set" "bad_ips" {
  name               = "bad-ips"
  scope              = "REGIONAL"
  ip_address_version = "IPV4"
  addresses          = []  # Populated from threat intelligence
}

# Associate WAF with ALB
resource "aws_wafv2_web_acl_association" "alb" {
  resource_arn = aws_lb.main.arn
  web_acl_arn  = aws_wafv2_web_acl.ddos_protection.arn
}
```

### 4.3 Cloudflare Configuration

```bash
# Cloudflare API: Enable Under Attack Mode
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/settings/security_level" \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{"value":"under_attack"}'

# Cloudflare API: Create rate limiting rule
curl -X POST "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/rate_limits" \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{
        "threshold": 100,
        "period": 60,
        "action": {
            "mode": "ban",
            "timeout": 600
        },
        "match": {
            "request": {
                "url": "example.com/api/*",
                "schemes": ["HTTP", "HTTPS"],
                "methods": ["GET", "POST"]
            }
        },
        "correlate": {
            "by": "ip"
        }
    }'

# Enable Bot Management
curl -X PUT "https://api.cloudflare.com/client/v4/zones/${ZONE_ID}/bot_management" \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{
        "enable_js": true,
        "fight_mode": true,
        "using_latest_model": true
    }'
```

### 4.4 Circuit Breaker Pattern

```python
"""
Circuit breaker for DDoS resilience.
Prevents cascading failures when downstream services are overwhelmed.
"""

import time
import threading
from enum import Enum
from typing import Callable, Any

class CircuitState(Enum):
    CLOSED = "closed"        # Normal operation
    OPEN = "open"            # Failing, reject requests
    HALF_OPEN = "half_open"  # Testing if service recovered

class CircuitBreaker:
    def __init__(
        self,
        failure_threshold: int = 5,
        recovery_timeout: int = 30,
        expected_exception: type = Exception
    ):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.expected_exception = expected_exception

        self._state = CircuitState.CLOSED
        self._failure_count = 0
        self._last_failure_time = None
        self._lock = threading.Lock()

    @property
    def state(self) -> CircuitState:
        if self._state == CircuitState.OPEN:
            if time.time() - self._last_failure_time > self.recovery_timeout:
                return CircuitState.HALF_OPEN
        return self._state

    def call(self, func: Callable, *args, **kwargs) -> Any:
        with self._lock:
            current_state = self.state

            if current_state == CircuitState.OPEN:
                raise CircuitBreakerOpenError(
                    f"Circuit breaker is OPEN. "
                    f"Retry after {self.recovery_timeout}s"
                )

            try:
                result = func(*args, **kwargs)
                self._on_success()
                return result
            except self.expected_exception as e:
                self._on_failure()
                raise

    def _on_success(self):
        self._failure_count = 0
        self._state = CircuitState.CLOSED

    def _on_failure(self):
        self._failure_count += 1
        self._last_failure_time = time.time()

        if self._failure_count >= self.failure_threshold:
            self._state = CircuitState.OPEN

class CircuitBreakerOpenError(Exception):
    pass


# Usage example with Flask
from flask import Flask, jsonify

app = Flask(__name__)

# Circuit breaker for database
db_circuit = CircuitBreaker(
    failure_threshold=5,
    recovery_timeout=30
)

@app.route('/api/users')
def get_users():
    try:
        # Try to get from database through circuit breaker
        users = db_circuit.call(get_users_from_db)
        return jsonify(users)
    except CircuitBreakerOpenError:
        # Return cached data or graceful degradation
        return jsonify({
            'users': get_cached_users(),
            'source': 'cache',
            'message': 'Database temporarily unavailable'
        }), 200
    except Exception as e:
        return jsonify({'error': str(e)}), 503
```

---

## 5. Hands-On Lab

### Lab: Implement Rate Limiting and DDoS Protection

**Objective:** Set up rate limiting, traffic analysis, and DDoS mitigation.

#### Step 1: Create the Lab Environment

```bash
mkdir -p lab-ddos-protection && cd lab-ddos-protection

cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    networks:
      - backend

  nginx:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app
    networks:
      - frontend
      - backend

  app:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
    networks:
      - backend

  # Load testing tool
  locust:
    image: locustio/locust:latest
    ports:
      - "8089:8089"
    volumes:
      - ./locustfile.py:/mnt/locust/locustfile.py
    command: -f /mnt/locust/locustfile.py --host=http://nginx
    networks:
      - frontend

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true
EOF

cat > Dockerfile << 'EOF'
FROM python:3.11-slim

WORKDIR /app
RUN pip install flask redis

COPY app.py .
EXPOSE 5000
CMD ["python", "app.py"]
EOF

cat > app.py << 'EOF'
from flask import Flask, jsonify, request
import redis
import time
import os

app = Flask(__name__)
redis_client = redis.Redis(
    host=os.getenv('REDIS_HOST', 'localhost'),
    port=int(os.getenv('REDIS_PORT', 6379)),
    decode_responses=True
)

@app.route('/health')
def health():
    return jsonify({'status': 'healthy'})

@app.route('/api/data')
def get_data():
    # Simulate expensive operation
    time.sleep(0.1)
    return jsonify({'data': 'some data', 'timestamp': time.time()})

@app.route('/api/expensive')
def expensive_operation():
    # Simulate very expensive operation
    time.sleep(1)
    return jsonify({'result': 'computed', 'timestamp': time.time()})

@app.route('/api/rate-limit-status')
def rate_limit_status():
    """Check rate limit status for current IP."""
    ip = request.remote_addr
    current = redis_client.get(f'rate:{ip}') or 0
    return jsonify({
        'ip': ip,
        'current_requests': int(current),
        'limit': 100,
        'remaining': max(0, 100 - int(current))
    })

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
EOF

cat > nginx.conf << 'EOF'
worker_processes auto;

events {
    worker_connections 1024;
}

http {
    # Rate limiting zones
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=strict:10m rate=1r/s;
    limit_conn_zone $binary_remote_addr zone=conn:10m;

    # Custom log format with timing
    log_format detailed '$remote_addr - $remote_user [$time_local] '
                        '"$request" $status $body_bytes_sent '
                        '"$http_referer" "$http_user_agent" '
                        'rt=$request_time';

    access_log /var/log/nginx/access.log detailed;

    server {
        listen 80;

        # Global connection limit
        limit_conn conn 20;

        # Health check - no rate limit
        location /health {
            proxy_pass http://app:5000;
        }

        # General API - moderate rate limit
        location /api/data {
            limit_req zone=api burst=20 nodelay;
            proxy_pass http://app:5000;
        }

        # Expensive endpoint - strict rate limit
        location /api/expensive {
            limit_req zone=strict burst=2 nodelay;
            proxy_pass http://app:5000;
        }

        # Rate limit status endpoint
        location /api/rate-limit-status {
            limit_req zone=api burst=5 nodelay;
            proxy_pass http://app:5000;
        }

        # Custom error page for rate limiting
        error_page 429 /429.html;
        location = /429.html {
            internal;
            default_type application/json;
            return 429 '{"error": "Rate limit exceeded", "retry_after": 1}';
        }
    }
}
EOF

cat > locustfile.py << 'EOF'
from locust import HttpUser, task, between

class NormalUser(HttpUser):
    wait_time = between(1, 3)

    @task(10)
    def get_data(self):
        self.client.get("/api/data")

    @task(1)
    def expensive_operation(self):
        self.client.get("/api/expensive")

    @task(5)
    def check_health(self):
        self.client.get("/health")

class AttackerUser(HttpUser):
    wait_time = between(0.01, 0.1)

    @task
    def flood_api(self):
        self.client.get("/api/data")

    @task
    def flood_expensive(self):
        self.client.get("/api/expensive")
EOF
```

#### Step 2: Start the Lab

```bash
# Build and start all services
docker-compose up -d

# Verify services are running
docker-compose ps

# Test basic connectivity
curl http://localhost:8080/health
curl http://localhost:8080/api/data
```

#### Step 3: Observe Normal Traffic

```bash
# Make normal requests
for i in {1..20}; do
    curl -s http://localhost:8080/api/data | jq .
    sleep 0.5
done

# Check rate limit status
curl http://localhost:8080/api/rate-limit-status | jq .
```

#### Step 4: Simulate DDoS Attack

```bash
# Simulate attack with Apache Bench
# 1000 requests, 100 concurrent
ab -n 1000 -c 100 http://localhost:8080/api/data

# Count successful vs rate-limited responses
echo "=== Response Analysis ==="
for i in {1..100}; do
    status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/data)
    echo "$status"
done | sort | uniq -c | sort -rn
```

#### Step 5: Monitor and Analyze

```bash
# Watch Nginx logs in real-time
docker-compose logs -f nginx | grep -E "(429|200|503)"

# Analyze access log for patterns
docker exec lab-ddos-protection-nginx-1 sh -c "
    echo '=== Top IPs ==='
    tail -10000 /var/log/nginx/access.log | awk '{print \$1}' | sort | uniq -c | sort -rn | head -10

    echo ''
    echo '=== Response Codes ==='
    awk '{print \$9}' /var/log/nginx/access.log | sort | uniq -c | sort -rn

    echo ''
    echo '=== Rate Limited Requests ==='
    grep '429' /var/log/nginx/access.log | wc -l
"
```

#### Step 6: Test Locust Load Testing

```bash
# Open Locust UI at http://localhost:8089
# Configure:
# - Number of users: 100
# - Spawn rate: 10
# - Host: http://nginx

# Or run headless
docker run --rm --network lab-ddos-protection_frontend \
    locustio/locust:latest \
    -f /dev/stdin \
    --host=http://nginx \
    --users=50 \
    --spawn-rate=10 \
    --run-time=30s \
    --headless < locustfile.py
```

#### Step 7: Cleanup

```bash
docker-compose down -v
```

**Expected Results:**
- Normal traffic passes through without rate limiting
- High-frequency requests trigger 429 responses
- Rate limiting protects expensive endpoints
- Traffic analysis reveals attack patterns
- Legitimate users are not affected during moderate attacks

---

## 6. Limitation -> Next Topic

DDoS protection handles volumetric and protocol-layer attacks by limiting traffic rates and absorbing excess load. However, it cannot protect against sophisticated application-layer attacks that send valid-looking requests with malicious payloads.

**What DDoS protection cannot do:**
- Cannot detect SQL injection in legitimate-looking requests
- Cannot prevent XSS attacks in normal traffic
- Cannot block CSRF attacks from authenticated users
- Cannot identify and block application logic abuse
- Cannot inspect encrypted payloads without a WAF

The next layer of defense is a Web Application Firewall that inspects application-layer traffic for attack patterns.

**Next Module:** [53 - WAF Rules](../53-waf-rules/README.md) -- SQL injection, XSS, and CSRF protection with Web Application Firewalls.
