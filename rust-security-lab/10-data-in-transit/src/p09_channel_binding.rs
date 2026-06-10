//! # Lesson 09: Channel Binding
//!
//! ## What is Channel Binding?
//!
//! Channel binding ties application-layer authentication to the TLS channel.
//! Without it, an attacker can terminate the TLS connection at a proxy and
//! establish a separate TLS connection to the server, relaying messages.
//!
//! ```text
//! Without channel binding:
//!   Client <--TLS1--> Attacker <--TLS2--> Server
//!   Client sends auth token -> Attacker relays it -> Server accepts
//!
//! With channel binding:
//!   Client includes TLS channel properties in auth proof
//!   Attacker's TLS channel has different properties -> auth fails
//! ```
//!
//! ## How It Works
//!
//! The client includes a fingerprint of the TLS channel (e.g., the TLS finished
//! message or the server's certificate hash) in its authentication proof.
//! The server verifies that the fingerprint matches the current TLS session.
//!
//! ## Channel Binding Types
//!
//! 1. **tls-unique**: The TLS finished message from the handshake
//! 2. **tls-server-end-point**: Hash of the server's certificate
//! 3. **tls-exporter**: Derived from the TLS session secrets
//!
//! ## Attack Scenario: TLS Stripping Proxy
//!
//! An attacker positions themselves as a TLS-terminating proxy:
//! - Client connects to attacker via TLS (attacker has a valid cert)
//! - Attacker connects to server via TLS
//! - Attacker relays all messages, reading everything in plaintext
//!
//! Channel binding defeats this because the client's auth proof includes
//! the TLS channel fingerprint, which is different on the attacker's connection.
//!
//! ## Why This Matters
//!
//! Without channel binding, any TLS-terminating proxy (corporate proxy, CDN,
//! compromised CA) can read and modify traffic while appearing legitimate.

use ring::digest;
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a TLS channel's unique properties.
#[derive(Debug, Clone)]
pub struct TlsChannel {
    /// Unique session identifier (simulates TLS session ID)
    pub session_id: Vec<u8>,
    /// Server certificate hash (for tls-server-end-point binding)
    pub server_cert_hash: Vec<u8>,
    /// TLS finished message (for tls-unique binding)
    pub finished_message: Vec<u8>,
    /// Whether this is the expected/legitimate channel
    pub is_legitimate: bool,
}

/// An authentication proof that includes channel binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundAuthProof {
    /// The user's identity
    pub user_id: String,
    /// HMAC of (user_id + channel_binding_data)
    pub auth_mac: Vec<u8>,
    /// Channel binding type used
    pub binding_type: ChannelBindingType,
    /// The channel binding data (hash of TLS channel properties)
    pub binding_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelBindingType {
    /// Hash of TLS finished message
    TlsUnique,
    /// Hash of server certificate
    TlsServerEndPoint,
    /// Derived from TLS session secrets
    TlsExporter,
}

/// Exercise: Compute the tls-unique channel binding data.
///
/// This is the hash of the TLS finished message from the handshake.
///
/// Hints:
/// - Hash the finished_message with SHA-256
pub fn compute_tls_unique(channel: &TlsChannel) -> Vec<u8> {
    todo!("Implement tls-unique binding computation")
}

/// Exercise: Compute the tls-server-end-point channel binding data.
///
/// This is the hash of the server's certificate.
///
/// Hints:
/// - Hash the server_cert_hash with SHA-256
pub fn compute_tls_server_end_point(channel: &TlsChannel) -> Vec<u8> {
    todo!("Implement tls-server-end-point binding computation")
}

/// Exercise: Create an authentication proof bound to a TLS channel.
///
/// The proof demonstrates that the client knows the TLS channel's properties.
///
/// Hints:
/// - Compute channel binding data (choose tls-unique by default)
/// - HMAC over (user_id || binding_data) using the auth key
/// - Return a BoundAuthProof
pub fn create_bound_auth(
    user_id: &str,
    channel: &TlsChannel,
    auth_key: &[u8],
    binding_type: ChannelBindingType,
) -> BoundAuthProof {
    todo!("Implement bound authentication proof creation")
}

