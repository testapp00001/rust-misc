//! # Lesson 04: Typosquatting Attack Detection (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Calculate the Levenshtein edit distance between two strings.
///
/// Uses dynamic programming with a 2D matrix.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();

    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    // Base cases
    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=n {
        for j in 1..=m {
            if a_chars[i - 1] == b_chars[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + dp[i - 1][j - 1]
                    .min(dp[i][j - 1])
                    .min(dp[i - 1][j]);
            }
        }
    }

    dp[n][m]
}

/// Calculate a normalized similarity score between two strings.
///
/// Returns 0.0 to 1.0 (1.0 = identical).
pub fn similarity_score(a: &str, b: &str) -> f64 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0;
    }
    let distance = levenshtein_distance(a, b);
    1.0 - (distance as f64 / max_len as f64)
}

/// Normalize a string by replacing common homoglyphs.
fn normalize_homoglyphs(s: &str) -> String {
    let mut result = s.to_lowercase();
    // Replace multi-char first (rn -> m), then single chars
    result = result.replace("rn", "m");
    result = result.replace("vv", "w");
    result = result.replace('0', "o");
    result = result.replace('1', "l");
    result
}

/// Check if two names are homoglyphs (visually similar characters).
pub fn are_homoglyphs(a: &str, b: &str) -> bool {
    let norm_a = normalize_homoglyphs(a);
    let norm_b = normalize_homoglyphs(b);
    norm_a == norm_b && a != b
}

/// Check if two names differ by exactly one adjacent character transposition.
pub fn is_transposition(a: &str, b: &str) -> bool {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.len() != b_chars.len() || a_chars.len() < 2 {
        return false;
    }

    let mut diff_positions: Vec<usize> = Vec::new();
    for (i, (ac, bc)) in a_chars.iter().zip(b_chars.iter()).enumerate() {
        if ac != bc {
            diff_positions.push(i);
        }
    }

    // Must differ in exactly 2 adjacent positions, and the chars must be swapped
    if diff_positions.len() != 2 {
        return false;
    }
    let i = diff_positions[0];
    let j = diff_positions[1];
    if j != i + 1 {
        return false;
    }
    a_chars[i] == b_chars[j] && a_chars[j] == b_chars[i]
}

/// Detect if a package name is a potential typosquat of a known package.
///
/// Returns the name of the closest matching known package if flagged, else None.
pub fn detect_typosquat(
    package_name: &str,
    known_packages: &[&str],
) -> Option<String> {
    let mut best_match: Option<(String, f64)> = None;

    for &known in known_packages {
        // Skip exact matches
        if package_name == known {
            return None;
        }

        let score = similarity_score(package_name, known);

        // Check homoglyphs
        if are_homoglyphs(package_name, known) {
            return Some(known.to_string());
        }

        // Check transposition
        if is_transposition(package_name, known) {
            return Some(known.to_string());
        }

        // Check similarity threshold
        if score >= 0.8 {
            match &best_match {
                None => best_match = Some((known.to_string(), score)),
                Some((_, best_score)) => {
                    if score > *best_score {
                        best_match = Some((known.to_string(), score));
                    }
                }
            }
        }
    }

    best_match.map(|(name, _)| name)
}

/// Score a list of package names for typosquatting risk.
pub fn score_packages(
    packages: &[&str],
    known_packages: &[&str],
) -> Vec<(String, f64)> {
    packages
        .iter()
        .filter_map(|&pkg| {
            if detect_typosquat(pkg, known_packages).is_some() {
                let best_score = known_packages
                    .iter()
                    .filter(|&&k| k != pkg)
                    .map(|&k| similarity_score(pkg, k))
                    .fold(0.0_f64, f64::max);
                Some((pkg.to_string(), best_score))
            } else {
                None
            }
        })
        .collect()
}

/// Generate a warning report for detected typosquats.
pub fn generate_typosquat_report(
    packages: &[&str],
    known_packages: &[&str],
) -> String {
    let detections: Vec<String> = packages
        .iter()
        .filter_map(|&pkg| {
            detect_typosquat(pkg, known_packages).map(|legitimate| {
                let score = similarity_score(pkg, &legitimate);
                format!(
                    "WARNING: '{}' may be typosquat of '{}' (similarity: {:.2})",
                    pkg, legitimate, score
                )
            })
        })
        .collect();

    if detections.is_empty() {
        "No typosquats detected.".to_string()
    } else {
        detections.join("\n")
    }
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
