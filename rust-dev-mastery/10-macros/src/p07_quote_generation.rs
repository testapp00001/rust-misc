//! # quote Crate: Code Generation
//!
//! The `quote` crate converts Rust syntax tree data structures into
//! `TokenStream`s of Rust source code. It uses `#variable` interpolation
//! similar to format strings.
//!
//! Key features:
//! - `#var` for interpolation
//! - `#(...)* ` for repetition
//! - Spans control error locations and hygiene
//! - `quote_spanned!` for precise error reporting

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_str, Ident, Type};

/// Demonstrates basic interpolation with `#var`.
pub fn generate_function(name: &str, body: &str) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());
    let body_expr: syn::Expr = parse_str(body).unwrap();

    quote! {
        pub fn #name_ident() -> i32 {
            #body_expr
        }
    }
}

/// Demonstrates repetition with `#(...)*`.
pub fn generate_struct_with_fields(name: &str, fields: &[(&str, &str)]) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());

    let field_decls: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, ftype)| {
            let f_ident = Ident::new(fname, Span::call_site());
            let f_type: Type = parse_str(ftype).unwrap();
            quote! { pub #f_ident: #f_type }
        })
        .collect();

    quote! {
        #[derive(Debug)]
        pub struct #name_ident {
            #(#field_decls),*
        }
    }
}

/// Demonstrates `format_ident!` for creating identifiers with formatting.
pub fn generate_getters(name: &str, fields: &[&str]) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());

    let getters: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_ident = Ident::new(field, Span::call_site());
            let getter_name = format_ident!("get_{}", field);
            quote! {
                pub fn #getter_name(&self) -> &#field_ident {
                    &self.#field_ident
                }
            }
        })
        .collect();

    quote! {
        impl #name_ident {
            #(#getters)*
        }
    }
}

/// Demonstrates conditional code generation with quote.
pub fn generate_conditional(feature: &str, code_if: TokenStream, code_else: TokenStream) -> TokenStream {
    let feature_str = feature.to_string();

    quote! {
        #[cfg(feature = #feature_str)]
        {
            #code_if
        }

        #[cfg(not(feature = #feature_str))]
        {
            #code_else
        }
    }
}

/// Demonstrates generating match arms dynamically.
pub fn generate_match(enum_name: &str, variants: &[(&str, &str)]) -> TokenStream {
    let enum_ident = Ident::new(enum_name, Span::call_site());

    let arms: Vec<TokenStream> = variants
        .iter()
        .map(|(variant, value)| {
            let variant_ident = Ident::new(variant, Span::call_site());
            quote! {
                #enum_ident::#variant_ident => #value
            }
        })
        .collect();

    quote! {
        match self {
            #(#arms),*
        }
    }
}

/// Demonstrates generating trait implementations.
pub fn generate_display_impl(name: &str, format_str: &str) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());

    quote! {
        impl std::fmt::Display for #name_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #format_str)
            }
        }
    }
}

/// Demonstrates generating generic implementations.
pub fn generate_from_impl(outer: &str, inner: &str) -> TokenStream {
    let outer_ident = Ident::new(outer, Span::call_site());
    let inner_type: Type = parse_str(inner).unwrap();

    quote! {
        impl From<#inner_type> for #outer_ident {
            fn from(value: #inner_type) -> Self {
                #outer_ident(value)
            }
        }
    }
}

/// Demonstrates `quote_spanned!` for precise error locations.
pub fn generate_with_span_error(name: &str, span: Span) -> TokenStream {
    let name_ident = Ident::new(name, span);

    quote_spanned! { span =>
        impl MyTrait for #name_ident {
            fn required_method(&self) {
                // Error will point to the original item
            }
        }
    }
}

/// Demonstrates generating code with multiple trait bounds.
pub fn generate_with_bounds() -> TokenStream {
    quote! {
        pub fn process<T>(items: Vec<T>) -> Vec<T>
        where
            T: Clone + std::fmt::Debug + Send + Sync + 'static,
        {
            items.into_iter().map(|item| {
                let cloned = item.clone();
                println!("{:?}", cloned);
                cloned
            }).collect()
        }
    }
}

/// Demonstrates generating a test function.
pub fn generate_test(test_name: &str, assertions: &[TokenStream]) -> TokenStream {
    let test_ident = Ident::new(test_name, Span::call_site());

    quote! {
        #[test]
        fn #test_ident() {
            #(#assertions)*
        }
    }
}

/// Demonstrates generating const declarations.
pub fn generate_constants(constants: &[(&str, &str, &str)]) -> TokenStream {
    let decls: Vec<TokenStream> = constants
        .iter()
        .map(|(name, type_str, value)| {
            let name_ident = Ident::new(name, Span::call_site());
            let ty: Type = parse_str(type_str).unwrap();
            let val: syn::Expr = parse_str(value).unwrap();
            quote! {
                pub const #name_ident: #ty = #val;
            }
        })
        .collect();

    quote! {
        #(#decls)*
    }
}

/// Demonstrates generating a module with items.
pub fn generate_module(name: &str, items: Vec<TokenStream>) -> TokenStream {
    let mod_ident = Ident::new(name, Span::call_site());

    quote! {
        pub mod #mod_ident {
            #(#items)*
        }
    }
}

