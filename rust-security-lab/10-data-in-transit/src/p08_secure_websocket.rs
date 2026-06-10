//! # Lesson 08: Secure WebSocket (wss://)
//!
//! ## What is WebSocket?
//!
//! WebSocket provides a persistent, full-duplex communication channel over a
//! single TCP connection. Unlike HTTP's request-response model, both client
//! and server can send messages at any time.
//!
//! ## Secure WebSocket (wss://)
//!
//! `wss://` is WebSocket over TLS. It is to `ws://` what HTTPS is to HTTP.
//! The TLS handshake occurs before the WebSocket handshake, so the entire
//! connection is encrypted.
//!
//! ```text
//! [TLS Handshake] -> [WebSocket Upgrade] -> [Encrypted Messages]
//! ```
//!
//! ## WebSocket Frame Format
//!
//! ```text
//!  0                   1                   2                   3
//!  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-------+-+-------------+-------------------------------+
//! |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
//! |I|S|S|S|  (4)  |A|     (7)     |           (16/64)             |
//! |N|V|V|V|       |S|             |   (if payload len==126/127)   |
//! | |1|2|3|       |K|             |                               |
//! +-+-+-+-+-------+-+-------------+-------------------------------+
//! ```
//!
//! ## Attack: Unencrypted WebSocket Data
//!
//! Using `ws://` instead of `wss://` exposes all WebSocket messages to
//! network eavesdroppers. Many developers forget that WebSocket needs TLS too.
//!
//! ## Why This Matters
//!
//! Real-time applications (chat, trading, gaming) use WebSockets heavily.
//! Without TLS, every message is visible on the network.

use ring::digest;
use serde::{Deserialize, Serialize};

/// WebSocket opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Opcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

/// A WebSocket frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketFrame {
    pub fin: bool,
    pub opcode: Opcode,
    pub masked: bool,
    pub mask_key: Option<[u8; 4]>,
    pub payload: Vec<u8>,
}

/// Exercise: Encode a WebSocket frame to bytes.
///
/// Format:
/// - Byte 0: FIN (1 bit) + RSV (3 bits, all 0) + Opcode (4 bits)
/// - Byte 1: MASK (1 bit) + Payload length (7 bits)
///   - If length <= 125: stored directly
///   - If length <= 65535: 126 + 2 bytes extended length
///   - If length > 65535: 127 + 8 bytes extended length
/// - If masked: 4 bytes mask key
/// - Payload (XOR with mask key if masked)
///
/// Hints:
/// - Use bitwise operations to set FIN and opcode
/// - Handle the three length cases
/// - If masked, XOR each payload byte with the mask key (cycling)
pub fn encode_frame(frame: &WebSocketFrame) -> Vec<u8> {
    todo!("Implement WebSocket frame encoding")
}

/// Exercise: Decode a WebSocket frame from bytes.
///
/// Hints:
/// - Parse byte 0: FIN = bit 7, opcode = bits 0-3
/// - Parse byte 1: MASK = bit 7, length = bits 0-6
/// - Handle extended length (126 -> 2 bytes, 127 -> 8 bytes)
/// - If masked, read 4-byte mask key and unmask payload
pub fn decode_frame(data: &[u8]) -> Result<WebSocketFrame, String> {
    todo!("Implement WebSocket frame decoding")
}

/// Exercise: Simulate a TLS-protected WebSocket connection.
///
/// This models the wss:// flow: TLS handshake first, then WebSocket upgrade
/// over the encrypted channel.
pub struct SecureWebSocket {
    /// Whether TLS has been established
    pub tls_established: bool,
    /// Whether WebSocket upgrade has completed
    pub ws_upgraded: bool,
    /// Session key derived from TLS handshake
    pub session_key: Option<[u8; 32]>,
    /// Messages sent over the connection
    pub messages: Vec<Vec<u8>>,
}

impl SecureWebSocket {
    /// Create a new secure WebSocket connection (client side).
    pub fn new() -> Self {
        todo!("Create a new SecureWebSocket")
    }

    /// Perform the TLS handshake (simulated).
    ///
    /// Exercise: Generate a session key and set tls_established = true.
    /// The session key should be derived from a random value.
    pub fn tls_handshake(&mut self) -> Result<(), String> {
        todo!("Implement simulated TLS handshake")
    }

