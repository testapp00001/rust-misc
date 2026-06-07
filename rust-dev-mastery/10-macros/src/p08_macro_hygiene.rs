//! # Macro Hygiene
//!
//! Macro hygiene prevents macro-generated identifiers from colliding with
//! identifiers in the calling code. Rust's hygiene rules are:
//!
//! - Identifiers in `macro_rules!` are hygienic: they can't see local variables
//! - `$crate` is always available and resolves to the crate defining the macro
//! - Proc macro identifiers use `Span::call_site()` (call-site hygiene)
//! - `Span::mixed_site()` provides partial hygiene
//!
//! Key concepts:
//! - **Call site**: Where the macro is invoked
//! - **Def site**: Where the macro is defined
//! - **Mixed site**: A mix of both (used in some contexts)

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Demonstrates hygiene in declarative macros.
/// Variables defined inside the macro don't conflict with caller's variables.
macro_rules! hygienic_macro {
    ($value:expr) => {
        {
            // This `x` is hygienic - it won't conflict with any `x` in the caller's scope
            let x = $value;
            x * 2
        }
    };
}

/// Demonstrates that macro_rules! variables are in a different scope.
macro_rules! let_bindings {
    ($($name:ident = $value:expr),*) => {
        {
            // Each `$name` is hygienic - created in the macro's scope
            $(let $name = $value;)*
            ($($name),*)
        }
    };
}

/// Demonstrates $crate for referencing the defining crate's items.
/// `$crate` always resolves to the crate where the macro is defined,
/// even when the macro is used in another crate.
macro_rules! with_crate_path {
    ($name:ident) => {
        // In real code: $crate::some_module::$name
        // For demonstration, we use the current module path
        $name
    };
}

/// Demonstrates hygiene issues with trait methods.
/// When a macro generates code that calls methods, it must use
/// fully qualified syntax to avoid ambiguity.
macro_rules! call_method {
    ($obj:expr, $method:ident) => {{
        // Use fully qualified syntax to avoid ambiguity
        // This is important when the caller might have a different
        // trait in scope with the same method name
        $obj.$method()
    }};
}

/// Demonstrates how to generate identifiers that interact with caller code.
/// Proc macros use `Span::call_site()` by default, which means generated
/// identifiers are in the caller's scope.
pub fn generate_with_call_site(name: &str) -> TokenStream {
    // This identifier will be in the caller's scope
    let ident = Ident::new(name, Span::call_site());

    quote! {
        let #ident = 42;
    }
}

/// Demonstrates `Span::mixed_site()` which provides partial hygiene.
/// Mixed site identifiers are visible to the macro's own code but
/// not to the caller's code.
pub fn generate_with_mixed_site(name: &str) -> TokenStream {
    let ident = Ident::new(name, Span::mixed_site());

    quote! {
        let #ident = 42;
    }
}

/// Demonstrates how to create a hygienic helper function in a macro.
/// The helper function name uses $crate to avoid conflicts.
macro_rules! with_helper {
    ($value:expr) => {{
        // Define a helper function with a unique name
        // In real code, this would use $crate::__helper_fn
        fn __macro_helper(x: i32) -> i32 {
            x * 2 + 1
        }
        __macro_helper($value)
    }};
}

/// Demonstrates the pattern of using unique identifier names to avoid conflicts.
/// This is sometimes necessary when hygiene alone isn't sufficient.
pub fn generate_with_unique_names(base_name: &str) -> TokenStream {
    let var_name = format!("__{base_name}_internal");
    let ident = Ident::new(&var_name, Span::call_site());

    quote! {
        let #ident = {
            // Internal implementation detail
            42
        };
        #ident
    }
}

/// Demonstrates how to handle type paths in macros.
/// Using `$crate::` ensures the path resolves correctly even when
/// the macro is used from another crate.
macro_rules! use_type {
    ($ty:ty) => {{
        let _: $ty = Default::default();
    }};
}

/// Demonstrates how to safely reference traits in macro output.
/// When calling trait methods, use fully qualified syntax.
macro_rules! debug_print {
    ($value:expr) => {{
        // Use fully qualified syntax to avoid ambiguity
        <_ as std::fmt::Debug>::fmt(&$value, &mut std::fmt::Formatter::new(
            &mut String::new(),
            std::fmt::ArgumentV1::new(&|_| Ok(()), &|_| Ok(())),
        ))
    }};
}

