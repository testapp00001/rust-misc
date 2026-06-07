//! # Deployment Strategies for Rust Services
//!
//! Safe deployment strategies minimize risk when rolling out changes to production.
//! This module covers blue-green deployments, canary releases, rolling updates,
//! and feature flags.
//!
//! ## Strategy Comparison:
//!
//! | Strategy | Downtime | Risk | Rollback Speed | Complexity |
//! |----------|----------|------|----------------|------------|
//! | Blue-Green | None | Low | Instant | Medium |
//! | Canary | None | Very Low | Fast | High |
//! | Rolling | None | Medium | Slow | Low |
//! | Recreate | Yes | High | Slow | Low |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a deployment environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEnvironment {
    pub name: String,
    pub version: String,
    pub replicas: u32,
    pub status: DeploymentStatus,
    pub traffic_weight: f64,
    pub health: EnvironmentHealth,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Active,
    Standby,
    Deploying,
    RollingBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentHealth {
    pub healthy_replicas: u32,
    pub total_replicas: u32,
    pub error_rate: f64,
    pub latency_p99_ms: f64,
}

impl EnvironmentHealth {
    pub fn is_healthy(&self) -> bool {
        let healthy_ratio = self.healthy_replicas as f64 / self.total_replicas as f64;
        healthy_ratio >= 0.8 && self.error_rate < 0.05
    }
}

/// Blue-Green deployment manager.
///
/// Maintains two identical environments: one serving production traffic (blue)
/// and one idle (green). Deployments go to the idle environment, and traffic
/// switches after validation.
#[derive(Debug)]
pub struct BlueGreenDeployment {
    pub blue: DeploymentEnvironment,
    pub green: DeploymentEnvironment,
    pub active_color: Color,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Blue,
    Green,
}

impl BlueGreenDeployment {
    pub fn new(version: &str, replicas: u32) -> Self {
        Self {
            blue: DeploymentEnvironment {
                name: "blue".into(),
                version: version.into(),
                replicas,
                status: DeploymentStatus::Active,
                traffic_weight: 1.0,
                health: EnvironmentHealth {
                    healthy_replicas: replicas,
                    total_replicas: replicas,
                    error_rate: 0.0,
                    latency_p99_ms: 50.0,
                },
                created_at: "2024-01-01T00:00:00Z".into(),
            },
            green: DeploymentEnvironment {
                name: "green".into(),
                version: version.into(),
                replicas,
                status: DeploymentStatus::Standby,
                traffic_weight: 0.0,
                health: EnvironmentHealth {
                    healthy_replicas: replicas,
                    total_replicas: replicas,
                    error_rate: 0.0,
                    latency_p99_ms: 50.0,
                },
                created_at: "2024-01-01T00:00:00Z".into(),
            },
            active_color: Color::Blue,
        }
    }

    /// Get the currently active (traffic-serving) environment.
    pub fn active(&self) -> &DeploymentEnvironment {
        match self.active_color {
            Color::Blue => &self.blue,
            Color::Green => &self.green,
        }
    }

    /// Get the standby (idle) environment.
    pub fn standby(&self) -> &DeploymentEnvironment {
        match self.active_color {
            Color::Blue => &self.green,
            Color::Green => &self.blue,
        }
    }

    /// Deploy a new version to the standby environment.
    pub fn deploy(&mut self, new_version: &str) -> Result<(), String> {
        let standby = match self.active_color {
            Color::Blue => &mut self.green,
            Color::Green => &mut self.blue,
        };
        standby.version = new_version.to_string();
        standby.status = DeploymentStatus::Deploying;
        Ok(())
    }

