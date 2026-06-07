//! # Protocol Design
//!
//! Custom network protocols are essential for building efficient, domain-specific
//! communication layers. This lesson covers framing, codec design, length-delimited
//! messages, and building a complete custom protocol from scratch.
//!
//! ## Key Concepts
//! - Message framing (length-delimited, delimiter-based, fixed-size)
//! - Codec pattern (encode/decode)
//! - Protocol versioning
//! - Binary message formats
//! - Error recovery and malformed message handling
//! - Protocol state machines

use bytes::{Buf, BufMut, BytesMut};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Protocol Message Types
// ---------------------------------------------------------------------------

/// Message types in our custom protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    Handshake = 0x01,
    HandshakeAck = 0x02,
    Data = 0x03,
    Heartbeat = 0x04,
    HeartbeatAck = 0x05,
    Error = 0x06,
    Disconnect = 0x07,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Handshake),
            0x02 => Some(Self::HandshakeAck),
            0x03 => Some(Self::Data),
            0x04 => Some(Self::Heartbeat),
            0x05 => Some(Self::HeartbeatAck),
            0x06 => Some(Self::Error),
            0x07 => Some(Self::Disconnect),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Protocol Frame
// ---------------------------------------------------------------------------

/// A single frame in our custom protocol.
///
/// Wire format:
/// ```text
/// [4 bytes: total length][1 byte: version][1 byte: msg_type][2 bytes: flags][4 bytes: sequence][N bytes: payload]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub version: u8,
    pub msg_type: MessageType,
    pub flags: u16,
    pub sequence: u32,
    pub payload: Vec<u8>,
}

impl Frame {
    pub const HEADER_SIZE: usize = 4 + 1 + 1 + 2 + 4; // length + version + type + flags + sequence = 12

    pub fn new(msg_type: MessageType, sequence: u32, payload: Vec<u8>) -> Self {
        Self {
            version: 1,
            msg_type,
            flags: 0,
            sequence,
            payload,
        }
    }

    pub fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    /// Total frame size on the wire (header + payload).
    pub fn wire_size(&self) -> usize {
        Self::HEADER_SIZE + self.payload.len()
    }
}

// ---------------------------------------------------------------------------
// 3. Flag Definitions
// ---------------------------------------------------------------------------

/// Protocol flags as bit flags.
pub struct Flags;

impl Flags {
    pub const NONE: u16 = 0x0000;
    pub const COMPRESSED: u16 = 0x0001;
    pub const ENCRYPTED: u16 = 0x0002;
    pub const PRIORITY_HIGH: u16 = 0x0004;
    pub const REQUIRES_ACK: u16 = 0x0008;
    pub const LAST_MESSAGE: u16 = 0x0010;

    pub fn has_flag(flags: u16, flag: u16) -> bool {
        flags & flag != 0
    }

    pub fn set_flag(flags: u16, flag: u16) -> u16 {
        flags | flag
    }

    pub fn clear_flag(flags: u16, flag: u16) -> u16 {
        flags & !flag
    }
}

// ---------------------------------------------------------------------------
// 4. Length-Delimited Codec
// ---------------------------------------------------------------------------

/// Errors that can occur during codec operations.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("incomplete frame: need {needed} more bytes")]
    Incomplete { needed: usize },

    #[error("invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("frame too large: {size} bytes (max {max})")]
    FrameTooLarge { size: usize, max: usize },

    #[error("invalid protocol version: {0}")]
    InvalidVersion(u8),

    #[error("buffer underflow")]
    BufferUnderflow,
}

/// The codec handles encoding and decoding frames from raw bytes.
pub struct LengthDelimitedCodec {
    max_frame_size: usize,
}

impl LengthDelimitedCodec {
    pub fn new(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }

    pub fn default_codec() -> Self {
        Self::new(1024 * 1024) // 1MB max frame
    }

    /// Encode a frame into bytes.
    pub fn encode(&self, frame: &Frame) -> Vec<u8> {
        let total_len = frame.wire_size() as u32;
        let mut buf = Vec::with_capacity(frame.wire_size());

        buf.put_u32(total_len);
        buf.put_u8(frame.version);
        buf.put_u8(frame.msg_type as u8);
        buf.put_u16(frame.flags);
        buf.put_u32(frame.sequence);
        buf.extend_from_slice(&frame.payload);

        buf
    }

