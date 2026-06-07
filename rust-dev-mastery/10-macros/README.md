# Module 10: Macros

Master Rust's macro system from declarative `macro_rules!` to procedural macros and build scripts.

## Lesson Index

1. **p01_declarative_macros.rs** - `macro_rules!`, patterns, repetitions, hygiene, `$crate`
2. **p02_proc_macro_basics.rs** - proc-macro crate setup, TokenStream, parsing, code generation
3. **p03_derive_macros.rs** - Custom derive, parsing structs/enums, generating impls, helper attributes
4. **p04_attribute_macros.rs** - `#[proc_macro_attribute]`, attribute arguments, item transformation
5. **p05_function_like_macros.rs** - `#[proc_macro]`, DSL macros, template macros
6. **p06_syn_parsing.rs** - syn crate, Parse trait, custom parsing, lookahead, error reporting
7. **p07_quote_generation.rs** - quote crate, interpolation, `#var`, spans, hygiene in generated code
8. **p08_macro_hygiene.rs** - Macro hygiene, span resolution, mixed site, def site, call site
9. **p09_macro_testing.rs** - Testing macros, trybuild, compile-test, macro expansion inspection
10. **p10_build_scripts.rs** - build.rs, `cargo:rustc-cfg`, code generation, linking native libs

## Key Concepts

- `macro_rules!` is for pattern-based metaprogramming within a single crate
- Procedural macros operate on `TokenStream` and can generate arbitrary code
- Hygiene prevents macro-generated identifiers from colliding with user code
- Build scripts run at compile time and can generate code, set cfg flags, and link native libraries
