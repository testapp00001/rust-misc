//! GET handler for reading key-value pairs from the database.
//!
//! Retrieves a value from the storage engine and returns it along with the current
//! HLC timestamp. The timestamp enables clients to perform causal consistency checks.

/// A GET request specifying the key to look up.
#[derive(Debug)]
pub struct GetRequest {
    /// The key to retrieve.
    pub key: String,
}

/// A GET response containing the looked-up value and the HLC timestamp.
#[derive(Debug)]
pub struct GetResponse {
    /// The key that was looked up.
    pub key: String,
    /// The value, or `None` if the key does not exist.
    pub value: Option<String>,
    /// The HLC timestamp at the time of the read.
    pub timestamp: u64,
}

/// Handle a GET request by reading from the storage engine and recording the HLC time.
///
/// # Arguments
///
/// * `state` - The shared database state.
/// * `key` - The key to look up.
///
/// # Returns
///
/// A `GetResponse` with the value (if any) and the current HLC timestamp.
pub fn handle_get(state: &super::DatabaseState, key: &str) -> GetResponse {
    let value = {
        let engine = state.engine.lock().expect("engine lock poisoned");
        engine.get(key)
    };
    let timestamp = {
        let mut hlc = state.hlc.lock().expect("hlc lock poisoned");
        hlc.now()
    };
    GetResponse {
        key: key.to_string(),
        value,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_existing_key() {
        let state = super::super::DatabaseState::new(1);
        {
            let mut engine = state.engine.lock().unwrap();
            engine.put("mykey".into(), "myval".into());
        }
        let resp = handle_get(&state, "mykey");
        assert_eq!(resp.value, Some("myval".into()));
        assert_eq!(resp.key, "mykey");
        assert!(resp.timestamp > 0);
    }

    #[test]
    fn test_get_nonexistent_key() {
        let state = super::super::DatabaseState::new(1);
        let resp = handle_get(&state, "missing");
        assert_eq!(resp.value, None);
    }
}
