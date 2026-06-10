//! # Lesson 08: PQC in TLS — Post-Quantum Transport Layer Security
//!
//! ## PQC in TLS 1.3
//!
//! TLS 1.3 key exchange uses Diffie-Hellman (ECDHE or DHE). To make TLS
//! post-quantum safe, we replace or augment the key exchange with a PQC KEM.
//!
//! ## TLS 1.3 Handshake (Classical)
//! ```
//! Client                              Server
//!   │                                    │
//!   │── ClientHello (supported_groups) ─→│
//!   │←─ ServerHello (key_share) ─────────│
//!   │←─ EncryptedExtensions ─────────────│
//!   │←─ Certificate ─────────────────────│
//!   │←─ CertificateVerify ──────────────│
//!   │←─ Finished ────────────────────────│
//!   │── Finished ───────────────────────→│
//!   │                                    │
//!   │←→ Application Data (encrypted) ←─→│
//! ```
//!
//! ## Hybrid TLS Handshake
//! ```
//! Client                              Server
//!   │                                    │
//!   │── ClientHello + PQC key_share ───→│
//!   │←─ ServerHello + PQC key_share ─────│
//!   │   (Both compute: ECDH_ss || KEM_ss)│
//!   │   ... rest same as TLS 1.3 ...     │
//! ```
//!
//! ## Challenge: Size
//!
//! PQC public keys and ciphertexts are much larger than ECDH.
//! This affects:
//! - Handshake latency (more bytes to transmit)
//! - Certificate chains (PQC signatures are huge)
//! - UDP-based protocols (DTLS, QUIC) with datagram size limits
//!
//! ## Attack: TLS Downgrade with PQC
//!
//! An attacker strips PQC algorithms from the ClientHello, forcing the server
//! to fall back to classical-only. Mitigation: require PQC in policy.

use sha2::{Digest, Sha256};

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

/// Exercise 1: Build a ClientHello with PQC support.
///
/// Create a ClientHello that advertises both classical and PQC key exchange:
/// - Supported groups: ["x25519", "ML-KEM-768", "secp256r1"]
/// - Key shares: generate random key_exchange bytes for each group
///   - x25519: 32 bytes
///   - ML-KEM-768: 1184 bytes (public key size)
///   - secp256r1: 33 bytes (compressed point)
/// - SNI: provided hostname
///
/// Mark ML-KEM-768 as is_pqc = true.
pub fn build_pqc_client_hello(hostname: &str) -> ClientHello {
    todo!("Build ClientHello with PQC key shares")
}

/// Exercise 2: Server selects the best key share from ClientHello.
///
/// The server should prefer PQC algorithms if available.
/// Selection priority:
/// 1. ML-KEM-768 (PQC preferred)
/// 2. x25519 (classical fallback)
/// 3. secp256r1 (classical fallback)
///
/// Return a ServerHello with the selected key share.
/// Generate a matching server key_share (same size random bytes).
pub fn server_select_key_share(ch: &ClientHello) -> ServerHello {
    todo!("Server selects best key share from ClientHello")
}

/// Exercise 3: Compute the TLS shared secret from key shares.
///
/// In a real TLS handshake, both sides compute the shared secret from
/// their private key and the peer's public key.
///
/// For simulation:
/// - Concatenate client and server key_exchange bytes
/// - SHA-256 hash the concatenation
/// - Return 32 bytes
pub fn compute_tls_shared_secret(
    client_ks: &KeyShareEntry,
    server_ks: &KeyShareEntry,
) -> Vec<u8> {
    todo!("Compute TLS shared secret from key shares")
}

/// Exercise 4: Determine if a TLS handshake result uses PQC.
///
/// Return true if the handshake used a PQC algorithm (either standalone or hybrid).
pub fn handshake_uses_pqc(result: &TlsHandshakeResult) -> bool {
    todo!("Check if TLS handshake uses PQC")
}

/// Exercise 5: Detect TLS downgrade attack.
///
/// Given the client's original offered groups and the server's selected group,
/// detect if a downgrade occurred.
///
/// A downgrade is when:
/// - The client offered a PQC group but the server selected a non-PQC group
/// AND
/// - The client did NOT explicitly request non-PQC (we assume it didn't)
///
/// Return true if downgrade detected.
pub fn detect_tls_downgrade(
    client_groups: &[String],
    selected_group: &str,
) -> bool {
    todo!("Detect TLS algorithm downgrade")
}

/// Exercise 6: Compute handshake bandwidth overhead for PQC vs classical.
///
/// Compare the total bytes transmitted during key exchange:
/// - Classical (x25519): client_share(32) + server_share(32) = 64 bytes
/// - PQC (ML-KEM-768): client_share(1184) + server_share(1088) = 2272 bytes
/// - Hybrid: both combined = 64 + 2272 = 2336 bytes
///
/// Return (classical_bytes, pqc_bytes, hybrid_bytes).
pub fn tls_key_exchange_sizes() -> (usize, usize, usize) {
    todo!("Compute TLS key exchange sizes for classical, PQC, and hybrid")
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
