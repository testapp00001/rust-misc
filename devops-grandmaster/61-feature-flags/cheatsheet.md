# Cheatsheet: Feature Flags

## Flag Types

| Type | Purpose | Example |
|------|---------|---------|
| Release | Decouple deploy from release | New UI behind flag |
| Experiment | A/B testing | New algorithm |
| Ops | Operational toggle | Maintenance mode |
| Permission | Premium features | Pro features |

## Unleash (Open Source)
```bash
# Run Unleash
docker run -d --name unleash -p 4242:4242 unleashorg/unleash-server

# Check flag
curl -H "Authorization: YOUR_TOKEN" \
  http://localhost:4242/api/client/features/new-checkout
```

## Python Example
```python
from unleash import UnleashClient

client = UnleashClient(
    url="http://unleash:4242",
    app_name="my-app"
)
client.initialize_client()

# Check flag
if client.is_enabled("new-checkout"):
    # New checkout flow
    return new_checkout()
else:
    # Old checkout flow
    return old_checkout()
```

## Percentage Rollout
```yaml
# 10% of users see new feature
{
  "name": "new-checkout",
  "enabled": true,
  "strategies": [
    {
      "name": "gradualRollout",
      "parameters": {
        "percentage": 10
      }
    }
  ]
}
```

## Flag Lifecycle
```
1. Create flag → Deploy code with flag
2. Enable for internal testing
3. Enable for 1% of users
4. Gradually increase to 100%
5. Remove flag and old code
```
