//! # Daemon Services
//!
//! Building background services and daemon processes in Rust. This lesson covers
//! PID file management, daemonization, systemd integration, and service lifecycle
//! management.
//!
//! ## Key Concepts
//! - Daemon process lifecycle
//! - PID file management
//! - Systemd service integration
//! - Service status reporting
//! - Log rotation and management
//! - Watchdog patterns

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// 1. Service State Machine
// ---------------------------------------------------------------------------

/// States of a service lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Service is not running.
    Stopped,
    /// Service is starting up.
    Starting,
    /// Service is running normally.
    Running,
    /// Service is reloading configuration.
    Reloading,
    /// Service is shutting down gracefully.
    Stopping,
    /// Service encountered an error.
    Failed,
}

impl ServiceState {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Reloading)
    }

    pub fn can_transition_to(&self, target: ServiceState) -> bool {
        matches!(
            (self, target),
            (Self::Stopped, Self::Starting)
                | (Self::Starting, Self::Running)
                | (Self::Starting, Self::Failed)
                | (Self::Running, Self::Reloading)
                | (Self::Running, Self::Stopping)
                | (Self::Running, Self::Failed)
                | (Self::Reloading, Self::Running)
                | (Self::Reloading, Self::Failed)
                | (Self::Stopping, Self::Stopped)
                | (Self::Stopping, Self::Failed)
                | (Self::Failed, Self::Stopped)
                | (Self::Failed, Self::Starting)
        )
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Stopped => "Service is stopped",
            Self::Starting => "Service is starting",
            Self::Running => "Service is running",
            Self::Reloading => "Service is reloading",
            Self::Stopping => "Service is stopping",
            Self::Failed => "Service has failed",
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Service Manager
// ---------------------------------------------------------------------------

/// Configuration for a managed service.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub description: String,
    pub pid_file: String,
    pub log_file: Option<String>,
    pub restart_delay: Duration,
    pub max_restarts: u32,
    pub health_check_interval: Duration,
    pub health_check_timeout: Duration,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "my-service".into(),
            description: "A managed service".into(),
            pid_file: "/tmp/service.pid".into(),
            log_file: None,
            restart_delay: Duration::from_secs(5),
            max_restarts: 3,
            health_check_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(5),
        }
    }
}

/// Manages the lifecycle of a service.
#[derive(Debug)]
pub struct ServiceManager {
    config: ServiceConfig,
    state: ServiceState,
    start_time: Option<Instant>,
    restart_count: u32,
    last_health_check: Option<Instant>,
    health_check_failures: u32,
    events: Vec<ServiceEvent>,
}

