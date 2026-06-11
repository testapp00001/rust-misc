# Cheatsheet: Traffic Surge Handling

## Black Friday Checklist
- [ ] Pre-warm infrastructure (scale up 24h before)
- [ ] Enable aggressive caching
- [ ] Increase connection pool sizes
- [ ] Enable rate limiting
- [ ] Disable non-critical features
- [ ] Monitor error rates closely
- [ ] Have rollback plan ready

## Queue-Based Architecture
```
User → Load Balancer → API → Queue → Worker → Database
```

## Rate Limiting
```nginx
limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s;

location /api/ {
    limit_req zone=api burst=200 nodelay;
    proxy_pass http://backend;
}
```

## Caching Strategy
```python
# Cache-aside pattern
def get_product(product_id):
    # Check cache first
    cached = cache.get(f'product:{product_id}')
    if cached:
        return cached

    # Query database
    product = db.query('SELECT * FROM products WHERE id = %s', product_id)

    # Cache for 5 minutes
    cache.setex(f'product:{product_id}', 300, json.dumps(product))
    return product
```

## Graceful Degradation
```python
# If service is overloaded, return cached/default data
def get_recommendations(user_id):
    try:
        return recommendation_service.get(user_id)
    except ServiceOverloaded:
        return get_cached_recommendations(user_id) or get_default_recommendations()
```
