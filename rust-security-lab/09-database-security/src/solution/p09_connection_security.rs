//! # Lesson 09: Database Connection Security (Reference Solution)
//!
//! See the exercise file for full documentation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsMode {
    None,
    Prefer,
    Require,
    VerifyFull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub tls_mode: TlsMode,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub verify_hostname: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityAssessment {
    pub secure: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Validate the security configuration of a database connection.
pub fn validate_connection_config(config: &ConnectionConfig) -> SecurityAssessment {
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Check TLS mode
    match config.tls_mode {
        TlsMode::None => {
            issues.push("CRITICAL: TLS is disabled — traffic is plaintext".to_string());
            recommendations.push("Set tls_mode to VerifyFull".to_string());
        }
        TlsMode::Prefer => {
            issues.push("WARNING: TLS is optional — may fall back to plaintext".to_string());
            recommendations.push("Set tls_mode to VerifyFull".to_string());
        }
        TlsMode::Require => {
            issues.push("WARNING: TLS required but not verifying certificates".to_string());
            recommendations.push("Set tls_mode to VerifyFull for certificate validation".to_string());
        }
        TlsMode::VerifyFull => {
            // This is the recommended mode
            if config.ca_cert_path.is_none() {
                issues.push("WARNING: VerifyFull mode without CA cert path".to_string());
                recommendations.push("Set ca_cert_path to your CA certificate".to_string());
            }
        }
    }

    // Check password
    if config.password.is_empty() {
        issues.push("CRITICAL: Password is empty".to_string());
        recommendations.push("Set a strong password".to_string());
    }

    // Check hostname verification
    if !config.verify_hostname {
        issues.push("WARNING: Hostname verification is disabled".to_string());
        recommendations.push("Enable verify_hostname to prevent MITM attacks".to_string());
    }

    // Check default credentials
    if config.username == "root" || config.username == "admin" {
        recommendations.push("Consider using a non-default username".to_string());
    }

    let secure = issues.iter().all(|i| !i.contains("CRITICAL"))
        && config.tls_mode == TlsMode::VerifyFull
        && !config.password.is_empty()
        && config.verify_hostname;

    SecurityAssessment {
        secure,
        issues,
        recommendations,
    }
}

/// Build a connection string without the password.
pub fn build_connection_string(config: &ConnectionConfig) -> String {
    let sslmode = match config.tls_mode {
        TlsMode::None => "disable",
        TlsMode::Prefer => "prefer",
        TlsMode::Require => "require",
        TlsMode::VerifyFull => "verify-full",
    };

    format!(
        "host={} port={} dbname={} user={} sslmode={}",
        config.host, config.port, config.database, config.username, sslmode
    )
}

/// Simulate a TLS handshake with certificate validation.
pub fn simulate_tls_handshake(
    config: &ConnectionConfig,
    server_cert_valid: bool,
    server_hostname: &str,
) -> Result<String, String> {
    // No TLS — return success but note it's insecure
    if config.tls_mode == TlsMode::None {
        return Ok(format!(
            "Connected to {}:{} WITHOUT TLS (insecure)",
            config.host, config.port
        ));
    }

    // Check CA cert for VerifyFull
    if config.tls_mode == TlsMode::VerifyFull && config.ca_cert_path.is_none() {
        return Err("VerifyFull mode requires a CA certificate path".to_string());
    }

    // Verify server certificate
    if !server_cert_valid {
        return Err("Server certificate is invalid or untrusted".to_string());
    }

    // Verify hostname
    if config.verify_hostname && server_hostname != config.host {
        return Err(format!(
            "Hostname mismatch: expected '{}', got '{}'",
            config.host, server_hostname
        ));
    }

    Ok(format!(
        "TLS connection established to {}:{} (mode: {:?})",
        config.host, config.port, config.tls_mode
    ))
}

/// Compare secure vs insecure connection configurations.
pub fn demonstrate_connection_security() -> (SecurityAssessment, SecurityAssessment) {
    let secure = ConnectionConfig {
        host: "db.example.com".to_string(),
        port: 5432,
        database: "production".to_string(),
        username: "app_user".to_string(),
        password: "strong_password_123!".to_string(),
        tls_mode: TlsMode::VerifyFull,
        ca_cert_path: Some("/etc/ssl/ca.crt".to_string()),
        client_cert_path: None,
        client_key_path: None,
        verify_hostname: true,
    };

    let insecure = ConnectionConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "test".to_string(),
        username: "root".to_string(),
        password: "".to_string(),
        tls_mode: TlsMode::None,
        ca_cert_path: None,
        client_cert_path: None,
        client_key_path: None,
        verify_hostname: false,
    };

    (
        validate_connection_config(&secure),
        validate_connection_config(&insecure),
    )
}

