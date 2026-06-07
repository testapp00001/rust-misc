//! # WebSocket Communication
//!
//! WebSockets provide full-duplex communication channels over a single TCP connection.
//! This lesson covers WebSocket fundamentals, message types, connection lifecycle,
//! broadcast patterns, and building a chat-like system.
//!
//! ## Key Concepts
//! - WebSocket upgrade handshake
//! - Text vs binary messages
//! - Ping/pong for keepalive
//! - Connection management and broadcast
//! - Room-based messaging
//! - Reconnection strategies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ---------------------------------------------------------------------------
// 1. WebSocket Message Types
// ---------------------------------------------------------------------------

/// Represents a WebSocket message as understood by our protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    /// A chat message from a user.
    Chat(ChatMessage),
    /// A user joined the room.
    Join { user: String, room: String },
    /// A user left the room.
    Leave { user: String, room: String },
    /// A system notification.
    System { text: String },
    /// Ping for keepalive.
    Ping { timestamp: u64 },
    /// Pong response.
    Pong { timestamp: u64 },
    /// Request to subscribe to a room.
    Subscribe { room: String },
    /// Request to unsubscribe from a room.
    Unsubscribe { room: String },
    /// Error message.
    Error { code: u16, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub user: String,
    pub room: String,
    pub text: String,
    pub timestamp: u64,
}

impl WsMessage {
    /// Serialize to JSON string for transmission.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Create a chat message.
    pub fn chat(id: &str, user: &str, room: &str, text: &str, timestamp: u64) -> Self {
        WsMessage::Chat(ChatMessage {
            id: id.into(),
            user: user.into(),
            room: room.into(),
            text: text.into(),
            timestamp,
        })
    }

    /// Create a system message.
    pub fn system(text: &str) -> Self {
        WsMessage::System {
            text: text.into(),
        }
    }

    /// Create a join notification.
    pub fn join(user: &str, room: &str) -> Self {
        WsMessage::Join {
            user: user.into(),
            room: room.into(),
        }
    }

    /// Create an error message.
    pub fn error(code: u16, message: &str) -> Self {
        WsMessage::Error {
            code,
            message: message.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Connection State
// ---------------------------------------------------------------------------

/// Represents the state of a WebSocket connection.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
}

/// Metadata for a connected client.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub user_id: String,
    pub state: ConnectionState,
    pub subscribed_rooms: Vec<String>,
    pub connected_at: u64,
    pub last_ping_at: u64,
}

impl ConnectionInfo {
    pub fn new(connection_id: String, user_id: String, timestamp: u64) -> Self {
        Self {
            connection_id,
            user_id,
            state: ConnectionState::Connected,
            subscribed_rooms: Vec::new(),
            connected_at: timestamp,
            last_ping_at: timestamp,
        }
    }

    pub fn subscribe(&mut self, room: &str) {
        if !self.subscribed_rooms.iter().any(|r| r == room) {
            self.subscribed_rooms.push(room.into());
        }
    }

    pub fn unsubscribe(&mut self, room: &str) {
        self.subscribed_rooms.retain(|r| r != room);
    }

