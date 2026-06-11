# Cheatsheet: DDoS Protection

## DDoS Types

| Type | Layer | Attack Vector |
|------|-------|---------------|
| Volumetric | L3/L4 | UDP flood, ICMP flood |
| Protocol | L3/L4 | SYN flood, Ping of Death |
| Application | L7 | HTTP flood, Slowloris |

## Rate Limiting (Nginx)
```nginx
# Define zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_conn_zone $binary_remote_addr zone=addr:10m;

server {
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        limit_conn addr 10;
        proxy_pass http://backend;
    }
}
```

## Cloud DDoS Protection
```
AWS Shield      → Automatic for AWS resources
Cloudflare      → CDN + DDoS protection
Akamai          → Enterprise DDoS mitigation
Google Cloud Armor → GCP DDoS protection
```

## Emergency Procedures
```bash
# 1. Identify attack type
tcpdump -i eth0 -n | head -100

# 2. Enable rate limiting
nginx -s reload

# 3. Block attacking IPs
iptables -A INPUT -s ATTACKER_IP -j DROP

# 4. Enable cloud DDoS protection
# 5. Contact ISP/hosting provider
# 6. Enable geo-blocking if applicable
```