    /// Switch traffic from active to standby (the "swap").
    pub fn switch_traffic(&mut self) -> Result<(), String> {
        let standby = match self.active_color {
            Color::Blue => &self.green,
            Color::Green => &self.blue,
        };

        if !standby.health.is_healthy() {
            return Err("Standby environment is not healthy".into());
        }

        match self.active_color {
            Color::Blue => {
                self.blue.traffic_weight = 0.0;
                self.blue.status = DeploymentStatus::Standby;
                self.green.traffic_weight = 1.0;
                self.green.status = DeploymentStatus::Active;
                self.active_color = Color::Green;
            }
            Color::Green => {
                self.green.traffic_weight = 0.0;
                self.green.status = DeploymentStatus::Standby;
                self.blue.traffic_weight = 1.0;
                self.blue.status = DeploymentStatus::Active;
                self.active_color = Color::Blue;
            }
        }
        Ok(())
    }

    /// Rollback by switching traffic back to the previous active.
    pub fn rollback(&mut self) {
        let _ = self.switch_traffic();
    }
}

/// Canary deployment manager.
///
/// Gradually shifts traffic from the old version to the new version in
/// controlled increments, with automated health checks at each stage.
#[derive(Debug)]
pub struct CanaryDeployment {
    pub stable: DeploymentEnvironment,
    pub canary: DeploymentEnvironment,
    pub stages: Vec<CanaryStage>,
    pub current_stage: usize,
    pub metrics: CanaryMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryStage {
    pub name: String,
    pub canary_traffic_weight: f64,
    pub duration_minutes: u32,
    pub success_criteria: SuccessCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub max_error_rate: f64,
    pub max_latency_p99_ms: f64,
    pub min_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryMetrics {
    pub canary_error_rate: f64,
    pub canary_latency_p99_ms: f64,
    pub canary_requests: u64,
    pub stable_error_rate: f64,
    pub stable_latency_p99_ms: f64,
}

impl CanaryDeployment {
    pub fn new(stable_version: &str, canary_version: &str, replicas: u32) -> Self {
        Self {
            stable: DeploymentEnvironment {
                name: "stable".into(),
                version: stable_version.into(),
                replicas,
                status: DeploymentStatus::Active,
                traffic_weight: 1.0,
                health: EnvironmentHealth {
                    healthy_replicas: replicas,
                    total_replicas: replicas,
                    error_rate: 0.01,
                    latency_p99_ms: 100.0,
                },
                created_at: "2024-01-01T00:00:00Z".into(),
            },
            canary: DeploymentEnvironment {
                name: "canary".into(),
                version: canary_version.into(),
                replicas: 1,
                status: DeploymentStatus::Deploying,
                traffic_weight: 0.0,
                health: EnvironmentHealth {
                    healthy_replicas: 1,
                    total_replicas: 1,
                    error_rate: 0.0,
                    latency_p99_ms: 0.0,
                },
                created_at: "2024-01-01T00:00:00Z".into(),
            },
            stages: Self::default_stages(),
            current_stage: 0,
            metrics: CanaryMetrics {
                canary_error_rate: 0.0,
                canary_latency_p99_ms: 0.0,
                canary_requests: 0,
                stable_error_rate: 0.01,
                stable_latency_p99_ms: 100.0,
            },
        }
    }

    fn default_stages() -> Vec<CanaryStage> {
        vec![
            CanaryStage {
                name: "Smoke test".into(),
                canary_traffic_weight: 0.01,
                duration_minutes: 5,
                success_criteria: SuccessCriteria {
                    max_error_rate: 0.05,
                    max_latency_p99_ms: 500.0,
                    min_success_rate: 0.95,
                },
            },
            CanaryStage {
                name: "Small traffic".into(),
                canary_traffic_weight: 0.10,
                duration_minutes: 15,
                success_criteria: SuccessCriteria {
                    max_error_rate: 0.02,
                    max_latency_p99_ms: 200.0,
                    min_success_rate: 0.98,
                },
            },
            CanaryStage {
                name: "Medium traffic".into(),
                canary_traffic_weight: 0.50,
                duration_minutes: 30,
                success_criteria: SuccessCriteria {
                    max_error_rate: 0.01,
                    max_latency_p99_ms: 150.0,
                    min_success_rate: 0.99,
                },
            },
            CanaryStage {
                name: "Full traffic".into(),
                canary_traffic_weight: 1.0,
                duration_minutes: 60,
                success_criteria: SuccessCriteria {
                    max_error_rate: 0.005,
                    max_latency_p99_ms: 120.0,
                    min_success_rate: 0.995,
                },
            },
        ]
    }

    /// Advance to the next canary stage if criteria are met.
    pub fn advance_stage(&mut self) -> Result<CanaryAction, String> {
        if self.current_stage >= self.stages.len() {
            return Ok(CanaryAction::Promote);
        }

        let stage = &self.stages[self.current_stage];

        if self.metrics.canary_error_rate > stage.success_criteria.max_error_rate {
            return Ok(CanaryAction::Rollback);
        }

        if self.metrics.canary_latency_p99_ms > stage.success_criteria.max_latency_p99_ms {
            return Ok(CanaryAction::Rollback);
        }

        // Advance to next stage
        self.current_stage += 1;
        if self.current_stage >= self.stages.len() {
            return Ok(CanaryAction::Promote);
        }

        let next_stage = &self.stages[self.current_stage];
        self.canary.traffic_weight = next_stage.canary_traffic_weight;
        self.stable.traffic_weight = 1.0 - next_stage.canary_traffic_weight;

        Ok(CanaryAction::Continue)
    }

    /// Get the current canary stage progress as a percentage.
    pub fn progress(&self) -> f64 {
        if self.stages.is_empty() {
            return 100.0;
        }
        (self.current_stage as f64 / self.stages.len() as f64) * 100.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanaryAction {
    Continue,
    Promote,
    Rollback,
}

/// Rolling update configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingUpdateConfig {
    /// Maximum number of pods that can be unavailable during update
    pub max_unavailable: String,
    /// Maximum number of pods over desired count
    pub max_surge: String,
    /// Minimum seconds a pod must be ready before continuing
    pub min_ready_seconds: u32,
    /// Seconds to wait for graceful shutdown
    pub termination_grace_period: u32,
}

impl Default for RollingUpdateConfig {
    fn default() -> Self {
        Self {
            max_unavailable: "25%".into(),
            max_surge: "25%".into(),
            min_ready_seconds: 10,
            termination_grace_period: 30,
        }
    }
}

impl RollingUpdateConfig {
    /// Conservative rolling update (zero downtime).
    pub fn zero_downtime() -> Self {
        Self {
            max_unavailable: "0".into(),
            max_surge: "1".into(),
            min_ready_seconds: 30,
            termination_grace_period: 60,
        }
    }

    /// Aggressive rolling update (fast but brief unavailability possible).
    pub fn aggressive() -> Self {
        Self {
            max_unavailable: "50%".into(),
            max_surge: "50%".into(),
            min_ready_seconds: 5,
            termination_grace_period: 15,
        }
    }

    /// Generate Kubernetes deployment strategy YAML snippet.
    pub fn to_k8s_yaml(&self) -> String {
        format!(
            r#"strategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: "{}"
    maxSurge: "{}"
minReadySeconds: {}
terminationGracePeriodSeconds: {}"#,
            self.max_unavailable,
            self.max_surge,
            self.min_ready_seconds,
            self.termination_grace_period
        )
    }
}

/// Feature flag system for controlling feature rollouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub rollout_percentage: f64,
    pub allowed_users: Vec<String>,
    pub environment: String,
}

impl FeatureFlag {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            enabled: false,
            rollout_percentage: 0.0,
            allowed_users: Vec::new(),
            environment: "development".into(),
        }
    }

