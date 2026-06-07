# Module 13: CLI Tools

Command-line application design with clap, interactive prompts, configuration management, cross-platform support, and testing.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `p01_clap_mastery.rs` | Derive API, subcommands, arguments, validation, help customization |
| 02 | `p02_interactive_prompts.rs` | Confirm, select, fuzzy select, password prompts, input patterns |
| 03 | `p03_progress_indicators.rs` | Progress bars, spinners, multi-progress, custom templates |
| 04 | `p04_config_management.rs` | TOML/YAML config, environment variables, layered configuration |
| 05 | `p05_cross_platform.rs` | Path handling, platform detection, conditional compilation, home directory |
| 06 | `p06_signal_handling.rs` | Ctrl+C, SIGTERM, graceful shutdown, signal hooks, cleanup |
| 07 | `p07_daemon_services.rs` | Background processes, PID files, systemd integration, service management |
| 08 | `p08_shell_completion.rs` | Shell completions for bash/zsh/fish, clap completions, dynamic completions |
| 09 | `p09_cli_architecture.rs` | Structuring CLI apps, command pattern, testable commands, separation of concerns |
| 10 | `p10_cli_testing.rs` | assert_cmd, predicates, stdin/stdout testing, snapshot testing CLI output |

## Running Tests

```bash
cargo test -p cli_tools
```
