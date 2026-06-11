# 61 - Feature Flags

## Decoupling Deploy from Release

> **Previous:** [60 - GitOps](../60-gitops/README.md) | **Next:** [62 - DNS Deep Dive](../62-dns-deep-dive/README.md)

Feature flags (also called feature toggles) allow you to control the visibility of functionality without deploying new code. This decouples deployment from release: you can deploy code to production with features hidden behind flags, then enable them independently. This is the foundation of progressive delivery.

---

## Problem

Without feature flags, deploying and releasing are the same event:

```
Traditional Flow:
  Code Change -> Build -> Test -> Deploy to Prod -> Feature is LIVE
                                                        |
                                                   No going back
                                                   without another deploy
```

This creates several problems:
- Long-lived feature branches cause merge conflicts
- Big-bang releases are high-risk
- No way to do A/B testing or canary releases for features
- Rollback requires a full redeployment
- Incomplete features cannot sit in the main branch
- Kill switches for problematic features require emergency deploys
- No way to target features to specific users or segments

---

## Naive Way: Compile-Time Flags and Branches

```rust
// Naive: compile-time feature flags
#[cfg(feature = "new-checkout")]
fn process_checkout() {
    // New checkout flow
}

#[cfg(not(feature = "new-checkout"))]
fn process_checkout() {
    // Old checkout flow
}
```

```bash
# Build with feature enabled
cargo build --features new-checkout

# Build without
cargo build
```

Problems:
- Requires separate builds for different configurations
- Cannot change at runtime
- Each combination requires its own binary
- No targeting to specific users
- Cannot do percentage rollouts

---

## Right Way: Runtime Feature Flags

### Feature Flag Architecture

```
                         +------------------+
                         |  Feature Flag    |
                         |  Service/SDK     |
                         +--------+---------+
                                  |
              +-------------------+-------------------+
              |                   |                   |
    +---------v-------+ +--------v--------+ +--------v--------+
    | User Targeting  | | Percentage      | | Environment     |
    | Rules           | | Rollout         | | Rules           |
    +-----------------+ +-----------------+ +-----------------+
    
    Application Code:
    if feature_flags.is_enabled("new-checkout", &user_context) {
        new_checkout_flow(user)
    } else {
        old_checkout_flow(user)
    }
```

### Feature Flag Types

```
| Type               | Lifetime    | Example                        |
|--------------------|-------------|--------------------------------|
| Release Toggle     | Days-Weeks  | Hide incomplete feature        |
| Experiment Toggle  | Weeks       | A/B test checkout flow         |
| Ops Toggle         | Minutes-Hrs | Circuit breaker, load shedding |
| Permission Toggle  | Permanent   | Premium feature access         |
```

