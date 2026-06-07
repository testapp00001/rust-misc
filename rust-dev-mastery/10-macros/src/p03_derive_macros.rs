//! # Derive Macros
//!
//! Derive macros are the most common kind of proc macro. They generate
//! trait implementations from struct/enum definitions.
//!
//! Key concepts:
//! - Parsing the input with `syn::DeriveInput`
//! - Generating implementations with `quote!`
//! - Helper attributes for customization
//! - Handling generics, lifetimes, and where clauses
//!
//! This module demonstrates the patterns used in derive macros using
//! simulated implementations (real ones require a proc-macro crate).

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_str, DeriveInput, Fields, FieldsNamed, Ident};

/// Simulates what a derive macro does: parse a struct/enum and generate code.
/// In a real proc macro, this would be annotated with `#[proc_macro_derive(MyTrait)]`.
pub fn simulate_derive(input: &str) -> Result<TokenStream, String> {
    let derive_input: DeriveInput = parse_str(input).map_err(|e| e.to_string())?;
    let name = &derive_input.ident;
    let name_str = name.to_string();

    // Generate a simple implementation based on the type
    match &derive_input.data {
        syn::Data::Struct(data_struct) => {
            generate_struct_impl(name, &name_str, data_struct)
        }
        syn::Data::Enum(_data_enum) => {
            Ok(quote! {
                impl MyTrait for #name {
                    fn trait_method(&self) -> String {
                        stringify!(#name).to_string()
                    }
                }
            })
        }
        syn::Data::Union(_) => Err("derive macro does not support unions".to_string()),
    }
}

/// Generate implementation for a struct type.
fn generate_struct_impl(
    name: &Ident,
    name_str: &str,
    data: &syn::DataStruct,
) -> Result<TokenStream, String> {
    let field_names = extract_field_names(&data.fields)?;

    let field_prints: Vec<TokenStream> = field_names
        .iter()
        .map(|f| {
            let f_str = f.to_string();
            quote! {
                result.push_str(&format!("  {}: {:?}\n", #f_str, &self.#f));
            }
        })
        .collect();

    Ok(quote! {
        impl MyTrait for #name {
            fn trait_method(&self) -> String {
                let mut result = format!("{} {{\n", #name_str);
                #(#field_prints)*
                result.push('}');
                result
            }
        }
    })
}

/// Extract field names from a struct's fields.
fn extract_field_names(fields: &syn::Fields) -> Result<Vec<Ident>, String> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            Ok(named
                .iter()
                .filter_map(|f| f.ident.clone())
                .collect())
        }
        Fields::Unit => Ok(vec![]),
        Fields::Unnamed(_) => Err("tuple structs not supported".to_string()),
    }
}

/// Demonstrates how derive macros handle generics.
pub fn generate_generic_derive(input: &str) -> Result<TokenStream, String> {
    let derive_input: DeriveInput = parse_str(input).map_err(|e| e.to_string())?;
    let name = &derive_input.ident;
    let (impl_generics, type_generics, where_clause) = derive_input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics MyTrait for #name #type_generics #where_clause {
            fn trait_method(&self) -> String {
                stringify!(#name).to_string()
            }
        }
    })
}

/// Demonstrates how derive macros handle helper attributes.
/// A derive macro can define custom attributes that users can place
/// on fields or the type itself.
pub fn parse_with_helper_attrs(input: &str) -> Result<TokenStream, String> {
    let derive_input: DeriveInput = parse_str(input).map_err(|e| e.to_string())?;
    let name = &derive_input.ident;

    // Check for container-level attributes
    let skip_debug = derive_input.attrs.iter().any(|attr| {
        attr.path().is_ident("my_derive")
            && attr.parse_args::<Ident>().map(|id| id == "skip_debug").unwrap_or(false)
    });

    if skip_debug {
        Ok(quote! {
            // Skip Debug implementation
        })
    } else {
        Ok(quote! {
            impl std::fmt::Debug for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(stringify!(#name))
                }
            }
        })
    }
}

/// Demonstrates generating a Builder derive macro.
/// This is a complex derive that generates a builder struct and its methods.
pub fn generate_builder_derive(input: &str) -> Result<TokenStream, String> {
    let derive_input: DeriveInput = parse_str(input).map_err(|e| e.to_string())?;
    let name = &derive_input.ident;
    let builder_name = Ident::new(&format!("{name}Builder"), Span::call_site());

    let fields = match &derive_input.data {
        syn::Data::Struct(data) => extract_field_info(&data.fields)?,
        _ => return Err("Builder derive only supports structs".to_string()),
    };

    let field_decls: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, ftype)| {
            quote! { #fname: Option<#ftype> }
        })
        .collect();

    let field_methods: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, ftype)| {
            quote! {
                pub fn #fname(mut self, value: #ftype) -> Self {
                    self.#fname = Some(value);
                    self
                }
            }
        })
        .collect();

    let field_inits: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, _)| {
            quote! { #fname: None }
        })
        .collect();

    let field_extracts: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, _)| {
            let err_msg = format!("field `{}` not set", fname);
            quote! {
                #fname: self.#fname.ok_or(#err_msg)?
            }
        })
        .collect();

    Ok(quote! {
        pub struct #builder_name {
            #(#field_decls),*
        }

        impl #builder_name {
            pub fn new() -> Self {
                #builder_name {
                    #(#field_inits),*
                }
            }

            #(#field_methods)*

            pub fn build(self) -> Result<#name, String> {
                Ok(#name {
                    #(#field_extracts),*
                })
            }
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    })
}