/// Demonstrates the pattern for generating code that uses external crates.
/// The macro must use $crate for its own crate's items and full paths
/// for other crates.
pub fn generate_with_external_crate() -> TokenStream {
    quote! {
        // When the macro's crate depends on serde, use the path
        // that will resolve correctly in the caller's context
        use serde::{Serialize, Deserialize};
    }
}

/// Demonstrates how to handle generic parameters in macro-generated code.
/// Generic parameters must be passed through correctly.
pub fn generate_generic_function() -> TokenStream {
    quote! {
        pub fn process<T, E>(value: T) -> Result<T, E>
        where
            T: Clone + std::fmt::Debug,
            E: std::fmt::Debug,
        {
            println!("{:?}", &value);
            Ok(value)
        }
    }
}

/// Demonstrates the `mbe` (macro-by-example) hygiene rules.
/// In `macro_rules!`, identifiers introduced by the macro are in
/// a different syntax context from the caller's identifiers.
macro_rules! hygiene_demo {
    ($x:expr) => {{
        // `result` is hygienic - in the macro's scope
        let result = $x + 1;
        result
    }};
}

/// Demonstrates how to work around hygiene when needed.
/// Sometimes you need identifiers to be in the caller's scope.
macro_rules! introduce_binding {
    ($name:ident, $value:expr) => {
        let $name = $value;
    };
}

/// Demonstrates the pattern for generating constants with unique names.
pub fn generate_const_array(name: &str, values: &[i32]) -> TokenStream {
    let const_name = format!("{name}_VALUES").to_uppercase();
    let ident = Ident::new(&const_name, Span::call_site());

    quote! {
        pub const #ident: [i32; #(#values.len())*] = [#(#values),*];
    }
}

/// Demonstrates how to generate code that interacts with the caller's scope.
/// This is the key challenge of macro hygiene: sometimes you WANT to
/// access the caller's scope.
pub fn generate_scope_interactive(var_name: &str) -> TokenStream {
    // Use call_site span to interact with caller's scope
    let ident = Ident::new(var_name, Span::call_site());

    quote! {
        // This will access the caller's `#ident` variable
        println!("{}", #ident);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hygienic_macro_no_conflict() {
        let x = 100;
        // The macro's `x` doesn't conflict with the outer `x`
        let result = hygienic_macro!(5);
        assert_eq!(result, 10); // 5 * 2
        assert_eq!(x, 100); // Outer x unchanged
    }

    #[test]
    fn test_let_bindings_hygiene() {
        let (a, b, c) = let_bindings!(a = 1, b = 2, c = 3);
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 3);
        // The outer scope's variables with the same names would be different
    }

    #[test]
    fn test_with_helper() {
        let result = with_helper!(10);
        assert_eq!(result, 21); // 10 * 2 + 1
    }

    #[test]
    fn test_hygiene_demo() {
        let result = hygiene_demo!(41);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_generate_with_call_site() {
        let code = generate_with_call_site("my_var");
        let code_str = code.to_string();
        assert!(code_str.contains("my_var"));
        assert!(code_str.contains("42"));
    }

    #[test]
    fn test_generate_with_unique_names() {
        let code = generate_with_unique_names("test");
        let code_str = code.to_string();
        assert!(code_str.contains("__test_internal"));
    }

    #[test]
    fn test_generate_generic_function() {
        let code = generate_generic_function();
        let code_str = code.to_string();
        assert!(code_str.contains("process"));
        assert!(code_str.contains("T"));
        assert!(code_str.contains("Clone"));
    }

    #[test]
    fn test_generate_const_array() {
        let code = generate_const_array("prime", &[2, 3, 5, 7, 11]);
        let code_str = code.to_string();
        assert!(code_str.contains("PRIME_VALUES"));
    }

    #[test]
    fn test_generate_scope_interactive() {
        let code = generate_scope_interactive("my_data");
        let code_str = code.to_string();
        assert!(code_str.contains("my_data"));
    }

    #[test]
    fn test_call_method() {
        let s = String::from("hello");
        let len = call_method!(s, len);
        assert_eq!(len, 5);
    }

    #[test]
    fn test_use_type() {
        use_type!(i32);
        use_type!(String);
        use_type!(Vec<u8>);
    }

    #[test]
    fn test_introduce_binding() {
        introduce_binding!(x, 42);
        assert_eq!(x, 42);

        introduce_binding!(name, "hello");
        assert_eq!(name, "hello");
    }
}
