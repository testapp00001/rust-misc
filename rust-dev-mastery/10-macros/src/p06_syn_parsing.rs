//! # syn Parsing
//!
//! The `syn` crate is the standard tool for parsing Rust code in proc macros.
//! It provides a complete Rust syntax tree and powerful parsing utilities.
//!
//! Key types:
//! - `syn::Item`: Any Rust item (fn, struct, enum, impl, etc.)
//! - `syn::DeriveInput`: Input to derive macros
//! - `syn::Expr`: Expressions
//! - `syn::Type`: Type expressions
//! - `syn::parse_str<T>()`: Parse a string into any parseable type
//!
//! This module demonstrates syn's parsing capabilities.

use syn::{
    parse::{Parse, ParseStream},
    parse_str,
    punctuated::Punctuated,
    Expr, ExprLit, Fields, FieldsNamed, Ident, Item, ItemFn, ItemStruct, Lit, LitStr, Meta,
    Result as SynResult, Token, Type,
};

/// Demonstrates parsing different Rust items.
pub fn parse_item(code: &str) -> Result<String, String> {
    let item: Item = parse_str(code).map_err(|e| e.to_string())?;

    match item {
        Item::Fn(item_fn) => Ok(format!("function: {}", item_fn.sig.ident)),
        Item::Struct(item_struct) => Ok(format!("struct: {}", item_struct.ident)),
        Item::Enum(item_enum) => Ok(format!("enum: {}", item_enum.ident)),
        Item::Impl(item_impl) => Ok(format!(
            "impl for: {}",
            quote::quote!(#(&item_impl.self_ty)).to_string()
        )),
        Item::Mod(item_mod) => Ok(format!("mod: {}", item_mod.ident)),
        Item::Trait(item_trait) => Ok(format!("trait: {}", item_trait.ident)),
        _ => Ok("other item".to_string()),
    }
}

/// Demonstrates parsing a function signature.
pub fn parse_function_signature(code: &str) -> Result<FunctionInfo, String> {
    let func: ItemFn = parse_str(code).map_err(|e| e.to_string())?;

    let params: Vec<ParamInfo> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                let name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    pat_ident.ident.to_string()
                } else {
                    "_".to_string()
                };
                let ty = quote::quote!(#(&pat_type.ty)).to_string();
                Some(ParamInfo { name, ty })
            } else {
                None
            }
        })
        .collect();

    let return_type = match &func.sig.output {
        syn::ReturnType::Default => "void".to_string(),
        syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
    };

    Ok(FunctionInfo {
        name: func.sig.ident.to_string(),
        params,
        return_type,
        is_async: func.sig.asyncness.is_some(),
        is_unsafe: func.sig.unsafety.is_some(),
    })
}

