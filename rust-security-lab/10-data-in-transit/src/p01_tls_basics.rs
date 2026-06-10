//! # Lesson 01: TLS 1.3 Handshake Concepts
//!
//! ## What is TLS?
//!
//! TLS (Transport Layer Security) creates a secure channel over an insecure network.
//! It provides confidentiality (encryption), integrity (tamper detection), and
//! authenticity (you know who you are talking to).
//!
//! ## TLS 1.3 Handshake Flow
//!
//! ```text
//! Client                              Server
//!   |                                    |
//!   |--- ClientHello (supported ciphers, key share) -->|
//!   |                                    |
//!   |<-- ServerHello (chosen cipher, key share) ------|
//!   |<-- EncryptedExtensions -----------|
//!   |<-- Certificate (server identity) -|
//!   |<-- CertificateVerify (signature) -|
//!   |<-- Finished (handshake MAC) ------|
//!   |                                    |
//!   |--- Finished (handshake MAC) ----->|
//!   |                                    |
//!   |<========= Encrypted Data ========>|
//! ```
//!
//! ## Key Concepts
//!
//! 1. **Key Exchange**: ECDHE (Elliptic Curve Diffie-Hellman Ephemeral) provides
//!    forward secrecy — even if the server's long-term key is compromised later,
//!    past sessions cannot be decrypted.
//!
//! 2. **Certificate Verification**: The server proves its identity by presenting
//!    a certificate signed by a trusted Certificate Authority (CA).
//!
//! 3. **Encrypted Channel**: After the handshake, all data is encrypted with
//!    AEAD ciphers (AES-256-GCM or ChaCha20-Poly1305).
//!
//! ## Attack Scenario: Downgrade Attack
//!
//! An attacker intercepts the ClientHello and removes TLS 1.3 from the supported
//! versions list, forcing the server to negotiate TLS 1.2 or lower. TLS 1.3
//! prevents this by requiring the server to sign the transcript hash, proving
//! it saw the original ClientHello.
//!
//! ## Why This Matters
//!
//! Without TLS, any network device between client and server can read all data:
//! passwords, tokens, personal information. Even with TLS, misconfiguration
//! (weak ciphers, disabled validation) can break the security model.

use ring::digest;
use ring::hkdf;
use ring::hmac;

/// A simplified TLS 1.3 handshake state machine.
///
/// This models the key exchange phase of a TLS 1.3 handshake using
/// HKDF (HMAC-based Key Derivation Function) to derive session keys.
#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeState {
    /// Initial state — no messages exchanged
    Initial,
    /// ClientHello sent — client has generated key share
    ClientHelloSent,
    /// ServerHello received — shared secret established
    ServerHelloReceived,
    /// Handshake complete — encrypted channel ready
    Connected,
    /// Handshake failed
    Failed(String),
}

/// Simulated TLS session that tracks handshake state and derives keys.
///
/// In a real TLS implementation, this would use ECDHE for key exchange.
/// We simulate it with HKDF for educational purposes.
pub struct TlsSession {
    pub state: HandshakeState,
    pub client_random: [u8; 32],
    pub server_random: [u8; 32],
    pub shared_secret: Vec<u8>,
    pub handshake_transcript: Vec<u8>,
    pub application_key: Option<[u8; 32]>,
}

impl TlsSession {
    /// Create a new TLS session in the Initial state.
    ///
    /// Exercise: Initialize all fields with appropriate default values.
    /// Generate random `client_random` using `ring::rand`.
    pub fn new() -> Self {
        todo!("Create a new TLS session with random client_random")
    }

    /// Simulate sending a ClientHello message.
    ///
    /// In TLS 1.3, the ClientHello includes:
    /// - Supported cipher suites
    /// - Key share (client's ECDHE public value)
    /// - Supported TLS versions
    /// - Random nonce (32 bytes)
    ///
    /// Exercise: Transition to ClientHelloSent state.
    /// Append a representation of the ClientHello to the transcript.
    pub fn send_client_hello(&mut self) -> Result<(), String> {
        todo!("Implement ClientHello sending")
    }

    /// Simulate receiving a ServerHello message.
    ///
    /// The ServerHello includes:
    /// - Chosen cipher suite
    /// - Server's key share
    /// - Server random nonce
    ///
    /// Exercise: Transition to ServerHelloReceived state.
    /// Compute a simulated shared secret from client_random and server_random.
    pub fn receive_server_hello(&mut self, server_random: [u8; 32]) -> Result<(), String> {
        todo!("Implement ServerHello processing")
    }

    /// Derive the application traffic key from the handshake transcript.
    ///
    /// In TLS 1.3, HKDF-Expand-Label derives separate keys for client→server
    /// and server→client traffic. We simplify to a single key.
    ///
    /// Exercise: Use HKDF to derive a 32-byte application key from
    /// the shared_secret and handshake_transcript.
    pub fn derive_application_key(&mut self) -> Result<(), String> {
        todo!("Implement application key derivation using HKDF")
    }

