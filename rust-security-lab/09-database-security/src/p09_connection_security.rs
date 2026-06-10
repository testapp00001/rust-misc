//! # Lesson 09: Database Connection Security
//!
//! ## Why Connection Security?
//!
//! Even if your database encrypts data at rest and your queries are parameterized,
//! an attacker on the network can sniff unencrypted database traffic. Database
//! connections carry:
//! - Authentication credentials (username/password)
//! - Query results (potentially sensitive data)
//! - Schema information
//!
//! ## TLS for Database Connections
//!
//! ```text
//! Client  ←—— TLS 1.3 ——→  Database Server
//!
//! TLS provides:
//!   - ENCRYPTION: Traffic is unreadable to network observers
//!   - AUTHENTICATION: Client verifies it's talking to the real server (cert pinning)
//!   - INTEGRITY: Any tampering with traffic is detected
//!
//! Without TLS:
//!   - Attacker on same network reads all queries and results
//!   - Attacker can modify queries in transit (MITM)
//!   - Attacker can steal credentials from connection handshake
//! ```
//!
//! ## Certificate Validation
//!
//! ```text
//! CORRECT: Verify server cert against trusted CA
//!   → Prevents MITM with self-signed cert
//!
//! WRONG: Accept any cert (sslmode=allow or verify-full=false)
//!   → MITM attacker presents their own cert
//!   → Client happily connects to attacker
//!   → Attacker proxies to real server, reading everything
//! ```
//!
//! ## This Module
//!
//! We simulate TLS connection configuration and certificate validation. Since we
//! cannot establish real TLS connections here, we focus on the configuration
//! validation and security checks that a connection library should perform.

use serde::{Deserialize, Serialize};

/// TLS mode for database connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsMode {
    /// No TLS — traffic is plaintext (NEVER use in production)
    None,
    /// TLS optional — use TLS if available, fall back to plaintext
    Prefer,
    /// TLS required — fail if TLS cannot be established
    Require,
    /// TLS required + certificate verification (RECOMMENDED)
    VerifyFull,
}

/// Configuration for a database connection.
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
    /// Whether to verify the server's hostname matches the certificate
    pub verify_hostname: bool,
}

/// Result of connection security validation.
#[derive(Debug, Clone)]
pub struct SecurityAssessment {
    pub secure: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Validate the security configuration of a database connection.
///
/// Checks:
/// 1. TLS mode must be VerifyFull for production
/// 2. Password must not be empty
/// 3. Hostname verification must be enabled
/// 4. CA certificate path must be specified for VerifyFull
/// 5. Default port warnings (5432 for PostgreSQL, 3306 for MySQL)
///
/// Exercise: Implement all validation checks.
///
/// Hints:
/// - Check TLS mode (None = critical, Prefer = warning, Require = warning, VerifyFull = OK)
/// - Check password is not empty
/// - Check verify_hostname is true
/// - Check CA cert is set for VerifyFull mode
pub fn validate_connection_config(config: &ConnectionConfig) -> SecurityAssessment {
    todo!("Validate database connection security configuration")
}

/// Build a connection string from configuration.
///
/// The connection string format varies by database. We use a generic format:
///   `host=HOST port=PORT dbname=DB user=USER sslmode=SSLMODE`
///
/// Exercise: Build the connection string, NEVER including the password in it.
///
/// Hints:
/// - Format each field as `key=value` separated by spaces
/// - Map TlsMode to sslmode strings: None→"disable", Prefer→"prefer",
///   Require→"require", VerifyFull→"verify-full"
/// - Do NOT include the password in the connection string
pub fn build_connection_string(config: &ConnectionConfig) -> String {
    todo!("Build connection string without password")
}

/// Simulate a TLS handshake and certificate validation.
///
/// Returns Ok(connection_info) if the handshake succeeds, Err if it fails.
///
/// We simulate the following checks:
/// 1. Server presents a certificate
/// 2. Certificate is signed by a trusted CA
/// 3. Certificate hostname matches the connection hostname
/// 4. Certificate is not expired
///
/// Exercise: Implement the simulated handshake.
///
/// Hints:
/// - If tls_mode is None, return Ok (but it's insecure)
/// - If tls_mode is VerifyFull, check that ca_cert is provided
/// - Check that server_hostname matches the expected host
/// - Return a descriptive connection info string
pub fn simulate_tls_handshake(
    config: &ConnectionConfig,
    server_cert_valid: bool,
    server_hostname: &str,
) -> Result<String, String> {
    todo!("Simulate TLS handshake with certificate validation")
}

/// Demonstrate the difference between secure and insecure connections.
///
/// Exercise: Create two configs (one secure, one insecure) and validate both.
///
/// Hints:
/// - Secure: VerifyFull, verify_hostname=true, CA cert set, strong password
/// - Insecure: None, no password, verify_hostname=false
/// - Return (secure_assessment, insecure_assessment)
pub fn demonstrate_connection_security() -> (SecurityAssessment, SecurityAssessment) {
    todo!("Compare secure vs insecure connection configurations")
}

/// Check if a connection string leaks sensitive information.
///
/// Exercise: Scan a connection string for common credential leak patterns.
///
/// Hints:
/// - Check for `password=` or `pwd=` in the string
/// - Check for embedded credentials in URL format (`user:pass@host`)
/// - Return true if sensitive data is found
pub fn leaks_credentials(connection_string: &str) -> bool {
    todo!("Check if connection string contains credentials")
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
