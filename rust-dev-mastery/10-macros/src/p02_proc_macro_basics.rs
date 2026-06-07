//! # Proc Macro Basics
//!
//! Procedural macros operate on `TokenStream` and can generate arbitrary code.
//! Unlike `macro_rules!`, they are full Rust programs that run at compile time.
//!
//! There are three kinds of proc macros:
//! 1. **Derive macros**: `#[derive(MyMacro)]`
//! 2. **Attribute macros**: `#[my_macro]`
//! 3. **Function-like macros**: `my_macro!(...)`
//!
//! This module demonstrates the concepts and patterns used in proc macros,
//! even though actual proc macros must live in a separate crate with
//! `proc-macro = true`.
//!
//! ## Crate Structure
//!
//! A proc macro crate typically has this structure:
//! ```toml
//! [lib]
//! proc-macro = true
//!
//! [dependencies]
//! syn = { version = "2", features = ["full"] }
//! quote = "1"
//! proc-macro2 = "1"
//! ```

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::{parse_str, Expr, Ident, ItemFn, Type};

/// Demonstrates how proc macros work with TokenStream.
/// In a real proc macro crate, this would be annotated with `#[proc_macro]`.
pub fn demonstrate_token_stream() -> TokenStream {
    let input: TokenStream = "fn hello() { }".parse().unwrap();

    // Count tokens
    let count = input.clone().into_iter().count();
    assert_eq!(count, 4); // fn, hello, (, ), { }

    // Iterate over tokens
    let mut tokens = Vec::new();
    for token in input {
        match token {
            TokenTree::Ident(ident) => tokens.push(format!("Ident({ident})")),
            TokenTree::Punct(punct) => tokens.push(format!("Punct({punct})")),
            TokenTree::Group(group) => tokens.push(format!("Group({:?})", group.delimiter())),
            TokenTree::Literal(lit) => tokens.push(format!("Literal({lit})")),
        }
    }
    assert!(tokens.contains(&"Ident(fn)".to_string()));
    assert!(tokens.contains(&"Ident(hello)".to_string()));

    // Generate code using quote!
    let generated = quote! {
        fn generated_function() -> i32 {
            42
        }
    };
    generated
}

/// Demonstrates parsing Rust code with syn.
/// This is what proc macros do internally: parse the input TokenStream
/// into a structured AST, then generate new code.
pub fn parse_and_generate(code: &str) -> Result<TokenStream, String> {
    // Parse the input as a function
    let func: ItemFn = parse_str(code).map_err(|e| e.to_string())?;

    let name = &func.sig.ident;
    let name_str = name.to_string();

    // Generate a wrapper that adds logging
    let output = quote! {
        pub fn #name() {
            println!("Entering function: {}", #name_str);
            // Original function body would go here
            println!("Exiting function: {}", #name_str);
        }
    };

    Ok(output)
}

/// Demonstrates creating identifiers programmatically.
/// Proc macros often need to create new identifiers for generated code.
pub fn create_identifiers() -> Vec<Ident> {
    let names = vec!["field_a", "field_b", "field_c"];
    names
        .into_iter()
        .map(|name| Ident::new(name, Span::call_site()))
        .collect()
}

/// Demonstrates generating code with different spans.
/// Spans control error reporting and hygiene.
pub fn generate_with_spans() -> TokenStream {
    let func_name = Ident::new("my_function", Span::call_site());
    let param_name = Ident::new("input", Span::mixed_site());
    let body: Expr = parse_str("input * 2").unwrap();

    quote! {
        pub fn #func_name(#param_name: i32) -> i32 {
            #body
        }
    }
}

/// Demonstrates how proc macros can parse custom syntax.
/// This simulates a derive macro that parses struct fields.
pub fn parse_struct_fields(code: &str) -> Result<Vec<(String, String)>, String> {
    let item: syn::Item = parse_str(code).map_err(|e| e.to_string())?;

    match item {
        syn::Item::Struct(item_struct) => {
            let mut fields = Vec::new();
            for field in &item_struct.fields {
                let name = field
                    .ident
                    .as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                let ty = quote!(#(&field.ty)).to_string();
                fields.push((name, ty));
            }
            Ok(fields)
        }
        _ => Err("expected a struct".to_string()),
    }
}

/// Demonstrates generating trait implementations.
/// This is the core pattern for derive macros.
pub fn generate_display_impl(struct_name: &str, fields: &[&str]) -> TokenStream {
    let name = Ident::new(struct_name, Span::call_site());
    let field_names: Vec<Ident> = fields
        .iter()
        .map(|f| Ident::new(f, Span::call_site()))
        .collect();

    quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(.field(stringify!(#field_names), &self.#field_names))*
                    .finish()
            }
        }
    }
}

/// Demonstrates conditional code generation.
/// Proc macros can generate different code based on configuration.
pub fn generate_with_cfg(feature: &str, code_if_enabled: TokenStream) -> TokenStream {
    let cfg_ident = Ident::new(feature, Span::call_site());

    quote! {
        #[cfg(feature = #cfg_ident)]
        #code_if_enabled

        #[cfg(not(feature = #cfg_ident))]
        compile_error!(concat!("Feature '", #cfg_ident, "' is required"));
    }
}

/// Demonstrates generating error messages from proc macros.
/// Good error messages are essential for proc macro usability.
pub fn generate_compile_error(message: &str) -> TokenStream {
    quote! {
        compile_error!(#message);
    }
}

/// Demonstrates how to generate code that references types from other crates.
/// The `#crate_name::Type` pattern ensures correct paths.
pub fn generate_serde_impl() -> TokenStream {
    quote! {
        impl serde::Serialize for MyType {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }
    }
}

/// Demonstrates generating code with multiple trait bounds.
pub fn generate_generic_impl() -> TokenStream {
    quote! {
        impl<T> MyTrait<T>
        where
            T: Clone + std::fmt::Debug + Send + Sync + 'static,
        {
            pub fn process(&self, value: &T) -> T {
                let cloned = value.clone();
                println!("Processing: {:?}", cloned);
                cloned
            }
        }
    }
}

/// Demonstrates generating match arms dynamically.
pub fn generate_match_arms(variants: &[(&str, &str)]) -> TokenStream {
    let arms: Vec<TokenStream> = variants
        .iter()
        .map(|(variant, value)| {
            let ident = Ident::new(variant, Span::call_site());
            quote! {
                Self::#ident => #value,
            }
        })
        .collect();

    quote! {
        match self {
            #(#arms)*
        }
    }
}

/// Demonstrates generating a newtype wrapper with conversions.
pub fn generate_newtype(name: &str, inner_type: &str) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());
    let inner_type_ident = Ident::new(inner_type, Span::call_site());

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #name_ident(pub #inner_type_ident);

        impl std::fmt::Display for #name_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl From<#inner_type_ident> for #name_ident {
            fn from(value: #inner_type_ident) -> Self {
                #name_ident(value)
            }
        }

        impl From<#name_ident> for #inner_type_ident {
            fn from(value: #name_ident) -> Self {
                value.0
            }
        }
    }
}

