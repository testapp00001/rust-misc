//! # Shell Completions
//!
//! Shell completions dramatically improve CLI usability. This lesson covers
//! generating completions for bash, zsh, and fish shells, dynamic completions
//! that query the application, and clap's built-in completion support.
//!
//! ## Key Concepts
//! - Static completion script generation
//! - Bash, Zsh, and Fish completion syntax
//! - Dynamic completions
//! - Clap completion integration
//! - Completion installation

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Completion Script Generator
// ---------------------------------------------------------------------------

/// Generates shell completion scripts for different shells.
pub struct CompletionGenerator {
    program_name: String,
    subcommands: Vec<SubcommandInfo>,
    global_flags: Vec<FlagInfo>,
}

#[derive(Debug, Clone)]
pub struct SubcommandInfo {
    pub name: String,
    pub description: String,
    pub flags: Vec<FlagInfo>,
    pub args: Vec<ArgInfo>,
}

#[derive(Debug, Clone)]
pub struct FlagInfo {
    pub short: Option<char>,
    pub long: String,
    pub description: String,
    pub takes_value: bool,
}

#[derive(Debug, Clone)]
pub struct ArgInfo {
    pub name: String,
    pub required: bool,
    pub possible_values: Option<Vec<String>>,
}

impl CompletionGenerator {
    pub fn new(program_name: impl Into<String>) -> Self {
        Self {
            program_name: program_name.into(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
        }
    }

    pub fn add_subcommand(mut self, cmd: SubcommandInfo) -> Self {
        self.subcommands.push(cmd);
        self
    }

    pub fn add_global_flag(mut self, flag: FlagInfo) -> Self {
        self.global_flags.push(flag);
        self
    }

    /// Generate bash completion script.
    pub fn generate_bash(&self) -> String {
        let prog = &self.program_name;
        let mut script = String::new();

        script.push_str(&format!("# Bash completion for {prog}\n\n"));
        script.push_str(&format!("_{prog}() {{\n"));
        script.push_str("    local cur prev commands\n");
        script.push_str("    COMPREPLY=()\n");
        script.push_str("    cur=\"${COMP_WORDS[COMP_CWORD]}\"\n");
        script.push_str("    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n\n");

        // Subcommands
        let cmd_names: Vec<&str> = self.subcommands.iter().map(|c| c.name.as_str()).collect();
        script.push_str(&format!(
            "    commands=\"{}\"\n\n",
            cmd_names.join(" ")
        ));

        // Complete subcommands at position 1
        script.push_str("    if [[ $COMP_CWORD -eq 1 ]]; then\n");
        script.push_str(&format!(
            "        COMPREPLY=($(compgen -W \"{cmds}\" -- \"$cur\"))\n",
            cmds = cmd_names.join(" ")
        ));
        script.push_str("        return 0\n");
        script.push_str("    fi\n\n");

        // Complete flags for each subcommand
        script.push_str("    local cmd=\"${COMP_WORDS[1]}\"\n");
        script.push_str("    case \"$cmd\" in\n");

        for cmd in &self.subcommands {
            let flags: Vec<String> = cmd
                .flags
                .iter()
                .flat_map(|f| {
                    let mut opts = vec![format!("--{}", f.long)];
                    if let Some(short) = f.short {
                        opts.push(format!("-{short}"));
                    }
                    opts
                })
                .collect();

            script.push_str(&format!("        {})\n", cmd.name));
            script.push_str(&format!(
                "            COMPREPLY=($(compgen -W \"{}\" -- \"$cur\"))\n",
                flags.join(" ")
            ));
            script.push_str("            ;;\n");
        }

        script.push_str("        *)\n            ;;\n");
        script.push_str("    esac\n");
        script.push_str("}\n\n");
        script.push_str(&format!("complete -F _{prog} {prog}\n"));

        script
    }

    /// Generate zsh completion script.
    pub fn generate_zsh(&self) -> String {
        let prog = &self.program_name;
        let mut script = String::new();

        script.push_str(&format!("#compdef {prog}\n\n"));
        script.push_str(&format!("_{prog}() {{\n"));
        script.push_str("    local -a commands\n\n");

        // Subcommand descriptions
        script.push_str("    commands=(\n");
        for cmd in &self.subcommands {
            script.push_str(&format!(
                "        '{}:{}'\n",
                cmd.name, cmd.description
            ));
        }
        script.push_str("    )\n\n");

        script.push_str("    _arguments -C \\\n");
        script.push_str("        '1: :->cmds' \\\n");
        script.push_str("        '*::arg:->args'\n\n");

        script.push_str("    case $state in\n");
        script.push_str("        cmds)\n");
        script.push_str("            _describe 'command' commands\n");
        script.push_str("            ;;\n");
        script.push_str("        args)\n");
        script.push_str("            case $words[1] in\n");

        for cmd in &self.subcommands {
            script.push_str(&format!("                {})\n", cmd.name));
            script.push_str("                    _arguments \\\n");
            for flag in &cmd.flags {
                let spec = if flag.takes_value {
                    format!(
                        "'--{long}[{desc}]: :'",
                        long = flag.long,
                        desc = flag.description
                    )
                } else {
                    format!(
                        "'--{long}[{desc}]'",
                        long = flag.long,
                        desc = flag.description
                    )
                };
                if let Some(short) = flag.short {
                    script.push_str(&format!(
                        "                        '(-{short} --{long})'{{-{short},--{long}}}'[{desc}]' \\\n",
                        short = short,
                        long = flag.long,
                        desc = flag.description
                    ));
                } else {
                    script.push_str(&format!("                        {spec} \\\n"));
                }
            }
            script.push_str("                    ;;\n");
        }

        script.push_str("            esac\n");
        script.push_str("            ;;\n");
        script.push_str("    esac\n");
        script.push_str("}\n\n");
        script.push_str(&format!("_{prog} \"$@\"\n"));

        script
    }

    /// Generate fish completion script.
    pub fn generate_fish(&self) -> String {
        let prog = &self.program_name;
        let mut script = String::new();

        script.push_str(&format!("# Fish completion for {prog}\n\n"));

        // Global flags
        for flag in &self.global_flags {
            if let Some(short) = flag.short {
                script.push_str(&format!(
                    "complete -c {prog} -s {short} -l {long} -d '{desc}'\n",
                    long = flag.long,
                    desc = flag.description
                ));
            } else {
                script.push_str(&format!(
                    "complete -c {prog} -l {long} -d '{desc}'\n",
                    long = flag.long,
                    desc = flag.description
                ));
            }
        }

        script.push('\n');

        // Subcommands
        for cmd in &self.subcommands {
            script.push_str(&format!(
                "complete -c {prog} -f -n '__fish_use_subcommand' -a {name} -d '{desc}'\n",
                name = cmd.name,
                desc = cmd.description
            ));

            // Subcommand flags
            for flag in &cmd.flags {
                let condition = format!("__fish_seen_subcommand_from {}", cmd.name);
                if let Some(short) = flag.short {
                    script.push_str(&format!(
                        "complete -c {prog} -f -n '{cond}' -s {short} -l {long} -d '{desc}'\n",
                        cond = condition,
                        short = short,
                        long = flag.long,
                        desc = flag.description
                    ));
                } else {
                    script.push_str(&format!(
                        "complete -c {prog} -f -n '{cond}' -l {long} -d '{desc}'\n",
                        cond = condition,
                        long = flag.long,
                        desc = flag.description
                    ));
                }
            }

            // Args with possible values
            for arg in &cmd.args {
                if let Some(ref values) = arg.possible_values {
                    let condition = format!("__fish_seen_subcommand_from {}", cmd.name);
                    for value in values {
                        script.push_str(&format!(
                            "complete -c {prog} -f -n '{cond}' -a '{value}'\n",
                            cond = condition,
                            value = value
                        ));
                    }
                }
            }
        }

        script
    }
}

// ---------------------------------------------------------------------------
// 2. Completion Installation
// ---------------------------------------------------------------------------

/// Information about where to install completions for different shells.
#[derive(Debug, Clone)]
pub struct CompletionPaths {
    pub bash: String,
    pub zsh: String,
    pub fish: String,
}

impl CompletionPaths {
    pub fn for_program(program_name: &str) -> Self {
        Self {
            bash: format!("/etc/bash_completion.d/{program_name}"),
            zsh: format!(
                "/usr/share/zsh/site-functions/_{program_name}"
            ),
            fish: format!(
                "~/.config/fish/completions/{program_name}.fish"
            ),
        }
    }

