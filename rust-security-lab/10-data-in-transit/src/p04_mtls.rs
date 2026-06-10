//! # Lesson 04: Mutual TLS (mTLS)
//!
//! ## What is mTLS?
//!
//! In standard TLS, only the server presents a certificate. The client verifies
//! the server's identity, but the server has no cryptographic proof of the
//! client's identity. Mutual TLS (mTLS) requires both sides to present certificates.
//!
//! ```text
//! Standard TLS:                    Mutual TLS:
//! Client verifies Server           Client verifies Server
//! Server trusts Client (token?)    Server verifies Client cert
//! ```
//!
//! ## mTLS Handshake
//!
//! ```text
//! Client                              Server
//!   |--- ClientHello ----------------->|
//!   |<-- ServerHello -----------------|
//!   |<-- Server Certificate ----------|
//!   |<-- CertificateRequest ----------|  <-- NEW in mTLS
//!   |--- Client Certificate --------->|  <-- NEW in mTLS
//!   |--- CertificateVerify ---------->|  <-- Client proves key ownership
//!   |--- Finished ------------------->|
//!   |<-- Finished --------------------|
//!   |<======== Encrypted Data ========>|
//! ```
//!
//! ## Use Cases
//!
//! - **Service-to-service**: Microservices authenticate each other
//! - **API security**: Clients present certificates instead of API keys
//! - **Zero trust networks**: Every connection is authenticated
//! - **IoT devices**: Devices have embedded client certificates
//!
//! ## Attack Scenario: No Client Authentication
//!
//! Without mTLS, a server must rely on bearer tokens (JWT, API keys) for client
//! identity. These can be stolen, leaked, or replayed. With mTLS, the client
//! proves identity through a certificate — stealing the TLS session doesn't
//! help because the attacker doesn't have the client's private key.
//!
//! ## Why This Matters
//!
//! mTLS eliminates entire classes of attacks: stolen API keys, leaked tokens,
//! and credential stuffing. It is the gold standard for service-to-service auth.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Represents a TLS certificate with a key pair.
#[derive(Debug, Clone)]
pub struct TlsIdentity {
    pub common_name: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub issuer: String,
    pub is_ca: bool,
}

/// Simplified mTLS connection state.
#[derive(Debug, Clone, PartialEq)]
pub enum MtlsState {
    Initial,
    ServerHelloSent,
    ClientCertSent,
    ServerVerified,
    ClientVerified,
    Connected,
    Failed(String),
}

/// An mTLS connection that authenticates both sides.
pub struct MtlsConnection {
    pub state: MtlsState,
    pub server_identity: TlsIdentity,
    pub client_identity: Option<TlsIdentity>,
    pub trusted_cas: Vec<TlsIdentity>,
    pub transcript: Vec<u8>,
}

impl MtlsConnection {
    /// Create a new mTLS connection from the server's perspective.
    ///
    /// Exercise: Initialize with server identity and trusted CAs.
    /// The client identity starts as None (not yet received).
    pub fn new_server(server_identity: TlsIdentity, trusted_cas: Vec<TlsIdentity>) -> Self {
        todo!("Create server-side mTLS connection")
    }

    /// Create a new mTLS connection from the client's perspective.
    ///
    /// Exercise: Initialize with client identity and trusted CAs.
    pub fn new_client(client_identity: TlsIdentity, trusted_cas: Vec<TlsIdentity>) -> Self {
        todo!("Create client-side mTLS connection")
    }

    /// Server sends its certificate to the client.
    ///
    /// Exercise: Serialize the server identity and append to transcript.
    /// Transition to ServerHelloSent state.
    pub fn server_send_certificate(&mut self) -> Vec<u8> {
        todo!("Implement server certificate sending")
    }

    /// Server requests a client certificate (CertificateRequest message).
    ///
    /// Exercise: Generate a CertificateRequest message listing acceptable CAs.
    pub fn server_request_client_cert(&self) -> Vec<u8> {
        todo!("Implement CertificateRequest generation")
    }

    /// Client receives and verifies the server's certificate.
    ///
    /// Exercise: Verify the server cert is issued by a trusted CA.
    /// Transition to ServerVerified on success, Failed on error.
    pub fn client_verify_server(&mut self, server_cert: &[u8]) -> Result<(), String> {
        todo!("Implement client-side server verification")
    }

    /// Client sends its certificate in response to CertificateRequest.
    ///
    /// Exercise: Serialize client identity and append to transcript.
    /// Transition to ClientCertSent state.
    pub fn client_send_certificate(&self) -> Vec<u8> {
        todo!("Implement client certificate sending")
    }

    /// Server verifies the client's certificate.
    ///
    /// Exercise: Verify the client cert is issued by a trusted CA.
    /// Transition to ClientVerified on success.
    pub fn server_verify_client(&mut self, client_cert: &[u8]) -> Result<(), String> {
        todo!("Implement server-side client verification")
    }

    /// Complete the handshake — both sides must be verified.
    ///
    /// Exercise: Check that both server and client are verified.
    /// Transition to Connected state.
    pub fn complete_handshake(&mut self) -> Result<(), String> {
        todo!("Implement mTLS handshake completion")
    }