    /// Try to decode a frame from a byte buffer.
    /// Returns Ok(Some(frame)) if a complete frame was decoded,
    /// Ok(None) if more data is needed, or Err on protocol errors.
    pub fn decode(&self, buf: &mut BytesMut) -> Result<Option<Frame>, CodecError> {
        // Need at least 4 bytes for the length prefix
        if buf.len() < 4 {
            return Ok(None);
        }

        // Read length without consuming bytes
        let total_len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

        if total_len > self.max_frame_size {
            return Err(CodecError::FrameTooLarge {
                size: total_len,
                max: self.max_frame_size,
            });
        }

        // Check if we have enough data
        if buf.len() < total_len {
            return Ok(None);
        }

        // Consume the length prefix
        buf.advance(4);

        let version = buf.get_u8();
        let msg_type_byte = buf.get_u8();
        let msg_type =
            MessageType::from_u8(msg_type_byte).ok_or(CodecError::InvalidMessageType(msg_type_byte))?;
        let flags = buf.get_u16();
        let sequence = buf.get_u32();

        let payload_len = total_len - Frame::HEADER_SIZE;
        let mut payload = vec![0u8; payload_len];
        buf.copy_to_slice(&mut payload);

        Ok(Some(Frame {
            version,
            msg_type,
            flags,
            sequence,
            payload,
        }))
    }
}

// ---------------------------------------------------------------------------
// 5. Protocol State Machine
// ---------------------------------------------------------------------------

/// States in the protocol connection lifecycle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolState {
    Disconnected,
    Handshaking,
    Connected,
    ShuttingDown,
}

/// Manages the protocol connection state machine.
#[derive(Debug)]
pub struct ProtocolStateMachine {
    state: ProtocolState,
    local_sequence: u32,
    remote_sequence: u32,
    pending_acks: HashMap<u32, Frame>,
}

impl ProtocolStateMachine {
    pub fn new() -> Self {
        Self {
            state: ProtocolState::Disconnected,
            local_sequence: 0,
            remote_sequence: 0,
            pending_acks: HashMap::new(),
        }
    }

    pub fn state(&self) -> ProtocolState {
        self.state
    }

    pub fn next_sequence(&mut self) -> u32 {
        let seq = self.local_sequence;
        self.local_sequence = self.local_sequence.wrapping_add(1);
        seq
    }

    /// Handle an incoming frame and return any response frames.
    pub fn handle_frame(&mut self, frame: &Frame) -> Result<Vec<Frame>, ProtocolError> {
        self.remote_sequence = frame.sequence;

        match (self.state, frame.msg_type) {
            // Disconnected -> receive handshake -> send ack -> connected
            (ProtocolState::Disconnected, MessageType::Handshake) => {
                self.state = ProtocolState::Connected;
                Ok(vec![Frame::new(
                    MessageType::HandshakeAck,
                    self.next_sequence(),
                    Vec::new(),
                )])
            }

            // Handshaking -> receive ack -> connected
            (ProtocolState::Handshaking, MessageType::HandshakeAck) => {
                self.state = ProtocolState::Connected;
                Ok(vec![])
            }

            // Connected -> receive heartbeat -> send ack
            (ProtocolState::Connected, MessageType::Heartbeat) => Ok(vec![Frame::new(
                MessageType::HeartbeatAck,
                self.next_sequence(),
                Vec::new(),
            )]),

            // Connected -> receive data -> process
            (ProtocolState::Connected, MessageType::Data) => {
                // Track if ack is required
                if Flags::has_flag(frame.flags, Flags::REQUIRES_ACK) {
                    self.pending_acks.insert(frame.sequence, frame.clone());
                }
                Ok(vec![])
            }

            // Connected -> receive disconnect -> shutdown
            (ProtocolState::Connected, MessageType::Disconnect) => {
                self.state = ProtocolState::Disconnected;
                Ok(vec![])
            }

            // Error in any state
            (_, MessageType::Error) => {
                Ok(vec![])
            }

            _ => Err(ProtocolError::UnexpectedMessage {
                state: self.state,
                msg_type: frame.msg_type,
            }),
        }
    }