### Building a Feature Flag System in Rust

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub key: String,
    pub enabled: bool,
    pub rules: Vec<TargetingRule>,
    pub rollout: Option<RolloutConfig>,
    pub variants: Option<Vec<Variant>>,
    pub default_variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingRule {
    pub attribute: String,       // e.g., "email", "plan", "country"
    pub operator: RuleOperator,
    pub values: Vec<String>,
    pub variant: Option<String>, // Which variant to serve
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutConfig {
    pub percentage: f64,         // 0.0 to 100.0
    pub seed: String,            // For consistent hashing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub name: String,
    pub payload: serde_json::Value,
    pub weight: f64,             // Traffic percentage for this variant
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub email: Option<String>,
    pub plan: Option<String>,
    pub country: Option<String>,
    pub custom_attributes: HashMap<String, String>,
}

pub struct FeatureFlagEvaluator {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl FeatureFlagEvaluator {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load flags from a configuration source
    pub async fn load_flags(&self, flags: Vec<FeatureFlag>) {
        let mut map = self.flags.write().await;
        for flag in flags {
            map.insert(flag.key.clone(), flag);
        }
    }

    /// Check if a feature is enabled for a user
    pub async fn is_enabled(&self, flag_key: &str, context: &UserContext) -> bool {
        let flags = self.flags.read().await;
        let flag = match flags.get(flag_key) {
            Some(f) => f,
            None => return false, // Unknown flag = disabled
        };

        // Flag globally disabled
        if !flag.enabled {
            return false;
        }

        // Check targeting rules first
        for rule in &flag.rules {
            if self.evaluate_rule(rule, context) {
                return true;
            }
        }

        // Check percentage rollout
        if let Some(rollout) = &flag.rollout {
            return self.evaluate_rollout(rollout, context);
        }

        false
    }

    /// Get the variant for a feature flag (for A/B testing)
    pub async fn get_variant(&self, flag_key: &str, context: &UserContext) -> Option<String> {
        let flags = self.flags.read().await;
        let flag = flags.get(flag_key)?;

        if !flag.enabled {
            return flag.default_variant.clone();
        }

        // Check targeting rules
        for rule in &flag.rules {
            if self.evaluate_rule(rule, context) {
                return rule.variant.clone().or_else(|| flag.default_variant.clone());
            }
        }

        // Check variants for A/B test
        if let Some(variants) = &flag.variants {
            return self.select_variant(variants, context, flag_key);
        }

        flag.default_variant.clone()
    }

    fn evaluate_rule(&self, rule: &TargetingRule, context: &UserContext) -> bool {
        let value = if rule.attribute == "user_id" {
            Some(&context.user_id)
        } else if rule.attribute == "email" {
            context.email.as_ref()
        } else if rule.attribute == "plan" {
            context.plan.as_ref()
        } else if rule.attribute == "country" {
            context.country.as_ref()
        } else {
            context.custom_attributes.get(&rule.attribute)
        };

        let value = match value {
            Some(v) => v,
            None => return false,
        };

        match &rule.operator {
            RuleOperator::Equals => value == &rule.values[0],
            RuleOperator::NotEquals => value != &rule.values[0],
            RuleOperator::Contains => value.contains(&rule.values[0]),
            RuleOperator::StartsWith => value.starts_with(&rule.values[0]),
            RuleOperator::EndsWith => value.ends_with(&rule.values[0]),
            RuleOperator::In => rule.values.contains(value),
            RuleOperator::NotIn => !rule.values.contains(value),
            RuleOperator::GreaterThan => {
                value.parse::<f64>().unwrap_or(0.0) > rule.values[0].parse::<f64>().unwrap_or(0.0)
            }
            RuleOperator::LessThan => {
                value.parse::<f64>().unwrap_or(0.0) < rule.values[0].parse::<f64>().unwrap_or(0.0)
            }
        }
    }

    fn evaluate_rollout(&self, rollout: &RolloutConfig, context: &UserContext) -> bool {
        // Consistent hashing: same user always gets same result
        let hash_input = format!("{}:{}", context.user_id, rollout.seed);
        let hash = self.hash_string(&hash_input);
        let bucket = (hash % 10000) as f64 / 100.0; // 0.0 to 99.99
        bucket < rollout.percentage
    }

    fn select_variant(&self, variants: &[Variant], context: &UserContext, flag_key: &str) -> Option<String> {
        let hash_input = format!("{}:{}", context.user_id, flag_key);
        let hash = self.hash_string(&hash_input);
        let bucket = (hash % 10000) as f64 / 100.0;

        let mut cumulative = 0.0;
        for variant in variants {
            cumulative += variant.weight;
            if bucket < cumulative {
                return Some(variant.name.clone());
            }
        }

        variants.last().map(|v| v.name.clone())
    }

    fn hash_string(&self, input: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}
```

### Using Feature Flags in Application Code

```rust
// Feature flag configuration
async fn setup_flags() -> FeatureFlagEvaluator {
    let evaluator = FeatureFlagEvaluator::new();

    let flags = vec![
        FeatureFlag {
            key: "new-checkout".to_string(),
            enabled: true,
            rules: vec![
                // Internal users always see new checkout
                TargetingRule {
                    attribute: "email".to_string(),
                    operator: RuleOperator::EndsWith,
                    values: vec!["@mycompany.com".to_string()],
                    variant: None,
                },
                // Beta testers
                TargetingRule {
                    attribute: "plan".to_string(),
                    operator: RuleOperator::Equals,
                    values: vec!["beta".to_string()],
                    variant: None,
                },
            ],
            rollout: Some(RolloutConfig {
                percentage: 10.0, // 10% of remaining users
                seed: "new-checkout-v2".to_string(),
            }),
            variants: None,
            default_variant: None,
        },
        FeatureFlag {
            key: "checkout-experiment".to_string(),
            enabled: true,
            rules: vec![],
            rollout: None,
            variants: Some(vec![
                Variant {
                    name: "control".to_string(),
                    payload: serde_json::json!({"button_color": "blue"}),
                    weight: 50.0,
                },
                Variant {
                    name: "treatment-a".to_string(),
                    payload: serde_json::json!({"button_color": "green"}),
                    weight: 25.0,
                },
                Variant {
                    name: "treatment-b".to_string(),
                    payload: serde_json::json!({"button_color": "red"}),
                    weight: 25.0,
                },
            ]),
            default_variant: Some("control".to_string()),
        },
        FeatureFlag {
            key: "maintenance-mode".to_string(),
            enabled: false, // Ops toggle: flip to true for emergency maintenance
            rules: vec![],
            rollout: None,
            variants: None,
            default_variant: None,
        },
    ];

    evaluator.load_flags(flags).await;
    evaluator
}

// Application handler using feature flags
async fn handle_checkout(
    evaluator: &FeatureFlagEvaluator,
    user: &UserContext,
) -> HttpResponse {
    // Check for maintenance mode (ops toggle)
    if evaluator.is_enabled("maintenance-mode", user).await {
        return HttpResponse::ServiceUnavailable()
            .body("System is under maintenance. Please try again later.");
    }

    // A/B test: which checkout variant?
    let variant = evaluator.get_variant("checkout-experiment", user).await;

    // Check if new checkout is enabled for this user
    if evaluator.is_enabled("new-checkout", user).await {
        let config = match variant.as_deref() {
            Some("treatment-a") => CheckoutConfig { button_color: "green" },
            Some("treatment-b") => CheckoutConfig { button_color: "red" },
            _ => CheckoutConfig { button_color: "blue" },
        };
        render_new_checkout(user, &config).await
    } else {
        render_old_checkout(user).await
    }
}
```

### Feature Flag Lifecycle Management

```rust
/// Feature flag lifecycle states
#[derive(Debug, Clone, PartialEq)]
enum FlagLifecycle {
    /// New flag, being tested internally
    InDevelopment,
    /// Rolling out to a percentage of users
    RollingOut,
    /// Fully launched to all users
    FullyLaunched,
    /// Ready to be cleaned up (flag always returns true)
    ReadyForCleanup,
    /// Deprecated, should be removed from code
    Deprecated,
}

struct FlagLifecycleManager {
    flags: HashMap<String, FlagLifecycle>,
    created_at: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl FlagLifecycleManager {
    /// Identify flags that should be cleaned up
    fn flags_needing_cleanup(&self) -> Vec<&String> {
        self.flags
            .iter()
            .filter(|(_, lifecycle)| **lifecycle == FlagLifecycle::ReadyForCleanup)
            .map(|(key, _)| key)
            .collect()
    }

    /// Generate a cleanup PR for fully launched flags
    fn generate_cleanup_plan(&self, flag_key: &str) -> String {
        format!(
            r#"## Feature Flag Cleanup: {}

This flag is fully launched and can be removed.

### Steps:
1. Remove all `{}` checks from application code
2. Remove the old code path
3. Remove the flag configuration
4. Delete the flag from the feature flag service

### Verification:
- All users are receiving the new behavior
- No code paths reference this flag
- Tests pass without the flag

### Risk: Low
The flag has been at 100% for {} days with no issues.
"#,
            flag_key,
            flag_key,
            self.days_since_creation(flag_key)
        )
    }

    fn days_since_creation(&self, flag_key: &str) -> i64 {
        self.created_at
            .get(flag_key)
            .map(|created| {
                (chrono::Utc::now() - created).num_days()
            })
            .unwrap_or(0)
    }
}
```

---

## Production Way: LaunchDarkly / Unleash Integration

### Unleash (Open Source)

```yaml
# docker-compose for Unleash
version: '3.8'
services:
  unleash:
    image: unleashorg/unleash-server:latest
    ports:
      - "4242:4242"
    environment:
      DATABASE_URL: postgres://unleash:password@postgres:5432/unleash
      INIT_ADMIN_API_TOKENS: "admin-token-123"
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: unleash
      POSTGRES_USER: unleash
      POSTGRES_PASSWORD: password
    volumes:
      - unleash-data:/var/lib/postgresql/data

volumes:
  unleash-data:
```

### LaunchDarkly SDK Integration

```rust
// Using LaunchDarkly Rust SDK
use launchdarkly_server_sdk as ld;

async fn setup_launchdarkly() -> ld::Client {
    let config = ld::Config::builder("sdk-key-123")
        .stream(true) // Real-time updates via streaming
        .build();

    let client = ld::Client::build(config).expect("Failed to init LaunchDarkly");
    client.start_streaming().await;

    client
}

async fn check_feature(client: &ld::Client, user: &ld::User) -> bool {
    let flag_key = "new-checkout";

    // Boolean variation
    let enabled = client.bool_variation(user, flag_key, false);

    // String variation (for A/B testing)
    let variant = client.string_variation(user, "checkout-experiment", "control");

    // JSON variation (for complex config)
    let config = client.json_variation(
        user,
        "checkout-config",
        serde_json::json!({"button_color": "blue"}),
    );

    enabled
}
```

### Percentage Rollouts with Targeting

```yaml
# Unleash strategy configuration
{
  "name": "new-checkout",
  "enabled": true,
  "strategies": [
    {
      "name": "userWithId",
      "parameters": {
        "userIds": "user-123,user-456,user-789"
      }
    },
    {
      "name": "gradualRollout",
      "parameters": {
        "percentage": "25",
        "groupId": "new-checkout"
      }
    },
    {
      "name": "flexibleRollout",
      "parameters": {
        "rollout": "50",
        "stickiness": "userId",
        "groupId": "new-checkout"
      }
    }
  ],
  "variants": [
    {
      "name": "control",
      "weight": 50,
      "payload": {
        "type": "json",
        "value": "{\"button_color\": \"blue\"}"
      }
    },
    {
      "name": "treatment",
      "weight": 50,
      "payload": {
        "type": "json",
        "value": "{\"button_color\": \"green\"}"
      }
    }
  ]
}
```

### Feature Flag Analytics Pipeline

```yaml
# Track feature flag evaluations for analytics
# .github/workflows/feature-analytics.yml
name: Feature Flag Analytics

on:
  schedule:
    - cron: '0 9 * * 1'  # Weekly Monday 9 AM

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - name: Fetch flag evaluations
        run: |
          # Query LaunchDarkly/Unleash API for evaluation counts
          curl -s -H "Authorization: ${{ secrets.UNLEASH_TOKEN }}" \
            "https://unleash.internal/api/admin/metrics/feature-toggles" \
            | jq '.lastHour[] | {name: .name, yes: .yes, no: .no, exposure: (.yes / (.yes + .no) * 100)}'

      - name: Identify stale flags
        run: |
          # Find flags that haven't been evaluated in 30 days
          curl -s -H "Authorization: ${{ secrets.UNLEASH_TOKEN }}" \
            "https://unleash.internal/api/admin/features" \
            | jq '.features[] | select(.stale == true) | .name'

      - name: Generate cleanup report
        run: |
          echo "## Feature Flag Cleanup Report" > report.md
          echo "" >> report.md
          echo "### Stale Flags (candidates for removal):" >> report.md
          # ... generate report
```

---

## Hands-On Lab

### Lab: Implement Feature Flags with A/B Testing

#### Part 1: Set Up Feature Flag Service

```bash
# Start Unleash locally
docker run -d --name unleash \
  -p 4242:4242 \
  -e DATABASE_URL=postgres://unleash:password@host.docker.internal:5432/unleash \
  unleashorg/unleash-server:latest

# Create a feature flag via API
curl -X POST http://localhost:4242/api/admin/features \
  -H "Authorization: admin-token-123" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-checkout",
    "description": "New checkout flow with improved UX",
    "type": "release",
    "enabled": false,
    "strategies": [
      {
        "name": "default",
        "parameters": {}
      }
    ]
  }'
```

#### Part 2: Build a Rust Service with Feature Flags

```rust
// Cargo.toml dependencies:
// unleash-api = "0.7"
// reqwest = { version = "0.11", features = ["json"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// tokio = { version = "1", features = ["full"] }
// warp = "0.3"

use std::sync::Arc;
use tokio::sync::RwLock;

struct FeatureClient {
    api_url: String,
    api_token: String,
    features: Arc<RwLock<HashMap<String, bool>>>,
}

impl FeatureClient {
    async fn new(api_url: &str, api_token: &str) -> Self {
        let client = Self {
            api_url: api_url.to_string(),
            api_token: api_token.to_string(),
            features: Arc::new(RwLock::new(HashMap::new())),
        };
        client.refresh().await;
        client
    }

    async fn refresh(&self) {
        let resp = reqwest::Client::new()
            .get(format!("{}/api/client/features", self.api_url))
            .header("Authorization", &self.api_token)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        let mut features = self.features.write().await;
        if let Some(feats) = resp["features"].as_array() {
            for feat in feats {
                let name = feat["name"].as_str().unwrap().to_string();
                let enabled = feat["enabled"].as_bool().unwrap_or(false);
                features.insert(name, enabled);
            }
        }
    }

    async fn is_enabled(&self, feature: &str) -> bool {
        let features = self.features.read().await;
        *features.get(feature).unwrap_or(&false)
    }
}

#[tokio::main]
async fn main() {
    let feature_client = Arc::new(
        FeatureClient::new("http://localhost:4242", "admin-token-123").await
    );

    // Periodic refresh
    let client_clone = feature_client.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            client_clone.refresh().await;
        }
    });

    // Routes
    let features = feature_client.clone();
    let checkout = warp::path("checkout")
        .and(warp::get())
        .and_then(move || {
            let features = features.clone();
            async move {
                if features.is_enabled("new-checkout").await {
                    Ok(warp::reply::html("<h1>New Checkout Flow</h1>"))
                } else {
                    Ok(warp::reply::html("<h1>Classic Checkout</h1>"))
                }
            }
        });

    warp::serve(checkout).run(([0, 0, 0, 0], 3030)).await;
}
```

#### Part 3: Test Feature Flag Toggling

```bash
# Test with flag disabled
curl http://localhost:3030/checkout
# Returns: Classic Checkout

