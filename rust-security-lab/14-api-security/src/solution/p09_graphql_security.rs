//! # Lesson 09: GraphQL Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};

pub fn calculate_query_depth(query: &str) -> usize {
    let mut max_depth = 0usize;
    let mut current_depth = 0usize;

    for ch in query.chars() {
        match ch {
            '{' => {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            }
            '}' => {
                current_depth = current_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    max_depth
}

pub fn calculate_complexity(query: &str) -> usize {
    // Count field names: identifiers that appear as field references
    // Simplified: count words that are valid identifiers (not inside string literals)
    let mut count = 0usize;
    let mut in_string = false;
    let mut current_word = String::new();
    let mut depth = 0usize;

    for ch in query.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                if !in_string && !current_word.is_empty() {
                    current_word.clear();
                }
            }
            _ if in_string => {}
            '{' => {
                depth += 1;
                if !current_word.is_empty() {
                    count += 1;
                    current_word.clear();
                }
            }
            '}' => {
                if !current_word.is_empty() {
                    count += 1;
                    current_word.clear();
                }
                depth = depth.saturating_sub(1);
            }
            ',' | '(' => {
                if !current_word.is_empty() {
                    count += 1;
                    current_word.clear();
                }
            }
            c if c.is_alphanumeric() || c == '_' => {
                current_word.push(c);
            }
            _ => {
                if !current_word.is_empty() {
                    count += 1;
                    current_word.clear();
                }
            }
        }
    }

    if !current_word.is_empty() {
        count += 1;
    }

    count
}

pub fn contains_introspection(query: &str) -> bool {
    query.contains("__schema") || query.contains("__type") || query.contains("__typename")
}

pub fn validate_graphql_query(
    query: &str,
    max_depth: usize,
    max_complexity: usize,
    allow_introspection: bool,
    max_query_length: usize,
) -> Result<(), &'static str> {
    if query.len() > max_query_length {
        return Err("query_too_long");
    }

    if !allow_introspection && contains_introspection(query) {
        return Err("introspection_disabled");
    }

    let depth = calculate_query_depth(query);
    if depth > max_depth {
        return Err("depth_exceeded");
    }

    let complexity = calculate_complexity(query);
    if complexity > max_complexity {
        return Err("complexity_exceeded");
    }

    Ok(())
}

pub fn count_batch_operations(batch_json: &str) -> Result<usize, &'static str> {
    let value: serde_json::Value = serde_json::from_str(batch_json)
        .map_err(|_| "invalid_batch")?;

    match value {
        serde_json::Value::Array(arr) => Ok(arr.len()),
        // A single object is a single query
        serde_json::Value::Object(_) => Ok(1),
        _ => Err("invalid_batch"),
    }
}

pub fn validate_persisted_query(
    allowlist: &std::collections::HashMap<String, String>,
    query_hash: &str,
) -> Result<String, &'static str> {
    allowlist.get(query_hash)
        .cloned()
        .ok_or("query_not_allowed")
}

pub fn hash_query(query: &str) -> String {
    let hash = Sha256::digest(query.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_simple() {
        assert_eq!(calculate_query_depth("{ user }"), 1);
    }

    #[test]
    fn test_depth_nested() {
        assert_eq!(calculate_query_depth("{ user { name } }"), 2);
    }

    #[test]
    fn test_depth_deep() {
        assert_eq!(calculate_query_depth("{ a { b { c { d { e } } } } }"), 5);
    }

    #[test]
    fn test_depth_with_multiple_fields() {
        assert_eq!(calculate_query_depth("{ user { name, email }, posts { title } }"), 2);
    }

    #[test]
    fn test_complexity_simple() {
        assert_eq!(calculate_complexity("{ user }"), 1);
    }

    #[test]
    fn test_complexity_multiple_fields() {
        let c = calculate_complexity("{ user { name, email } }");
        assert!(c >= 3, "Expected at least 3, got {}", c);
    }

    #[test]
    fn test_introspection_detected() {
        assert!(contains_introspection("{ __schema { types { name } } }"));
        assert!(contains_introspection("{ __type(name: \"User\") { name } }"));
    }

    #[test]
    fn test_introspection_not_present() {
        assert!(!contains_introspection("{ user { name, email } }"));
    }

    #[test]
    fn test_validate_query_valid() {
        let query = "{ user { name, email } }";
        assert!(validate_graphql_query(query, 10, 100, false, 1000).is_ok());
    }

    #[test]
    fn test_validate_query_too_long() {
        let query = "{ user }";
        assert_eq!(
            validate_graphql_query(query, 10, 100, false, 3).unwrap_err(),
            "query_too_long"
        );
    }

    #[test]
    fn test_validate_query_depth_exceeded() {
        let query = "{ a { b { c } } }";
        assert_eq!(
            validate_graphql_query(query, 2, 100, false, 1000).unwrap_err(),
            "depth_exceeded"
        );
    }

    #[test]
    fn test_validate_query_introspection_blocked() {
        let query = "{ __schema { types { name } } }";
        assert_eq!(
            validate_graphql_query(query, 10, 100, false, 1000).unwrap_err(),
            "introspection_disabled"
        );
    }

    #[test]
    fn test_validate_query_introspection_allowed() {
        let query = "{ __schema { types { name } } }";
        assert!(validate_graphql_query(query, 10, 100, true, 1000).is_ok());
    }

    #[test]
    fn test_count_batch_operations() {
        let batch = r#"[
            { "query": "{ user { name } }" },
            { "query": "mutation { createPost }" },
            { "query": "{ posts { title } }" }
        ]"#;
        assert_eq!(count_batch_operations(batch).unwrap(), 3);
    }

    #[test]
    fn test_count_batch_single() {
        let single = r#"{ "query": "{ user { name } }" }"#;
        assert_eq!(count_batch_operations(single).unwrap(), 1);
    }

    #[test]
    fn test_count_batch_invalid() {
        assert_eq!(count_batch_operations("not json").unwrap_err(), "invalid_batch");
    }

    #[test]
    fn test_persisted_query_allowed() {
        let mut allowlist = std::collections::HashMap::new();
        let hash = hash_query("{ user { name } }");
        allowlist.insert(hash.clone(), "q1".to_string());
        assert_eq!(validate_persisted_query(&allowlist, &hash).unwrap(), "q1");
    }

    #[test]
    fn test_persisted_query_not_allowed() {
        let allowlist = std::collections::HashMap::new();
        assert_eq!(
            validate_persisted_query(&allowlist, "deadbeef").unwrap_err(),
            "query_not_allowed"
        );
    }

    #[test]
    fn test_hash_query_deterministic() {
        let h1 = hash_query("{ user { name } }");
        let h2 = hash_query("{ user { name } }");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_query_is_sha256() {
        let h = hash_query("test");
        assert_eq!(h.len(), 64);
        assert!(hex::decode(&h).is_ok());
    }
}