    /// Initiate a handshake (client side).
    pub fn initiate_handshake(&mut self) -> Result<Frame, ProtocolError> {
        if self.state != ProtocolState::Disconnected {
            return Err(ProtocolError::InvalidStateTransition {
                from: self.state,
                to: ProtocolState::Handshaking,
            });
        }
        self.state = ProtocolState::Handshaking;
        Ok(Frame::new(
            MessageType::Handshake,
            self.next_sequence(),
            Vec::new(),
        ))
    }

    /// Initiate disconnect.
    pub fn initiate_disconnect(&mut self) -> Result<Frame, ProtocolError> {
        if self.state != ProtocolState::Connected {
            return Err(ProtocolError::InvalidStateTransition {
                from: self.state,
                to: ProtocolState::ShuttingDown,
            });
        }
        self.state = ProtocolState::ShuttingDown;
        Ok(Frame::new(
            MessageType::Disconnect,
            self.next_sequence(),
            Vec::new(),
        ))
    }

    /// Create a data frame.
    pub fn create_data_frame(&mut self, payload: Vec<u8>, requires_ack: bool) -> Frame {
        let mut flags = Flags::NONE;
        if requires_ack {
            flags = Flags::set_flag(flags, Flags::REQUIRES_ACK);
        }
        Frame::new(MessageType::Data, self.next_sequence(), payload).with_flags(flags)
    }

    pub fn local_sequence(&self) -> u32 {
        self.local_sequence
    }

    pub fn remote_sequence(&self) -> u32 {
        self.remote_sequence
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("unexpected message {msg_type:?} in state {state:?}")]
    UnexpectedMessage {
        state: ProtocolState,
        msg_type: MessageType,
    },

    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: ProtocolState,
        to: ProtocolState,
    },
}

// ---------------------------------------------------------------------------
// 6. Payload Serialization Helpers
// ---------------------------------------------------------------------------

/// Encode a string payload with a 2-byte length prefix.
pub fn encode_string_payload(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut buf = Vec::with_capacity(2 + bytes.len());
    buf.put_u16(bytes.len() as u16);
    buf.extend_from_slice(bytes);
    buf
}

/// Decode a length-prefixed string from a payload.
pub fn decode_string_payload(payload: &[u8]) -> Result<String, CodecError> {
    if payload.len() < 2 {
        return Err(CodecError::BufferUnderflow);
    }
    let len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + len {
        return Err(CodecError::Incomplete {
            needed: 2 + len - payload.len(),
        });
    }
    String::from_utf8(payload[2..2 + len].to_vec()).map_err(|_| CodecError::BufferUnderflow)
}

/// Encode a key-value map payload.
pub fn encode_map_payload(map: &HashMap<String, String>) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.put_u32(map.len() as u32);
    for (key, value) in map {
        let key_bytes = key.as_bytes();
        let val_bytes = value.as_bytes();
        buf.put_u16(key_bytes.len() as u16);
        buf.extend_from_slice(key_bytes);
        buf.put_u16(val_bytes.len() as u16);
        buf.extend_from_slice(val_bytes);
    }
    buf
}

// ---------------------------------------------------------------------------
// 7. Connection Info
// ---------------------------------------------------------------------------

/// Information about a protocol connection.
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub frames_sent: u64,
    pub frames_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
    pub reconnects: u64,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            frames_sent: 0,
            frames_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            errors: 0,
            reconnects: 0,
        }
    }
}

impl ConnectionStats {
    pub fn record_sent(&mut self, frame: &Frame) {
        self.frames_sent += 1;
        self.bytes_sent += frame.wire_size() as u64;
    }

