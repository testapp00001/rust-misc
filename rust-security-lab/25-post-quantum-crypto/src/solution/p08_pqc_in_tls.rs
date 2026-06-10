//! # Lesson 08: PQC in TLS — Post-Quantum Transport Layer Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use rand::Rng;

/// A TLS supported_group / key_share entry.
#[derive(Debug, Clone)]
pub struct KeyShareEntry {
    pub group: String,
    pub key_exchange: Vec<u8>,
    pub is_pqc: bool,
}

/// A TLS ClientHello message (simplified).
#[derive(Debug, Clone)]
pub struct ClientHello {
    pub supported_groups: Vec<String>,
    pub key_shares: Vec<KeyShareEntry>,
    pub sni: String,
}

/// A TLS ServerHello message (simplified).
#[derive(Debug, Clone)]
pub struct ServerHello {
    pub selected_group: String,
    pub key_share: KeyShareEntry,
}

/// The result of a TLS handshake.
#[derive(Debug, Clone)]
pub struct TlsHandshakeResult {
    pub shared_secret: Vec<u8>,
    pub is_pqc: bool,
    pub is_hybrid: bool,
    pub algorithm: String,
}

/// Build a ClientHello with PQC key shares.
pub fn build_pqc_client_hello(hostname: &str) -> ClientHello {
    let mut rng = rand::thread_rng();

    let groups = vec![
        ("x25519", 32, false),
        ("ML-KEM-768", 1184, true),
        ("secp256r1", 33, false),
    ];

    let key_shares: Vec<KeyShareEntry> = groups
        .iter()
        .map(|(name, size, pqc)| KeyShareEntry {
            group: name.to_string(),
            key_exchange: (0..*size).map(|_| rng.gen()).collect(),
            is_pqc: *pqc,
        })
        .collect();

    let supported_groups = key_shares.iter().map(|ks| ks.group.clone()).collect();

    ClientHello {
        supported_groups,
        key_shares,
        sni: hostname.to_string(),
    }
}

/// Server selects the best key share (preferring PQC).
pub fn server_select_key_share(ch: &ClientHello) -> ServerHello {
    let mut rng = rand::thread_rng();

    // Priority order: PQC first, then classical
    let priority = ["ML-KEM-768", "x25519", "secp256r1"];

    let selected = priority
        .iter()
        .find(|g| ch.key_shares.iter().any(|ks| ks.group == **g))
        .expect("No supported group found");

    let size = match *selected {
        "ML-KEM-768" => 1088, // ciphertext size
        "x25519" => 32,
        "secp256r1" => 33,
        _ => 32,
    };

    let is_pqc = *selected == "ML-KEM-768";

    ServerHello {
        selected_group: selected.to_string(),
        key_share: KeyShareEntry {
            group: selected.to_string(),
            key_exchange: (0..size).map(|_| rng.gen()).collect(),
            is_pqc,
        },
    }
}

/// Compute TLS shared secret from key shares.
pub fn compute_tls_shared_secret(
    client_ks: &KeyShareEntry,
    server_ks: &KeyShareEntry,
) -> Vec<u8> {
    let mut combined = Vec::new();
    combined.extend_from_slice(&client_ks.key_exchange);
    combined.extend_from_slice(&server_ks.key_exchange);
    Sha256::digest(&combined).to_vec()
}

/// Check if a TLS handshake result uses PQC.
pub fn handshake_uses_pqc(result: &TlsHandshakeResult) -> bool {
    result.is_pqc || result.is_hybrid
}

/// Detect TLS downgrade: client offered PQC but server didn't select it.
pub fn detect_tls_downgrade(
    client_groups: &[String],
    selected_group: &str,
) -> bool {
    let pqc_groups = ["ML-KEM-768", "ML-KEM-1024", "Kyber768", "Kyber1024"];

    let offered_pqc = client_groups.iter().any(|g| pqc_groups.contains(&g.as_str()));
    let selected_pqc = pqc_groups.contains(&selected_group);

    offered_pqc && !selected_pqc
}

