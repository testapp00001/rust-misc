//! Binary RPC protocol for the message queue.
//!
//! All messages are serialized with `bincode` and transmitted over TCP using
//! length-prefixed framing:
//!
//! ```text
//! [4 bytes: payload length in big-endian][payload bytes]
//! ```
//!
//! The maximum message size is enforced at the server level to prevent
//! memory exhaustion from malicious or buggy clients.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Request / Response enums
// ---------------------------------------------------------------------------

/// A client-to-server RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcRequest {
    // -- Producer API -------------------------------------------------------
    /// Produce one or more messages to a topic.  If `partition` is `None` the
    /// broker selects one via the partitioning strategy (key-hash or
    /// round-robin).
    Produce {
        topic: String,
        partition: Option<u32>,
        messages: Vec<Vec<u8>>,
        acks: Acks,
    },

    // -- Consumer API -------------------------------------------------------
    /// Fetch messages starting at `offset` for the given topic/partition.
    /// `max_bytes` caps the total payload size of the response batch.
    Fetch {
        topic: String,
        partition: u32,
        offset: u64,
        max_bytes: u64,
    },

    /// Commit a consumer offset for a consumer group.
    CommitOffset {
        topic: String,
        partition: u32,
        consumer_group: String,
        offset: u64,
    },

    /// Fetch the committed offset for a consumer group.
    FetchOffset {
        topic: String,
        partition: u32,
        consumer_group: String,
    },

    // -- Admin API ----------------------------------------------------------
    /// Create a new topic with the given partition count and replication factor.
    CreateTopic {
        topic: String,
        partitions: u32,
        replication_factor: u32,
    },

    /// List all topics known to this broker.
    ListTopics,

    /// Get cluster metadata, optionally filtered to a single topic.
    GetMetadata { topic: Option<String> },

    // -- Replication API (Raft) ---------------------------------------------
    /// Append entries from the leader to a follower.
    AppendEntries {
        term: u64,
        leader_id: u64,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    },

    /// Request a vote during leader election.
    RequestVote {
        term: u64,
        candidate_id: u64,
        last_log_index: u64,
        last_log_term: u64,
    },

    /// Install a snapshot from the leader to a lagging follower.
    InstallSnapshot {
        term: u64,
        leader_id: u64,
        last_included_index: u64,
        last_included_term: u64,
        data: Vec<u8>,
    },

    // -- Health / Monitoring ------------------------------------------------
    /// Liveness probe -- the server replies with `Pong`.
    Ping,

    /// Return a snapshot of the broker's runtime metrics.
    GetMetrics,
}

/// A server-to-client RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcResponse {
    // -- Producer -----------------------------------------------------------
    Produce {
        offsets: Vec<u64>,
    },

    // -- Consumer -----------------------------------------------------------
    Fetch {
        messages: Vec<FetchMessage>,
        high_watermark: u64,
    },

    CommitOffset {
        success: bool,
    },

    FetchOffset {
        offset: u64,
    },

    // -- Admin --------------------------------------------------------------
    CreateTopic {
        success: bool,
    },

    ListTopics {
        topics: Vec<TopicInfo>,
    },

    GetMetadata {
        brokers: Vec<BrokerInfo>,
        partitions: Vec<PartitionInfo>,
    },

    // -- Replication --------------------------------------------------------
    AppendEntries {
        term: u64,
        success: bool,
    },

    RequestVote {
        term: u64,
        vote_granted: bool,
    },

    InstallSnapshot {
        term: u64,
        success: bool,
    },

    // -- Health / Monitoring ------------------------------------------------
    Pong,

    Metrics(crate::metrics::MetricsSnapshot),

    // -- Error --------------------------------------------------------------
    Error(String),
}

// ---------------------------------------------------------------------------
// Acknowledgement level
// ---------------------------------------------------------------------------

/// How many replicas must acknowledge a produce before the broker responds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Acks {
    /// Fire and forget -- the broker acknowledges immediately without waiting
    /// for any replica.
    None,
    /// Wait only for the leader partition to persist the message.
    Leader,
    /// Wait for a majority (quorum) of replicas to acknowledge.
    Majority,
    /// Wait for every replica in the ISR to acknowledge.
    All,
}

// ---------------------------------------------------------------------------
// Wire-format types
// ---------------------------------------------------------------------------