/// Check if a connection string leaks credentials.
pub fn leaks_credentials(connection_string: &str) -> bool {
    let lower = connection_string.to_lowercase();

    // Check for password in key=value format
    if lower.contains("password=") || lower.contains("pwd=") {
        return true;
    }

    // Check for credentials in URL format (user:pass@host)
    if lower.contains("://") {
        // Parse the part between :// and @
        if let Some(after_scheme) = lower.split("://").nth(1) {
            if after_scheme.contains('@') {
                let before_at = after_scheme.split('@').next().unwrap_or("");
                if before_at.contains(':') {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secure_config() -> ConnectionConfig {
        ConnectionConfig {
            host: "db.example.com".to_string(),
            port: 5432,
            database: "production".to_string(),
            username: "app_user".to_string(),
            password: "strong_password_123!".to_string(),
            tls_mode: TlsMode::VerifyFull,
            ca_cert_path: Some("/etc/ssl/ca.crt".to_string()),
            client_cert_path: None,
            client_key_path: None,
            verify_hostname: true,
        }
    }

    fn insecure_config() -> ConnectionConfig {
        ConnectionConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "test".to_string(),
            username: "root".to_string(),
            password: "".to_string(),
            tls_mode: TlsMode::None,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            verify_hostname: false,
        }
    }

    #[test]
    fn test_secure_config_valid() {
        let assessment = validate_connection_config(&secure_config());
        assert!(assessment.secure, "Secure config should pass validation");
        assert!(assessment.issues.is_empty() || assessment.issues.iter().all(|i| !i.contains("CRITICAL")));
    }

    #[test]
    fn test_insecure_config_flagged() {
        let assessment = validate_connection_config(&insecure_config());
        assert!(!assessment.secure, "Insecure config should fail validation");
        assert!(!assessment.issues.is_empty());
    }

    #[test]
    fn test_no_tls_flagged() {
        let mut config = secure_config();
        config.tls_mode = TlsMode::None;
        let assessment = validate_connection_config(&config);
        assert!(
            assessment.issues.iter().any(|i| i.contains("TLS") || i.contains("tls") || i.contains("encrypt")),
            "No TLS should be flagged as an issue"
        );
    }

    #[test]
    fn test_empty_password_flagged() {
        let mut config = secure_config();
        config.password = String::new();
        let assessment = validate_connection_config(&config);
        assert!(
            assessment.issues.iter().any(|i| i.contains("password") || i.contains("Password")),
            "Empty password should be flagged"
        );
    }

    #[test]
    fn test_connection_string_no_password() {
        let conn_str = build_connection_string(&secure_config());
        assert!(!conn_str.contains("strong_password_123!"), "Connection string must not contain password");
        assert!(conn_str.contains("db.example.com"));
        assert!(conn_str.contains("5432"));
    }

    #[test]
    fn test_tls_handshake_verify_full() {
        let config = secure_config();
        let result = simulate_tls_handshake(&config, true, "db.example.com");
        assert!(result.is_ok(), "Valid TLS handshake should succeed");
    }

    #[test]
    fn test_tls_handshake_hostname_mismatch() {
        let config = secure_config();
        let result = simulate_tls_handshake(&config, true, "evil.example.com");
        assert!(result.is_err(), "Hostname mismatch should fail");
    }

    #[test]
    fn test_leak_detection() {
        assert!(leaks_credentials("host=db password=secret123 user=admin"));
        assert!(leaks_credentials("postgres://admin:secret@db.example.com/mydb"));
        assert!(!leaks_credentials("host=db.example.com port=5432 dbname=mydb"));
    }

    #[test]
    fn test_demonstrate_connection_security() {
        let (secure, insecure) = demonstrate_connection_security();
        assert!(secure.secure, "Secure config should be assessed as secure");
        assert!(!insecure.secure, "Insecure config should be assessed as insecure");
    }
}
