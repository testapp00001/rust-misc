//! # Lesson 04: Typosquatting Attack Detection
//!
//! ## What is Typosquatting?
//!
//! Typosquatting in software supply chains involves publishing malicious packages
//! with names very similar to popular ones. Developers who make a typo when adding
//! a dependency may accidentally install the malicious package.
//!
//! ## Real-World Examples
//!
//! | Legitimate | Typosquat | Attack |
//! |------------|-----------|--------|
//! | `requests` | `request` | Python, 2017 |
//! | `cross-env` | `crossenv` | npm, 2017 |
//! | `eslint-scope` | `eslint-scope` (compromised) | npm, 2018 |
//! | `ua-parser-js` | Various misspellings | npm, 2021 |
//!
//! ## Detection Strategies
//!
//! 1. **Edit distance**: Calculate Levenshtein distance between package names
//! 2. **Prefix/suffix matching**: Check for common typo patterns (doubled letters, swaps)
//! 3. **Known package matching**: Compare against a list of popular packages
//! 4. **Character substitution**: Detect homoglyph attacks (e.g., `rn` vs `m`)
//!
//! ## Defense: Name Similarity Analysis
//!
//! In this lesson, you will implement edit distance calculation and a typosquatting
//! detector that flags suspicious package names.

/// Exercise 1: Calculate the Levenshtein edit distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string into another.
///
/// Hints:
/// - Use dynamic programming with a 2D matrix
/// - Matrix dimensions: (len_a + 1) x (len_b + 1)
/// - Base cases: distance from empty string to string of length n is n
/// - Recurrence: if chars match, take diagonal; otherwise, 1 + min(top, left, diagonal)
///
/// This is a classic DP problem — the matrix approach is O(n*m) time and space.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    todo!("Calculate Levenshtein edit distance")
}

/// Exercise 2: Calculate a normalized similarity score between two strings.
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
///
/// Formula: 1.0 - (distance as f64 / max(len_a, len_b) as f64)
/// If both strings are empty, return 1.0.
///
/// Hints:
/// - Use `levenshtein_distance` from Exercise 1
/// - Handle the edge case where both strings are empty
pub fn similarity_score(a: &str, b: &str) -> f64 {
    todo!("Calculate normalized similarity score")
}

/// Exercise 3: Check if two package names are homoglyphs (visually similar characters).
///
/// Common homoglyph pairs (lowercase):
/// - 'o' and '0' (letter o vs zero)
/// - 'l' and '1' (lowercase L vs one)
/// - 'i' and '1' (lowercase i vs one)
/// - 'm' and 'rn' (m vs r+n)
/// - 'vv' and 'w'
///
/// Normalize both strings by replacing right-side chars with left-side equivalents,
/// then check if the normalized forms are equal.
///
/// Hints:
/// - Create a normalization function that replaces homoglyphs
/// - Replace: "0" -> "o", "1" -> "l", "rn" -> "m", "vv" -> "w"
/// - Compare normalized forms
pub fn are_homoglyphs(a: &str, b: &str) -> bool {
    todo!("Check if two names are homoglyphs")
}

/// Exercise 4: Check for common typo patterns between two names.
///
/// Common patterns:
/// - **Transposition**: Two adjacent characters swapped (e.g., "sered" vs "serde")
/// - **Missing letter**: One character missing (e.g., "sere" vs "serde")
/// - **Doubled letter**: An extra character that's a repeat (e.g., "serdde" vs "serde")
/// - **Adjacent key**: A character replaced by an adjacent keyboard key
///
/// For this exercise, check if the names differ by exactly one transposition.
/// Two strings are transpositions if they have the same length and differ
/// in exactly two adjacent positions.
///
/// Hints:
/// - Check lengths are equal
/// - Find positions where characters differ
/// - Check exactly 2 differences at positions i and i+1
/// - Verify a[i] == b[i+1] and a[i+1] == b[i]
pub fn is_transposition(a: &str, b: &str) -> bool {
    todo!("Check for adjacent character transposition")
}

