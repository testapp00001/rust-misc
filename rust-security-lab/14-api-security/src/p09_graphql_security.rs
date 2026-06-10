//! # Lesson 09: GraphQL Security
//!
//! ## GraphQL's Unique Security Challenges
//!
//! GraphQL gives clients the power to query exactly what they need. But this
//! flexibility creates unique attack surfaces:
//!
//! ### 1. Query Depth Attack (Alias Bomb)
//!
//! ```graphql
//! query {
//!   user {
//!     friends {
//!       friends {
//!         friends {
//!           friends {  # ... 100 levels deep
//!             name
//!           }
//!         }
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! Each level multiplies the data fetched. Deep queries cause exponential
//! database load and memory usage.
//!
//! ### 2. Introspection Leak
//!
//! ```graphql
//! query {
//!   __schema {
//!     types {
//!       name
//!       fields {
//!         name
//!         type { name }
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! Introspection reveals your entire schema — every type, field, and argument.
//! Attackers use this to map your API.
//!
//! ### 3. Batch Query Attack
//!
//! ```graphql
//! [
//!   { query: "mutation { login(...) }" },
//!   { query: "mutation { login(...) }" },
//!   { query: "mutation { login(...) }" },
//!   # ... 1000 login attempts in one request
//! ]
//! ```
//!
//! Batching multiple queries in one HTTP request bypasses per-request rate limits.
//!
//! ## Defense
//!
//! 1. Limit query depth (typically 5-10 levels)
//! 2. Limit query complexity (assign costs to fields)
//! 3. Disable introspection in production
//! 4. Limit batch size (or disable batching)
//! 5. Rate limit by query cost, not just request count
//! 6. Use persisted queries (only allow pre-approved queries)

/// Exercise 1: Calculate the depth of a GraphQL query string.
///
/// Parse a simplified GraphQL query and calculate its maximum nesting depth.
///
/// We use a simplified format where:
/// - Fields are identifiers (letters, digits, underscores)
/// - Nesting is indicated by braces `{ }`
/// - Fragments, variables, arguments are ignored for simplicity
///
/// Examples:
/// - `{ user }` → depth 1
/// - `{ user { name } }` → depth 2
/// - `{ user { friends { name, email } } }` → depth 3
///
/// Hints:
/// - Track brace nesting level with a counter
/// - '{' increments, '}' decrements
/// - Track the maximum level seen
/// - Skip non-brace characters
pub fn calculate_query_depth(query: &str) -> usize {
    todo!("Calculate the maximum nesting depth of a GraphQL query")
}

/// Exercise 2: Calculate the complexity score of a GraphQL query.
///
/// Each field has a cost. Nested fields multiply their parent's cost.
///
/// Rules:
/// - Each top-level field has a base cost of 1
/// - Each level of nesting multiplies the cost by the number of sibling fields
/// - Total complexity = sum of all field costs
///
/// Simplified model:
/// - Split the query into a tree of fields
/// - Count the number of fields at each nesting level
/// - Multiply across levels
///
/// For this exercise, use a simpler metric:
/// - Count total field references in the query (every identifier that's a field name)
/// - This is a proxy for how many resolver calls will be made
///
/// Examples:
/// - `{ user }` → 1 field → complexity 1
/// - `{ user { name, email } }` → 3 fields → complexity 3
/// - `{ users { friends { name } } }` → 3 fields → complexity 3
///
/// Hints:
/// - Tokenize the query into identifiers and braces
/// - Count identifiers that appear to be field names (not inside arguments)
pub fn calculate_complexity(query: &str) -> usize {
    todo!("Calculate the complexity score of a GraphQL query")
}

/// Exercise 3: Check if a query contains introspection fields.
///
/// Introspection fields start with `__`:
/// - `__schema`
/// - `__type`
/// - `__typename`
///
/// Returns `true` if the query contains any introspection field.
///
/// This is used to disable introspection in production.
///
/// Hints:
/// - Search for `__schema`, `__type`, or `__typename` in the query string
/// - Use `.contains()` or scan for the pattern
/// - Be careful not to match inside string literals (but for this exercise,
///   a simple string search is acceptable)
pub fn contains_introspection(query: &str) -> bool {
    todo!("Check if a GraphQL query uses introspection")
}

/// Exercise 4: Validate a GraphQL query against security limits.
///
/// A one-stop validation function that checks:
/// 1. Query depth <= `max_depth`
/// 2. Query complexity <= `max_complexity`
/// 3. If `allow_introspection` is false, no introspection fields
/// 4. Query string length <= `max_query_length`
///
/// Returns:
/// - `Ok(())` if all checks pass
/// - `Err("query_too_long")` if the string is too long
/// - `Err("depth_exceeded")` if nesting is too deep
/// - `Err("complexity_exceeded")` if complexity is too high
/// - `Err("introspection_disabled")` if introspection is used when not allowed
///
/// Hints:
/// - Check string length first (cheapest)
/// - Then check introspection (string search)
/// - Then compute depth and complexity
pub fn validate_graphql_query(
    query: &str,
    max_depth: usize,
    max_complexity: usize,
    allow_introspection: bool,
    max_query_length: usize,
) -> Result<(), &'static str> {
    todo!("Validate a GraphQL query against all security limits")
}

/// Exercise 5: Count the number of operations in a GraphQL batch.
///
/// A GraphQL batch is a JSON array of query objects:
/// ```json
/// [
///   { "query": "{ user { name } }" },
///   { "query": "mutation { createPost(...) }" }
/// ]
/// ```
///
/// Given a JSON string representing a batch, return the number of operations.
///
/// Returns:
/// - `Ok(count)` if the JSON is valid and is an array
/// - `Err("invalid_batch")` if the JSON is not a valid array
///
/// Hints:
/// - Parse with `serde_json::from_str::<serde_json::Value>()`
/// - Check if it's an array
/// - Return the array length
/// - Handle the case where it's a single query (not an array)
pub fn count_batch_operations(batch_json: &str) -> Result<usize, &'static str> {
    todo!("Count operations in a GraphQL batch request")
}

/// Exercise 6: Implement a persisted query allowlist.
///
/// Instead of allowing arbitrary queries, only allow pre-approved queries
/// identified by their hash. This is a strong security measure.
///
/// The allowlist maps query hashes (SHA-256 of the query string) to query IDs.
///
/// Given:
/// - `allowlist`: HashMap of hash → query_id
/// - `query_hash`: the hash provided by the client (hex-encoded SHA-256)
///
/// Returns:
/// - `Ok(query_id)` if the hash is in the allowlist
/// - `Err("query_not_allowed")` if the hash is not found
///
/// This prevents arbitrary queries from being executed.
///
/// Hints:
/// - Look up the hash in the allowlist HashMap
/// - Return the query_id if found
pub fn validate_persisted_query(
    allowlist: &std::collections::HashMap<String, String>,
    query_hash: &str,
) -> Result<String, &'static str> {
    todo!("Validate a persisted query against the allowlist")
}

/// Exercise 7: Generate a query hash for the persisted query allowlist.
///
/// Given a query string, compute its SHA-256 hash as a hex string.
/// This is what the client sends instead of the full query.
///
/// Hints:
/// - Use `sha2::Sha256`
/// - Hash the query string bytes
/// - Return hex-encoded hash
pub fn hash_query(query: &str) -> String {
    todo!("Compute SHA-256 hash of a GraphQL query")
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
        assert_eq!(calculate_complexity("{ user { name, email } }"), 3);
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
        // This is not an array, so it's a single query
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