/// Demonstrates iterating over token streams with filtering.
pub fn filter_idents(stream: &TokenStream) -> Vec<String> {
    stream
        .clone()
        .into_iter()
        .filter_map(|token| match token {
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None,
        })
        .collect()
}

/// Demonstrates concatenating token streams.
pub fn concat_streams(streams: Vec<TokenStream>) -> TokenStream {
    let mut result = TokenStream::new();
    for stream in streams {
        result.extend(stream);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demonstrate_token_stream() {
        let generated = demonstrate_token_stream();
        let code = generated.to_string();
        assert!(code.contains("generated_function"));
        assert!(code.contains("42"));
    }

    #[test]
    fn test_parse_and_generate() {
        let result = parse_and_generate("fn my_func() { }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("my_func"));
        assert!(code.contains("Entering function"));
    }

    #[test]
    fn test_parse_and_generate_invalid() {
        let result = parse_and_generate("not valid rust");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_identifiers() {
        let idents = create_identifiers();
        assert_eq!(idents.len(), 3);
        assert_eq!(idents[0].to_string(), "field_a");
        assert_eq!(idents[1].to_string(), "field_b");
        assert_eq!(idents[2].to_string(), "field_c");
    }

    #[test]
    fn test_generate_with_spans() {
        let code = generate_with_spans().to_string();
        assert!(code.contains("my_function"));
        assert!(code.contains("input"));
        assert!(code.contains("input * 2"));
    }

    #[test]
    fn test_parse_struct_fields() {
        let result = parse_struct_fields("struct MyStruct { name: String, age: u32 }");
        assert!(result.is_ok());
        let fields = result.unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "name");
        assert_eq!(fields[1].0, "age");
    }

    #[test]
    fn test_generate_display_impl() {
        let code = generate_display_impl("Point", &["x", "y"]).to_string();
        assert!(code.contains("Display"));
        assert!(code.contains("Point"));
        assert!(code.contains("x"));
        assert!(code.contains("y"));
    }

    #[test]
    fn test_generate_compile_error() {
        let code = generate_compile_error("this is an error").to_string();
        assert!(code.contains("compile_error"));
        assert!(code.contains("this is an error"));
    }

    #[test]
    fn test_generate_serde_impl() {
        let code = generate_serde_impl().to_string();
        assert!(code.contains("Serialize"));
        assert!(code.contains("serialize"));
    }

    #[test]
    fn test_generate_generic_impl() {
        let code = generate_generic_impl().to_string();
        assert!(code.contains("Clone"));
        assert!(code.contains("Debug"));
        assert!(code.contains("Send"));
        assert!(code.contains("Sync"));
    }

    #[test]
    fn test_generate_match_arms() {
        let variants = vec![("Red", "red"), ("Green", "green"), ("Blue", "blue")];
        let code = generate_match_arms(&variants).to_string();
        assert!(code.contains("Red"));
        assert!(code.contains("red"));
        assert!(code.contains("Green"));
        assert!(code.contains("blue"));
    }

    #[test]
    fn test_generate_newtype() {
        let code = generate_newtype("UserId", "u64").to_string();
        assert!(code.contains("UserId"));
        assert!(code.contains("Display"));
        assert!(code.contains("From"));
    }

    #[test]
    fn test_filter_idents() {
        let stream: TokenStream = "fn hello(x: i32) -> i32".parse().unwrap();
        let idents = filter_idents(&stream);
        assert!(idents.contains(&"fn".to_string()));
        assert!(idents.contains(&"hello".to_string()));
        assert!(idents.contains(&"i32".to_string()));
    }

    #[test]
    fn test_concat_streams() {
        let s1: TokenStream = "fn a()".parse().unwrap();
        let s2: TokenStream = "{ }".parse().unwrap();
        let result = concat_streams(vec![s1, s2]);
        assert!(result.to_string().contains("fn a ()"));
    }
}