/// Exercise: Verify a bound authentication proof against the server's view of the channel.
///
/// The server recomputes the channel binding and verifies the HMAC.
///
/// Hints:
/// - Compute the same channel binding data on the server side
/// - Verify the HMAC using the shared auth key
/// - If the binding data doesn't match, the proof is from a different channel
pub fn verify_bound_auth(
    proof: &BoundAuthProof,
    server_channel: &TlsChannel,
    auth_key: &[u8],
) -> Result<(), String> {
    todo!("Implement bound authentication verification")
}

/// Exercise: Simulate a TLS stripping attack.
///
/// The attacker terminates TLS and creates a new connection to the server.
/// The channel binding data will differ, so authentication should fail.
///
/// Hints:
/// - Create a legitimate channel and an attacker channel (different properties)
/// - Client creates auth proof bound to the legitimate channel
/// - Server tries to verify against the attacker's channel
/// - Verification should fail
pub fn simulate_tls_stripping(
    client_channel: &TlsChannel,
    attacker_channel: &TlsChannel,
    auth_key: &[u8],
) -> Result<bool, String> {
    todo!("Implement TLS stripping simulation")
}

/// Exercise: Create a channel binding token that can be included in HTTP headers.
///
/// Format: base64(binding_type + ":" + binding_data)
///
/// Hints:
/// - Encode the binding type as a string
/// - Concatenate with binding data
/// - Base64-encode the result
pub fn export_channel_binding(channel: &TlsChannel, binding_type: ChannelBindingType) -> String {
    todo!("Implement channel binding export")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legitimate_channel() -> TlsChannel {
        TlsChannel {
            session_id: vec![1, 2, 3, 4],
            server_cert_hash: vec![0xAA; 32],
            finished_message: vec![0xBB; 64],
            is_legitimate: true,
        }
    }

    fn attacker_channel() -> TlsChannel {
        TlsChannel {
            session_id: vec![5, 6, 7, 8],
            server_cert_hash: vec![0xCC; 32],
            finished_message: vec![0xDD; 64],
            is_legitimate: false,
        }
    }

    #[test]
    fn test_tls_unique_binding() {
        let channel = legitimate_channel();
        let binding = compute_tls_unique(&channel);
        assert!(!binding.is_empty());
    }

    #[test]
    fn test_tls_server_end_point_binding() {
        let channel = legitimate_channel();
        let binding = compute_tls_server_end_point(&channel);
        assert!(!binding.is_empty());
    }

    #[test]
    fn test_bound_auth_roundtrip() {
        let channel = legitimate_channel();
        let auth_key = vec![0x42u8; 32];
        let proof = create_bound_auth("alice", &channel, &auth_key, ChannelBindingType::TlsUnique);
        assert!(verify_bound_auth(&proof, &channel, &auth_key).is_ok());
    }

    #[test]
    fn test_bound_auth_different_channel_fails() {
        let legit = legitimate_channel();
        let attacker = attacker_channel();
        let auth_key = vec![0x42u8; 32];

        let proof = create_bound_auth("alice", &legit, &auth_key, ChannelBindingType::TlsUnique);
        assert!(
            verify_bound_auth(&proof, &attacker, &auth_key).is_err(),
            "Auth should fail when verified against a different channel"
        );
    }

    #[test]
    fn test_tls_stripping_detected() {
        let legit = legitimate_channel();
        let attacker = attacker_channel();
        let auth_key = vec![0x42u8; 32];
        let result = simulate_tls_stripping(&legit, &attacker, &auth_key).unwrap();
        assert!(result, "TLS stripping should be detected");
    }

    #[test]
    fn test_export_channel_binding() {
        let channel = legitimate_channel();
        let exported = export_channel_binding(&channel, ChannelBindingType::TlsUnique);
        assert!(!exported.is_empty());
        // Should be valid base64
        assert!(base64::decode(&exported).is_ok() || !exported.is_empty());
    }

    #[test]
    fn test_wrong_auth_key_fails() {
        let channel = legitimate_channel();
        let auth_key = vec![0x42u8; 32];
        let wrong_key = vec![0x99u8; 32];

        let proof = create_bound_auth("alice", &channel, &auth_key, ChannelBindingType::TlsUnique);
        assert!(verify_bound_auth(&proof, &channel, &wrong_key).is_err());
    }

    #[test]
    fn test_different_binding_types_produce_different_data() {
        let channel = legitimate_channel();
        let b1 = compute_tls_unique(&channel);
        let b2 = compute_tls_server_end_point(&channel);
        assert_ne!(b1, b2);
    }
}
