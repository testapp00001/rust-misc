//! # Lesson 07: Input Size Limits
//!
//! ## Why Limit Input Size?
//!
//! Without size limits, an attacker can send massive requests to exhaust
//! server resources. This is a simple but effective denial-of-service attack.
//!
//! ## Attack: Payload Bomb
//!
//! ```text
//! Attacker sends a 1 GB JSON body:
//! curl -X POST https://api.example.com/data -d @huge_file.json
//!
//! Server tries to parse the entire body into memory → OOM crash
//! ```
//!
//! Even with streaming parsers, large bodies consume:
//! - Memory (buffering)
//! - CPU (parsing)
//! - Disk (temporary storage)
//! - Network bandwidth (denying legitimate users)
//!
//! ## Defense Layers
//!
//! 1. **Reverse proxy**: Nginx/Cloudflare rejects oversized requests early
//! 2. **Application middleware**: Check Content-Length before reading body
//! 3. **Streaming limits**: Limit how much data is read into memory
//! 4. **Per-field limits**: Limit individual fields (e.g., bio max 1000 chars)
//!
//! ## Other DoS Vectors
//!
//! Size limits aren't just about total bytes:
//! - **Header size**: Too many headers or very large header values
//! - **Query parameters**: Too many or too long query parameters
//! - **Multipart files**: Upload size limits per file and total
//! - **URL length**: Very long URLs consume memory in logging

use std::collections::HashMap;

/// Exercise 1: Check if a request body exceeds the size limit.
///
/// Given the `Content-Length` header value (as a string) and the maximum
/// allowed size in bytes, determine if the request should be rejected.
///
/// Returns:
/// - `Ok(size)` — the parsed size, which is within limits
/// - `Err("missing_content_length")` — Content-Length header is missing
/// - `Err("invalid_content_length")` — can't parse as a number
/// - `Err("payload_too_large")` — exceeds max_size
///
/// Hints:
/// - Parse the Content-Length string to `u64`
/// - Compare against `max_size`
pub fn check_content_length(
    content_length: Option<&str>,
    max_size: u64,
) -> Result<u64, &'static str> {
    todo!("Check Content-Length against maximum allowed size")
}

/// Exercise 2: Validate multiple request size constraints.
///
/// Check all size-related constraints for a request. Returns `Ok(())` if
/// all constraints pass, or `Err(description)` for the first failure.
///
/// Constraints to check (in order):
/// 1. Content-Length must be present and parseable
/// 2. Content-Length must not exceed `max_body_size`
/// 3. The number of headers must not exceed `max_headers`
/// 4. Each header value must not exceed `max_header_value_bytes`
/// 5. The URL path must not exceed `max_url_length` characters
///
/// Parameters:
/// - `headers`: map of header name → value
/// - `url_path`: the request URL path
/// - `max_body_size`: maximum body size in bytes
/// - `max_headers`: maximum number of headers
/// - `max_header_value_bytes`: maximum size of any single header value
/// - `max_url_length`: maximum URL path length
///
/// Hints:
/// - Check each constraint in order, return early on first failure
/// - Use `check_content_length` for the first check
/// - `headers.len()` for header count
/// - `.len()` on header value strings for byte count
pub fn validate_request_sizes(
    headers: &HashMap<String, String>,
    url_path: &str,
    max_body_size: u64,
    max_headers: usize,
    max_header_value_bytes: usize,
    max_url_length: usize,
) -> Result<(), &'static str> {
    todo!("Validate all request size constraints")
}

/// Exercise 3: Implement a streaming size limiter.
///
/// Simulate reading data in chunks with a cumulative size limit.
/// Given a vector of chunks (byte slices), read them one by one until
/// the total bytes exceed `max_size`.
///
/// Returns:
/// - `Ok(total_bytes)` if all chunks fit within the limit
/// - `Err((bytes_read, "payload_too_large"))` if the limit is exceeded
///
/// This simulates what a streaming body reader does — it doesn't need
/// the full body in memory, just tracks cumulative bytes.
///
/// Hints:
/// - Track cumulative bytes
/// - For each chunk, add its length
/// - If cumulative > max_size, return error with bytes_read
pub fn check_streaming_size(chunks: &[&[u8]], max_size: u64) -> Result<u64, (u64, &'static str)> {
    todo!("Check cumulative size of streaming data")
}