#[derive(Debug, Clone)]
pub struct ServiceEvent {
    pub timestamp: u64,
    pub event_type: ServiceEventType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ServiceEventType {
    Started,
    Stopped,
    Failed,
    Reloaded,
    HealthCheckPassed,
    HealthCheckFailed,
    Restarted,
}

impl ServiceManager {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            state: ServiceState::Stopped,
            start_time: None,
            restart_count: 0,
            last_health_check: None,
            health_check_failures: 0,
            events: Vec::new(),
        }
    }

    pub fn state(&self) -> ServiceState {
        self.state
    }

    pub fn config(&self) -> &ServiceConfig {
        &self.config
    }

    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    pub fn events(&self) -> &[ServiceEvent] {
        &self.events
    }

    /// Start the service.
    pub fn start(&mut self) -> Result<(), ServiceError> {
        if !self.state.can_transition_to(ServiceState::Starting) {
            return Err(ServiceError::InvalidTransition {
                from: self.state,
                to: ServiceState::Starting,
            });
        }

        self.transition(ServiceState::Starting, "Service starting");
        // Simulate startup
        self.start_time = Some(Instant::now());
        self.transition(ServiceState::Running, "Service started successfully");
        Ok(())
    }

    /// Stop the service.
    pub fn stop(&mut self) -> Result<(), ServiceError> {
        if !self.state.can_transition_to(ServiceState::Stopping) {
            return Err(ServiceError::InvalidTransition {
                from: self.state,
                to: ServiceState::Stopping,
            });
        }

        self.transition(ServiceState::Stopping, "Service stopping");
        self.start_time = None;
        self.transition(ServiceState::Stopped, "Service stopped");
        Ok(())
    }

    /// Reload the service configuration.
    pub fn reload(&mut self) -> Result<(), ServiceError> {
        if !self.state.can_transition_to(ServiceState::Reloading) {
            return Err(ServiceError::InvalidTransition {
                from: self.state,
                to: ServiceState::Reloading,
            });
        }

        self.transition(ServiceState::Reloading, "Service reloading");
        self.transition(ServiceState::Running, "Service reloaded successfully");
        Ok(())
    }

    /// Report a failure.
    pub fn fail(&mut self, reason: &str) {
        self.transition(ServiceState::Failed, &format!("Service failed: {reason}"));
    }

    /// Restart the service (stop then start).
    pub fn restart(&mut self) -> Result<(), ServiceError> {
        if self.restart_count >= self.config.max_restarts {
            return Err(ServiceError::MaxRestartsExceeded);
        }

        if self.state.is_active() {
            self.stop()?;
        }

        self.restart_count += 1;
        self.record_event(ServiceEventType::Restarted, &format!("Restart #{}", self.restart_count));
        self.start()
    }

    /// Perform a health check.
    pub fn health_check(&mut self, healthy: bool) {
        self.last_health_check = Some(Instant::now());

        if healthy {
            self.health_check_failures = 0;
            self.record_event(ServiceEventType::HealthCheckPassed, "Health check passed");
        } else {
            self.health_check_failures += 1;
            self.record_event(
                ServiceEventType::HealthCheckFailed,
                &format!("Health check failed (consecutive: {})", self.health_check_failures),
            );
        }
    }

    pub fn consecutive_health_failures(&self) -> u32 {
        self.health_check_failures
    }

    /// Generate a systemd service unit file.
    pub fn generate_systemd_unit(&self) -> String {
        format!(
            "[Unit]\n\
             Description={description}\n\
             After=network.target\n\
             \n\
             [Service]\n\
             Type=simple\n\
             ExecStart=/usr/local/bin/{name}\n\
             PIDFile={pid_file}\n\
             Restart=on-failure\n\
             RestartSec={restart_secs}\n\
             \n\
             [Install]\n\
             WantedBy=multi-user.target\n",
            description = self.config.description,
            name = self.config.name,
            pid_file = self.config.pid_file,
            restart_secs = self.config.restart_delay.as_secs(),
        )
    }

    fn transition(&mut self, state: ServiceState, message: &str) {
        let event_type = match state {
            ServiceState::Running => ServiceEventType::Started,
            ServiceState::Stopped => ServiceEventType::Stopped,
            ServiceState::Failed => ServiceEventType::Failed,
            ServiceState::Reloading => ServiceEventType::Reloaded,
            _ => ServiceEventType::Started,
        };
        self.state = state;
        self.record_event(event_type, message);
    }

    fn record_event(&mut self, event_type: ServiceEventType, message: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.events.push(ServiceEvent {
            timestamp,
            event_type,
            message: message.into(),
        });
    }
}

// ---------------------------------------------------------------------------
// 3. Service Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ServiceState,
        to: ServiceState,
    },

    #[error("maximum restart attempts exceeded")]
    MaxRestartsExceeded,

    #[error("service startup failed: {0}")]
    StartupFailed(String),

    #[error("PID file error: {0}")]
    PidFileError(String),
}

// ---------------------------------------------------------------------------
// 4. Service Status Report
// ---------------------------------------------------------------------------