    pub fn generate_install_script(&self, program_name: &str) -> String {
        format!(
            "#!/bin/bash\n\
             # Install shell completions for {program_name}\n\n\
             echo \"Installing bash completion...\"\n\
             mkdir -p /etc/bash_completion.d\n\
             {program_name} completion bash > {bash_path}\n\n\
             echo \"Installing zsh completion...\"\n\
             mkdir -p /usr/share/zsh/site-functions\n\
             {program_name} completion zsh > {zsh_path}\n\n\
             echo \"Installing fish completion...\"\n\
             mkdir -p ~/.config/fish/completions\n\
             {program_name} completion fish > {fish_path}\n\n\
             echo \"Done! Restart your shell to activate completions.\"\n",
            bash_path = self.bash,
            zsh_path = self.zsh,
            fish_path = self.fish,
        )
    }
}

// ---------------------------------------------------------------------------
// 3. Dynamic Completion Provider
// ---------------------------------------------------------------------------

/// A trait for providing dynamic completions at runtime.
pub trait CompletionProvider {
    /// Provide completions for a given context.
    fn complete(&self, context: &CompletionContext) -> Vec<CompletionItem>;
}

#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub current_word: String,
    pub previous_word: String,
    pub subcommand: Option<String>,
    pub all_words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    pub value: String,
    pub description: Option<String>,
}

