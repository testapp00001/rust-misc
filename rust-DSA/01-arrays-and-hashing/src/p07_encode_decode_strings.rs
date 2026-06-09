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
    todo!("Implement encode")
}

pub fn decode(s: &str) -> Vec<String> {
    todo!("Implement decode")
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