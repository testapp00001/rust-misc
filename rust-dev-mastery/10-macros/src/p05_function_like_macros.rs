//! # Function-Like Macros
//!
//! Function-like macros are invoked like function calls: `my_macro!(args)`.
//! They can take any number of tokens as input and produce any tokens as output.
//!
//! Common uses:
//! - DSLs (Domain Specific Languages)
//! - SQL query builders
//! - Template engines
//! - Code generation from inline specifications

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_str, Expr, Ident, LitStr};

/// Simulates a SQL-like DSL macro.
/// `sql!(SELECT * FROM users WHERE age > 18)`
pub fn simulate_sql_dsl(input: &str) -> Result<TokenStream, String> {
    // In a real proc macro, we'd parse the SQL-like syntax
    // Here we just wrap the input in a query builder call
    let query = input.trim();

    Ok(quote! {
        {
            let __query = #query;
            // In real code, this would parse and validate the SQL
            println!("Executing query: {}", __query);
            __query.to_string()
        }
    })
}

/// Simulates a JSON-like DSL macro.
/// `json!({"name": "Alice", "age": 30})`
pub fn simulate_json_dsl(key: &str, value: &str) -> Result<TokenStream, String> {
    Ok(quote! {
        {
            let mut __map = std::collections::HashMap::new();
            __map.insert(#key.to_string(), #value.to_string());
            __map
        }
    })
}

/// Simulates a logging macro with levels.
/// `log!(Info, "Processing {} items", count)`
pub fn simulate_log_macro(level: &str, message: &str) -> Result<TokenStream, String> {
    let level_ident = Ident::new(level, Span::call_site());

    Ok(quote! {
        println!("[{}] {}", stringify!(#level_ident), #message);
    })
}

/// Simulates a regex compilation macro.
/// `regex!(r"\d+\.\d+")`
pub fn simulate_regex_compile(pattern: &str) -> Result<TokenStream, String> {
    Ok(quote! {
        {
            // In real code, this would compile the regex at compile time
            let __pattern = #pattern;
            println!("Compiled regex: {}", __pattern);
            __pattern
        }
    })
}

/// Simulates a format template macro.
/// `template!("Hello, {name}! You are {age} years old.", name, age)`
pub fn simulate_template(template: &str, vars: &[&str]) -> Result<TokenStream, String> {
    let var_idents: Vec<Ident> = vars
        .iter()
        .map(|v| Ident::new(v, Span::call_site()))
        .collect();

    let replacements: Vec<TokenStream> = var_idents
        .iter()
        .map(|ident| {
            let placeholder = format!("{{{ident}}}");
            quote! {
                __result = __result.replace(#placeholder, &format!("{}", #ident));
            }
        })
        .collect();

    Ok(quote! {
        {
            let __template = #template;
            let mut __result = __template.to_string();
            #(#replacements)*
            __result
        }
    })
}

/// Simulates a bitflags macro.
/// `bitflags! { pub struct Flags: u32 { A = 1, B = 2, C = 4 } }`
pub fn simulate_bitflags(
    name: &str,
    backing: &str,
    flags: &[(&str, u64)],
) -> Result<TokenStream, String> {
    let name_ident = Ident::new(name, Span::call_site());
    let backing_type: syn::Type = parse_str(backing).unwrap();

    let flag_consts: Vec<TokenStream> = flags
        .iter()
        .map(|(fname, fval)| {
            let ident = Ident::new(fname, Span::call_site());
            quote! {
                pub const #ident: #name_ident = #name_ident(#fval as #backing_type);
            }
        })
        .collect();

    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct #name_ident(pub #backing_type);

        impl #name_ident {
            #(#flag_consts)*

            pub fn contains(self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            pub fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }

            pub fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }
        }

        impl std::ops::BitOr for #name_ident {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                #name_ident(self.0 | rhs.0)
            }
        }

        impl std::ops::BitAnd for #name_ident {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                #name_ident(self.0 & rhs.0)
            }
        }
    })
}