impl CompletionItem {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A file-path completion provider.
pub struct PathCompleter {
    show_hidden: bool,
}

impl PathCompleter {
    pub fn new(show_hidden: bool) -> Self {
        Self { show_hidden }
    }
}

impl CompletionProvider for PathCompleter {
    fn complete(&self, context: &CompletionContext) -> Vec<CompletionItem> {
        let input = &context.current_word;
        let (dir, prefix) = if let Some(pos) = input.rfind('/') {
            (&input[..pos + 1], &input[pos + 1..])
        } else {
            (".", input.as_str())
        };

        let path = std::path::Path::new(dir);
        let mut items = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();

                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }

                if name.starts_with(prefix) {
                    let is_dir = entry.path().is_dir();
                    let display = if is_dir {
                        format!("{name}/")
                    } else {
                        name.clone()
                    };
                    items.push(CompletionItem::new(display));
                }
            }
        }

        items
    }
}

// ---------------------------------------------------------------------------
// 4. Completion Cache
// ---------------------------------------------------------------------------

/// Caches completion results for performance.
pub struct CompletionCache {
    cache: HashMap<String, (Vec<CompletionItem>, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl CompletionCache {
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<CompletionItem>> {
        self.cache
            .get(key)
            .filter(|(_, time)| time.elapsed() < self.ttl)
            .map(|(items, _)| items)
    }

    pub fn set(&mut self, key: String, items: Vec<CompletionItem>) {
        self.cache
            .insert(key, (items, std::time::Instant::now()));
    }

    pub fn invalidate(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_generator() -> CompletionGenerator {
        CompletionGenerator::new("myapp")
            .add_global_flag(FlagInfo {
                short: Some('v'),
                long: "verbose".into(),
                description: "Enable verbose output".into(),
                takes_value: false,
            })
            .add_global_flag(FlagInfo {
                short: Some('f'),
                long: "format".into(),
                description: "Output format".into(),
                takes_value: true,
            })
            .add_subcommand(SubcommandInfo {
                name: "init".into(),
                description: "Initialize a new project".into(),
                flags: vec![FlagInfo {
                    short: Some('t'),
                    long: "template".into(),
                    description: "Template to use".into(),
                    takes_value: true,
                }],
                args: vec![ArgInfo {
                    name: "name".into(),
                    required: true,
                    possible_values: None,
                }],
            })
            .add_subcommand(SubcommandInfo {
                name: "build".into(),
                description: "Build the project".into(),
                flags: vec![
                    FlagInfo {
                        short: Some('r'),
                        long: "release".into(),
                        description: "Build in release mode".into(),
                        takes_value: false,
                    },
                    FlagInfo {
                        short: None,
                        long: "target".into(),
                        description: "Target platform".into(),
                        takes_value: true,
                    },
                ],
                args: vec![],
            })
    }

    #[test]
    fn test_bash_completion_contains_program_name() {
        let gen = test_generator();
        let script = gen.generate_bash();
        assert!(script.contains("# Bash completion for myapp"));
        assert!(script.contains("_myapp()"));
        assert!(script.contains("complete -F _myapp myapp"));
    }

    #[test]
    fn test_bash_completion_contains_subcommands() {
        let gen = test_generator();
        let script = gen.generate_bash();
        assert!(script.contains("init"));
        assert!(script.contains("build"));
    }

    #[test]
    fn test_bash_completion_contains_flags() {
        let gen = test_generator();
        let script = gen.generate_bash();
        assert!(script.contains("--release"));
        assert!(script.contains("--target"));
        assert!(script.contains("--template"));
    }

    #[test]
    fn test_zsh_completion_contains_compdef() {
        let gen = test_generator();
        let script = gen.generate_zsh();
        assert!(script.contains("#compdef myapp"));
    }

    #[test]
    fn test_zsh_completion_contains_descriptions() {
        let gen = test_generator();
        let script = gen.generate_zsh();
        assert!(script.contains("Initialize a new project"));
        assert!(script.contains("Build the project"));
    }

    #[test]
    fn test_fish_completion_contains_subcommands() {
        let gen = test_generator();
        let script = gen.generate_fish();
        assert!(script.contains("complete -c myapp"));
        assert!(script.contains("__fish_use_subcommand"));
        assert!(script.contains("-a init"));
        assert!(script.contains("-a build"));
    }

    #[test]
    fn test_fish_completion_contains_flags() {
        let gen = test_generator();
        let script = gen.generate_fish();
        assert!(script.contains("-l release"));
        assert!(script.contains("-l target"));
        assert!(script.contains("-l verbose"));
    }

    #[test]
    fn test_fish_completion_contains_global_flags() {
        let gen = test_generator();
        let script = gen.generate_fish();
        assert!(script.contains("-s v -l verbose"));
        assert!(script.contains("-s f -l format"));
    }

    #[test]
    fn test_completion_paths() {
        let paths = CompletionPaths::for_program("myapp");
        assert!(paths.bash.contains("myapp"));
        assert!(paths.zsh.contains("myapp"));
        assert!(paths.fish.contains("myapp"));
    }

    #[test]
    fn test_install_script() {
        let paths = CompletionPaths::for_program("mytool");
        let script = paths.generate_install_script("mytool");
        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("mytool completion bash"));
        assert!(script.contains("mytool completion zsh"));
        assert!(script.contains("mytool completion fish"));
    }

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new("value").with_description("A value");
        assert_eq!(item.value, "value");
        assert_eq!(item.description.as_deref(), Some("A value"));
    }

    #[test]
    fn test_completion_item_no_description() {
        let item = CompletionItem::new("test");
        assert!(item.description.is_none());
    }

    #[test]
    fn test_path_completer() {
        let completer = PathCompleter::new(false);
        let ctx = CompletionContext {
            current_word: "/".into(),
            previous_word: String::new(),
            subcommand: None,
            all_words: vec!["myapp".into()],
        };

        let items = completer.complete(&ctx);
        // Root directory should have entries
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completion_cache() {
        let mut cache = CompletionCache::new(std::time::Duration::from_secs(60));

        // Cache miss
        assert!(cache.get("key").is_none());

        // Set and get
        let items = vec![
            CompletionItem::new("a"),
            CompletionItem::new("b"),
        ];
        cache.set("key".into(), items);
        assert!(cache.get("key").is_some());
        assert_eq!(cache.get("key").unwrap().len(), 2);

        // Invalidate
        cache.invalidate("key");
        assert!(cache.get("key").is_none());
    }

    #[test]
    fn test_completion_cache_clear() {
        let mut cache = CompletionCache::new(std::time::Duration::from_secs(60));
        cache.set("a".into(), vec![]);
        cache.set("b".into(), vec![]);

        cache.clear();
        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_none());
    }

    #[test]
    fn test_completion_generator_empty() {
        let gen = CompletionGenerator::new("empty");
        let bash = gen.generate_bash();
        assert!(bash.contains("empty"));

        let zsh = gen.generate_zsh();
        assert!(zsh.contains("empty"));

        let fish = gen.generate_fish();
        assert!(fish.contains("empty"));
    }

    #[test]
    fn test_subcommand_info_debug() {
        let cmd = SubcommandInfo {
            name: "test".into(),
            description: "Test command".into(),
            flags: vec![],
            args: vec![],
        };
        let debug = format!("{cmd:?}");
        assert!(debug.contains("test"));
    }
}
