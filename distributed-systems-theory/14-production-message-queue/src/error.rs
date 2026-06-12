use thiserror::Error;

#[derive(Error, Debug)]
pub enum MqError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Replication error: {0}")]
    Replication(String),

    #[error("Partition error: {0}")]
    Partition(String),

    #[error("Consumer error: {0}")]
    Consumer(String),

    #[error("Producer error: {0}")]
    Producer(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Not leader: current leader is {leader_id:?}")]
    NotLeader { leader_id: Option<u64> },

    #[error("Quorum not reached: {got}/{needed}")]
    QuorumNotReached { got: usize, needed: usize },

    #[error("Offset out of range: {offset} (valid: 0..{high_watermark})")]
    OffsetOutOfRange {
        offset: u64,
        high_watermark: u64,
    },

    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    #[error("Partition not found: {topic}/{partition}")]
    PartitionNotFound { topic: String, partition: u32 },

    #[error("Duplicate message: {0}")]
    DuplicateMessage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, MqError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_variants() {
        let err = MqError::Network("connection refused".into());
        assert_eq!(err.to_string(), "Network error: connection refused");

        let err = MqError::NotLeader {
            leader_id: Some(42),
        };
        assert_eq!(
            err.to_string(),
            "Not leader: current leader is Some(42)"
        );

        let err = MqError::QuorumNotReached { got: 1, needed: 3 };
        assert_eq!(err.to_string(), "Quorum not reached: 1/3");

        let err = MqError::OffsetOutOfRange {
            offset: 100,
            high_watermark: 50,
        };
        assert_eq!(
            err.to_string(),
            "Offset out of range: 100 (valid: 0..50)"
        );

        let err = MqError::PartitionNotFound {
            topic: "orders".into(),
            partition: 7,
        };
        assert_eq!(err.to_string(), "Partition not found: orders/7");
    }

    #[test]
    fn result_type_alias() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(MqError::Internal("boom".into()));
        assert!(err.is_err());
    }
}
