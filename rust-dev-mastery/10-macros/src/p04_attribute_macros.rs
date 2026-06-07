//! # Attribute Macros
//!
//! Attribute macros are applied to items (functions, structs, modules, etc.)
//! and can transform the item arbitrarily. They're defined with
//! `#[proc_macro_attribute]` and take two TokenStreams: the attribute arguments
//! and the item being decorated.
//!
//! Common uses:
//! - Web framework route handlers (`#[get("/path")]`)
//! - Test frameworks (`#[test]`)
//! - Logging/tracing instrumentation
//! - Code generation from annotations

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_str, Attribute, Expr, Ident, ItemFn, Meta};

/// Simulates an attribute macro that adds timing to a function.
/// In a real proc macro: `#[time_it] fn my_function() { ... }`
pub fn simulate_time_attribute(input: &str) -> Result<TokenStream, String> {
    let func: ItemFn = parse_str(input).map_err(|e| e.to_string())?;
    let name = &func.sig.ident;
    let name_str = name.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            let __start = std::time::Instant::now();
            let __result = (|| #block)();
            let __elapsed = __start.elapsed();
            println!("{} took {:?}", #name_str, __elapsed);
            __result
        }
    })
}

/// Simulates an attribute macro that wraps a function in error handling.
/// `#[catch_errors] fn risky() -> Result<T, E> { ... }`
pub fn simulate_catch_errors(input: &str) -> Result<TokenStream, String> {
    let func: ItemFn = parse_str(input).map_err(|e| e.to_string())?;
    let name = &func.sig.ident;
    let name_str = name.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            let result = (|| #block)();
            match &result {
                Ok(val) => println!("{} succeeded", #name_str),
                Err(e) => eprintln!("{} failed: {:?}", #name_str, e),
            }
            result
        }
    })
}

/// Simulates an attribute macro that adds a retry mechanism.
/// `#[retry(max_attempts = 3)] fn flaky() -> Result<T, E> { ... }`
pub fn simulate_retry(input: &str, max_attempts: u32) -> Result<TokenStream, String> {
    let func: ItemFn = parse_str(input).map_err(|e| e.to_string())?;
    let name = &func.sig.ident;
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            let mut __attempt = 0;
            loop {
                __attempt += 1;
                let __result = (|| #block)();
                match __result {
                    Ok(val) => break Ok(val),
                    Err(e) if __attempt < #max_attempts => {
                        eprintln!("Attempt {} failed, retrying...", __attempt);
                        std::thread::sleep(std::time::Duration::from_millis(100 * __attempt as u64));
                    }
                    Err(e) => break Err(e),
                }
            }
        }
    })
}

/// Simulates an attribute macro that adds logging.
/// `#[log_calls] fn process(data: &[u8]) -> usize { ... }`
pub fn simulate_log_calls(input: &str) -> Result<TokenStream, String> {
    let func: ItemFn = parse_str(input).map_err(|e| e.to_string())?;
    let name = &func.sig.ident;
    let name_str = name.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    // Extract parameter names for logging
    let param_names: Vec<String> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    return Some(pat_ident.ident.to_string());
                }
            }
            None
        })
        .collect();

    let log_params: Vec<TokenStream> = param_names
        .iter()
        .map(|name| {
            let ident = Ident::new(name, Span::call_site());
            quote! { stringify!(#ident) }
        })
        .collect();

    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            println!("{} called with params: [{}]", #name_str, [#(#log_params),*].join(", "));
            let __result = (|| #block)();
            println!("{} returned: {:?}", #name_str, &__result);
            __result
        }
    })
}

/// Simulates an attribute macro that adds a cache layer.
/// `#[cached] fn expensive(x: u64) -> u64 { ... }`
pub fn simulate_cached(input: &str) -> Result<TokenStream, String> {
    let func: ItemFn = parse_str(input).map_err(|e| e.to_string())?;
    let name = &func.sig.ident;
    let cache_name = Ident::new(&format!("__{}_cache", name), Span::call_site());
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    Ok(quote! {
        thread_local! {
            static #cache_name: std::cell::RefCell<std::collections::HashMap<String, String>>
                = std::cell::RefCell::new(std::collections::HashMap::new());
        }

        #(#attrs)*
        #vis #sig {
            let __key = stringify!(#name).to_string();
            #cache_name.with(|cache| {
                let mut cache = cache.borrow_mut();
                if let Some(cached) = cache.get(&__key) {
                    return cached.clone();
                }
                drop(cache);
                let result = (|| #block)();
                let result_str = format!("{:?}", result);
                #cache_name.with(|cache| {
                    cache.borrow_mut().insert(__key, result_str.clone());
                });
                result
            })
        }
    })
}

/// Demonstrates parsing attribute arguments.
/// Attribute macros can take arguments like `#[my_attr(key = "value", count = 5)]`
pub fn parse_attribute_args(args: &str) -> Result<Vec<(String, String)>, String> {
    let meta: Meta = parse_str(args).map_err(|e| e.to_string())?;

    match meta {
        Meta::List(list) => {
            let mut args = Vec::new();
            let tokens = list.tokens.to_string();
            // Simple key=value parsing
            for pair in tokens.split(',') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();
                    args.push((key, value));
                }
            }
            Ok(args)
        }
        _ => Err("expected attribute list".to_string()),
    }
}