/// Exercise 4: Validate multipart form field sizes.
///
/// A multipart form has multiple fields. Each field has a name and content.
/// Validate:
/// 1. Total number of fields doesn't exceed `max_fields`
/// 2. No single field content exceeds `max_field_size`
/// 3. Total content across all fields doesn't exceed `max_total_size`
///
/// Returns:
/// - `Ok(())` if all limits are respected
/// - `Err("too_many_fields")` if field count exceeds limit
/// - `Err("field_too_large:{name}")` if a field exceeds its limit
/// - `Err("total_too_large")` if total exceeds limit
///
/// Hints:
/// - Check fields.len() first
/// - Iterate over fields, check each field's content length
/// - Sum all content lengths for total check
pub fn validate_multipart_fields(
    fields: &HashMap<String, Vec<u8>>,
    max_fields: usize,
    max_field_size: usize,
    max_total_size: usize,
) -> Result<(), String> {
    todo!("Validate multipart form field sizes")
}

/// Exercise 5: Implement a URL query parameter size validator.
///
/// Validate that query parameters don't exceed size limits:
/// 1. Total number of parameters doesn't exceed `max_params`
/// 2. Each parameter name doesn't exceed `max_name_length`
/// 3. Each parameter value doesn't exceed `max_value_length`
/// 4. Total query string size doesn't exceed `max_total_length`
///
/// Query parameters are provided as a Vec of (name, value) pairs.
///
/// Returns:
/// - `Ok(())` if valid
/// - `Err("too_many_params")` if parameter count exceeds limit
/// - `Err("param_name_too_long")` if any name exceeds limit
/// - `Err("param_value_too_long")` if any value exceeds limit
/// - `Err("query_too_long")` if total exceeds limit
///
/// Hints:
/// - Total size = sum of (name.len() + value.len() + 2) for each param (2 for '=' and '&')
/// - Check each constraint in order
pub fn validate_query_params(
    params: &[(&str, &str)],
    max_params: usize,
    max_name_length: usize,
    max_value_length: usize,
    max_total_length: usize,
) -> Result<(), &'static str> {
    todo!("Validate query parameter sizes")
}

/// Exercise 6: Implement a recursive depth/size validator for nested data.
///
/// Some data structures (JSON, XML) can be deeply nested. This function
/// validates that a tree structure doesn't exceed a maximum depth.
///
/// The tree is represented as a `NestedNode` enum. Count the maximum depth
/// of nesting.
///
/// Returns:
/// - `Ok(depth)` if depth is within `max_depth`
/// - `Err(depth)` if depth exceeds `max_depth`
///
/// Hints:
/// - A Leaf node has depth 0
/// - A Container node has depth = 1 + max(child depths)
/// - Use recursion or a stack-based approach
pub enum NestedNode {
    Leaf,
    Container(Vec<NestedNode>),
}

