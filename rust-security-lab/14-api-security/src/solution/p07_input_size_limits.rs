//! # Lesson 07: Input Size Limits (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

pub fn check_content_length(
    content_length: Option<&str>,
    max_size: u64,
) -> Result<u64, &'static str> {
    let cl_str = content_length.ok_or("missing_content_length")?;
    let size: u64 = cl_str.parse().map_err(|_| "invalid_content_length")?;

    if size > max_size {
        Err("payload_too_large")
    } else {
        Ok(size)
    }
}

pub fn validate_request_sizes(
    headers: &HashMap<String, String>,
    url_path: &str,
    max_body_size: u64,
    max_headers: usize,
    max_header_value_bytes: usize,
    max_url_length: usize,
) -> Result<(), &'static str> {
    // Check URL length first
    if url_path.len() > max_url_length {
        return Err("url_too_long");
    }

    // Check Content-Length
    check_content_length(headers.get("Content-Length").map(|s| s.as_str()), max_body_size)?;

    // Check header count
    if headers.len() > max_headers {
        return Err("too_many_headers");
    }

    // Check individual header value sizes
    for (_, value) in headers {
        if value.len() > max_header_value_bytes {
            return Err("header_value_too_large");
        }
    }

    Ok(())
}

pub fn check_streaming_size(chunks: &[&[u8]], max_size: u64) -> Result<u64, (u64, &'static str)> {
    let mut total: u64 = 0;
    for chunk in chunks {
        total += chunk.len() as u64;
        if total > max_size {
            return Err((total, "payload_too_large"));
        }
    }
    Ok(total)
}

pub fn validate_multipart_fields(
    fields: &HashMap<String, Vec<u8>>,
    max_fields: usize,
    max_field_size: usize,
    max_total_size: usize,
) -> Result<(), String> {
    if fields.len() > max_fields {
        return Err("too_many_fields".to_string());
    }

    let mut total = 0usize;
    for (name, content) in fields {
        if content.len() > max_field_size {
            return Err(format!("field_too_large:{}", name));
        }
        total += content.len();
    }

    if total > max_total_size {
        return Err("total_too_large".to_string());
    }

    Ok(())
}

pub fn validate_query_params(
    params: &[(&str, &str)],
    max_params: usize,
    max_name_length: usize,
    max_value_length: usize,
    max_total_length: usize,
) -> Result<(), &'static str> {
    if params.len() > max_params {
        return Err("too_many_params");
    }

    let mut total = 0usize;
    for (i, (name, value)) in params.iter().enumerate() {
        if name.len() > max_name_length {
            return Err("param_name_too_long");
        }
        if value.len() > max_value_length {
            return Err("param_value_too_long");
        }
        total += name.len() + value.len();
        if i > 0 {
            total += 1; // '&' separator
        }
        total += 1; // '=' separator
    }

    if total > max_total_length {
        return Err("query_too_long");
    }

    Ok(())
}

pub enum NestedNode {
    Leaf,
    Container(Vec<NestedNode>),
}

pub fn check_nesting_depth(node: &NestedNode, max_depth: usize) -> Result<usize, usize> {
    let depth = match node {
        NestedNode::Leaf => 0,
        NestedNode::Container(children) => {
            let max_child = children
                .iter()
                .map(|child| check_nesting_depth(child, max_depth))
                .map(|r| r.unwrap_or_else(|d| d))
                .max()
                .unwrap_or(0);
            1 + max_child
        }
    };

    if depth > max_depth {
        Err(depth)
    } else {
        Ok(depth)
    }
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
        assert_eq!(bytes, 10);
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