/// Compute TLS key exchange sizes: (classical, pqc, hybrid).
pub fn tls_key_exchange_sizes() -> (usize, usize, usize) {
    let classical = 32 + 32; // x25519 client + server
    let pqc = 1184 + 1088; // ML-KEM-768 client pk + server ct
    let hybrid = classical + pqc;
    (classical, pqc, hybrid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_client_hello_groups() {
        let ch = build_pqc_client_hello("example.com");
        assert_eq!(ch.supported_groups.len(), 3);
        assert!(ch.supported_groups.contains(&"ML-KEM-768".to_string()));
        assert_eq!(ch.sni, "example.com");
    }

    #[test]
    fn test_build_client_hello_key_share_sizes() {
        let ch = build_pqc_client_hello("example.com");
        for ks in &ch.key_shares {
            match ks.group.as_str() {
                "x25519" => assert_eq!(ks.key_exchange.len(), 32),
                "ML-KEM-768" => assert_eq!(ks.key_exchange.len(), 1184),
                "secp256r1" => assert_eq!(ks.key_exchange.len(), 33),
                _ => {}
            }
        }
    }

    #[test]
    fn test_server_prefers_pqc() {
        let ch = build_pqc_client_hello("example.com");
        let sh = server_select_key_share(&ch);
        assert_eq!(sh.selected_group, "ML-KEM-768");
    }

    #[test]
    fn test_server_fallback_classical() {
        let ch = ClientHello {
            supported_groups: vec!["x25519".to_string()],
            key_shares: vec![KeyShareEntry {
                group: "x25519".to_string(),
                key_exchange: vec![0u8; 32],
                is_pqc: false,
            }],
            sni: "example.com".to_string(),
        };
        let sh = server_select_key_share(&ch);
        assert_eq!(sh.selected_group, "x25519");
    }

    #[test]
    fn test_shared_secret_deterministic() {
        let ks1 = KeyShareEntry { group: "test".to_string(), key_exchange: vec![1, 2, 3], is_pqc: false };
        let ks2 = KeyShareEntry { group: "test".to_string(), key_exchange: vec![4, 5, 6], is_pqc: false };
        let ss1 = compute_tls_shared_secret(&ks1, &ks2);
        let ss2 = compute_tls_shared_secret(&ks1, &ks2);
        assert_eq!(ss1, ss2);
        assert_eq!(ss1.len(), 32);
    }

    #[test]
    fn test_handshake_uses_pqc() {
        let result = TlsHandshakeResult {
            shared_secret: vec![0u8; 32],
            is_pqc: true,
            is_hybrid: false,
            algorithm: "ML-KEM-768".to_string(),
        };
        assert!(handshake_uses_pqc(&result));
    }

    #[test]
    fn test_handshake_no_pqc() {
        let result = TlsHandshakeResult {
            shared_secret: vec![0u8; 32],
            is_pqc: false,
            is_hybrid: false,
            algorithm: "x25519".to_string(),
        };
        assert!(!handshake_uses_pqc(&result));
    }

    #[test]
    fn test_detect_downgrade() {
        let groups = vec!["x25519".to_string(), "ML-KEM-768".to_string()];
        assert!(detect_tls_downgrade(&groups, "x25519"));
    }

    #[test]
    fn test_no_downgrade() {
        let groups = vec!["x25519".to_string(), "ML-KEM-768".to_string()];
        assert!(!detect_tls_downgrade(&groups, "ML-KEM-768"));
    }

    #[test]
    fn test_key_exchange_sizes() {
        let (classical, pqc, hybrid) = tls_key_exchange_sizes();
        assert_eq!(classical, 64);
        assert_eq!(pqc, 2272);
        assert_eq!(hybrid, 2336);
    }
}
