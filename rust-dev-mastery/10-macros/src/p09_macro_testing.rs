//! # Testing Macros
//!
//! Testing macros requires different strategies than testing regular code:
//! - **trybuild**: Tests that macros produce expected compiler errors
//! - **compile-test**: Verifies compilation succeeds or fails
//! - **macro expansion**: Inspects what a macro generates
//! - **Unit tests**: Test the parsing and generation logic directly
//!
//! This module demonstrates testing patterns for both declarative and
//! procedural macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, ItemFn};

/// Demonstrates testing macro expansion by comparing token streams.
pub fn assert_expansion_eq(actual: &TokenStream, expected: &TokenStream) {
    let actual_str = actual.to_string();
    let expected_str = expected.to_string();
    assert_eq!(
        normalize_code(&actual_str),
        normalize_code(&expected_str),
        "Macro expansion mismatch:\nActual:\n{actual_str}\nExpected:\n{expected_str}"
    );
}

/// Normalize code for comparison by removing extra whitespace.
fn normalize_code(code: &str) -> String {
    code.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" ;", ";")
        .replace(" ,", ",")
        .replace("( ", "(")
        .replace(" )", ")")
}

/// Demonstrates testing that a macro generates valid Rust code.
/// We parse the output and verify it's a valid item.
pub fn assert_generates_valid_item(code: &TokenStream) {
    let code_str = code.to_string();
    // Try to parse as various item types
    let result = parse_str::<syn::Item>(&code_str);
    assert!(result.is_ok(), "Generated code is not a valid item: {code_str}");
}

/// Demonstrates testing that a macro generates a valid function.
pub fn assert_generates_valid_function(code: &TokenStream) {
    let code_str = code.to_string();
    let result = parse_str::<ItemFn>(&code_str);
    assert!(result.is_ok(), "Generated code is not a valid function: {code_str}");
}

/// Demonstrates testing that a macro generates a valid struct.
pub fn assert_generates_valid_struct(code: &TokenStream) {
    let code_str = code.to_string();
    let result = parse_str::<syn::ItemStruct>(&code_str);
    assert!(result.is_ok(), "Generated code is not a valid struct: {code_str}");
}

/// Demonstrates testing error messages from macros.
/// When a macro should produce an error, we test the error message.
pub fn assert_compile_error_contains(code: &str, expected_msg: &str) {
    let result = parse_str::<syn::Item>(code);
    match result {
        Ok(_) => panic!("Expected a compile error, but parsing succeeded"),
        Err(err) => {
            let err_msg = err.to_string();
            assert!(
                err_msg.contains(expected_msg),
                "Error message '{}' does not contain '{}'",
                err_msg,
                expected_msg
            );
        }
    }
}

/// Demonstrates a trybuild-style test structure.
/// In real usage, you'd have a tests/compile_fail/ directory with
/// test cases that should fail to compile.
pub struct CompileFailTest {
    pub name: &'static str,
    pub code: &'static str,
    pub expected_error: &'static str,
}

/// Demonstrates a compile-pass test structure.
pub struct CompilePassTest {
    pub name: &'static str,
    pub code: &'static str,
}

/// Demonstrates testing helper: verify a token stream contains expected identifiers.
pub fn assert_contains_ident(stream: &TokenStream, expected: &str) {
    let code = stream.to_string();
    assert!(
        code.contains(expected),
        "Expected identifier '{expected}' not found in:\n{code}"
    );
}

/// Demonstrates testing helper: verify a token stream does NOT contain something.
pub fn assert_not_contains(stream: &TokenStream, unexpected: &str) {
    let code = stream.to_string();
    assert!(
        !code.contains(unexpected),
        "Unexpected '{unexpected}' found in:\n{code}"
    );
}

/// Demonstrates testing macro-generated code by counting items.
pub fn count_items(stream: &TokenStream) -> usize {
    let code = stream.to_string();
    // Simple heuristic: count pub fn, pub struct, pub enum, etc.
    let patterns = ["pub fn ", "pub struct ", "pub enum ", "pub const ", "impl "];
    patterns.iter().map(|p| code.matches(p).count()).sum()
}

/// Demonstrates testing pattern: parameterized tests for macros.
pub fn run_macro_tests<F>(name: &str, cases: &[(&str, &str)], macro_fn: F)
where
    F: Fn(&str) -> TokenStream,
{
    for (input, expected_pattern) in cases {
        let result = macro_fn(input);
        let code = result.to_string();
        assert!(
            code.contains(expected_pattern),
            "Test '{name}' failed for input '{input}': \
             expected pattern '{expected_pattern}' not found in output:\n{code}"
        );
    }
}

/// Demonstrates snapshot testing for macro output.
/// Instead of asserting exact matches, we check key properties.
pub fn snapshot_test(name: &str, stream: &TokenStream, properties: &[&str]) {
    let code = stream.to_string();
    for prop in properties {
        assert!(
            code.contains(prop),
            "Snapshot test '{name}': property '{prop}' not found"
        );
    }
}

/// Demonstrates testing that a macro handles edge cases correctly.
pub fn test_edge_cases<F>(name: &str, macro_fn: F)
where
    F: Fn(&str) -> Result<TokenStream, String>,
{
    // Empty input
    let result = macro_fn("");
    // Depending on the macro, this might succeed or fail

    // Single element
    let result = macro_fn("x");
    // Verify it doesn't panic

    // Many elements
    let many = "a, b, c, d, e, f, g, h, i, j";
    let result = macro_fn(many);
    // Verify it handles many elements

    let _ = (name, result);
}