/// A single entry in the replicated log (Raft).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log index (monotonically increasing per partition).
    pub index: u64,
    /// Raft term when the entry was created.
    pub term: u64,
    /// Serialized command payload.
    pub data: Vec<u8>,
}

/// A message returned by a `Fetch` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMessage {
    /// Offset of this message within its partition.
    pub offset: u64,
    /// Optional message key (used for key-based partitioning).
    pub key: Option<Vec<u8>>,
    /// Message payload.
    pub value: Vec<u8>,
    /// Unix timestamp in milliseconds when the message was produced.
    pub timestamp: u64,
    /// Arbitrary key-value headers attached by the producer.
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
}

/// Topic metadata returned by `ListTopics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    /// Topic name.
    pub name: String,
    /// Number of partitions.
    pub partitions: u32,
    /// Replication factor.
    pub replication_factor: u32,
}

/// Broker information returned by `GetMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerInfo {
    /// Unique broker node ID.
    pub id: u64,
    /// Broker bind address (`host:port`).
    pub address: String,
    /// Optional rack ID for rack-aware replication.
    pub rack: Option<String>,
}

/// Per-partition metadata returned by `GetMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Topic name.
    pub topic: String,
    /// Partition index within the topic.
    pub partition: u32,
    /// Node ID of the current leader.
    pub leader: u64,
    /// All replica node IDs.
    pub replicas: Vec<u64>,
    /// In-sync replica node IDs (subset of `replicas`).
    pub isr: Vec<u64>,
    /// High watermark: the next offset that will be written.
    pub high_watermark: u64,
}

// ---------------------------------------------------------------------------
// Wire encoding / decoding  (length-prefixed binary)
// ---------------------------------------------------------------------------

