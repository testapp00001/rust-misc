//! PUT handler for inserting or updating key-value pairs in the database.
//!
//! Writes the key-value pair to the storage engine and returns the HLC timestamp
//! for causal tracking.

/// A PUT request specifying the key and value to store.
#[derive(Debug)]
pub struct PutRequest {
    /// The key to insert or update.
    pub key: String,
    /// The value to associate with the key.
    pub value: String,
}

/// A PUT response indicating success and the HLC timestamp.
#[derive(Debug)]
pub struct PutResponse {
    /// The key that was written.
    pub key: String,
    /// Whether the write succeeded (always `true` in this implementation).
    pub success: bool,
    /// The HLC timestamp at the time of the write.
    pub timestamp: u64,
}

/// Handle a PUT request by writing to the storage engine and incrementing the HLC.
///
/// # Arguments
///
/// * `state` - The shared database state.
/// * `key` - The key to insert or update.
/// * `value` - The value to store.
///
/// # Returns
///
/// A `PutResponse` indicating success and the current HLC timestamp.
pub fn handle_put(state: &super::DatabaseState, key: String, value: String) -> PutResponse {
    {
        let mut engine = state.engine.lock().expect("engine lock poisoned");
        engine.put(key.clone(), value);
    }
    let timestamp = {
        let mut hlc = state.hlc.lock().expect("hlc lock poisoned");
        hlc.now()
    };
    PutResponse {
        key,
        success: true,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_succeeds() {
        let state = super::super::DatabaseState::new(1);
        let resp = handle_put(&state, "k".into(), "v".into());
        assert!(resp.success);
        assert_eq!(resp.key, "k");
        assert!(resp.timestamp > 0);
    }

    #[test]
    fn test_put_overwrites() {
        let state = super::super::DatabaseState::new(1);
        handle_put(&state, "k".into(), "v1".into());
        handle_put(&state, "k".into(), "v2".into());

        let engine = state.engine.lock().unwrap();
        assert_eq!(engine.get("k"), Some("v2".into()));
    }
}