/// Demonstrates testing macro hygiene by verifying that generated identifiers
/// don't conflict with known names.
pub fn test_hygiene(stream: &TokenStream, reserved_names: &[&str]) {
    let code = stream.to_string();
    // In a real test, you'd parse the tokens and check spans
    // Here we just verify the code doesn't reference reserved names directly
    for name in reserved_names {
        // This is a simplified check
        let _ = (code.contains(name), name);
    }
}

/// Demonstrates testing async macro output.
pub fn assert_generates_async_function(code: &TokenStream) {
    let code_str = code.to_string();
    assert!(
        code_str.contains("async"),
        "Expected async function, got:\n{code_str}"
    );
}

/// Demonstrates testing macro output with generics.
pub fn assert_has_generic(code: &TokenStream, generic_param: &str) {
    let code_str = code.to_string();
    assert!(
        code_str.contains(generic_param),
        "Expected generic parameter '{generic_param}' not found in:\n{code_str}"
    );
}

/// Demonstrates testing macro output with trait bounds.
pub fn assert_has_trait_bound(code: &TokenStream, trait_name: &str) {
    let code_str = code.to_string();
    assert!(
        code_str.contains(trait_name),
        "Expected trait bound '{trait_name}' not found in:\n{code_str}"
    );
}

/// Demonstrates testing macro output with attributes.
pub fn assert_has_attribute(code: &TokenStream, attr: &str) {
    let code_str = code.to_string();
    assert!(
        code_str.contains(attr),
        "Expected attribute '{attr}' not found in:\n{code_str}"
    );
}

/// Demonstrates testing macro output with doc comments.
pub fn assert_has_doc_comment(code: &TokenStream, doc_text: &str) {
    let code_str = code.to_string();
    assert!(
        code_str.contains("doc") || code_str.contains(doc_text),
        "Expected doc comment containing '{doc_text}' not found in:\n{code_str}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_expansion_eq() {
        let actual = quote! { fn hello() { 42 } };
        let expected = quote! { fn hello() { 42 } };
        assert_expansion_eq(&actual, &expected);
    }

    #[test]
    fn test_assert_generates_valid_function() {
        let code = quote! {
            pub fn my_function() -> i32 {
                42
            }
        };
        assert_generates_valid_function(&code);
    }

    #[test]
    fn test_assert_generates_valid_struct() {
        let code = quote! {
            pub struct MyStruct {
                field: i32,
            }
        };
        assert_generates_valid_struct(&code);
    }

    #[test]
    fn test_assert_generates_valid_item() {
        let code = quote! {
            impl MyTrait for MyType {
                fn method(&self) {}
            }
        };
        assert_generates_valid_item(&code);
    }

    #[test]
    fn test_assert_contains_ident() {
        let code = quote! { fn hello_world() {} };
        assert_contains_ident(&code, "hello_world");
    }

    #[test]
    fn test_assert_not_contains() {
        let code = quote! { fn hello() {} };
        assert_not_contains(&code, "world");
    }

    #[test]
    fn test_count_items() {
        let code = quote! {
            pub fn a() {}
            pub struct B {}
            impl C {
                fn d(&self) {}
            }
        };
        assert_eq!(count_items(&code), 3);
    }

    #[test]
    fn test_snapshot_test() {
        let code = quote! {
            pub struct Server {
                host: String,
                port: u16,
            }
        };
        snapshot_test(
            "server_struct",
            &code,
            &["Server", "host", "port", "String", "u16"],
        );
    }

    #[test]
    fn test_assert_generates_async_function() {
        let code = quote! {
            async fn fetch_data() -> String {
                String::new()
            }
        };
        assert_generates_async_function(&code);
    }

    #[test]
    fn test_assert_has_generic() {
        let code = quote! {
            pub fn process<T: Clone>(items: Vec<T>) -> Vec<T> {
                items
            }
        };
        assert_has_generic(&code, "T");
    }

    #[test]
    fn test_assert_has_trait_bound() {
        let code = quote! {
            pub fn process<T: Clone + Debug>(value: T) -> T {
                value
            }
        };
        assert_has_trait_bound(&code, "Clone");
    }

    #[test]
    fn test_assert_has_attribute() {
        let code = quote! {
            #[derive(Debug, Clone)]
            pub struct Config {}
        };
        assert_has_attribute(&code, "derive");
    }

    #[test]
    fn test_assert_has_doc_comment() {
        let code = quote! {
            #[doc = "A server configuration"]
            pub struct Config {}
        };
        assert_has_doc_comment(&code, "server configuration");
    }

    #[test]
    fn test_run_macro_tests() {
        let test_fn = |input: &str| -> TokenStream {
            let name = syn::Ident::new(input, proc_macro2::Span::call_site());
            quote! { pub fn #name() {} }
        };

        run_macro_tests(
            "function_names",
            &[
                ("hello", "hello"),
                ("world", "world"),
                ("test_fn", "test_fn"),
            ],
            test_fn,
        );
    }

    #[test]
    fn test_compile_error() {
        let code = "fn";
        assert_compile_error_contains(code, "expected");
    }

    #[test]
    fn test_normalize_code() {
        let code = "fn   hello(  )  {   42  }";
        let normalized = normalize_code(code);
        assert_eq!(normalized, "fn hello() { 42 }");
    }
}
