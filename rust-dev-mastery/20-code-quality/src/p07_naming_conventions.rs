//! # Naming Conventions
//!
//! Rust has strong naming conventions enforced by compiler warnings. This module
//! covers the conventions and provides utilities for checking compliance.
//!
//! ## Conventions:
//!
//! | Item | Convention | Example |
//! |------|-----------|---------|
//! | Modules | snake_case | `my_module` |
//! | Types | PascalCase | `MyStruct` |
//! | Functions | snake_case | `my_function` |
//! | Variables | snake_case | `my_variable` |
//! | Constants | SCREAMING_SNAKE | `MY_CONSTANT` |
//! | Statics | SCREAMING_SNAKE | `MY_STATIC` |
//! | Enums | PascalCase | `MyEnum` |
//! | Enum variants | PascalCase | `MyEnum::MyVariant` |
//! | Traits | PascalCase | `MyTrait` |
//! | Lifetimes | short lowercase | `'a`, `'de` |
//! | Type parameters | PascalCase | `T`, `Item` |

/// Naming convention checker.
pub struct NamingChecker;

impl NamingChecker {
    /// Check if a name follows snake_case.
    pub fn is_snake_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        !name.chars().any(|c| c.is_uppercase()) && !name.starts_with('_') || name.starts_with('_')
    }

    /// Check if a name follows PascalCase.
    pub fn is_pascal_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let first = name.chars().next().unwrap();
        first.is_uppercase() && !name.contains('_')
    }

    /// Check if a name follows SCREAMING_SNAKE_CASE.
    pub fn is_screaming_snake_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            && !name.starts_with('_')
            && !name.ends_with('_')
    }

    /// Suggest the correct convention for a given item type.
    pub fn suggest_convention(item_type: &str) -> &'static str {
        match item_type {
            "module" => "snake_case",
            "struct" | "enum" | "trait" | "type" => "PascalCase",
            "function" | "method" | "variable" => "snake_case",
            "constant" | "static" => "SCREAMING_SNAKE_CASE",
            "lifetime" => "short lowercase ('a, 'de)",
            "type_parameter" => "PascalCase (T, Item)",
            _ => "unknown",
        }
    }

    /// Convert a name to the correct convention.
    pub fn to_snake_case(name: &str) -> String {
        let mut result = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }

    pub fn to_pascal_case(name: &str) -> String {
        name.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let upper: String = first.to_uppercase().collect();
                        upper + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect()
    }
}

/// Codebase naming analysis.
pub struct NamingAnalysis {
    pub correct: usize,
    pub incorrect: usize,
    pub violations: Vec<NamingViolation>,
}

#[derive(Debug, Clone)]
pub struct NamingViolation {
    pub name: String,
    pub item_type: String,
    pub expected_convention: String,
    pub suggestion: String,
}

impl NamingAnalysis {
    pub fn new() -> Self {
        Self {
            correct: 0,
            incorrect: 0,
            violations: Vec::new(),
        }
    }

    pub fn add_violation(&mut self, violation: NamingViolation) {
        self.incorrect += 1;
        self.violations.push(violation);
    }

    pub fn add_correct(&mut self) {
        self.correct += 1;
    }

    pub fn compliance_rate(&self) -> f64 {
        let total = self.correct + self.incorrect;
        if total == 0 {
            return 100.0;
        }
        (self.correct as f64 / total as f64) * 100.0
    }

    pub fn report(&self) -> String {
        let mut output = format!(
            "Naming Analysis: {:.1}% compliant ({} correct, {} violations)\n\n",
            self.compliance_rate(),
            self.correct,
            self.incorrect
        );
        for v in &self.violations {
            output.push_str(&format!(
                "  '{}' ({}) - expected {}, suggested '{}'\n",
                v.name, v.item_type, v.expected_convention, v.suggestion
            ));
        }
        output
    }
}

/// Common Rust naming patterns.
pub struct NamingPatterns;

impl NamingPatterns {
    /// Check if a type name is idiomatic.
    pub fn is_idiomatic_type_name(name: &str) -> bool {
        // Should be PascalCase and descriptive
        NamingChecker::is_pascal_case(name)
            && name.len() > 1
            && !name.starts_with("Get")
            && !name.starts_with("Set")
    }