    pub fn is_subscribed(&self, room: &str) -> bool {
        self.subscribed_rooms.iter().any(|r| r == room)
    }
}

// ---------------------------------------------------------------------------
// 3. Message History
// ---------------------------------------------------------------------------

/// Stores recent messages for a room (for new subscribers to catch up).
#[derive(Debug, Clone)]
pub struct MessageHistory {
    max_messages: usize,
    messages: Vec<ChatMessage>,
}

impl MessageHistory {
    pub fn new(max_messages: usize) -> Self {
        Self {
            max_messages,
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn recent(&self, count: usize) -> Vec<ChatMessage> {
        let start = self.messages.len().saturating_sub(count);
        self.messages[start..].to_vec()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 4. Hub (Connection Manager)
// ---------------------------------------------------------------------------

/// The central hub that manages all WebSocket connections and rooms.
/// This is the core of any WebSocket server.
#[derive(Debug)]
pub struct Hub {
    connections: HashMap<String, ConnectionInfo>,
    rooms: HashMap<String, Vec<String>>, // room -> connection_ids
    room_history: HashMap<String, MessageHistory>,
    max_history_per_room: usize,
}

impl Hub {
    pub fn new(max_history_per_room: usize) -> Self {
        Self {
            connections: HashMap::new(),
            rooms: HashMap::new(),
            room_history: HashMap::new(),
            max_history_per_room,
        }
    }

    /// Register a new connection.
    pub fn connect(&mut self, conn: ConnectionInfo) -> String {
        let id = conn.connection_id.clone();
        self.connections.insert(id.clone(), conn);
        id
    }

    /// Remove a connection and unsubscribe from all rooms.
    pub fn disconnect(&mut self, connection_id: &str) -> Vec<String> {
        let mut affected_rooms = Vec::new();

        if let Some(conn) = self.connections.remove(connection_id) {
            for room in &conn.subscribed_rooms {
                if let Some(members) = self.rooms.get_mut(room) {
                    members.retain(|id| id != connection_id);
                    if members.is_empty() {
                        self.rooms.remove(room);
                    }
                    affected_rooms.push(room.clone());
                }
            }
        }

        affected_rooms
    }

    /// Subscribe a connection to a room.
    pub fn subscribe(&mut self, connection_id: &str, room: &str) -> Result<(), HubError> {
        let conn = self
            .connections
            .get_mut(connection_id)
            .ok_or(HubError::ConnectionNotFound)?;

        conn.subscribe(room);

        self.rooms
            .entry(room.into())
            .or_default()
            .push(connection_id.into());

        Ok(())
    }

    /// Unsubscribe a connection from a room.
    pub fn unsubscribe(&mut self, connection_id: &str, room: &str) {
        if let Some(conn) = self.connections.get_mut(connection_id) {
            conn.unsubscribe(room);
        }

        if let Some(members) = self.rooms.get_mut(room) {
            members.retain(|id| id != connection_id);
            if members.is_empty() {
                self.rooms.remove(room);
            }
        }
    }

    /// Get all connection IDs subscribed to a room.
    pub fn room_members(&self, room: &str) -> Vec<String> {
        self.rooms.get(room).cloned().unwrap_or_default()
    }

    /// Store a message in room history.
    pub fn record_message(&mut self, msg: ChatMessage) {
        let history = self
            .room_history
            .entry(msg.room.clone())
            .or_insert_with(|| MessageHistory::new(self.max_history_per_room));
        history.push(msg);
    }

    /// Get recent messages for a room.
    pub fn room_history(&self, room: &str, count: usize) -> Vec<ChatMessage> {
        self.room_history
            .get(room)
            .map(|h| h.recent(count))
            .unwrap_or_default()
    }

    /// Get the number of active connections.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get the number of active rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Get connection info.
    pub fn get_connection(&self, connection_id: &str) -> Option<&ConnectionInfo> {
        self.connections.get(connection_id)
    }

    /// List all room names.
    pub fn list_rooms(&self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }

    /// Handle an incoming message and return broadcast targets.
    pub fn handle_message(
        &mut self,
        connection_id: &str,
        message: WsMessage,
    ) -> Result<BroadcastPlan, HubError> {
        match message {
            WsMessage::Subscribe { room } => {
                self.subscribe(connection_id, &room)?;
                let user = self
                    .connections
                    .get(connection_id)
                    .map(|c| c.user_id.clone())
                    .unwrap_or_default();
                let join_msg = WsMessage::join(&user, &room);
                let members = self.room_members(&room);
                Ok(BroadcastPlan {
                    messages: vec![(members, join_msg)],
                    response: None,
                })
            }
            WsMessage::Unsubscribe { room } => {
                self.unsubscribe(connection_id, &room);
                let user = self
                    .connections
                    .get(connection_id)
                    .map(|c| c.user_id.clone())
                    .unwrap_or_default();
                let leave_msg = WsMessage::Leave {
                    user,
                    room: room.clone(),
                };
                let members = self.room_members(&room);
                Ok(BroadcastPlan {
                    messages: vec![(members, leave_msg)],
                    response: None,
                })
            }
            WsMessage::Chat(chat) => {
                let conn = self
                    .connections
                    .get(connection_id)
                    .ok_or(HubError::ConnectionNotFound)?;

                if !conn.is_subscribed(&chat.room) {
                    return Err(HubError::NotSubscribed);
                }

                self.record_message(chat.clone());
                let members = self.room_members(&chat.room);
                Ok(BroadcastPlan {
                    messages: vec![(members, WsMessage::Chat(chat))],
                    response: None,
                })
            }
            WsMessage::Ping { timestamp } => Ok(BroadcastPlan {
                messages: vec![],
                response: Some(WsMessage::Pong { timestamp }),
            }),
            _ => Ok(BroadcastPlan {
                messages: vec![],
                response: None,
            }),
        }
    }
}

/// Describes where to send messages after processing.
#[derive(Debug)]
pub struct BroadcastPlan {
    /// (target_connection_ids, message) pairs to broadcast.
    pub messages: Vec<(Vec<String>, WsMessage)>,
    /// A direct response to send back to the sender.
    pub response: Option<WsMessage>,
}

#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("connection not found")]
    ConnectionNotFound,

    #[error("not subscribed to this room")]
    NotSubscribed,

    #[error("room is full")]
    RoomFull,
}

// ---------------------------------------------------------------------------
// 5. Rate Limiting for Messages
// ---------------------------------------------------------------------------

/// Per-connection message rate limiter.
#[derive(Debug)]
pub struct MessageRateLimiter {
    max_per_window: u32,
    window_secs: u64,
    timestamps: Vec<u64>,
}

impl MessageRateLimiter {
    pub fn new(max_per_window: u32, window_secs: u64) -> Self {
        Self {
            max_per_window,
            window_secs,
            timestamps: Vec::new(),
        }
    }

    /// Try to consume a message slot. Returns true if allowed.
    pub fn try_consume(&mut self, now: u64) -> bool {
        let window_start = now.saturating_sub(self.window_secs);
        self.timestamps.retain(|t| *t > window_start);

        if self.timestamps.len() < self.max_per_window as usize {
            self.timestamps.push(now);
            true
        } else {
            false
        }
    }

    pub fn current_count(&self, now: u64) -> usize {
        let window_start = now.saturating_sub(self.window_secs);
        self.timestamps.iter().filter(|t| **t > window_start).count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_chat_serialization() {
        let msg = WsMessage::chat("msg-1", "alice", "general", "hello!", 1000);
        let json = msg.to_json();
        let parsed = WsMessage::from_json(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_ws_message_join_serialization() {
        let msg = WsMessage::join("alice", "general");
        let json = msg.to_json();
        assert!(json.contains("\"type\":\"Join\""));
        let parsed = WsMessage::from_json(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_ws_message_system() {
        let msg = WsMessage::system("server restarting");
        let json = msg.to_json();
        let parsed = WsMessage::from_json(&json).unwrap();
        match parsed {
            WsMessage::System { text } => assert_eq!(text, "server restarting"),
            _ => panic!("expected System variant"),
        }
    }

    #[test]
    fn test_ws_message_error() {
        let msg = WsMessage::error(403, "forbidden");
        match msg {
            WsMessage::Error { code, message } => {
                assert_eq!(code, 403);
                assert_eq!(message, "forbidden");
            }
            _ => panic!("expected Error variant"),
        }
    }

    #[test]
    fn test_connection_info_subscribe() {
        let mut conn = ConnectionInfo::new("c1".into(), "user1".into(), 1000);
        conn.subscribe("general");
        conn.subscribe("random");
        conn.subscribe("general"); // duplicate
        assert_eq!(conn.subscribed_rooms.len(), 2);
        assert!(conn.is_subscribed("general"));
        assert!(conn.is_subscribed("random"));
        assert!(!conn.is_subscribed("other"));
    }

    #[test]
    fn test_connection_info_unsubscribe() {
        let mut conn = ConnectionInfo::new("c1".into(), "user1".into(), 1000);
        conn.subscribe("general");
        conn.subscribe("random");
        conn.unsubscribe("general");
        assert!(!conn.is_subscribed("general"));
        assert!(conn.is_subscribed("random"));
    }

    #[test]
    fn test_message_history() {
        let mut history = MessageHistory::new(3);
        assert!(history.is_empty());

        history.push(ChatMessage {
            id: "1".into(),
            user: "a".into(),
            room: "r".into(),
            text: "msg1".into(),
            timestamp: 1,
        });
        history.push(ChatMessage {
            id: "2".into(),
            user: "b".into(),
            room: "r".into(),
            text: "msg2".into(),
            timestamp: 2,
        });
        history.push(ChatMessage {
            id: "3".into(),
            user: "c".into(),
            room: "r".into(),
            text: "msg3".into(),
            timestamp: 3,
        });
        history.push(ChatMessage {
            id: "4".into(),
            user: "d".into(),
            room: "r".into(),
            text: "msg4".into(),
            timestamp: 4,
        });

        assert_eq!(history.len(), 3); // capped at 3
        assert_eq!(history.recent(2).len(), 2);
        assert_eq!(history.recent(10).len(), 3);
    }

    #[test]
    fn test_hub_connect_disconnect() {
        let mut hub = Hub::new(100);

        let conn = ConnectionInfo::new("c1".into(), "alice".into(), 1000);
        hub.connect(conn);
        assert_eq!(hub.connection_count(), 1);

        hub.disconnect("c1");
        assert_eq!(hub.connection_count(), 0);
    }

    #[test]
    fn test_hub_subscribe_unsubscribe() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));

        hub.subscribe("c1", "general").unwrap();
        assert_eq!(hub.room_members("general"), vec!["c1"]);
        assert_eq!(hub.room_count(), 1);

        hub.unsubscribe("c1", "general");
        assert!(hub.room_members("general").is_empty());
    }

    #[test]
    fn test_hub_multiple_subscribers() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));
        hub.connect(ConnectionInfo::new("c2".into(), "bob".into(), 1001));
        hub.connect(ConnectionInfo::new("c3".into(), "charlie".into(), 1002));

        hub.subscribe("c1", "general").unwrap();
        hub.subscribe("c2", "general").unwrap();
        hub.subscribe("c3", "random").unwrap();

        let general_members = hub.room_members("general");
        assert_eq!(general_members.len(), 2);
        assert!(general_members.contains(&"c1".to_string()));
        assert!(general_members.contains(&"c2".to_string()));

        assert_eq!(hub.room_members("random").len(), 1);
    }

    #[test]
    fn test_hub_disconnect_removes_from_rooms() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));
        hub.subscribe("c1", "general").unwrap();
        hub.subscribe("c1", "random").unwrap();