/// Demonstrates generating code with `stringify!` for compile-time string conversion.
pub fn generate_with_stringify(ident_name: &str) -> TokenStream {
    let ident = Ident::new(ident_name, Span::call_site());

    quote! {
        pub fn name() -> &'static str {
            stringify!(#ident)
        }
    }
}

/// Demonstrates generating a vector initialization.
pub fn generate_vec_init(items: &[i32]) -> TokenStream {
    quote! {
        vec![#(#items),*]
    }
}

/// Demonstrates generating nested structures.
pub fn generate_nested() -> TokenStream {
    quote! {
        pub struct Outer {
            pub inner: Inner,
        }

        pub struct Inner {
            pub value: i32,
        }

        impl Outer {
            pub fn new(value: i32) -> Self {
                Outer {
                    inner: Inner { value },
                }
            }

            pub fn get_value(&self) -> i32 {
                self.inner.value
            }
        }
    }
}

/// Demonstrates generating code with doc comments.
pub fn generate_documented(name: &str, doc: &str) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());

    quote! {
        #[doc = #doc]
        pub struct #name_ident;

        impl #name_ident {
            #[doc = concat!("Create a new `", stringify!(#name_ident), "` instance.")]
            pub fn new() -> Self {
                #name_ident
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_function() {
        let code = generate_function("double", "x * 2");
        let code_str = code.to_string();
        assert!(code_str.contains("double"));
        assert!(code_str.contains("i32"));
    }

    #[test]
    fn test_generate_struct_with_fields() {
        let fields = vec![("x", "f64"), ("y", "f64")];
        let code = generate_struct_with_fields("Point", &fields);
        let code_str = code.to_string();
        assert!(code_str.contains("Point"));
        assert!(code_str.contains("x"));
        assert!(code_str.contains("y"));
        assert!(code_str.contains("f64"));
    }

    #[test]
    fn test_generate_getters() {
        let fields = vec!["name", "age", "email"];
        let code = generate_getters("User", &fields);
        let code_str = code.to_string();
        assert!(code_str.contains("get_name"));
        assert!(code_str.contains("get_age"));
        assert!(code_str.contains("get_email"));
    }

    #[test]
    fn test_generate_match() {
        let variants = vec![("Red", "red"), ("Green", "green"), ("Blue", "blue")];
        let code = generate_match("Color", &variants);
        let code_str = code.to_string();
        assert!(code_str.contains("Color :: Red"));
        assert!(code_str.contains("red"));
    }

    #[test]
    fn test_generate_display_impl() {
        let code = generate_display_impl("Point", "({}, {})");
        let code_str = code.to_string();
        assert!(code_str.contains("Display"));
        assert!(code_str.contains("Point"));
    }

    #[test]
    fn test_generate_from_impl() {
        let code = generate_from_impl("UserId", "u64");
        let code_str = code.to_string();
        assert!(code_str.contains("From"));
        assert!(code_str.contains("UserId"));
        assert!(code_str.contains("u64"));
    }

    #[test]
    fn test_generate_with_bounds() {
        let code = generate_with_bounds();
        let code_str = code.to_string();
        assert!(code_str.contains("Clone"));
        assert!(code_str.contains("Debug"));
        assert!(code_str.contains("Send"));
    }

    #[test]
    fn test_generate_test() {
        let assertions = vec![quote! { assert_eq!(1 + 1, 2); }];
        let code = generate_test("test_basic_math", &assertions);
        let code_str = code.to_string();
        assert!(code_str.contains("test_basic_math"));
        assert!(code_str.contains("assert_eq"));
    }

    #[test]
    fn test_generate_constants() {
        let constants = vec![
            ("MAX_SIZE", "usize", "1024"),
            ("DEFAULT_PORT", "u16", "8080"),
        ];
        let code = generate_constants(&constants);
        let code_str = code.to_string();
        assert!(code_str.contains("MAX_SIZE"));
        assert!(code_str.contains("DEFAULT_PORT"));
    }

    #[test]
    fn test_generate_module() {
        let items = vec![quote! { pub fn hello() {} }];
        let code = generate_module("api", items);
        let code_str = code.to_string();
        assert!(code_str.contains("mod api"));
        assert!(code_str.contains("hello"));
    }

    #[test]
    fn test_generate_with_stringify() {
        let code = generate_with_stringify("MyType");
        let code_str = code.to_string();
        assert!(code_str.contains("stringify"));
        assert!(code_str.contains("MyType"));
    }

    #[test]
    fn test_generate_vec_init() {
        let code = generate_vec_init(&[1, 2, 3, 4, 5]);
        let code_str = code.to_string();
        assert!(code_str.contains("vec"));
    }

    #[test]
    fn test_generate_nested() {
        let code = generate_nested();
        let code_str = code.to_string();
        assert!(code_str.contains("Outer"));
        assert!(code_str.contains("Inner"));
        assert!(code_str.contains("get_value"));
    }

    #[test]
    fn test_generate_documented() {
        let code = generate_documented("Server", "A TCP server that handles connections.");
        let code_str = code.to_string();
        assert!(code_str.contains("Server"));
        assert!(code_str.contains("doc"));
    }
}