pub fn check_nesting_depth(node: &NestedNode, max_depth: usize) -> Result<usize, usize> {
    todo!("Check the nesting depth of a tree structure")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_length_valid() {
        assert_eq!(check_content_length(Some("1024"), 2048).unwrap(), 1024);
    }

    #[test]
    fn test_content_length_exceeds() {
        assert_eq!(check_content_length(Some("4096"), 2048).unwrap_err(), "payload_too_large");
    }

    #[test]
    fn test_content_length_missing() {
        assert_eq!(check_content_length(None, 2048).unwrap_err(), "missing_content_length");
    }

    #[test]
    fn test_content_length_invalid() {
        assert_eq!(check_content_length(Some("not_a_number"), 2048).unwrap_err(), "invalid_content_length");
    }

    #[test]
    fn test_validate_request_sizes_valid() {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), "100".to_string());
        headers.insert("Host".to_string(), "example.com".to_string());
        assert!(validate_request_sizes(&headers, "/api/data", 1024, 100, 8192, 2048).is_ok());
    }

    #[test]
    fn test_validate_request_sizes_too_many_headers() {
        let mut headers = HashMap::new();
        for i in 0..10 {
            headers.insert(format!("Header-{}", i), "value".to_string());
        }
        headers.insert("Content-Length".to_string(), "100".to_string());
        assert_eq!(
            validate_request_sizes(&headers, "/api", 1024, 5, 8192, 2048).unwrap_err(),
            "too_many_headers"
        );
    }

    #[test]
    fn test_validate_request_sizes_url_too_long() {
        let headers = HashMap::new();
        let long_url = "/".to_string() + &"a".repeat(5000);
        assert_eq!(
            validate_request_sizes(&headers, &long_url, 1024, 100, 8192, 100).unwrap_err(),
            "url_too_long"
        );
    }

    #[test]
    fn test_streaming_size_ok() {
        let chunks: Vec<&[u8]> = vec![b"hello", b" ", b"world"];
        assert_eq!(check_streaming_size(&chunks, 20).unwrap(), 11);
    }

    #[test]
    fn test_streaming_size_exceeded() {
        let chunks: Vec<&[u8]> = vec![b"hello", b"world", b"test"];
        let result = check_streaming_size(&chunks, 5);
        assert!(result.is_err());
        let (bytes, _) = result.unwrap_err();
        assert_eq!(bytes, 10); // "hello" (5) + "world" (5) = 10 > 5
    }

    #[test]
    fn test_multipart_valid() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), b"Alice".to_vec());
        fields.insert("bio".to_string(), b"Hello world".to_vec());
        assert!(validate_multipart_fields(&fields, 10, 100, 1000).is_ok());
    }

    #[test]
    fn test_multipart_too_many_fields() {
        let mut fields = HashMap::new();
        for i in 0..20 {
            fields.insert(format!("field_{}", i), b"data".to_vec());
        }
        assert_eq!(validate_multipart_fields(&fields, 5, 100, 1000).unwrap_err(), "too_many_fields");
    }

    #[test]
    fn test_multipart_field_too_large() {
        let mut fields = HashMap::new();
        fields.insert("big".to_string(), vec![0u8; 1000]);
        let result = validate_multipart_fields(&fields, 10, 100, 10000);
        assert!(result.unwrap_err().starts_with("field_too_large:"));
    }

    #[test]
    fn test_query_params_valid() {
        let params = vec![("name", "alice"), ("page", "1")];
        assert!(validate_query_params(&params, 10, 50, 100, 1000).is_ok());
    }

    #[test]
    fn test_query_params_too_many() {
        let params: Vec<(&str, &str)> = (0..20).map(|i| ("key", "val")).collect();
        assert_eq!(validate_query_params(&params, 5, 50, 100, 10000).unwrap_err(), "too_many_params");
    }

    #[test]
    fn test_query_params_name_too_long() {
        let long_name = "a".repeat(100);
        let params = vec![(long_name.as_str(), "val")];
        assert_eq!(validate_query_params(&params, 10, 10, 100, 10000).unwrap_err(), "param_name_too_long");
    }

    #[test]
    fn test_nesting_depth_shallow() {
        let node = NestedNode::Leaf;
        assert_eq!(check_nesting_depth(&node, 10).unwrap(), 0);
    }

    #[test]
    fn test_nesting_depth_nested() {
        let node = NestedNode::Container(vec![
            NestedNode::Leaf,
            NestedNode::Container(vec![
                NestedNode::Leaf,
                NestedNode::Container(vec![NestedNode::Leaf]),
            ]),
        ]);
        assert_eq!(check_nesting_depth(&node, 10).unwrap(), 3);
    }

    #[test]
    fn test_nesting_depth_exceeded() {
        let node = NestedNode::Container(vec![
            NestedNode::Container(vec![NestedNode::Leaf]),
        ]);
        assert_eq!(check_nesting_depth(&node, 1).unwrap_err(), 2);
    }
}