/// Simulates a table/row macro for data structures.
/// `table!(users { id: u64, name: String, email: String })`
pub fn simulate_table(table_name: &str, columns: &[(&str, &str)]) -> Result<TokenStream, String> {
    let table_ident = Ident::new(table_name, Span::call_site());
    let row_ident = Ident::new(&format!("{table_name}Row"), Span::call_site());

    let field_decls: Vec<TokenStream> = columns
        .iter()
        .map(|(col_name, col_type)| {
            let ident = Ident::new(col_name, Span::call_site());
            let ty: syn::Type = parse_str(col_type).unwrap();
            quote! { pub #ident: #ty }
        })
        .collect();

    let col_names: Vec<&str> = columns.iter().map(|(name, _)| *name).collect();

    Ok(quote! {
        pub struct #table_ident;

        impl #table_ident {
            pub fn table_name() -> &'static str {
                stringify!(#table_ident)
            }

            pub fn columns() -> &'static [&'static str] {
                &[#(stringify!(#col_names)),*]
            }
        }

        #[derive(Debug, Clone)]
        pub struct #row_ident {
            #(#field_decls),*
        }
    })
}

/// Simulates a permissions/roles macro.
/// `permissions!(Admin can [read, write, delete], User can [read])`
pub fn simulate_permissions(
    roles: &[(&str, &[&str])],
) -> TokenStream {
    let role_impls: Vec<TokenStream> = roles
        .iter()
        .map(|(role, perms)| {
            let role_ident = Ident::new(role, Span::call_site());
            let perm_checks: Vec<TokenStream> = perms
                .iter()
                .map(|perm| {
                    let perm_ident = Ident::new(perm, Span::call_site());
                    let method = Ident::new(&format!("can_{perm}"), Span::call_site());
                    quote! {
                        pub fn #method(&self) -> bool { true }
                    }
                })
                .collect();

            quote! {
                pub struct #role_ident;
                impl #role_ident {
                    #(#perm_checks)*
                }
            }
        })
        .collect();

    quote! {
        #(#role_impls)*
    }
}

/// Simulates an HTML template macro.
/// `html!(<div class="container"><h1>"Hello"</h1></div>)`
pub fn simulate_html(
    tag: &str,
    class: Option<&str>,
    children: &[&str],
) -> TokenStream {
    let class_attr = class.map(|c| format!(" class=\"{c}\"")).unwrap_or_default();
    let children_html: String = children.join("");

    let html = format!("<{tag}{class_attr}>{children_html}</{tag}>");

    quote! {
        {
            #html.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_sql_dsl() {
        let result = simulate_sql_dsl("SELECT * FROM users WHERE age > 18");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("SELECT"));
        assert!(code.contains("Executing query"));
    }

    #[test]
    fn test_simulate_json_dsl() {
        let result = simulate_json_dsl("name", "Alice");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("HashMap"));
        assert!(code.contains("name"));
    }

    #[test]
    fn test_simulate_log_macro() {
        let result = simulate_log_macro("Info", "Starting process");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Info"));
        assert!(code.contains("Starting process"));
    }

    #[test]
    fn test_simulate_regex_compile() {
        let result = simulate_regex_compile(r"\d+\.\d+");
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Compiled regex"));
    }

    #[test]
    fn test_simulate_bitflags() {
        let result = simulate_bitflags(
            "Permissions",
            "u32",
            &[("READ", 1), ("WRITE", 2), ("EXECUTE", 4)],
        );
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Permissions"));
        assert!(code.contains("READ"));
        assert!(code.contains("WRITE"));
        assert!(code.contains("contains"));
    }

    #[test]
    fn test_simulate_table() {
        let result = simulate_table(
            "users",
            &[("id", "u64"), ("name", "String"), ("email", "String")],
        );
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("users"));
        assert!(code.contains("usersRow"));
        assert!(code.contains("table_name"));
    }

    #[test]
    fn test_simulate_permissions() {
        let roles = vec![
            ("Admin", ["read", "write", "delete"].as_slice()),
            ("User", ["read"].as_slice()),
        ];
        let code = simulate_permissions(&roles);
        let code_str = code.to_string();
        assert!(code_str.contains("Admin"));
        assert!(code_str.contains("User"));
        assert!(code_str.contains("can_read"));
        assert!(code_str.contains("can_write"));
    }

    #[test]
    fn test_simulate_html() {
        let code = simulate_html("div", Some("container"), &["<p>Hello</p>"]);
        let code_str = code.to_string();
        assert!(code_str.contains("div"));
        assert!(code_str.contains("container"));
        assert!(code_str.contains("Hello"));
    }

    #[test]
    fn test_simulate_html_no_class() {
        let code = simulate_html("span", None, &["text"]);
        let code_str = code.to_string();
        assert!(code_str.contains("span"));
        assert!(code_str.contains("text"));
    }
}
