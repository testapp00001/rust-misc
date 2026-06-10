//! # Lesson 08: Secure WebSocket (wss://) (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Opcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl Opcode {
    fn from_u8(val: u8) -> Result<Self, String> {
        match val {
            0x0 => Ok(Opcode::Continuation),
            0x1 => Ok(Opcode::Text),
            0x2 => Ok(Opcode::Binary),
            0x8 => Ok(Opcode::Close),
            0x9 => Ok(Opcode::Ping),
            0xA => Ok(Opcode::Pong),
            _ => Err(format!("Unknown opcode: {}", val)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketFrame {
    pub fin: bool,
    pub opcode: Opcode,
    pub masked: bool,
    pub mask_key: Option<[u8; 4]>,
    pub payload: Vec<u8>,
}

pub fn encode_frame(frame: &WebSocketFrame) -> Vec<u8> {
    let mut out = Vec::new();

    // Byte 0: FIN + opcode
    let byte0 = if frame.fin { 0x80 } else { 0x00 } | (frame.opcode as u8);
    out.push(byte0);

    // Byte 1+: MASK bit + payload length
    let mask_bit: u8 = if frame.masked { 0x80 } else { 0x00 };
    let len = frame.payload.len();

    if len <= 125 {
        out.push(mask_bit | len as u8);
    } else if len <= 65535 {
        out.push(mask_bit | 126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(mask_bit | 127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }

    // Mask key
    if let Some(key) = &frame.mask_key {
        out.extend_from_slice(key);
    }

    // Payload (XOR with mask if masked)
    if let Some(key) = &frame.mask_key {
        for (i, &byte) in frame.payload.iter().enumerate() {
            out.push(byte ^ key[i % 4]);
        }
    } else {
        out.extend_from_slice(&frame.payload);
    }

    out
}

pub fn decode_frame(data: &[u8]) -> Result<WebSocketFrame, String> {
    if data.len() < 2 {
        return Err("Frame too short".to_string());
    }

    let fin = (data[0] & 0x80) != 0;
    let opcode = Opcode::from_u8(data[0] & 0x0F)?;
    let masked = (data[1] & 0x80) != 0;
    let mut payload_len = (data[1] & 0x7F) as usize;
    let mut offset = 2;

    if payload_len == 126 {
        if data.len() < offset + 2 {
            return Err("Frame too short for extended length".to_string());
        }
        payload_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
    } else if payload_len == 127 {
        if data.len() < offset + 8 {
            return Err("Frame too short for extended length".to_string());
        }
        payload_len = u64::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;
        offset += 8;
    }

    let mask_key = if masked {
        if data.len() < offset + 4 {
            return Err("Frame too short for mask key".to_string());
        }
        let key = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
        offset += 4;
        Some(key)
    } else {
        None
    };

    if data.len() < offset + payload_len {
        return Err("Frame too short for payload".to_string());
    }

    let mut payload = data[offset..offset + payload_len].to_vec();

    // Unmask
    if let Some(key) = &mask_key {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= key[i % 4];
        }
    }

    Ok(WebSocketFrame {
        fin,
        opcode,
        masked,
        mask_key,
        payload,
    })
}

pub struct SecureWebSocket {
    pub tls_established: bool,
    pub ws_upgraded: bool,
    pub session_key: Option<[u8; 32]>,
    pub messages: Vec<Vec<u8>>,
}

impl SecureWebSocket {
    pub fn new() -> Self {
        SecureWebSocket {
            tls_established: false,
            ws_upgraded: false,
            session_key: None,
            messages: Vec::new(),
        }
    }

    pub fn tls_handshake(&mut self) -> Result<(), String> {
        let rng = SystemRandom::new();
        let mut session_key = [0u8; 32];
        rng.fill(&mut session_key).unwrap();
        self.session_key = Some(session_key);
        self.tls_established = true;
        Ok(())
    }

    pub fn ws_upgrade(&mut self) -> Result<(), String> {
        if !self.tls_established {
            return Err("Cannot upgrade: TLS not established".to_string());
        }
        self.ws_upgraded = true;
        Ok(())
    }

    pub fn send_message(&mut self, message: &str) -> Result<(), String> {
        if !self.tls_established || !self.ws_upgraded {
            return Err("Cannot send: connection not ready".to_string());
        }

        let frame = WebSocketFrame {
            fin: true,
            opcode: Opcode::Text,
            masked: true,
            mask_key: Some([0x01, 0x02, 0x03, 0x04]),
            payload: message.as_bytes().to_vec(),
        };

        let encoded = encode_frame(&frame);
        self.messages.push(encoded);
        Ok(())
    }

    pub fn receive_messages(&self) -> Result<&Vec<Vec<u8>>, String> {
        if !self.tls_established || !self.ws_upgraded {
            return Err("Cannot receive: connection not ready".to_string());
        }
        Ok(&self.messages)
    }

    pub fn close(&mut self) -> Result<(), String> {
        if !self.ws_upgraded {
            return Err("Cannot close: not upgraded".to_string());
        }
        let close_frame = WebSocketFrame {
            fin: true,
            opcode: Opcode::Close,
            masked: false,
            mask_key: None,
            payload: vec![],
        };
        self.messages.push(encode_frame(&close_frame));
        self.ws_upgraded = false;
        Ok(())
    }
}

pub fn encrypt_payload(payload: &[u8], session_key: &[u8]) -> Vec<u8> {
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, session_key);
    let mac = hmac::sign(&hmac_key, payload);

    let mut encrypted = Vec::with_capacity(payload.len());
    for (i, &byte) in payload.iter().enumerate() {
        encrypted.push(byte ^ mac.as_ref()[i % 32]);
    }
    encrypted
}

pub fn decrypt_payload(ciphertext: &[u8], session_key: &[u8]) -> Vec<u8> {
    // XOR is its own inverse — same operation
    encrypt_payload(ciphertext, session_key)
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