/// Simulates a module-level attribute macro.
/// `#[generate_routes] mod api { ... }`
pub fn simulate_module_attribute(module_code: &str) -> Result<TokenStream, String> {
    let item: syn::ItemMod = parse_str(module_code).map_err(|e| e.to_string())?;
    let mod_name = &item.ident;

    // In a real implementation, we'd parse the module contents and
    // generate route handlers, middleware, etc.
    Ok(quote! {
        mod #mod_name {
            pub fn init() {
                println!("Module {} initialized", stringify!(#mod_name));
            }

            pub fn routes() -> Vec<(&'static str, fn())> {
                vec![]
            }
        }
    })
}

/// Simulates a conditional compilation attribute.
/// `#[cfg_feature("advanced")] fn advanced_feature() { ... }`
pub fn simulate_cfg_feature(input: &str, feature: &str) -> Result<TokenStream, String> {
    let func: ItemFn = parse_str(input).map_err(|e| e.to_string())?;
    let name = &func.sig.ident;
    let feature_str = feature.to_string();

    Ok(quote! {
        #[cfg(feature = #feature_str)]
        #func

        #[cfg(not(feature = #feature_str))]
        pub fn #name() -> &'static str {
            "Feature not enabled"
        }
    })
}

/// Demonstrates how to generate a struct from an attribute macro.
/// `#[derive_struct] struct Config { ... }` could generate a ConfigBuilder.
pub fn generate_struct_from_attr(
    name: &str,
    fields: &[(&str, &str)],
) -> TokenStream {
    let name_ident = Ident::new(name, Span::call_site());
    let builder_name = Ident::new(&format!("{name}Builder"), Span::call_site());

    let field_decls: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, ftype)| {
            let ident = Ident::new(fname, Span::call_site());
            let ty_ident = Ident::new(ftype, Span::call_site());
            quote! { pub #ident: #ty_ident }
        })
        .collect();

    let builder_fields: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, ftype)| {
            let ident = Ident::new(fname, Span::call_site());
            let ty_ident = Ident::new(ftype, Span::call_site());
            quote! { #ident: Option<#ty_ident> }
        })
        .collect();

    let builder_methods: Vec<TokenStream> = fields
        .iter()
        .map(|(fname, ftype)| {
            let ident = Ident::new(fname, Span::call_site());
            let ty_ident = Ident::new(ftype, Span::call_site());
            quote! {
                pub fn #ident(mut self, value: #ty_ident) -> Self {
                    self.#ident = Some(value);
                    self
                }
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone)]
        pub struct #name_ident {
            #(#field_decls),*
        }

        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            pub fn new() -> Self {
                unimplemented!("builder initialization")
            }

            #(#builder_methods)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_time_attribute() {
        let result = simulate_time_attribute("fn hello() { 42 }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Instant"));
        assert!(code.contains("elapsed"));
    }

    #[test]
    fn test_simulate_catch_errors() {
        let result = simulate_catch_errors(
            "fn risky() -> Result<i32, String> { Ok(42) }",
        );
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("succeeded"));
        assert!(code.contains("failed"));
    }

    #[test]
    fn test_simulate_retry() {
        let result = simulate_retry(
            "fn flaky() -> Result<i32, String> { Ok(42) }",
            3,
        );
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("attempt"));
        assert!(code.contains("retrying"));
    }

    #[test]
    fn test_simulate_log_calls() {
        let result = simulate_log_calls("fn process(x: i32, y: i32) -> i32 { x + y }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("process"));
        assert!(code.contains("params"));
    }

    #[test]
    fn test_parse_attribute_args() {
        let result = parse_attribute_args("route(method = \"GET\", path = \"/users\")");
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], ("method".to_string(), "GET".to_string()));
        assert_eq!(args[1], ("path".to_string(), "/users".to_string()));
    }

    #[test]
    fn test_simulate_module_attribute() {
        let result = simulate_module_attribute("mod api { }");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("api"));
        assert!(code.contains("init"));
        assert!(code.contains("routes"));
    }

    #[test]
    fn test_simulate_cfg_feature() {
        let result = simulate_cfg_feature("fn advanced() { }", "advanced");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("cfg"));
        assert!(code.contains("advanced"));
    }

    #[test]
    fn test_generate_struct_from_attr() {
        let code = generate_struct_from_attr(
            "Server",
            &[("host", "String"), ("port", "u16")],
        );
        let code_str = code.to_string();
        assert!(code_str.contains("Server"));
        assert!(code_str.contains("ServerBuilder"));
    }

    #[test]
    fn test_simulate_retry_no_args() {
        let result = simulate_retry("fn f() -> Result<(), String> { Ok(()) }", 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simulate_log_calls_no_params() {
        let result = simulate_log_calls("fn no_params() -> i32 { 42 }");
        assert!(result.is_ok());
    }
}
