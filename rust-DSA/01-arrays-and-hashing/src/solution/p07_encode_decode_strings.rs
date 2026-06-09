// ============================================================================
// Problem: Encode and Decode Strings (LeetCode #271)
// ============================================================================
// Design an algorithm to encode a list of strings to a single string, and
// decode that string back to the original list of strings.
//
// Example:
//   Input:  ["Hello", "World"]
//   Encoded: "5#Hello5#World"
//   Decoded: ["Hello", "World"]
//
// ============================================================================
// APPROACH: Length-Prefix Encoding
// ============================================================================
//
// Encoding strategy: "<length>#<string>" for each string.
//   "Hello" → "5#Hello"
//   "World" → "5#World"
//   Full:    "5#Hello5#World"
//
// Decoding: Read the length before '#', then read that many characters.
//
// Why this works: The '#' delimiter separates the length from the content.
// We know exactly how many characters to read after '#'.
//
// Edge cases: Empty strings, strings containing '#', special characters.
// ============================================================================

pub fn encode(strs: Vec<&str>) -> String {
    let mut result = String::new();
    for s in strs {
        result.push_str(&format!("{}#{}", s.len(), s));
    }
    result
}

pub fn decode(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Find the '#' delimiter
        let mut j = i;
        while bytes[j] != b'#' {
            j += 1;
        }
        // Parse the length
        let len: usize = s[i..j].parse().unwrap();
        // Extract the string
        let start = j + 1;
        let end = start + len;
        result.push(s[start..end].to_string());
        i = end;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let strs = vec!["Hello", "World"];
        let encoded = encode(strs);
        assert_eq!(decode(&encoded), vec!["Hello", "World"]);
    }

    #[test]
    fn test_empty_string() {
        let strs = vec!["", "abc", ""];
        let encoded = encode(strs);
        assert_eq!(decode(&encoded), vec!["", "abc", ""]);
    }

    #[test]
    fn test_special_chars() {
        let strs = vec!["#", "##", "###"];
        let encoded = encode(strs);
        assert_eq!(decode(&encoded), vec!["#", "##", "###"]);
    }

    #[test]
    fn test_single_empty() {
        let strs = vec![""];
        let encoded = encode(strs);
        assert_eq!(decode(&encoded), vec![""]);
    }

    #[test]
    fn test_long_strings() {
        let long = "a".repeat(1000);
        let strs = vec![long.as_str()];
        let encoded = encode(strs);
        assert_eq!(decode(&encoded), vec![long]);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = encode(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}