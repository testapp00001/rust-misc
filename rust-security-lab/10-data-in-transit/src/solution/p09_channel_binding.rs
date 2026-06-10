//! # Lesson 09: Channel Binding (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TlsChannel {
    pub session_id: Vec<u8>,
    pub server_cert_hash: Vec<u8>,
    pub finished_message: Vec<u8>,
    pub is_legitimate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundAuthProof {
    pub user_id: String,
    pub auth_mac: Vec<u8>,
    pub binding_type: ChannelBindingType,
    pub binding_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelBindingType {
    TlsUnique,
    TlsServerEndPoint,
    TlsExporter,
}

pub fn compute_tls_unique(channel: &TlsChannel) -> Vec<u8> {
    digest::digest(&digest::SHA256, &channel.finished_message)
        .as_ref()
        .to_vec()
}

pub fn compute_tls_server_end_point(channel: &TlsChannel) -> Vec<u8> {
    digest::digest(&digest::SHA256, &channel.server_cert_hash)
        .as_ref()
        .to_vec()
}

pub fn create_bound_auth(
    user_id: &str,
    channel: &TlsChannel,
    auth_key: &[u8],
    binding_type: ChannelBindingType,
) -> BoundAuthProof {
    let binding_data = match binding_type {
        ChannelBindingType::TlsUnique => compute_tls_unique(channel),
        ChannelBindingType::TlsServerEndPoint => compute_tls_server_end_point(channel),
        ChannelBindingType::TlsExporter => {
            // Simulate exporter: hash of session_id
            digest::digest(&digest::SHA256, &channel.session_id)
                .as_ref()
                .to_vec()
        }
    };

    // HMAC over user_id + binding_data
    let key = hmac::Key::new(hmac::HMAC_SHA256, auth_key);
    let mut mac_data = Vec::new();
    mac_data.extend_from_slice(user_id.as_bytes());
    mac_data.extend_from_slice(&binding_data);
    let auth_mac = hmac::sign(&key, &mac_data).as_ref().to_vec();

    BoundAuthProof {
        user_id: user_id.to_string(),
        auth_mac,
        binding_type,
        binding_data,
    }
}

pub fn verify_bound_auth(
    proof: &BoundAuthProof,
    server_channel: &TlsChannel,
    auth_key: &[u8],
) -> Result<(), String> {
    // Recompute binding data on the server side
    let server_binding_data = match proof.binding_type {
        ChannelBindingType::TlsUnique => compute_tls_unique(server_channel),
        ChannelBindingType::TlsServerEndPoint => compute_tls_server_end_point(server_channel),
        ChannelBindingType::TlsExporter => {
            digest::digest(&digest::SHA256, &server_channel.session_id)
                .as_ref()
                .to_vec()
        }
    };

    // Verify binding data matches
    if proof.binding_data != server_binding_data {
        return Err("Channel binding data mismatch — possible MITM".to_string());
    }

    // Verify HMAC
    let key = hmac::Key::new(hmac::HMAC_SHA256, auth_key);
    let mut mac_data = Vec::new();
    mac_data.extend_from_slice(proof.user_id.as_bytes());
    mac_data.extend_from_slice(&server_binding_data);
    let computed_mac = hmac::sign(&key, &mac_data);

    ring::constant_time::verify_slices_are_equal(computed_mac.as_ref(), &proof.auth_mac)
        .map_err(|_| "Auth MAC verification failed".to_string())
}

pub fn simulate_tls_stripping(
    client_channel: &TlsChannel,
    attacker_channel: &TlsChannel,
    auth_key: &[u8],
) -> Result<bool, String> {
    // Client creates proof bound to the legitimate channel
    let proof = create_bound_auth("alice", client_channel, auth_key, ChannelBindingType::TlsUnique);

    // Server tries to verify against the attacker's channel
    let result = verify_bound_auth(&proof, attacker_channel, auth_key);

    // Should fail — binding data differs
    Ok(result.is_err())
}

pub fn export_channel_binding(channel: &TlsChannel, binding_type: ChannelBindingType) -> String {
    let binding_data = match binding_type {
        ChannelBindingType::TlsUnique => compute_tls_unique(channel),
        ChannelBindingType::TlsServerEndPoint => compute_tls_server_end_point(channel),
        ChannelBindingType::TlsExporter => {
            digest::digest(&digest::SHA256, &channel.session_id)
                .as_ref()
                .to_vec()
        }
    };

    let type_str = match binding_type {
        ChannelBindingType::TlsUnique => "tls-unique",
        ChannelBindingType::TlsServerEndPoint => "tls-server-end-point",
        ChannelBindingType::TlsExporter => "tls-exporter",
    };

    let mut combined = Vec::new();
    combined.extend_from_slice(type_str.as_bytes());
    combined.push(b':');
    combined.extend_from_slice(&binding_data);

    base64::engine::general_purpose::STANDARD.encode(&combined)
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
        assert!(base64::engine::general_purpose::STANDARD.decode(&exported).is_ok());
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