/// A status report for a running service.
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
    pub uptime_secs: Option<u64>,
    pub restart_count: u32,
    pub health_status: HealthStatus,
    pub recent_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl ServiceStatus {
    pub fn from_manager(mgr: &ServiceManager) -> Self {
        let health = if mgr.consecutive_health_failures() == 0 {
            HealthStatus::Healthy
        } else if mgr.consecutive_health_failures() < 3 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let recent: Vec<String> = mgr
            .events()
            .iter()
            .rev()
            .take(5)
            .map(|e| format!("[{:?}] {}", e.event_type, e.message))
            .collect();

        Self {
            name: mgr.config().name.clone(),
            state: mgr.state(),
            uptime_secs: mgr.uptime().map(|d| d.as_secs()),
            restart_count: mgr.restart_count(),
            health_status: health,
            recent_events: recent,
        }
    }

    pub fn format_status(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Service: {}\n", self.name));
        out.push_str(&format!("State: {:?}\n", self.state));
        if let Some(uptime) = self.uptime_secs {
            out.push_str(&format!("Uptime: {uptime}s\n"));
        }
        out.push_str(&format!("Restarts: {}\n", self.restart_count));
        out.push_str(&format!("Health: {:?}\n", self.health_status));
        if !self.recent_events.is_empty() {
            out.push_str("Recent events:\n");
            for event in &self.recent_events {
                out.push_str(&format!("  {event}\n"));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// 5. Log Rotation
// ---------------------------------------------------------------------------

/// Configuration for log rotation.
#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    pub max_size_bytes: u64,
    pub max_files: usize,
    pub compress: bool,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            compress: true,
        }
    }
}

/// Tracks log file information for rotation decisions.
#[derive(Debug, Clone)]
pub struct LogFileInfo {
    pub path: String,
    pub size_bytes: u64,
    pub created_at: u64,
}

/// Determine if log rotation is needed.
pub fn should_rotate(log_file: &LogFileInfo, config: &LogRotationConfig) -> bool {
    log_file.size_bytes >= config.max_size_bytes
}

/// Generate the rotated log file name.
pub fn rotated_log_name(base_path: &str, index: usize) -> String {
    format!("{base_path}.{index}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_state_transitions() {
        assert!(ServiceState::Stopped.can_transition_to(ServiceState::Starting));
        assert!(ServiceState::Starting.can_transition_to(ServiceState::Running));
        assert!(ServiceState::Running.can_transition_to(ServiceState::Stopping));
        assert!(ServiceState::Stopping.can_transition_to(ServiceState::Stopped));
        assert!(ServiceState::Running.can_transition_to(ServiceState::Reloading));
        assert!(ServiceState::Reloading.can_transition_to(ServiceState::Running));
        assert!(ServiceState::Running.can_transition_to(ServiceState::Failed));
        assert!(ServiceState::Failed.can_transition_to(ServiceState::Starting));
    }

    #[test]
    fn test_service_state_invalid_transitions() {
        assert!(!ServiceState::Stopped.can_transition_to(ServiceState::Running));
        assert!(!ServiceState::Stopped.can_transition_to(ServiceState::Stopping));
        assert!(!ServiceState::Running.can_transition_to(ServiceState::Starting));
        assert!(!ServiceState::Failed.can_transition_to(ServiceState::Running));
    }

    #[test]
    fn test_service_state_is_active() {
        assert!(ServiceState::Running.is_active());
        assert!(ServiceState::Reloading.is_active());
        assert!(!ServiceState::Stopped.is_active());
        assert!(!ServiceState::Failed.is_active());
    }

    #[test]
    fn test_service_manager_lifecycle() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);

        assert_eq!(mgr.state(), ServiceState::Stopped);
        assert!(mgr.uptime().is_none());

        mgr.start().unwrap();
        assert_eq!(mgr.state(), ServiceState::Running);
        assert!(mgr.uptime().is_some());

        mgr.reload().unwrap();
        assert_eq!(mgr.state(), ServiceState::Running);

        mgr.stop().unwrap();
        assert_eq!(mgr.state(), ServiceState::Stopped);
        assert!(mgr.uptime().is_none());
    }

    #[test]
    fn test_service_manager_invalid_start() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();

        // Can't start while running
        let result = mgr.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_manager_fail() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();
        mgr.fail("disk full");

        assert_eq!(mgr.state(), ServiceState::Failed);
    }

    #[test]
    fn test_service_manager_restart() {
        let config = ServiceConfig {
            max_restarts: 3,
            ..Default::default()
        };
        let mut mgr = ServiceManager::new(config);

        mgr.start().unwrap();
        mgr.restart().unwrap();
        assert_eq!(mgr.state(), ServiceState::Running);
        assert_eq!(mgr.restart_count(), 1);
    }

    #[test]
    fn test_service_manager_max_restarts() {
        let config = ServiceConfig {
            max_restarts: 2,
            ..Default::default()
        };
        let mut mgr = ServiceManager::new(config);

        mgr.start().unwrap();
        mgr.restart().unwrap();
        mgr.restart().unwrap();

        let result = mgr.restart();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_manager_health_check() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();

        mgr.health_check(true);
        assert_eq!(mgr.consecutive_health_failures(), 0);

        mgr.health_check(false);
        assert_eq!(mgr.consecutive_health_failures(), 1);

        mgr.health_check(false);
        assert_eq!(mgr.consecutive_health_failures(), 2);

        mgr.health_check(true);
        assert_eq!(mgr.consecutive_health_failures(), 0);
    }

    #[test]
    fn test_service_manager_events() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();
        mgr.stop().unwrap();

        let events = mgr.events();
        assert!(events.len() >= 2); // at least starting and stopped
    }

    #[test]
    fn test_systemd_unit_generation() {
        let config = ServiceConfig {
            name: "my-app".into(),
            description: "My Application".into(),
            pid_file: "/run/my-app.pid".into(),
            restart_delay: Duration::from_secs(10),
            ..Default::default()
        };
        let mgr = ServiceManager::new(config);
        let unit = mgr.generate_systemd_unit();

        assert!(unit.contains("Description=My Application"));
        assert!(unit.contains("ExecStart=/usr/local/bin/my-app"));
        assert!(unit.contains("PIDFile=/run/my-app.pid"));
        assert!(unit.contains("RestartSec=10"));
        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("[Install]"));
    }

    #[test]
    fn test_service_status_from_manager() {
        let config = ServiceConfig {
            name: "test-svc".into(),
            ..Default::default()
        };
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();
        mgr.health_check(true);

        let status = ServiceStatus::from_manager(&mgr);
        assert_eq!(status.name, "test-svc");
        assert_eq!(status.state, ServiceState::Running);
        assert_eq!(status.health_status, HealthStatus::Healthy);
        assert!(status.uptime_secs.is_some());
    }

    #[test]
    fn test_service_status_degraded() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();
        mgr.health_check(false);
        mgr.health_check(false);

        let status = ServiceStatus::from_manager(&mgr);
        assert_eq!(status.health_status, HealthStatus::Degraded);
    }

    #[test]
    fn test_service_status_unhealthy() {
        let config = ServiceConfig::default();
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();
        for _ in 0..3 {
            mgr.health_check(false);
        }

        let status = ServiceStatus::from_manager(&mgr);
        assert_eq!(status.health_status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_service_status_format() {
        let config = ServiceConfig {
            name: "my-svc".into(),
            ..Default::default()
        };
        let mut mgr = ServiceManager::new(config);
        mgr.start().unwrap();

        let status = ServiceStatus::from_manager(&mgr);
        let formatted = status.format_status();
        assert!(formatted.contains("my-svc"));
        assert!(formatted.contains("Running"));
    }

    #[test]
    fn test_log_rotation_needed() {
        let log_file = LogFileInfo {
            path: "/var/log/app.log".into(),
            size_bytes: 10 * 1024 * 1024,
            created_at: 0,
        };
        let config = LogRotationConfig::default();
        assert!(should_rotate(&log_file, &config));

        let small_file = LogFileInfo {
            path: "/var/log/app.log".into(),
            size_bytes: 1024,
            created_at: 0,
        };
        assert!(!should_rotate(&small_file, &config));
    }

    #[test]
    fn test_rotated_log_name() {
        assert_eq!(rotated_log_name("/var/log/app.log", 1), "/var/log/app.log.1");
        assert_eq!(rotated_log_name("/var/log/app.log", 3), "/var/log/app.log.3");
    }

    #[test]
    fn test_service_state_description() {
        assert!(!ServiceState::Running.description().is_empty());
        assert!(!ServiceState::Failed.description().is_empty());
    }
}