/// Demonstrates parsing a struct definition.
pub fn parse_struct(code: &str) -> Result<StructInfo, String> {
    let item_struct: ItemStruct = parse_str(code).map_err(|e| e.to_string())?;

    let fields = match &item_struct.fields {
        Fields::Named(FieldsNamed { named, .. }) => named
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().map(|id| id.to_string()).unwrap_or_default();
                let ty = quote::quote!(&f.ty).to_string();
                FieldInfo { name, ty }
            })
            .collect(),
        _ => vec![],
    };

    let generics: Vec<String> = item_struct
        .generics
        .params
        .iter()
        .map(|p| quote::quote!(#p).to_string())
        .collect();

    Ok(StructInfo {
        name: item_struct.ident.to_string(),
        fields,
        generics,
    })
}

/// Demonstrates parsing expressions.
pub fn parse_expression(code: &str) -> Result<ExprInfo, String> {
    let expr: Expr = parse_str(code).map_err(|e| e.to_string())?;

    match &expr {
        Expr::Lit(ExprLit { lit, .. }) => match lit {
            Lit::Int(i) => Ok(ExprInfo::Literal(i.base10_digits().to_string())),
            Lit::Float(f) => Ok(ExprInfo::Literal(f.base10_digits().to_string())),
            Lit::Str(s) => Ok(ExprInfo::Literal(s.value())),
            Lit::Bool(b) => Ok(ExprInfo::Literal(b.value.to_string())),
            _ => Ok(ExprInfo::Other),
        },
        Expr::Path(path) => Ok(ExprInfo::Path(
            path.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        )),
        Expr::Binary(binary) => {
            let op = quote::quote!(#binary.op).to_string();
            Ok(ExprInfo::Binary(op))
        }
        Expr::Call(call) => {
            let func = quote::quote!(#(&call.func)).to_string();
            Ok(ExprInfo::Call(func))
        }
        _ => Ok(ExprInfo::Other),
    }
}

/// Demonstrates parsing with the `Parse` trait.
/// This is how you define custom parsing logic for proc macro arguments.
pub struct MacroArgs {
    pub key: String,
    pub value: String,
    pub count: Option<u32>,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: LitStr = input.parse()?;

        let count = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let count_ident: Ident = input.parse()?;
            if count_ident == "count" {
                input.parse::<Token![=]>()?;
                let count_lit: syn::LitInt = input.parse()?;
                Some(count_lit.base10_parse()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(MacroArgs {
            key: key.to_string(),
            value: value.value(),
            count,
        })
    }
}

/// Demonstrates parsing comma-separated lists.
pub struct CommaList<T: Parse> {
    pub items: Punctuated<T, Token![,]>,
}

impl<T: Parse> Parse for CommaList<T> {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(CommaList {
            items: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Demonstrates lookahead parsing.
/// Lookahead lets you peek at the next token to decide how to parse.
pub fn parse_with_lookahead(code: &str) -> Result<String, String> {
    let expr: Expr = parse_str(code).map_err(|e| e.to_string())?;

    match &expr {
        Expr::Lit(lit) => {
            match &lit.lit {
                Lit::Str(s) => Ok(format!("string: {}", s.value())),
                Lit::Int(i) => Ok(format!("int: {}", i.base10_digits())),
                Lit::Float(f) => Ok(format!("float: {}", f.base10_digits())),
                Lit::Bool(b) => Ok(format!("bool: {}", b.value)),
                _ => Ok("other literal".to_string()),
            }
        }
        Expr::Path(path) => Ok(format!(
            "path: {}",
            path.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        )),
        _ => Ok("other expression".to_string()),
    }
}

/// Demonstrates parsing nested structures.
pub fn parse_nested_items(code: &str) -> Result<Vec<String>, String> {
    let item: Item = parse_str(code).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    match item {
        Item::Mod(item_mod) => {
            result.push(format!("mod: {}", item_mod.ident));
            if let Some((_, items)) = item_mod.content {
                for item in items {
                    match item {
                        Item::Fn(f) => result.push(format!("  fn: {}", f.sig.ident)),
                        Item::Struct(s) => result.push(format!("  struct: {}", s.ident)),
                        _ => {}
                    }
                }
            }
        }
        Item::Impl(item_impl) => {
            result.push(format!(
                "impl for: {}",
                quote::quote!(#(&item_impl.self_ty))
            ));
            for item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = item {
                    result.push(format!("  method: {}", method.sig.ident));
                }
            }
        }
        _ => result.push("other".to_string()),
    }

    Ok(result)
}

/// Demonstrates parsing types.
pub fn parse_type(code: &str) -> Result<TypeInfo, String> {
    let ty: Type = parse_str(code).map_err(|e| e.to_string())?;

    match &ty {
        Type::Path(path) => {
            let name = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            Ok(TypeInfo::Path(name))
        }
        Type::Reference(reference) => {
            let inner = quote::quote!(#(&reference.elem)).to_string();
            Ok(TypeInfo::Reference {
                inner,
                mutable: reference.mutability.is_some(),
            })
        }
        Type::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(|e| quote::quote!(#e).to_string())
                .collect();
            Ok(TypeInfo::Tuple(elems))
        }
        Type::Slice(slice) => {
            let elem = quote::quote!(&slice.elem).to_string();
            Ok(TypeInfo::Slice(elem))
        }
        _ => Ok(TypeInfo::Other),
    }
}

// Helper types for parsed information

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub return_type: String,
    pub is_async: bool,
    pub is_unsafe: bool,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub enum ExprInfo {
    Literal(String),
    Path(String),
    Binary(String),
    Call(String),
    Other,
}

#[derive(Debug, Clone)]
pub enum TypeInfo {
    Path(String),
    Reference { inner: String, mutable: bool },
    Tuple(Vec<String>),
    Slice(String),
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_item_function() {
        let result = parse_item("fn hello() { }");
        assert_eq!(result.unwrap(), "function: hello");
    }

    #[test]
    fn test_parse_item_struct() {
        let result = parse_item("struct Point { x: f64, y: f64 }");
        assert_eq!(result.unwrap(), "struct: Point");
    }

    #[test]
    fn test_parse_item_enum() {
        let result = parse_item("enum Color { Red, Green, Blue }");
        assert_eq!(result.unwrap(), "enum: Color");
    }

    #[test]
    fn test_parse_function_signature() {
        let result = parse_function_signature("fn add(a: i32, b: i32) -> i32 { a + b }");
        let info = result.unwrap();
        assert_eq!(info.name, "add");
        assert_eq!(info.params.len(), 2);
        assert_eq!(info.params[0].name, "a");
        assert_eq!(info.return_type, "i32");
        assert!(!info.is_async);
        assert!(!info.is_unsafe);
    }

    #[test]
    fn test_parse_function_async() {
        let result = parse_function_signature("async fn fetch() -> String { todo!() }");
        let info = result.unwrap();
        assert!(info.is_async);
    }

    #[test]
    fn test_parse_function_unsafe() {
        let result = parse_function_signature("unsafe fn dangerous() { }");
        let info = result.unwrap();
        assert!(info.is_unsafe);
    }

    #[test]
    fn test_parse_struct() {
        let result = parse_struct("struct Config { host: String, port: u16 }");
        let info = result.unwrap();
        assert_eq!(info.name, "Config");
        assert_eq!(info.fields.len(), 2);
        assert_eq!(info.fields[0].name, "host");
        assert_eq!(info.fields[1].name, "port");
    }

    #[test]
    fn test_parse_struct_generics() {
        let result = parse_struct("struct Wrapper<T> { inner: T }");
        let info = result.unwrap();
        assert_eq!(info.name, "Wrapper");
        assert_eq!(info.generics.len(), 1);
    }

    #[test]
    fn test_parse_expression_literal() {
        let result = parse_expression("42");
        assert!(matches!(result.unwrap(), ExprInfo::Literal(s) if s == "42"));
    }

    #[test]
    fn test_parse_expression_string() {
        let result = parse_expression("\"hello\"");
        assert!(matches!(result.unwrap(), ExprInfo::Literal(s) if s == "hello"));
    }

    #[test]
    fn test_parse_expression_path() {
        let result = parse_expression("std::io::stdout");
        assert!(matches!(result.unwrap(), ExprInfo::Path(s) if s == "std::io::stdout"));
    }

    #[test]
    fn test_parse_expression_binary() {
        let result = parse_expression("a + b");
        match result.unwrap() {
            ExprInfo::Binary(s) => assert!(s.contains("+")),
            other => panic!("expected binary expression, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_with_lookahead_string() {
        let result = parse_with_lookahead("\"hello\"");
        assert_eq!(result.unwrap(), "string: hello");
    }

    #[test]
    fn test_parse_with_lookahead_int() {
        let result = parse_with_lookahead("42");
        assert_eq!(result.unwrap(), "int: 42");
    }

    #[test]
    fn test_parse_with_lookahead_float() {
        let result = parse_with_lookahead("3.14");
        assert_eq!(result.unwrap(), "float: 3.14");
    }

    #[test]
    fn test_parse_nested_mod() {
        let result = parse_nested_items("mod api { fn handler() {} struct Request {} }");
        let items = result.unwrap();
        assert!(items.contains(&"mod: api".to_string()));
        assert!(items.contains(&"  fn: handler".to_string()));
        assert!(items.contains(&"  struct: Request".to_string()));
    }

    #[test]
    fn test_parse_type_path() {
        let result = parse_type("String");
        assert!(matches!(result.unwrap(), TypeInfo::Path(s) if s == "String"));
    }

    #[test]
    fn test_parse_type_reference() {
        let result = parse_type("&str");
        match result.unwrap() {
            TypeInfo::Reference { inner, mutable } => {
                // The inner type representation may vary
                assert!(!inner.is_empty());
                assert!(!mutable);
            }
            _ => panic!("expected reference"),
        }
    }

    #[test]
    fn test_parse_type_mutable_reference() {
        let result = parse_type("&mut Vec<u8>");
        match result.unwrap() {
            TypeInfo::Reference { mutable, .. } => {
                assert!(mutable);
            }
            _ => panic!("expected reference"),
        }
    }

    #[test]
    fn test_parse_type_tuple() {
        let result = parse_type("(i32, String)");
        match result.unwrap() {
            TypeInfo::Tuple(elems) => {
                assert_eq!(elems.len(), 2);
            }
            _ => panic!("expected tuple"),
        }
    }
}