    /// Complete the handshake and transition to Connected state.
    ///
    /// Exercise: Verify that application_key has been derived,
    /// then transition to Connected.
    pub fn complete_handshake(&mut self) -> Result<(), String> {
        todo!("Implement handshake completion")
    }

    /// Encrypt application data using the derived key.
    ///
    /// Uses HMAC-SHA256 as an authenticated encryption construction
    /// (simplified — real TLS uses AES-GCM or ChaCha20-Poly1305).
    ///
    /// Exercise: If connected, encrypt data using HMAC-based encryption.
    /// Return an error if the session is not in Connected state.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        todo!("Implement data encryption")
    }

    /// Decrypt application data using the derived key.
    ///
    /// Exercise: Verify HMAC and decrypt. Return error if tampered or not connected.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        todo!("Implement data decryption")
    }
}

/// Exercise: Implement a function that verifies a TLS handshake transcript
/// has not been tampered with.
///
/// The transcript hash covers all handshake messages in order.
/// If even one byte is modified, the hash changes completely.
///
/// Hints:
/// - Hash all handshake messages together using SHA-256
/// - Compare the computed hash with the expected hash
pub fn verify_transcript(messages: &[&[u8]], expected_hash: &[u8]) -> bool {
    todo!("Implement transcript verification")
}

/// Exercise: Simulate a downgrade attack detection.
///
/// In TLS 1.3, the server signs the transcript hash including the ClientHello.
/// If an attacker modified the ClientHello (e.g., removed TLS 1.3 support),
/// the signature won't match.
///
/// This function simulates detection by comparing two transcript hashes.
///
/// Hints:
/// - Hash the original transcript
/// - Hash the modified transcript (what the attacker forwarded)
/// - If they differ, a downgrade attack was detected
pub fn detect_downgrade(original_client_hello: &[u8], modified_client_hello: &[u8]) -> bool {
    todo!("Implement downgrade attack detection")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session_initial_state() {
        let session = TlsSession::new();
        assert_eq!(session.state, HandshakeState::Initial);
    }

    #[test]
    fn test_client_hello_transitions() {
        let mut session = TlsSession::new();
        assert_eq!(session.state, HandshakeState::Initial);

        session.send_client_hello().unwrap();
        assert_eq!(session.state, HandshakeState::ClientHelloSent);
    }

    #[test]
    fn test_full_handshake() {
        let mut session = TlsSession::new();
        session.send_client_hello().unwrap();
        let server_random = [42u8; 32];
        session.receive_server_hello(server_random).unwrap();
        assert_eq!(session.state, HandshakeState::ServerHelloReceived);

        session.derive_application_key().unwrap();
        session.complete_handshake().unwrap();
        assert_eq!(session.state, HandshakeState::Connected);
    }

    #[test]
    fn test_encrypt_requires_connected() {
        let session = TlsSession::new();
        let result = session.encrypt(b"hello");
        assert!(result.is_err(), "Should fail when not connected");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut session = TlsSession::new();
        session.send_client_hello().unwrap();
        session.receive_server_hello([1u8; 32]).unwrap();
        session.derive_application_key().unwrap();
        session.complete_handshake().unwrap();

        let plaintext = b"secret message";
        let ciphertext = session.encrypt(plaintext).unwrap();
        let decrypted = session.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_detect_downgrade_attack() {
        let original = b"ClientHello: TLS 1.3, TLS 1.2";
        let modified = b"ClientHello: TLS 1.2"; // Attacker removed TLS 1.3
        assert!(detect_downgrade(original, modified));
    }

    #[test]
    fn test_detect_no_downgrade() {
        let original = b"ClientHello: TLS 1.3, TLS 1.2";
        assert!(!detect_downgrade(original, original));
    }

    #[test]
    fn test_transcript_verification() {
        let msg1 = b"ClientHello";
        let msg2 = b"ServerHello";
        let messages: Vec<&[u8]> = vec![msg1, msg2];

        // Compute expected hash
        let mut hasher = digest::Context::new(&digest::SHA256);
        hasher.update(msg1);
        hasher.update(msg2);
        let expected = hasher.finish().as_ref().to_vec();

        assert!(verify_transcript(&messages, &expected));
    }

    #[test]
    fn test_transcript_tamper_detected() {
        let msg1 = b"ClientHello";
        let msg2 = b"ServerHello";
        let messages: Vec<&[u8]> = vec![msg1, msg2];
        let wrong_hash = vec![0u8; 32];
        assert!(!verify_transcript(&messages, &wrong_hash));
    }
}
