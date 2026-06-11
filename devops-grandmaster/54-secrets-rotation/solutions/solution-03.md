# Solution 03: API Key Rotation Pipeline

## Part A: Key Rotation Strategy

| State | Old Key | New Key | Partner Action Required | Duration |
|-------|---------|---------|------------------------|----------|
| Active | Valid | Does not exist | None | Normal operation |
| Pending Rotation | Valid | Valid (generated) | None yet | 0-7 days |
| Dual-Key Active | Valid (deprecating) | Valid (active) | Switch to new key | 7-30 days |
| Deprecating | Valid (expiring soon) | Valid (active) | Must switch immediately | 30-37 days |
| Rotated | Revoked | Valid (active) | Must use new key | Permanent |

### Why This Works

The dual-key strategy ensures zero-downtime rotation. Both keys are valid during the grace period. Partners can switch at their convenience. The 30-day grace period gives ample time for integration updates.

### Common Mistakes

- **No grace period.** Revoking immediately breaks all partner integrations.
- **Too long a grace period.** 90 days is a long exposure window.
- **Not monitoring deprecated key usage.** Partners who do not switch need escalation.

---

## Part B: Key Generation and Storage

```python
# key_manager.py
import secrets
import hashlib
import time
import hvac

class APIKeyManager:
    def __init__(self, vault_client: hvac.Client):
        self.client = vault_client
        self.key_prefix = "pk_live_"
        self.key_length = 32

    def generate_key(self, partner_id: str) -> dict:
        random_part = secrets.token_urlsafe(self.key_length)
        api_key = f"{self.key_prefix}{random_part}"
        key_hash = hashlib.sha256(api_key.encode()).hexdigest()
        key_id = f"key_{secrets.token_hex(8)}"

        self.client.secrets.kv.v2.create_or_update_secret(
            path=f"api-keys/{partner_id}/{key_id}",
            secret={
                'key_hash': key_hash,
                'partner_id': partner_id,
                'key_id': key_id,
                'status': 'active',
                'created_at': time.time(),
                'last_used': None,
                'usage_count': 0
            }
        )
        return {'key_id': key_id, 'api_key': api_key, 'partner_id': partner_id, 'status': 'active'}

    def rotate_key(self, partner_id: str) -> dict:
        current_keys = self._list_keys(partner_id)
        active_key = next((k for k in current_keys if k['status'] == 'active'), None)
        if not active_key:
            raise ValueError(f"No active key found for partner {partner_id}")

        new_key = self.generate_key(partner_id)
        self._update_key_status(partner_id, active_key['key_id'], 'deprecating')
        return {
            'partner_id': partner_id,
            'old_key_id': active_key['key_id'],
            'new_key': new_key,
            'grace_period_days': 30
        }

    def validate_key(self, api_key: str) -> dict:
        key_hash = hashlib.sha256(api_key.encode()).hexdigest()
        partners = self.client.secrets.kv.v2.list_secrets(path="api-keys")
        for partner_id in partners['data']['keys']:
            keys = self._list_keys(partner_id)
            for key_info in keys:
                if key_info['key_hash'] == key_hash:
                    if key_info['status'] in ('active', 'deprecating'):
                        return {'valid': True, 'partner_id': partner_id, 'key_id': key_info['key_id']}
                    return {'valid': False, 'reason': 'key_revoked'}
        return {'valid': False, 'reason': 'key_not_found'}

    def revoke_key(self, partner_id: str, key_id: str) -> bool:
        self._update_key_status(partner_id, key_id, 'revoked')
        return True
```

### Why This Works

Key generation uses `secrets.token_urlsafe()` which is cryptographically secure. The key is hashed before storage -- the raw key is only returned once. Even if Vault is compromised, the attacker gets hashes, not usable keys.

---

## Part C: Notification System

Notification schedule:

| Days Before Revocation | Notification Type | Channel | Urgency |
|----------------------|-------------------|---------|---------|
| 30 days | Key rotation initiated | Email + Webhook | Low |
| 14 days | Expiry warning | Email + Webhook | Medium |
| 7 days | Expiry warning | Email + Webhook | High |
| 3 days | Final warning | Email + Webhook + Dashboard | Urgent |
| 1 day | Last chance | Email + Webhook + SMS | Critical |
| 0 (revocation) | Key revoked | All channels | Critical |

### Why This Works

Escalating notifications ensure partners are aware. Multiple channels (email, webhook, dashboard, SMS) ensure the message reaches the partner regardless of their monitoring setup.

---

## Part D: Monitoring and Rollback

| Trigger | Action | Result |
|---------|--------|--------|
| New key has > 5% error rate | Re-activate old key, mark new key as revoked | Partners continue using old key, investigate new key issue |
| Partner reports key not working | Generate new key, send to partner | Partner gets working key |
| Old key still used after grace period | Escalate to partner success team | Human intervention |
| Vault unavailable during rotation | Abort rotation, keep current state | Retry when Vault is available |

### Why This Works

Monitoring deprecated key usage reveals which partners need escalation. Rollback is simple because the dual-key design makes it a status change.

---

## Part E: CI/CD Integration

```yaml
# .github/workflows/key-rotation.yaml
name: API Key Rotation
on:
  schedule:
    - cron: "0 2 1 * *"
  workflow_dispatch:
    inputs:
      partner_id:
        description: "Partner ID to rotate (empty for all)"
        required: false
      dry_run:
        description: "Dry run (no actual rotation)"
        required: false
        default: "true"

jobs:
  rotate:
    runs-on: ubuntu-latest
    steps:
      - name: Authenticate to Vault
        uses: hashicorp/vault-action@v2
        with:
          url: https://vault.example.com
          method: kubernetes
          role: key-rotation

      - name: Get partners due for rotation
        id: partners
        run: |
          PARTNERS=$(python scripts/get_rotation_candidates.py --interval 30d)
          echo "partners=$PARTNERS" >> $GITHUB_OUTPUT

      - name: Rotate keys
        if: github.event.inputs.dry_run != 'true'
        run: |
          python scripts/rotate_keys.py --partners "${{ steps.partners.outputs.partners }}" --grace-period 30d --notify

      - name: Generate rotation report
        if: always()
        run: python scripts/generate_report.py --output rotation-report.md
```

### Common Mistakes to Avoid

1. **No grace period.** Revoking keys immediately breaks partner integrations.
2. **Not monitoring deprecated key usage.** Partners who do not switch need escalation.
3. **No rollback procedure.** A documented rollback is essential.
4. **Storing raw API keys.** Always store hashes.
5. **Not tracking key usage.** Usage statistics reveal partner migration progress.

## Key Takeaway

API key rotation requires a dual-key strategy that balances security with partner continuity. The grace period gives partners time to switch, monitoring reveals who has switched, and escalating notifications ensure no one is surprised.