    pub fn record_received(&mut self, frame: &Frame) {
        self.frames_received += 1;
        self.bytes_received += frame.wire_size() as u64;
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_from_u8() {
        assert_eq!(MessageType::from_u8(0x01), Some(MessageType::Handshake));
        assert_eq!(MessageType::from_u8(0x03), Some(MessageType::Data));
        assert_eq!(MessageType::from_u8(0x07), Some(MessageType::Disconnect));
        assert_eq!(MessageType::from_u8(0xFF), None);
    }

    #[test]
    fn test_frame_wire_size() {
        let frame = Frame::new(MessageType::Data, 1, vec![0u8; 100]);
        assert_eq!(frame.wire_size(), Frame::HEADER_SIZE + 100);
    }

    #[test]
    fn test_frame_builder() {
        let frame = Frame::new(MessageType::Data, 42, vec![1, 2, 3])
            .with_flags(Flags::COMPRESSED | Flags::PRIORITY_HIGH)
            .with_version(2);

        assert_eq!(frame.version, 2);
        assert_eq!(frame.msg_type, MessageType::Data);
        assert_eq!(frame.flags, Flags::COMPRESSED | Flags::PRIORITY_HIGH);
        assert_eq!(frame.sequence, 42);
        assert_eq!(frame.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_flags() {
        let mut flags = Flags::NONE;
        assert!(!Flags::has_flag(flags, Flags::COMPRESSED));

        flags = Flags::set_flag(flags, Flags::COMPRESSED);
        assert!(Flags::has_flag(flags, Flags::COMPRESSED));
        assert!(!Flags::has_flag(flags, Flags::ENCRYPTED));

        flags = Flags::set_flag(flags, Flags::ENCRYPTED);
        assert!(Flags::has_flag(flags, Flags::COMPRESSED));
        assert!(Flags::has_flag(flags, Flags::ENCRYPTED));

        flags = Flags::clear_flag(flags, Flags::COMPRESSED);
        assert!(!Flags::has_flag(flags, Flags::COMPRESSED));
        assert!(Flags::has_flag(flags, Flags::ENCRYPTED));
    }

    #[test]
    fn test_codec_encode_decode_roundtrip() {
        let codec = LengthDelimitedCodec::default_codec();
        let frame = Frame::new(MessageType::Data, 42, vec![1, 2, 3, 4, 5]);

        let encoded = codec.encode(&frame);
        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded, frame);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_codec_decode_incomplete() {
        let codec = LengthDelimitedCodec::default_codec();
        let mut buf = BytesMut::from(&[0, 0, 0, 20][..]); // says 20 bytes but only 4 available
        let result = codec.decode(&mut buf).unwrap();
        assert!(result.is_none()); // need more data
    }

    #[test]
    fn test_codec_decode_too_large() {
        let codec = LengthDelimitedCodec::new(100);
        let mut buf = BytesMut::from(&[0, 0, 1, 0][..]); // 256 bytes > 100 max
        let result = codec.decode(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_codec_multiple_frames() {
        let codec = LengthDelimitedCodec::default_codec();
        let frame1 = Frame::new(MessageType::Data, 1, vec![10, 20]);
        let frame2 = Frame::new(MessageType::Heartbeat, 2, vec![]);

        let mut encoded = codec.encode(&frame1);
        encoded.extend_from_slice(&codec.encode(&frame2));

        let mut buf = BytesMut::from(&encoded[..]);
        let decoded1 = codec.decode(&mut buf).unwrap().unwrap();
        let decoded2 = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded1, frame1);
        assert_eq!(decoded2, frame2);
    }

    #[test]
    fn test_codec_empty_payload() {
        let codec = LengthDelimitedCodec::default_codec();
        let frame = Frame::new(MessageType::Heartbeat, 0, vec![]);

        let encoded = codec.encode(&frame);
        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded.payload.len(), 0);
        assert_eq!(decoded.msg_type, MessageType::Heartbeat);
    }

    #[test]
    fn test_codec_invalid_message_type() {
        let codec = LengthDelimitedCodec::default_codec();
        // Manually craft a frame with invalid type
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&(12u32.to_be_bytes())); // length = header only
        encoded.push(1); // version
        encoded.push(0xFF); // invalid type
        encoded.extend_from_slice(&0u16.to_be_bytes()); // flags
        encoded.extend_from_slice(&0u32.to_be_bytes()); // sequence

        let mut buf = BytesMut::from(&encoded[..]);
        let result = codec.decode(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_protocol_state_machine_handshake() {
        let mut server = ProtocolStateMachine::new();
        assert_eq!(server.state(), ProtocolState::Disconnected);

        // Client sends handshake
        let handshake = Frame::new(MessageType::Handshake, 0, vec![]);
        let responses = server.handle_frame(&handshake).unwrap();

        assert_eq!(server.state(), ProtocolState::Connected);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].msg_type, MessageType::HandshakeAck);
    }

    #[test]
    fn test_protocol_state_machine_client_handshake() {
        let mut client = ProtocolStateMachine::new();
        let frame = client.initiate_handshake().unwrap();
        assert_eq!(frame.msg_type, MessageType::Handshake);
        assert_eq!(client.state(), ProtocolState::Handshaking);

        // Receive ack
        let ack = Frame::new(MessageType::HandshakeAck, 0, vec![]);
        client.handle_frame(&ack).unwrap();
        assert_eq!(client.state(), ProtocolState::Connected);
    }

    #[test]
    fn test_protocol_state_machine_heartbeat() {
        let mut sm = ProtocolStateMachine::new();
        // Get to connected state
        sm.handle_frame(&Frame::new(MessageType::Handshake, 0, vec![]))
            .unwrap();

        // Send heartbeat
        let hb = Frame::new(MessageType::Heartbeat, 1, vec![]);
        let responses = sm.handle_frame(&hb).unwrap();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].msg_type, MessageType::HeartbeatAck);
    }