    /// Check if the connection is fully established.
    pub fn is_connected(&self) -> bool {
        self.state == MtlsState::Connected
    }
}

/// Exercise: Create a simple CA that can issue identities.
///
/// Hints:
/// - Generate a key pair for the CA
/// - Set is_ca = true
/// - Self-sign (issuer = own CN)
pub fn create_ca(common_name: &str) -> TlsIdentity {
    todo!("Implement CA creation")
}

/// Exercise: Issue a certificate (identity) signed by a CA.
///
/// Hints:
/// - Use the CA's private key to sign the new identity's public key
/// - Set issuer to the CA's common name
/// - Set is_ca = false for end-entity certs
pub fn issue_certificate(ca: &TlsIdentity, common_name: &str, public_key: Vec<u8>) -> TlsIdentity {
    todo!("Implement certificate issuance")
}

/// Exercise: Verify that an identity's certificate chain leads to a trusted CA.
///
/// Hints:
/// - Check that the issuer matches a trusted CA's common name
/// - Verify the signature using the CA's public key
pub fn verify_identity(identity: &TlsIdentity, trusted_cas: &[TlsIdentity]) -> bool {
    todo!("Implement identity verification")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity(cn: &str, key: u8, issuer: &str, is_ca: bool) -> TlsIdentity {
        TlsIdentity {
            common_name: cn.to_string(),
            public_key: vec![key; 32],
            private_key: vec![key + 100; 32],
            issuer: issuer.to_string(),
            is_ca,
        }
    }

    #[test]
    fn test_create_ca() {
        let ca = create_ca("Test CA");
        assert!(ca.is_ca);
        assert_eq!(ca.common_name, "Test CA");
    }

    #[test]
    fn test_issue_certificate() {
        let ca = create_ca("Test CA");
        let cert = issue_certificate(&ca, "server.example.com", vec![42u8; 32]);
        assert!(!cert.is_ca);
        assert_eq!(cert.issuer, "Test CA");
    }

    #[test]
    fn test_mtls_server_init() {
        let server = make_identity("server.com", 1, "CA", false);
        let ca = make_identity("CA", 2, "CA", true);
        let conn = MtlsConnection::new_server(server.clone(), vec![ca]);
        assert_eq!(conn.state, MtlsState::Initial);
        assert!(conn.client_identity.is_none());
    }

    #[test]
    fn test_mtls_full_handshake() {
        let ca = create_ca("Test CA");
        let server_id = issue_certificate(&ca, "server.com", vec![1u8; 32]);
        let client_id = issue_certificate(&ca, "client.com", vec![2u8; 32]);

        let mut server_conn = MtlsConnection::new_server(server_id.clone(), vec![ca.clone()]);
        let mut client_conn = MtlsConnection::new_client(client_id.clone(), vec![ca.clone()]);

        // Server sends certificate
        let server_cert = server_conn.server_send_certificate();

        // Client verifies server
        client_conn.client_verify_server(&server_cert).unwrap();

        // Client sends certificate
        let client_cert = client_conn.client_send_certificate();

        // Server verifies client
        server_conn.server_verify_client(&client_cert).unwrap();

        // Both complete handshake
        server_conn.complete_handshake().unwrap();
        client_conn.complete_handshake().unwrap();

        assert!(server_conn.is_connected());
        assert!(client_conn.is_connected());
    }

    #[test]
    fn test_mtls_rejects_untrusted_client() {
        let ca = create_ca("Trusted CA");
        let rogue_ca = create_ca("Rogue CA");
        let server_id = issue_certificate(&ca, "server.com", vec![1u8; 32]);
        let rogue_client = issue_certificate(&rogue_ca, "evil.com", vec![2u8; 32]);

        let mut server_conn = MtlsConnection::new_server(server_id, vec![ca]);
        let client_cert_bytes = serde_json::to_vec(&serde_json::json!({
            "cn": rogue_client.common_name,
            "public_key": rogue_client.public_key,
            "issuer": rogue_client.issuer,
        }))
        .unwrap();

        let result = server_conn.server_verify_client(&client_cert_bytes);
        assert!(result.is_err(), "Should reject client from untrusted CA");
    }

    #[test]
    fn test_verify_identity_valid() {
        let ca = create_ca("Test CA");
        let cert = issue_certificate(&ca, "server.com", vec![42u8; 32]);
        assert!(verify_identity(&cert, &[ca]));
    }

    #[test]
    fn test_verify_identity_untrusted() {
        let ca = create_ca("Test CA");
        let rogue_ca = create_ca("Rogue CA");
        let cert = issue_certificate(&rogue_ca, "evil.com", vec![42u8; 32]);
        assert!(!verify_identity(&cert, &[ca]));
    }

    #[test]
    fn test_mtls_cannot_connect_without_both_verified() {
        let ca = create_ca("Test CA");
        let server_id = issue_certificate(&ca, "server.com", vec![1u8; 32]);
        let mut server_conn = MtlsConnection::new_server(server_id, vec![ca]);
        let result = server_conn.complete_handshake();
        assert!(result.is_err(), "Cannot complete without client verification");
    }
}