    /// Perform the WebSocket upgrade over the TLS channel.
    ///
    /// Exercise: Verify TLS is established, then set ws_upgraded = true.
    pub fn ws_upgrade(&mut self) -> Result<(), String> {
        todo!("Implement WebSocket upgrade")
    }

    /// Send a message over the secure WebSocket.
    ///
    /// Exercise: Verify both TLS and WebSocket are ready.
    /// Create a text frame and store it.
    pub fn send_message(&mut self, message: &str) -> Result<(), String> {
        todo!("Implement message sending")
    }

    /// Receive a message (simulated — just check state and return stored messages).
    pub fn receive_messages(&self) -> Result<&Vec<Vec<u8>>, String> {
        todo!("Implement message receiving")
    }

    /// Close the connection with a Close frame.
    pub fn close(&mut self) -> Result<(), String> {
        todo!("Implement connection close")
    }
}

/// Exercise: Encrypt a WebSocket payload using the session key.
///
/// Uses HMAC-based encryption (simplified — real wss uses TLS encryption).
///
/// Hints:
/// - XOR payload bytes with HMAC-derived keystream
pub fn encrypt_payload(payload: &[u8], session_key: &[u8]) -> Vec<u8> {
    todo!("Implement payload encryption")
}

/// Exercise: Decrypt a WebSocket payload.
///
/// Hints:
/// - XOR is its own inverse, so decryption is the same as encryption
pub fn decrypt_payload(ciphertext: &[u8], session_key: &[u8]) -> Vec<u8> {
    todo!("Implement payload decryption")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_frame(payload: &str) -> WebSocketFrame {
        WebSocketFrame {
            fin: true,
            opcode: Opcode::Text,
            masked: false,
            mask_key: None,
            payload: payload.as_bytes().to_vec(),
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let frame = text_frame("Hello, WebSocket!");
        let encoded = encode_frame(&frame);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.fin, true);
        assert_eq!(decoded.opcode, Opcode::Text);
        assert_eq!(decoded.payload, b"Hello, WebSocket!");
    }

    #[test]
    fn test_encode_decode_masked() {
        let frame = WebSocketFrame {
            fin: true,
            opcode: Opcode::Text,
            masked: true,
            mask_key: Some([0x12, 0x34, 0x56, 0x78]),
            payload: b"masked message".to_vec(),
        };
        let encoded = encode_frame(&frame);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.payload, b"masked message");
        assert!(decoded.masked);
    }

    #[test]
    fn test_extended_length_126() {
        let payload = vec![0xAAu8; 200];
        let frame = WebSocketFrame {
            fin: true,
            opcode: Opcode::Binary,
            masked: false,
            mask_key: None,
            payload,
        };
        let encoded = encode_frame(&frame);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.payload.len(), 200);
    }

    #[test]
    fn test_close_frame() {
        let frame = WebSocketFrame {
            fin: true,
            opcode: Opcode::Close,
            masked: false,
            mask_key: None,
            payload: vec![],
        };
        let encoded = encode_frame(&frame);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.opcode, Opcode::Close);
    }

    #[test]
    fn test_ping_pong() {
        let ping = WebSocketFrame {
            fin: true,
            opcode: Opcode::Ping,
            masked: false,
            mask_key: None,
            payload: b"ping data".to_vec(),
        };
        let encoded = encode_frame(&ping);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded.opcode, Opcode::Ping);
        assert_eq!(decoded.payload, b"ping data");
    }

    #[test]
    fn test_secure_ws_full_flow() {
        let mut ws = SecureWebSocket::new();
        assert!(!ws.tls_established);

        ws.tls_handshake().unwrap();
        assert!(ws.tls_established);

        ws.ws_upgrade().unwrap();
        assert!(ws.ws_upgraded);

        ws.send_message("Hello!").unwrap();
        assert_eq!(ws.messages.len(), 1);
    }

    #[test]
    fn test_secure_ws_rejects_without_tls() {
        let mut ws = SecureWebSocket::new();
        assert!(ws.ws_upgrade().is_err(), "Should fail without TLS");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = vec![0x42u8; 32];
        let plaintext = b"secret websocket message";
        let ciphertext = encrypt_payload(plaintext, &key);
        let decrypted = decrypt_payload(&ciphertext, &key);
        assert_eq!(decrypted, plaintext);
    }
}