# Enable the flag in Unleash
curl -X PUT http://localhost:4242/api/admin/features/new-checkout/toggle/on \
  -H "Authorization: admin-token-123"

# Wait for refresh (15 seconds) or trigger manual refresh
sleep 16

# Test with flag enabled
curl http://localhost:3030/checkout
# Returns: New Checkout Flow

# Disable the flag (kill switch)
curl -X PUT http://localhost:4242/api/admin/features/new-checkout/toggle/off \
  -H "Authorization: admin-token-123"
```

#### Part 4: Percentage Rollout

```bash
# Update flag with percentage rollout strategy
curl -X PUT http://localhost:4242/api/admin/features/new-checkout \
  -H "Authorization: admin-token-123" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-checkout",
    "enabled": true,
    "strategies": [
      {
        "name": "gradualRollout",
        "parameters": {
          "percentage": "25",
          "groupId": "new-checkout"
        }
      }
    ]
  }'

# Test with multiple users
for i in $(seq 1 20); do
  RESULT=$(curl -s -H "X-User-ID: user-$i" http://localhost:3030/checkout)
  echo "user-$i: $RESULT"
done
# Approximately 25% should see the new checkout
```

---

## Limitation

Feature flags give you control over feature rollout but do not solve networking complexity. As you add more services, percentage rollouts, and A/B tests, the network paths your traffic takes become increasingly complex. DNS resolution, service discovery, and traffic routing all need to work correctly for your feature flags to have any effect.

---

## Next Topic

[62 - DNS Deep Dive](../62-dns-deep-dive/README.md) -- Understanding DNS resolution, service discovery, and how feature flags interact with traffic routing at the DNS layer.
