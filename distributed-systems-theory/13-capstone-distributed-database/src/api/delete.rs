//! DELETE handler for removing key-value pairs from the database.
//!
//! Removes the key from the storage engine and reports whether the key existed
//! before deletion. The HLC timestamp is included for causal tracking.

/// A DELETE request specifying the key to remove.
#[derive(Debug)]
pub struct DeleteRequest {
    /// The key to delete.
    pub key: String,
}

/// A DELETE response indicating whether the key existed and the HLC timestamp.
#[derive(Debug)]
pub struct DeleteResponse {
    /// The key that was targeted.
    pub key: String,
    /// `true` if the key existed before the delete; `false` otherwise.
    pub existed: bool,
    /// The HLC timestamp at the time of the delete.
    pub timestamp: u64,
}

/// Handle a DELETE request by removing the key from the storage engine.
///
/// # Arguments
///
/// * `state` - The shared database state.
/// * `key` - The key to delete.
///
/// # Returns
///
/// A `DeleteResponse` indicating whether the key existed and the current HLC timestamp.
pub fn handle_delete(state: &super::DatabaseState, key: &str) -> DeleteResponse {
    let existed = {
        let mut engine = state.engine.lock().expect("engine lock poisoned");
        engine.delete(key)
    };
    let timestamp = {
        let mut hlc = state.hlc.lock().expect("hlc lock poisoned");
        hlc.now()
    };
    DeleteResponse {
        key: key.to_string(),
        existed,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_existing_key() {
        let state = super::super::DatabaseState::new(1);
        {
            let mut engine = state.engine.lock().unwrap();
            engine.put("k".into(), "v".into());
        }
        let resp = handle_delete(&state, "k");
        assert!(resp.existed);
        assert_eq!(resp.key, "k");

        // Confirm it is gone.
        let engine = state.engine.lock().unwrap();
        assert_eq!(engine.get("k"), None);
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let state = super::super::DatabaseState::new(1);
        let resp = handle_delete(&state, "missing");
        assert!(!resp.existed);
    }
}
