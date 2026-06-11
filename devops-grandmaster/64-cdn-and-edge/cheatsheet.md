# Cheatsheet: CDN & Edge

## CDN Benefits
- Reduced latency (edge locations)
- Reduced origin load
- DDoS protection
- SSL termination
- Global distribution

## Cache Headers
```http
# Cache for 1 year (immutable assets)
Cache-Control: public, max-age=31536000, immutable

# Cache for 1 hour
Cache-Control: public, max-age=3600

# No cache (dynamic content)
Cache-Control: no-store

# Revalidate
Cache-Control: no-cache
ETag: "abc123"
Last-Modified: Wed, 21 Oct 2015 07:28:00 GMT
```

## Cloudflare Setup
```bash
# 1. Add domain to Cloudflare
# 2. Update nameservers
# 3. Enable caching
# 4. Configure page rules
```

## Edge Computing
```javascript
// Cloudflare Worker
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const response = await fetch(request)
  const html = await response.text()
  return new Response(html, {
    headers: { 'Content-Type': 'text/html' }
  })
}
```

## Cache Invalidation
```bash
# Purge specific URL
curl -X POST "https://api.cloudflare.com/client/v4/zones/ZONE_ID/purge_cache" \
  -H "Authorization: Bearer TOKEN" \
  -d '{"files":["https://example.com/style.css"]}'

# Purge everything
curl -X POST "https://api.cloudflare.com/client/v4/zones/ZONE_ID/purge_cache" \
  -H "Authorization: Bearer TOKEN" \
  -d '{"purge_everything":true}'
```