    pub fn with_rollout(mut self, percentage: f64) -> Self {
        self.rollout_percentage = percentage.clamp(0.0, 100.0);
        self
    }

    pub fn enabled_in(mut self, env: &str) -> Self {
        self.environment = env.to_string();
        self.enabled = true;
        self
    }

    pub fn allow_user(mut self, user: &str) -> Self {
        self.allowed_users.push(user.to_string());
        self
    }

    /// Check if a feature is enabled for a specific user.
    pub fn is_enabled_for(&self, user_id: &str, current_env: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if self.environment != current_env && self.environment != "*" {
            return false;
        }
        if self.allowed_users.contains(&user_id.to_string()) {
            return true;
        }
        // Simple hash-based percentage rollout
        let hash = user_id.bytes().map(|b| b as u64).sum::<u64>();
        let bucket = (hash % 100) as f64;
        bucket < self.rollout_percentage
    }
}

/// Feature flag store for managing multiple flags.
#[derive(Debug)]
pub struct FeatureFlagStore {
    flags: HashMap<String, FeatureFlag>,
}

impl FeatureFlagStore {
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    pub fn register(&mut self, flag: FeatureFlag) {
        self.flags.insert(flag.name.clone(), flag);
    }

    pub fn is_enabled(&self, flag_name: &str, user_id: &str, env: &str) -> bool {
        self.flags
            .get(flag_name)
            .map(|f| f.is_enabled_for(user_id, env))
            .unwrap_or(false)
    }

