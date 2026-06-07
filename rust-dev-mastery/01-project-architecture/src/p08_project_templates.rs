//! # Lesson 8: Project Templates
//!
//! Standard project layouts and scaffolding patterns ensure consistency
//! across teams. This lesson covers project templates, conventions,
//! and how to generate project boilerplate.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A file in a project template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub is_template: bool, // contains {{variables}}
}

/// A project template with configurable variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub name: String,
    pub description: String,
    pub variables: Vec<TemplateVariable>,
    pub files: Vec<TemplateFile>,
    pub post_create_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
}

impl ProjectTemplate {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            variables: Vec::new(),
            files: Vec::new(),
            post_create_commands: Vec::new(),
        }
    }

    pub fn add_variable(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        default: Option<impl Into<String>>,
        required: bool,
    ) {
        self.variables.push(TemplateVariable {
            name: name.into(),
            description: description.into(),
            default_value: default.map(|d| d.into()),
            required,
        });
    }

    pub fn add_file(
        &mut self,
        path: impl Into<String>,
        content: impl Into<String>,
        is_template: bool,
    ) {
        self.files.push(TemplateFile {
            path: path.into(),
            content: content.into(),
            is_template,
        });
    }

    pub fn add_post_command(&mut self, cmd: impl Into<String>) {
        self.post_create_commands.push(cmd.into());
    }

    /// Render a template file by replacing {{variables}} with values.
    pub fn render_file(
        &self,
        file: &TemplateFile,
        values: &BTreeMap<String, String>,
    ) -> Result<String, TemplateError> {
        if !file.is_template {
            return Ok(file.content.clone());
        }

        let mut result = file.content.clone();

        // Check all required variables are provided
        for var in &self.variables {
            let placeholder = format!("{{{{{}}}}}", var.name);
            if result.contains(&placeholder) && !values.contains_key(&name_of(var)) {
                if let Some(ref default) = var.default_value {
                    result = result.replace(&placeholder, default);
                } else if var.required {
                    return Err(TemplateError::MissingVariable(var.name.clone()));
                }
            }
        }

        // Replace all variables
        for (key, value) in values {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Generate the complete project with given variable values.
    pub fn generate(
        &self,
        values: &BTreeMap<String, String>,
    ) -> Result<Vec<(String, String)>, TemplateError> {
        let mut files = Vec::new();
        for file in &self.files {
            let rendered = self.render_file(file, values)?;
            files.push((file.path.clone(), rendered));
        }
        Ok(files)
    }
}

fn name_of(var: &TemplateVariable) -> String {
    var.name.clone()
}

#[derive(Debug, PartialEq)]
pub enum TemplateError {
    MissingVariable(String),
}

/// A standard Rust library project template.
pub fn library_template() -> ProjectTemplate {
    let mut tmpl = ProjectTemplate::new(
        "rust-library",
        "Standard Rust library project with tests and examples",
    );

    tmpl.add_variable("name", "Crate name", Some("my-lib"), true);
    tmpl.add_variable("author", "Author name", Some("Author"), true);
    tmpl.add_variable("description", "Crate description", Some("A Rust library"), false);

    tmpl.add_file(
        "Cargo.toml",
        "[package]\nname = \"{{name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\nauthors = [\"{{author}}\"]\ndescription = \"{{description}}\"\n\n[dependencies]\n\n[dev-dependencies]\n",
        true,
    );

    tmpl.add_file(
        "src/lib.rs",
        "//! {{description}}\n\n/// Add two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_add() {\n        assert_eq!(add(2, 3), 5);\n    }\n}\n",
        true,
    );

    tmpl.add_file("README.md", "# {{name}}\n\n{{description}}\n", true);
    tmpl.add_file(".gitignore", "/target\nCargo.lock\n", false);

    tmpl.add_post_command("cargo fmt");
    tmpl.add_post_command("cargo test");

    tmpl
}

/// A standard Rust CLI project template.
pub fn cli_template() -> ProjectTemplate {
    let mut tmpl = ProjectTemplate::new(
        "rust-cli",
        "Standard Rust CLI project with clap",
    );

    tmpl.add_variable("name", "Binary name", Some("my-cli"), true);
    tmpl.add_variable("author", "Author name", Some("Author"), true);

    tmpl.add_file(
        "Cargo.toml",
        "[package]\nname = \"{{name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\nauthors = [\"{{author}}\"]\n\n[[bin]]\nname = \"{{name}}\"\npath = \"src/main.rs\"\n\n[dependencies]\nanyhow = \"1\"\nserde = { version = \"1\", features = [\"derive\"] }\nserde_json = \"1\"\n",
        true,
    );

    tmpl.add_file(
        "src/main.rs",
        "use anyhow::Result;\nuse clap::Parser;\n\n#[derive(Parser, Debug)]\n#[command(name = \"{{name}}\", version, about)]\nstruct Args {\n    /// Input file path\n    #[arg(short, long)]\n    input: Option<String>,\n\n    /// Enable verbose output\n    #[arg(short, long, default_value_t = false)]\n    verbose: bool,\n}\n\nfn main() -> Result<()> {\n    let args = Args::parse();\n    if args.verbose {\n        eprintln!(\"Verbose mode enabled\");\n    }\n    println!(\"Hello from {{name}}!\");\n    Ok(())\n}\n",
        true,
    );

    tmpl.add_file(".gitignore", "/target\n", false);

    tmpl
}

/// A workspace project template for monorepos.
pub fn workspace_template() -> ProjectTemplate {
    let mut tmpl = ProjectTemplate::new(
        "rust-workspace",
        "Rust workspace with shared dependencies",
    );

    tmpl.add_variable("name", "Workspace name", Some("my-workspace"), true);

    tmpl.add_file(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\n    \"crates/core\",\n    \"crates/cli\",\n]\n\n[workspace.dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\nanyhow = \"1\"\n",
        true,
    );

    tmpl.add_file(
        "crates/core/Cargo.toml",
        "[package]\nname = \"{{name}}-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde.workspace = true\nanyhow.workspace = true\n",
        true,
    );

    tmpl.add_file("crates/core/src/lib.rs", "//! Core library\n\npub fn hello() -> &'static str {\n    \"Hello from core\"\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_hello() {\n        assert_eq!(hello(), \"Hello from core\");\n    }\n}\n", true);

    tmpl.add_file(
        "crates/cli/Cargo.toml",
        "[package]\nname = \"{{name}}-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"{{name}}\"\npath = \"src/main.rs\"\n\n[dependencies]\n{{name}}-core = { path = \"../core\" }\nanyhow.workspace = true\n",
        true,
    );

    tmpl.add_file(
        "crates/cli/src/main.rs",
        "use anyhow::Result;\n\nfn main() -> Result<()> {\n    println!(\"{}\", {{name}}_core::hello());\n    Ok(())\n}\n",
        true,
    );

    tmpl.add_file(".gitignore", "/target\nCargo.lock\n", false);

    tmpl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let tmpl = ProjectTemplate::new("test", "A test template");
        assert_eq!(tmpl.name, "test");
        assert!(tmpl.files.is_empty());
        assert!(tmpl.variables.is_empty());
    }

    #[test]
    fn test_render_simple_template() {
        let mut tmpl = ProjectTemplate::new("test", "test");
        tmpl.add_variable("name", "Name", Some("default"), true);
        tmpl.add_file("out.txt", "Hello {{name}}!", true);

        let mut values = BTreeMap::new();
        values.insert("name".to_string(), "World".to_string());

        let result = tmpl.render_file(&tmpl.files[0].clone(), &values).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_with_default() {
        let mut tmpl = ProjectTemplate::new("test", "test");
        tmpl.add_variable("name", "Name", Some("Default"), true);
        tmpl.add_file("out.txt", "Hello {{name}}!", true);

        let values = BTreeMap::new();
        let result = tmpl.render_file(&tmpl.files[0].clone(), &values).unwrap();
        assert_eq!(result, "Hello Default!");
    }

    #[test]
    fn test_render_missing_required_variable() {
        let mut tmpl = ProjectTemplate::new("test", "test");
        tmpl.add_variable("name", "Name", None::<&str>, true);
        tmpl.add_file("out.txt", "Hello {{name}}!", true);

        let values = BTreeMap::new();
        let result = tmpl.render_file(&tmpl.files[0].clone(), &values);
        assert!(matches!(result, Err(TemplateError::MissingVariable(_))));
    }

    #[test]
    fn test_render_non_template_file() {
        let mut tmpl = ProjectTemplate::new("test", "test");
        tmpl.add_file("raw.txt", "No {{variables}} here", false);

        let values = BTreeMap::new();
        let result = tmpl.render_file(&tmpl.files[0].clone(), &values).unwrap();
        assert_eq!(result, "No {{variables}} here");
    }

    #[test]
    fn test_generate_all_files() {
        let mut tmpl = ProjectTemplate::new("test", "test");
        tmpl.add_variable("name", "Name", Some("app"), true);
        tmpl.add_file("Cargo.toml", "[package]\nname = \"{{name}}\"", true);
        tmpl.add_file("src/lib.rs", "// {{name}}", true);

        let mut values = BTreeMap::new();
        values.insert("name".to_string(), "myapp".to_string());

        let files = tmpl.generate(&values).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files[0].1.contains("myapp"));
        assert!(files[1].1.contains("myapp"));
    }

    #[test]
    fn test_library_template() {
        let tmpl = library_template();
        assert!(!tmpl.files.is_empty());
        assert!(!tmpl.variables.is_empty());
        let var_names: Vec<&str> = tmpl.variables.iter().map(|v| v.name.as_str()).collect();
        assert!(var_names.contains(&"name"));
        assert!(var_names.contains(&"author"));
    }

    #[test]
    fn test_cli_template() {
        let tmpl = cli_template();
        assert!(tmpl.files.iter().any(|f| f.path == "src/main.rs"));
        assert!(tmpl.files.iter().any(|f| f.path == "Cargo.toml"));
    }

    #[test]
    fn test_workspace_template() {
        let tmpl = workspace_template();
        let paths: Vec<&str> = tmpl.files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"Cargo.toml"));
        assert!(paths.contains(&"crates/core/Cargo.toml"));
        assert!(paths.contains(&"crates/cli/Cargo.toml"));
    }

    #[test]
    fn test_post_commands() {
        let tmpl = library_template();
        assert!(!tmpl.post_create_commands.is_empty());
        assert!(tmpl.post_create_commands.contains(&"cargo test".to_string()));
    }

    #[test]
    fn test_multiple_variables() {
        let mut tmpl = ProjectTemplate::new("test", "test");
        tmpl.add_variable("name", "Name", Some("app"), true);
        tmpl.add_variable("version", "Version", Some("0.1.0"), false);
        tmpl.add_file("out", "{{name}} v{{version}}", true);

        let mut values = BTreeMap::new();
        values.insert("name".to_string(), "myapp".to_string());
        values.insert("version".to_string(), "1.0.0".to_string());

        let result = tmpl.render_file(&tmpl.files[0].clone(), &values).unwrap();
        assert_eq!(result, "myapp v1.0.0");
    }
}