/// Encode a serializable value into a length-prefixed frame.
///
/// The returned `Vec<u8>` is ready to be written to a TCP stream:
/// `[4-byte big-endian length][bincode payload]`.
pub fn encode_message<T: Serialize>(msg: &T) -> crate::error::Result<Vec<u8>> {
    let payload = bincode::serialize(msg).map_err(|e| {
        crate::error::MqError::Serialization(format!("bincode encode failed: {}", e))
    })?;
    let len = (payload.len() as u32).to_be_bytes();
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&len);
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decode a bincode-serialized value from raw payload bytes.
///
/// The `data` slice must **not** include the 4-byte length prefix -- the caller
/// is expected to have read that already.
pub fn decode_message<T: for<'de> Deserialize<'de>>(
    data: &[u8],
) -> crate::error::Result<T> {
    bincode::deserialize(data).map_err(|e| {
        crate::error::MqError::Serialization(format!("bincode decode failed: {}", e))
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- encode / decode round-trip -----------------------------------------

    #[test]
    fn encode_decode_roundtrip() {
        let req = RpcRequest::Ping;
        let frame = encode_message(&req).unwrap();

        // Frame must start with a 4-byte big-endian length.
        let len = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]);
        assert_eq!(len as usize, frame.len() - 4);

        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();
        assert!(matches!(decoded, RpcRequest::Ping));
    }

    #[test]
    fn pong_roundtrip() {
        let resp = RpcResponse::Pong;
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();
        assert!(matches!(decoded, RpcResponse::Pong));
    }

    // -- request variants ---------------------------------------------------

    #[test]
    fn produce_request_roundtrip() {
        let req = RpcRequest::Produce {
            topic: "orders".into(),
            partition: Some(3),
            messages: vec![b"msg1".to_vec(), b"msg2".to_vec()],
            acks: Acks::Majority,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::Produce {
            topic,
            partition,
            messages,
            acks,
        } = decoded
        {
            assert_eq!(topic, "orders");
            assert_eq!(partition, Some(3));
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0], b"msg1");
            assert_eq!(acks, Acks::Majority);
        } else {
            panic!("expected Produce");
        }
    }

    #[test]
    fn fetch_request_roundtrip() {
        let req = RpcRequest::Fetch {
            topic: "events".into(),
            partition: 0,
            offset: 42,
            max_bytes: 1024 * 1024,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::Fetch {
            topic,
            partition,
            offset,
            max_bytes,
        } = decoded
        {
            assert_eq!(topic, "events");
            assert_eq!(partition, 0);
            assert_eq!(offset, 42);
            assert_eq!(max_bytes, 1_048_576);
        } else {
            panic!("expected Fetch");
        }
    }

    #[test]
    fn commit_offset_roundtrip() {
        let req = RpcRequest::CommitOffset {
            topic: "logs".into(),
            partition: 1,
            consumer_group: "analytics".into(),
            offset: 999,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::CommitOffset {
            topic,
            partition,
            consumer_group,
            offset,
        } = decoded
        {
            assert_eq!(topic, "logs");
            assert_eq!(partition, 1);
            assert_eq!(consumer_group, "analytics");
            assert_eq!(offset, 999);
        } else {
            panic!("expected CommitOffset");
        }
    }

    #[test]
    fn create_topic_roundtrip() {
        let req = RpcRequest::CreateTopic {
            topic: "users".into(),
            partitions: 12,
            replication_factor: 3,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::CreateTopic {
            topic,
            partitions,
            replication_factor,
        } = decoded
        {
            assert_eq!(topic, "users");
            assert_eq!(partitions, 12);
            assert_eq!(replication_factor, 3);
        } else {
            panic!("expected CreateTopic");
        }
    }

    #[test]
    fn append_entries_roundtrip() {
        let req = RpcRequest::AppendEntries {
            term: 5,
            leader_id: 1,
            prev_log_index: 10,
            prev_log_term: 4,
            entries: vec![
                LogEntry {
                    index: 11,
                    term: 5,
                    data: vec![1, 2, 3],
                },
                LogEntry {
                    index: 12,
                    term: 5,
                    data: vec![4, 5, 6],
                },
            ],
            leader_commit: 12,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::AppendEntries {
            term,
            leader_id,
            entries,
            leader_commit,
            ..
        } = decoded
        {
            assert_eq!(term, 5);
            assert_eq!(leader_id, 1);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].index, 11);
            assert_eq!(entries[1].data, vec![4, 5, 6]);
            assert_eq!(leader_commit, 12);
        } else {
            panic!("expected AppendEntries");
        }
    }

    #[test]
    fn request_vote_roundtrip() {
        let req = RpcRequest::RequestVote {
            term: 3,
            candidate_id: 2,
            last_log_index: 20,
            last_log_term: 3,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::RequestVote {
            term,
            candidate_id,
            last_log_index,
            last_log_term,
        } = decoded
        {
            assert_eq!(term, 3);
            assert_eq!(candidate_id, 2);
            assert_eq!(last_log_index, 20);
            assert_eq!(last_log_term, 3);
        } else {
            panic!("expected RequestVote");
        }
    }

    #[test]
    fn install_snapshot_roundtrip() {
        let req = RpcRequest::InstallSnapshot {
            term: 10,
            leader_id: 1,
            last_included_index: 500,
            last_included_term: 9,
            data: vec![0u8; 256],
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::InstallSnapshot {
            term,
            leader_id,
            last_included_index,
            last_included_term,
            data,
        } = decoded
        {
            assert_eq!(term, 10);
            assert_eq!(leader_id, 1);
            assert_eq!(last_included_index, 500);
            assert_eq!(last_included_term, 9);
            assert_eq!(data.len(), 256);
        } else {
            panic!("expected InstallSnapshot");
        }
    }

    // -- response variants --------------------------------------------------

    #[test]
    fn produce_response_roundtrip() {
        let resp = RpcResponse::Produce {
            offsets: vec![0, 1, 2],
        };
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();

        if let RpcResponse::Produce { offsets } = decoded {
            assert_eq!(offsets, vec![0, 1, 2]);
        } else {
            panic!("expected Produce response");
        }
    }

    #[test]
    fn fetch_response_roundtrip() {
        let resp = RpcResponse::Fetch {
            messages: vec![FetchMessage {
                offset: 42,
                key: Some(b"k".to_vec()),
                value: b"payload".to_vec(),
                timestamp: 1_000_000,
                headers: vec![(b"h".to_vec(), b"v".to_vec())],
            }],
            high_watermark: 100,
        };
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();

        if let RpcResponse::Fetch {
            messages,
            high_watermark,
        } = decoded
        {
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].offset, 42);
            assert_eq!(messages[0].key, Some(b"k".to_vec()));
            assert_eq!(messages[0].value, b"payload");
            assert_eq!(high_watermark, 100);
        } else {
            panic!("expected Fetch response");
        }
    }

    #[test]
    fn error_response_roundtrip() {
        let resp = RpcResponse::Error("something went wrong".into());
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();

        if let RpcResponse::Error(msg) = decoded {
            assert_eq!(msg, "something went wrong");
        } else {
            panic!("expected Error response");
        }
    }

    #[test]
    fn list_topics_response_roundtrip() {
        let resp = RpcResponse::ListTopics {
            topics: vec![
                TopicInfo {
                    name: "orders".into(),
                    partitions: 6,
                    replication_factor: 3,
                },
                TopicInfo {
                    name: "payments".into(),
                    partitions: 3,
                    replication_factor: 2,
                },
            ],
        };
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();

        if let RpcResponse::ListTopics { topics } = decoded {
            assert_eq!(topics.len(), 2);
            assert_eq!(topics[0].name, "orders");
            assert_eq!(topics[1].partitions, 3);
        } else {
            panic!("expected ListTopics response");
        }
    }

    // -- Acks enum ----------------------------------------------------------

    #[test]
    fn acks_all_variants() {
        let variants = [Acks::None, Acks::Leader, Acks::Majority, Acks::All];
        for acks in &variants {
            let req = RpcRequest::Produce {
                topic: "t".into(),
                partition: None,
                messages: vec![],
                acks: *acks,
            };
            let frame = encode_message(&req).unwrap();
            let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();
            if let RpcRequest::Produce { acks: decoded_acks, .. } = decoded {
                assert_eq!(acks, &decoded_acks);
            } else {
                panic!("expected Produce");
            }
        }
    }

    // -- Wire format edge cases ---------------------------------------------

    #[test]
    fn empty_payload() {
        // An empty bincode-serialized value (e.g., ListTopics with no data).
        let req = RpcRequest::ListTopics;
        let frame = encode_message(&req).unwrap();
        // The frame must be at least 4 bytes (length prefix).
        assert!(frame.len() >= 4);
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();
        assert!(matches!(decoded, RpcRequest::ListTopics));
    }

    #[test]
    fn large_message() {
        let big_value = vec![0xABu8; 1024 * 1024]; // 1 MiB
        let req = RpcRequest::Produce {
            topic: "big".into(),
            partition: None,
            messages: vec![big_value.clone()],
            acks: Acks::None,
        };
        let frame = encode_message(&req).unwrap();
        let decoded: RpcRequest = decode_message(&frame[4..]).unwrap();

        if let RpcRequest::Produce { messages, .. } = decoded {
            assert_eq!(messages[0].len(), 1024 * 1024);
            assert_eq!(messages[0], big_value);
        } else {
            panic!("expected Produce");
        }
    }

    #[test]
    fn decode_corrupted_data_fails() {
        let result: crate::error::Result<RpcRequest> = decode_message(&[0xFF, 0xFE, 0xFD]);
        assert!(result.is_err());
    }

    #[test]
    fn metadata_response_roundtrip() {
        let resp = RpcResponse::GetMetadata {
            brokers: vec![
                BrokerInfo {
                    id: 1,
                    address: "10.0.0.1:9092".into(),
                    rack: Some("us-east-1a".into()),
                },
                BrokerInfo {
                    id: 2,
                    address: "10.0.0.2:9092".into(),
                    rack: None,
                },
            ],
            partitions: vec![PartitionInfo {
                topic: "events".into(),
                partition: 0,
                leader: 1,
                replicas: vec![1, 2, 3],
                isr: vec![1, 2],
                high_watermark: 5000,
            }],
        };
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();

        if let RpcResponse::GetMetadata { brokers, partitions } = decoded {
            assert_eq!(brokers.len(), 2);
            assert_eq!(brokers[0].id, 1);
            assert_eq!(brokers[1].rack, None);
            assert_eq!(partitions.len(), 1);
            assert_eq!(partitions[0].isr, vec![1, 2]);
        } else {
            panic!("expected GetMetadata response");
        }
    }

    // -- install_snapshot response ------------------------------------------

    #[test]
    fn install_snapshot_response_roundtrip() {
        let resp = RpcResponse::InstallSnapshot {
            term: 10,
            success: true,
        };
        let frame = encode_message(&resp).unwrap();
        let decoded: RpcResponse = decode_message(&frame[4..]).unwrap();

        if let RpcResponse::InstallSnapshot { term, success } = decoded {
            assert_eq!(term, 10);
            assert!(success);
        } else {
            panic!("expected InstallSnapshot response");
        }
    }
}