        let affected = hub.disconnect("c1");
        assert_eq!(affected.len(), 2);
        assert!(hub.room_members("general").is_empty());
    }

    #[test]
    fn test_hub_handle_subscribe_message() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));

        let msg = WsMessage::Subscribe {
            room: "general".into(),
        };
        let plan = hub.handle_message("c1", msg).unwrap();

        // Should broadcast join message to room members (just alice so far)
        assert_eq!(plan.messages.len(), 1);
        let (targets, _) = &plan.messages[0];
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], "c1");
    }

    #[test]
    fn test_hub_handle_chat_message() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));
        hub.connect(ConnectionInfo::new("c2".into(), "bob".into(), 1001));
        hub.subscribe("c1", "general").unwrap();
        hub.subscribe("c2", "general").unwrap();

        let msg = WsMessage::chat("m1", "alice", "general", "hello!", 1000);
        let plan = hub.handle_message("c1", msg).unwrap();

        assert_eq!(plan.messages.len(), 1);
        let (targets, chat_msg) = &plan.messages[0];
        assert_eq!(targets.len(), 2);
        match chat_msg {
            WsMessage::Chat(c) => assert_eq!(c.text, "hello!"),
            _ => panic!("expected Chat"),
        }

        // Message should be in history
        let history = hub.room_history("general", 10);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_hub_chat_requires_subscription() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));

        let msg = WsMessage::chat("m1", "alice", "general", "hello!", 1000);
        let result = hub.handle_message("c1", msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_hub_handle_ping() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));

        let msg = WsMessage::Ping { timestamp: 12345 };
        let plan = hub.handle_message("c1", msg).unwrap();

        assert!(plan.messages.is_empty());
        assert_eq!(plan.response, Some(WsMessage::Pong { timestamp: 12345 }));
    }

    #[test]
    fn test_hub_list_rooms() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));
        hub.subscribe("c1", "general").unwrap();
        hub.subscribe("c1", "random").unwrap();

        let rooms = hub.list_rooms();
        assert_eq!(rooms.len(), 2);
        assert!(rooms.contains(&"general".to_string()));
        assert!(rooms.contains(&"random".to_string()));
    }

    #[test]
    fn test_rate_limiter_within_limit() {
        let mut limiter = MessageRateLimiter::new(3, 60);
        assert!(limiter.try_consume(100));
        assert!(limiter.try_consume(101));
        assert!(limiter.try_consume(102));
        assert!(!limiter.try_consume(103));
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let mut limiter = MessageRateLimiter::new(2, 60);
        assert!(limiter.try_consume(100));
        assert!(limiter.try_consume(101));
        assert!(!limiter.try_consume(102));

        // After window passes
        assert!(limiter.try_consume(200));
    }

    #[test]
    fn test_rate_limiter_count() {
        let mut limiter = MessageRateLimiter::new(10, 60);
        limiter.try_consume(100);
        limiter.try_consume(101);
        limiter.try_consume(102);
        assert_eq!(limiter.current_count(150), 3);
        assert_eq!(limiter.current_count(200), 0); // outside window
    }

    #[test]
    fn test_message_history_empty() {
        let history = MessageHistory::new(10);
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert!(history.recent(5).is_empty());
    }

    #[test]
    fn test_hub_get_connection() {
        let mut hub = Hub::new(100);
        hub.connect(ConnectionInfo::new("c1".into(), "alice".into(), 1000));

        let conn = hub.get_connection("c1").unwrap();
        assert_eq!(conn.user_id, "alice");
        assert!(hub.get_connection("nonexistent").is_none());
    }
}