/// Helper to extract field name and type pairs.
fn extract_field_info(fields: &syn::Fields) -> Result<Vec<(Ident, syn::Type)>, String> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => Ok(named
            .iter()
            .filter_map(|f| {
                let name = f.ident.as_ref()?;
                let ty = &f.ty;
                Some((name.clone(), ty.clone()))
            })
            .collect()),
        _ => Err("only named fields supported".to_string()),
    }
}

/// Demonstrates generating an enum with helper methods.
pub fn generate_enum_methods(input: &str) -> Result<TokenStream, String> {
    let derive_input: DeriveInput = parse_str(input).map_err(|e| e.to_string())?;
    let name = &derive_input.ident;

    let variants = match &derive_input.data {
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .map(|v| v.ident.clone())
            .collect::<Vec<_>>(),
        _ => return Err("only enums supported".to_string()),
    };

    let variant_arms: Vec<TokenStream> = variants
        .iter()
        .map(|v| {
            let v_str = v.to_string();
            quote! { Self::#v => #v_str }
        })
        .collect();

    let count = variants.len();

    Ok(quote! {
        impl #name {
            pub fn variant_name(&self) -> &'static str {
                match self {
                    #(#variant_arms),*
                }
            }

            pub fn variant_count() -> usize {
                #count
            }

            pub fn all_variants() -> &'static [Self] {
                &[#(Self::#variants),*]
            }
        }
    })
}

/// Demonstrates how to handle attributes on fields for serialization.
pub fn generate_serde_like_derive(input: &str) -> Result<TokenStream, String> {
    let derive_input: DeriveInput = parse_str(input).map_err(|e| e.to_string())?;
    let name = &derive_input.ident;

    let fields = match &derive_input.data {
        syn::Data::Struct(data) => extract_field_info(&data.fields)?,
        _ => return Err("only structs supported".to_string()),
    };

    let serialize_fields: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, _)| {
            let key = fname.to_string();
            quote! {
                map.insert(#key.to_string(), format!("{:?}", &self.#fname));
            }
        })
        .collect();

    Ok(quote! {
        impl #name {
            pub fn to_map(&self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #(#serialize_fields)*
                map
            }
        }
    })
}

/// A trait that the derive macro would implement.
pub trait MyTrait {
    fn trait_method(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_derive_struct() {
        let result = simulate_derive("struct Point { x: f64, y: f64 }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("MyTrait"));
        assert!(code.contains("Point"));
    }

    #[test]
    fn test_simulate_derive_enum() {
        let result = simulate_derive("enum Color { Red, Green, Blue }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("MyTrait"));
        assert!(code.contains("Color"));
    }

    #[test]
    fn test_simulate_derive_unit_struct() {
        let result = simulate_derive("struct Marker;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_generic_derive() {
        let result = generate_generic_derive("struct Wrapper<T> { inner: T }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("impl"));
        assert!(code.contains("Wrapper"));
    }

    #[test]
    fn test_parse_with_helper_attrs_skip() {
        let result = parse_with_helper_attrs("#[my_derive(skip_debug)] struct MyType {}");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        // Should be empty (skip_debug was set)
        assert!(code.trim().is_empty());
    }

    #[test]
    fn test_parse_with_helper_attrs_no_skip() {
        let result = parse_with_helper_attrs("struct MyType {}");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Debug"));
    }

    #[test]
    fn test_generate_builder_derive() {
        let result = generate_builder_derive("struct Config { host: String, port: u16 }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("ConfigBuilder"));
        assert!(code.contains("build"));
        assert!(code.contains("host"));
        assert!(code.contains("port"));
    }

    #[test]
    fn test_generate_builder_derive_not_struct() {
        let result = generate_builder_derive("enum Foo { A, B }");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_enum_methods() {
        let result = generate_enum_methods("enum Status { Active, Inactive, Pending }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("variant_name"));
        assert!(code.contains("variant_count"));
        assert!(code.contains("all_variants"));
    }

    #[test]
    fn test_generate_serde_like_derive() {
        let result = generate_serde_like_derive("struct User { name: String, age: u32 }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("to_map"));
        assert!(code.contains("HashMap"));
    }

    #[test]
    fn test_extract_field_names() {
        let item_struct: syn::ItemStruct =
            parse_str("struct Foo { bar: i32, baz: String }").unwrap();
        let names = extract_field_names(&item_struct.fields).unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].to_string(), "bar");
        assert_eq!(names[1].to_string(), "baz");
    }

    #[test]
    fn test_extract_field_info() {
        let item_struct: syn::ItemStruct =
            parse_str("struct Foo { bar: i32, baz: String }").unwrap();
        let info = extract_field_info(&item_struct.fields).unwrap();
        assert_eq!(info.len(), 2);
        assert_eq!(info[0].0.to_string(), "bar");
        assert_eq!(info[1].0.to_string(), "baz");
    }
}