/// Exercise 5: Detect if a package name is a potential typosquat of a known package.
///
/// A package is flagged if ANY of these conditions are true:
/// - Similarity score >= 0.8 with any known package (and not identical)
/// - Is a homoglyph of any known package
/// - Is a transposition of any known package
///
/// Return the name of the closest matching known package if flagged, else None.
///
/// Hints:
/// - Iterate over known_packages
/// - Check each condition
/// - Track the highest similarity score for the "closest" match
pub fn detect_typosquat(
    package_name: &str,
    known_packages: &[&str],
) -> Option<String> {
    todo!("Detect if a package name is a typosquat")
}

/// Exercise 6: Score a list of package names for typosquatting risk.
///
/// For each package, compute a risk score:
/// - 0.0 if not a typosquat
/// - similarity_score if it's a typosquat (higher = more suspicious)
///
/// Return Vec of (package_name, risk_score) for packages with risk > 0.0.
///
/// Hints:
/// - Use `detect_typosquat` and `similarity_score`
/// - Filter out packages with 0.0 risk
pub fn score_packages(
    packages: &[&str],
    known_packages: &[&str],
) -> Vec<(String, f64)> {
    todo!("Score packages for typosquatting risk")
}

/// Exercise 7: Generate a warning report for detected typosquats.
///
/// Format each detection as:
/// "WARNING: '{suspicious}' may be typosquat of '{legitimate}' (similarity: {score:.2})"
///
/// Return "No typosquats detected." if empty.
pub fn generate_typosquat_report(
    packages: &[&str],
    known_packages: &[&str],
) -> String {
    todo!("Generate typosquat warning report")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_single_insert() {
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
    }

    #[test]
    fn test_levenshtein_single_delete() {
        assert_eq!(levenshtein_distance("abcd", "abc"), 1);
    }

    #[test]
    fn test_levenshtein_single_substitute() {
        assert_eq!(levenshtein_distance("abc", "axc"), 1);
    }

    #[test]
    fn test_levenshtein_classic() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_similarity_identical() {
        let score = similarity_score("serde", "serde");
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_similar() {
        let score = similarity_score("serde", "sre");
        assert!(score > 0.5);
        assert!(score < 1.0);
    }

    #[test]
    fn test_similarity_completely_different() {
        let score = similarity_score("abc", "xyz");
        assert!(score < 0.5);
    }

    #[test]
    fn test_homoglyphs_rn_m() {
        assert!(are_homoglyphs("forrn", "form"));
    }

    #[test]
    fn test_homoglyphs_0_o() {
        assert!(are_homoglyphs("hell0", "hello"));
    }

    #[test]
    fn test_homoglyphs_not_similar() {
        assert!(!are_homoglyphs("serde", "tokio"));
    }

    #[test]
    fn test_transposition_valid() {
        assert!(is_transposition("sered", "serde"));
        assert!(is_transposition("ab", "ba"));
    }

    #[test]
    fn test_transposition_invalid() {
        assert!(!is_transposition("serde", "serde"));
        assert!(!is_transposition("abcd", "adbc")); // non-adjacent swap (b and d)
    }

    #[test]
    fn test_detect_typosquat_found() {
        let known = &["serde", "tokio", "reqwest"];
        let result = detect_typosquat("srede", known);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "serde");
    }

    #[test]
    fn test_detect_typosquat_exact_match() {
        let known = &["serde", "tokio"];
        let result = detect_typosquat("serde", known);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_typosquat_unrelated() {
        let known = &["serde", "tokio"];
        let result = detect_typosquat("zzzzzz", known);
        assert!(result.is_none());
    }

    #[test]
    fn test_score_packages() {
        let packages = &["serde", "srede", "tokio"];
        let known = &["serde", "tokio"];
        let scores = score_packages(packages, known);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].0, "srede");
        assert!(scores[0].1 > 0.0);
    }

    #[test]
    fn test_typosquat_report() {
        let packages = &["serde", "srede"];
        let known = &["serde", "tokio"];
        let report = generate_typosquat_report(packages, known);
        assert!(report.contains("WARNING"));
        assert!(report.contains("srede"));
    }

    #[test]
    fn test_typosquat_report_clean() {
        let packages = &["serde", "tokio"];
        let known = &["serde", "tokio"];
        let report = generate_typosquat_report(packages, known);
        assert_eq!(report, "No typosquats detected.");
    }
}