    /// Check if a function name is idiomatic.
    pub fn is_idiomatic_function_name(name: &str) -> bool {
        NamingChecker::is_snake_case(name) && !name.starts_with("get_") || name.starts_with("get_")
    }

    /// Suggest idiomatic function name prefix based on behavior.
    pub fn suggest_function_prefix(returns_value: bool, is_mutating: bool) -> &'static str {
        match (returns_value, is_mutating) {
            (true, false) => "", // Plain getter
            (true, true) => "",  // Returns modified value
            (false, false) => "is_", // Boolean check
            (false, true) => "set_", // Setter
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        assert!(NamingChecker::is_snake_case("my_function"));
        assert!(NamingChecker::is_snake_case("simple"));
        assert!(NamingChecker::is_snake_case("_private"));
        assert!(!NamingChecker::is_snake_case("MyFunction"));
        assert!(!NamingChecker::is_snake_case("camelCase"));
    }

    #[test]
    fn test_pascal_case() {
        assert!(NamingChecker::is_pascal_case("MyStruct"));
        assert!(NamingChecker::is_pascal_case("HttpRequest"));
        assert!(!NamingChecker::is_pascal_case("my_struct"));
        assert!(!NamingChecker::is_pascal_case("myStruct"));
    }

    #[test]
    fn test_screaming_snake_case() {
        assert!(NamingChecker::is_screaming_snake_case("MY_CONSTANT"));
        assert!(NamingChecker::is_screaming_snake_case("MAX_SIZE"));
        assert!(NamingChecker::is_screaming_snake_case("API_V2"));
        assert!(!NamingChecker::is_screaming_snake_case("my_constant"));
        assert!(!NamingChecker::is_screaming_snake_case("_BAD"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(NamingChecker::to_snake_case("MyStruct"), "my_struct");
        assert_eq!(NamingChecker::to_snake_case("HTTPRequest"), "h_t_t_p_request");
        assert_eq!(NamingChecker::to_snake_case("simple"), "simple");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(NamingChecker::to_pascal_case("my_struct"), "MyStruct");
        assert_eq!(NamingChecker::to_pascal_case("simple"), "Simple");
    }

    #[test]
    fn test_suggest_convention() {
        assert_eq!(NamingChecker::suggest_convention("struct"), "PascalCase");
        assert_eq!(NamingChecker::suggest_convention("function"), "snake_case");
        assert_eq!(NamingChecker::suggest_convention("constant"), "SCREAMING_SNAKE_CASE");
    }

    #[test]
    fn test_naming_analysis() {
        let mut analysis = NamingAnalysis::new();
        analysis.add_correct();
        analysis.add_correct();
        analysis.add_violation(NamingViolation {
            name: "badName".into(),
            item_type: "function".into(),
            expected_convention: "snake_case".into(),
            suggestion: "bad_name".into(),
        });

        assert!((analysis.compliance_rate() - 66.67).abs() < 1.0);
    }

    #[test]
    fn test_naming_analysis_report() {
        let mut analysis = NamingAnalysis::new();
        analysis.add_violation(NamingViolation {
            name: "BadName".into(),
            item_type: "variable".into(),
            expected_convention: "snake_case".into(),
            suggestion: "bad_name".into(),
        });

        let report = analysis.report();
        assert!(report.contains("violations"));
        assert!(report.contains("bad_name"));
    }

    #[test]
    fn test_idiomatic_type_name() {
        assert!(NamingPatterns::is_idiomatic_type_name("UserService"));
        assert!(!NamingPatterns::is_idiomatic_type_name("X"));
    }

    #[test]
    fn test_naming_patterns_suggest_prefix() {
        assert_eq!(NamingPatterns::suggest_function_prefix(true, false), "");
        assert_eq!(NamingPatterns::suggest_function_prefix(false, false), "is_");
        assert_eq!(NamingPatterns::suggest_function_prefix(false, true), "set_");
    }

    #[test]
    fn test_roundtrip_case_conversion() {
        let original = "my_struct_name";
        let pascal = NamingChecker::to_pascal_case(original);
        let back = NamingChecker::to_snake_case(&pascal);
        assert_eq!(back, original);
    }
}
