//! # Lesson 01: TLS 1.3 Handshake Concepts (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::hkdf;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};

#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeState {
    Initial,
    ClientHelloSent,
    ServerHelloReceived,
    Connected,
    Failed(String),
}

pub struct TlsSession {
    pub state: HandshakeState,
    pub client_random: [u8; 32],
    pub server_random: [u8; 32],
    pub shared_secret: Vec<u8>,
    pub handshake_transcript: Vec<u8>,
    pub application_key: Option<[u8; 32]>,
}

impl TlsSession {
    pub fn new() -> Self {
        let rng = SystemRandom::new();
        let mut client_random = [0u8; 32];
        rng.fill(&mut client_random).unwrap();

        TlsSession {
            state: HandshakeState::Initial,
            client_random,
            server_random: [0u8; 32],
            shared_secret: Vec::new(),
            handshake_transcript: Vec::new(),
            application_key: None,
        }
    }

    pub fn send_client_hello(&mut self) -> Result<(), String> {
        if self.state != HandshakeState::Initial {
            return Err("Cannot send ClientHello: not in Initial state".to_string());
        }
        self.handshake_transcript.extend_from_slice(b"ClientHello:");
        self.handshake_transcript.extend_from_slice(&self.client_random);
        self.state = HandshakeState::ClientHelloSent;
        Ok(())
    }

    pub fn receive_server_hello(&mut self, server_random: [u8; 32]) -> Result<(), String> {
        if self.state != HandshakeState::ClientHelloSent {
            return Err("Cannot process ServerHello: not in ClientHelloSent state".to_string());
        }
        self.server_random = server_random;
        self.handshake_transcript.extend_from_slice(b"ServerHello:");
        self.handshake_transcript.extend_from_slice(&server_random);

        // Derive shared secret (simplified — real TLS uses ECDHE)
        let mut hasher = digest::Context::new(&digest::SHA256);
        hasher.update(&self.client_random);
        hasher.update(&server_random);
        self.shared_secret = hasher.finish().as_ref().to_vec();

        self.state = HandshakeState::ServerHelloReceived;
        Ok(())
    }

    pub fn derive_application_key(&mut self) -> Result<(), String> {
        if self.state != HandshakeState::ServerHelloReceived {
            return Err("Cannot derive key: not in ServerHelloReceived state".to_string());
        }

        // Use HKDF to derive application key
        let salt = hmac::Key::new(hmac::HMAC_SHA256, &self.handshake_transcript);
        let prk = hmac::sign(&salt, &self.shared_secret);

        let key_material = hkdf::Salt::new(hkdf::HKDF_SHA256, prk.as_ref());
        let okm = key_material.expand(&[b"application key"], hkdf::HKDF_SHA256).unwrap();

        let mut app_key = [0u8; 32];
        okm.fill(&mut app_key).unwrap();
        self.application_key = Some(app_key);

        Ok(())
    }

    pub fn complete_handshake(&mut self) -> Result<(), String> {
        if self.application_key.is_none() {
            return Err("Cannot complete: application key not derived".to_string());
        }
        self.state = HandshakeState::Connected;
        Ok(())
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        if self.state != HandshakeState::Connected {
            return Err("Cannot encrypt: not connected".to_string());
        }
        let key = self.application_key.unwrap();
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key);
        let mac = hmac::sign(&hmac_key, plaintext);

        let mut ciphertext = Vec::with_capacity(4 + plaintext.len() + 32);
        ciphertext.extend_from_slice(&(plaintext.len() as u32).to_be_bytes());
        // XOR-encrypt with HMAC-derived keystream
        for (i, &byte) in plaintext.iter().enumerate() {
            ciphertext.push(byte ^ mac.as_ref()[i % 32]);
        }
        ciphertext.extend_from_slice(mac.as_ref());
        Ok(ciphertext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        if self.state != HandshakeState::Connected {
            return Err("Cannot decrypt: not connected".to_string());
        }
        if ciphertext.len() < 4 + 32 {
            return Err("Ciphertext too short".to_string());
        }

        let key = self.application_key.unwrap();
        let len = u32::from_be_bytes([ciphertext[0], ciphertext[1], ciphertext[2], ciphertext[3]]) as usize;
        let enc_data = &ciphertext[4..4 + len];
        let stored_mac = &ciphertext[4 + len..];

        // Decrypt
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key);
        let mut plaintext = Vec::with_capacity(len);
        for (i, &byte) in enc_data.iter().enumerate() {
            plaintext.push(byte ^ stored_mac[i % 32]);
        }

        // Verify MAC
        let computed_mac = hmac::sign(&hmac_key, &plaintext);
        if ring::constant_time::verify_slices_are_equal(computed_mac.as_ref(), stored_mac).is_err() {
            return Err("MAC verification failed".to_string());
        }

        Ok(plaintext)
    }
}

pub fn verify_transcript(messages: &[&[u8]], expected_hash: &[u8]) -> bool {
    let mut hasher = digest::Context::new(&digest::SHA256);
    for msg in messages {
        hasher.update(msg);
    }
    let computed = hasher.finish();
    ring::constant_time::verify_slices_are_equal(computed.as_ref(), expected_hash).is_ok()
}

pub fn detect_downgrade(original_client_hello: &[u8], modified_client_hello: &[u8]) -> bool {
    let hash_original = digest::digest(&digest::SHA256, original_client_hello);
    let hash_modified = digest::digest(&digest::SHA256, modified_client_hello);
    ring::constant_time::verify_slices_are_equal(hash_original.as_ref(), hash_modified.as_ref()).is_err()
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
        let modified = b"ClientHello: TLS 1.2";
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