    #[test]
    fn test_protocol_state_machine_disconnect() {
        let mut sm = ProtocolStateMachine::new();
        sm.handle_frame(&Frame::new(MessageType::Handshake, 0, vec![]))
            .unwrap();

        let disconnect = Frame::new(MessageType::Disconnect, 1, vec![]);
        sm.handle_frame(&disconnect).unwrap();
        assert_eq!(sm.state(), ProtocolState::Disconnected);
    }

    #[test]
    fn test_protocol_state_machine_unexpected_message() {
        let mut sm = ProtocolStateMachine::new();
        // Try to send data while disconnected
        let data = Frame::new(MessageType::Data, 0, vec![1, 2, 3]);
        let result = sm.handle_frame(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_protocol_initiate_disconnect_wrong_state() {
        let mut sm = ProtocolStateMachine::new();
        let result = sm.initiate_disconnect();
        assert!(result.is_err());
    }

    #[test]
    fn test_protocol_sequence_numbers() {
        let mut sm = ProtocolStateMachine::new();
        assert_eq!(sm.next_sequence(), 0);
        assert_eq!(sm.next_sequence(), 1);
        assert_eq!(sm.next_sequence(), 2);
    }

    #[test]
    fn test_protocol_create_data_frame() {
        let mut sm = ProtocolStateMachine::new();
        sm.handle_frame(&Frame::new(MessageType::Handshake, 0, vec![]))
            .unwrap();

        let frame = sm.create_data_frame(vec![1, 2, 3], true);
        assert_eq!(frame.msg_type, MessageType::Data);
        assert!(Flags::has_flag(frame.flags, Flags::REQUIRES_ACK));
    }

    #[test]
    fn test_string_payload_roundtrip() {
        let original = "Hello, world!";
        let encoded = encode_string_payload(original);
        let decoded = decode_string_payload(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_string_payload_empty() {
        let encoded = encode_string_payload("");
        let decoded = decode_string_payload(&encoded).unwrap();
        assert_eq!(decoded, "");
    }

    #[test]
    fn test_string_payload_too_short() {
        let result = decode_string_payload(&[0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_stats() {
        let mut stats = ConnectionStats::default();
        let frame = Frame::new(MessageType::Data, 0, vec![1, 2, 3, 4, 5]);

        stats.record_sent(&frame);
        stats.record_received(&frame);
        stats.record_error();

        assert_eq!(stats.frames_sent, 1);
        assert_eq!(stats.frames_received, 1);
        assert_eq!(stats.bytes_sent, frame.wire_size() as u64);
        assert_eq!(stats.bytes_received, frame.wire_size() as u64);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    fn test_codec_large_payload() {
        let codec = LengthDelimitedCodec::default_codec();
        let payload = vec![0xAB; 10_000];
        let frame = Frame::new(MessageType::Data, 1, payload.clone());

        let encoded = codec.encode(&frame);
        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_protocol_full_lifecycle() {
        let mut server = ProtocolStateMachine::new();

        // 1. Client connects and handshakes
        server
            .handle_frame(&Frame::new(MessageType::Handshake, 0, vec![]))
            .unwrap();
        assert_eq!(server.state(), ProtocolState::Connected);

        // 2. Exchange data
        let data = Frame::new(MessageType::Data, 1, vec![1, 2, 3]);
        server.handle_frame(&data).unwrap();

        // 3. Heartbeat
        server
            .handle_frame(&Frame::new(MessageType::Heartbeat, 2, vec![]))
            .unwrap();

        // 4. Disconnect
        server
            .handle_frame(&Frame::new(MessageType::Disconnect, 3, vec![]))
            .unwrap();
        assert_eq!(server.state(), ProtocolState::Disconnected);
    }
}
