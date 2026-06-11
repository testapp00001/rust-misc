# Cheatsheet: Load Balancing Algorithms

## Algorithms

| Algorithm | Description | Best For |
|-----------|-------------|----------|
| Round Robin | Rotate through servers | Equal servers |
| Weighted Round Robin | Rotate with weights | Unequal servers |
| Least Connections | Send to least busy | Varying request times |
| IP Hash | Same client → same server | Sticky sessions |
| Random | Random selection | Simple, effective |
| Consistent Hashing | Hash-based distribution | Caching |

## HAProxy Configuration
```haproxy
frontend http_front
    bind *:80
    default_backend servers

backend servers
    balance roundrobin
    # balance leastconn
    # balance source
    server server1 10.0.0.1:8080 check weight 3
    server server2 10.0.0.2:8080 check weight 2
    server server3 10.0.0.3:8080 check weight 1
```

## Nginx Configuration
```nginx
upstream backend {
    # Round Robin (default)
    server backend1:8080;
    server backend2:8080;

    # Least Connections
    # least_conn;

    # IP Hash
    # ip_hash;

    # Weighted
    # server backend1:8080 weight=3;
    # server backend2:8080 weight=1;
}
```

## Health Checks
```nginx
upstream backend {
    server backend1:8080 max_fails=3 fail_timeout=30s;
    server backend2:8080 max_fails=3 fail_timeout=30s;
}
```