    pub fn get_flag(&self, name: &str) -> Option<&FeatureFlag> {
        self.flags.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blue_green_initialization() {
        let bg = BlueGreenDeployment::new("1.0.0", 3);
        assert_eq!(bg.active().version, "1.0.0");
        assert_eq!(bg.active_color, Color::Blue);
        assert_eq!(bg.blue.traffic_weight, 1.0);
        assert_eq!(bg.green.traffic_weight, 0.0);
    }

    #[test]
    fn test_blue_green_deploy_and_switch() {
        let mut bg = BlueGreenDeployment::new("1.0.0", 3);
        bg.deploy("2.0.0").unwrap();
        assert_eq!(bg.green.version, "2.0.0");
        assert_eq!(bg.green.status, DeploymentStatus::Deploying);

        // Simulate green becoming healthy
        bg.green.health.healthy_replicas = 3;
        bg.green.status = DeploymentStatus::Active;

        bg.switch_traffic().unwrap();
        assert_eq!(bg.active_color, Color::Green);
        assert_eq!(bg.active().version, "2.0.0");
        assert_eq!(bg.active().traffic_weight, 1.0);
    }

    #[test]
    fn test_blue_green_rollback() {
        let mut bg = BlueGreenDeployment::new("1.0.0", 3);
        bg.deploy("2.0.0").unwrap();
        bg.green.health.healthy_replicas = 3;
        bg.green.status = DeploymentStatus::Active;
        bg.switch_traffic().unwrap();
        assert_eq!(bg.active_color, Color::Green);

        bg.rollback();
        assert_eq!(bg.active_color, Color::Blue);
        assert_eq!(bg.active().version, "1.0.0");
    }

    #[test]
    fn test_blue_green_unhealthy_blocks_switch() {
        let mut bg = BlueGreenDeployment::new("1.0.0", 3);
        bg.deploy("2.0.0").unwrap();
        // Green is unhealthy
        bg.green.health.healthy_replicas = 1;
        bg.green.health.total_replicas = 3;

        let result = bg.switch_traffic();
        assert!(result.is_err());
        assert_eq!(bg.active_color, Color::Blue); // Still blue
    }

    #[test]
    fn test_canary_initialization() {
        let canary = CanaryDeployment::new("1.0.0", "2.0.0", 5);
        assert_eq!(canary.stable.version, "1.0.0");
        assert_eq!(canary.canary.version, "2.0.0");
        assert_eq!(canary.current_stage, 0);
        assert_eq!(canary.stages.len(), 4);
    }

    #[test]
    fn test_canary_advance_healthy() {
        let mut canary = CanaryDeployment::new("1.0.0", "2.0.0", 5);
        canary.metrics.canary_error_rate = 0.001;
        canary.metrics.canary_latency_p99_ms = 100.0;

        let action = canary.advance_stage().unwrap();
        assert_eq!(action, CanaryAction::Continue);
        assert_eq!(canary.current_stage, 1);
    }

    #[test]
    fn test_canary_advance_unhealthy_rollback() {
        let mut canary = CanaryDeployment::new("1.0.0", "2.0.0", 5);
        canary.metrics.canary_error_rate = 0.10; // 10% error rate, above threshold

        let action = canary.advance_stage().unwrap();
        assert_eq!(action, CanaryAction::Rollback);
    }

    #[test]
    fn test_canary_full_promotion() {
        let mut canary = CanaryDeployment::new("1.0.0", "2.0.0", 5);
        canary.metrics.canary_error_rate = 0.001;
        canary.metrics.canary_latency_p99_ms = 50.0;

        // Advance through all stages
        for _ in 0..canary.stages.len() {
            let action = canary.advance_stage().unwrap();
            if action == CanaryAction::Promote {
                break;
            }
        }

        assert!(canary.progress() >= 100.0);
    }

    #[test]
    fn test_canary_progress() {
        let mut canary = CanaryDeployment::new("1.0.0", "2.0.0", 5);
        assert!((canary.progress() - 0.0).abs() < f64::EPSILON);

        canary.current_stage = 2;
        assert!((canary.progress() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rolling_update_default() {
        let config = RollingUpdateConfig::default();
        assert_eq!(config.max_unavailable, "25%");
        assert_eq!(config.max_surge, "25%");
    }

    #[test]
    fn test_rolling_update_zero_downtime() {
        let config = RollingUpdateConfig::zero_downtime();
        assert_eq!(config.max_unavailable, "0");
        assert_eq!(config.max_surge, "1");
    }

    #[test]
    fn test_rolling_update_k8s_yaml() {
        let config = RollingUpdateConfig::zero_downtime();
        let yaml = config.to_k8s_yaml();
        assert!(yaml.contains("RollingUpdate"));
        assert!(yaml.contains("maxUnavailable: \"0\""));
        assert!(yaml.contains("minReadySeconds: 30"));
    }

    #[test]
    fn test_feature_flag_basic() {
        let flag = FeatureFlag::new("dark-mode", "Enable dark mode theme")
            .enabled_in("production")
            .with_rollout(50.0);

        assert!(flag.enabled);
        assert_eq!(flag.rollout_percentage, 50.0);
    }

    #[test]
    fn test_feature_flag_disabled() {
        let flag = FeatureFlag::new("dark-mode", "Enable dark mode");
        assert!(!flag.is_enabled_for("user1", "production"));
    }

    #[test]
    fn test_feature_flag_allowed_user() {
        let flag = FeatureFlag::new("beta", "Beta features")
            .enabled_in("production")
            .with_rollout(0.0)
            .allow_user("admin-user");

        assert!(flag.is_enabled_for("admin-user", "production"));
        // Non-allowed user with 0% rollout
        assert!(!flag.is_enabled_for("random-user", "production"));
    }

    #[test]
    fn test_feature_flag_wrong_environment() {
        let flag = FeatureFlag::new("feature", "desc")
            .enabled_in("staging")
            .with_rollout(100.0);

        assert!(!flag.is_enabled_for("user1", "production"));
        assert!(flag.is_enabled_for("user1", "staging"));
    }

    #[test]
    fn test_feature_flag_store() {
        let mut store = FeatureFlagStore::new();
        store.register(
            FeatureFlag::new("feature-a", "Feature A")
                .enabled_in("production")
                .with_rollout(100.0),
        );
        store.register(
            FeatureFlag::new("feature-b", "Feature B")
                .enabled_in("staging")
                .with_rollout(50.0),
        );

        assert!(store.is_enabled("feature-a", "user1", "production"));
        assert!(!store.is_enabled("feature-b", "user1", "production"));
        assert!(!store.is_enabled("nonexistent", "user1", "production"));
    }

    #[test]
    fn test_environment_health() {
        let healthy = EnvironmentHealth {
            healthy_replicas: 5,
            total_replicas: 5,
            error_rate: 0.01,
            latency_p99_ms: 100.0,
        };
        assert!(healthy.is_healthy());

        let unhealthy = EnvironmentHealth {
            healthy_replicas: 2,
            total_replicas: 5,
            error_rate: 0.10,
            latency_p99_ms: 500.0,
        };
        assert!(!unhealthy.is_healthy());
    }
}
