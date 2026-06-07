//! # Cross-Platform Development
//!
//! Writing CLI tools that work across Linux, macOS, and Windows requires handling
//! differences in paths, file systems, shell behavior, and platform APIs. This
//! lesson covers the key patterns for cross-platform Rust CLI development.
//!
//! ## Key Concepts
//! - Path handling with `std::path` and the `dirs` crate
//! - Conditional compilation with `#[cfg()]`
//! - Platform detection at runtime
//! - Home directory, config directory, data directory discovery
//! - Line ending handling
//! - Process execution differences

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 1. Platform Detection
// ---------------------------------------------------------------------------

/// Runtime platform information.
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Unknown(String),
}

impl Platform {
    /// Detect the current platform at runtime.
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Unknown(std::env::consts::OS.to_string())
        }
    }

    pub fn is_unix(&self) -> bool {
        matches!(self, Platform::Linux | Platform::MacOS)
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows)
    }

    /// Get the path separator for this platform.
    pub fn path_separator(&self) -> char {
        if self.is_windows() {
            '\\'
        } else {
            '/'
        }
    }

    /// Get the executable file extension for this platform.
    pub fn exe_extension(&self) -> &'static str {
        if self.is_windows() {
            ".exe"
        } else {
            ""
        }
    }

    /// Get the line ending for this platform.
    pub fn line_ending(&self) -> &'static str {
        if self.is_windows() {
            "\r\n"
        } else {
            "\n"
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Directory Discovery
// ---------------------------------------------------------------------------

/// Standard directories for application data.
#[derive(Debug)]
pub struct AppDirectories {
    pub home: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
    pub temp: PathBuf,
}

impl AppDirectories {
    /// Discover standard directories for an application.
    pub fn discover(app_name: &str) -> Self {
        let home = dirs_home();

        let config = dirs_config_dir()
            .unwrap_or_else(|| home.join(".config"))
            .join(app_name);

        let data = dirs_data_dir()
            .unwrap_or_else(|| home.join(".local/share"))
            .join(app_name);

        let cache = dirs_cache_dir()
            .unwrap_or_else(|| home.join(".cache"))
            .join(app_name);

        let temp = std::env::temp_dir().join(app_name);

        Self {
            home,
            config,
            data,
            cache,
            temp,
        }
    }

    /// Ensure all directories exist.
    pub fn ensure_all(&self) -> std::io::Result<()> {
        for dir in [&self.config, &self.data, &self.cache] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

// Directory discovery functions (simulating the `dirs` crate)
fn dirs_home() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .or_else(|_| {
                let drive = std::env::var("HOMEDRIVE").unwrap_or_default();
                let path = std::env::var("HOMEPATH").unwrap_or_default();
                Ok(format!("{drive}{path}"))
            })
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }
}

fn dirs_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| Some(dirs_home().join(".config")))
    }

    #[cfg(target_os = "macos")]
    {
        Some(dirs_home().join("Library/Application Support"))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn dirs_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| Some(dirs_home().join(".local/share")))
    }

    #[cfg(target_os = "macos")]
    {
        Some(dirs_home().join("Library/Application Support"))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn dirs_cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_CACHE_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| Some(dirs_home().join(".cache")))
    }

    #[cfg(target_os = "macos")]
    {
        Some(dirs_home().join("Library/Caches"))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok().map(|p| PathBuf::from(p).join("Cache"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

// ---------------------------------------------------------------------------
// 3. Path Utilities
// ---------------------------------------------------------------------------

/// Normalize a path to use forward slashes (for consistent cross-platform display).
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Expand `~` in a path to the home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs_home().join(rest)
    } else if path == "~" {
        dirs_home()
    } else {
        PathBuf::from(path)
    }
}

/// Join paths in a cross-platform way.
pub fn join_paths(base: &str, segments: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(base);
    for segment in segments {
        path = path.join(segment);
    }
    path
}

/// Check if a path is absolute.
pub fn is_absolute(path: &str) -> bool {
    let p = Path::new(path);
    if p.is_absolute() {
        return true;
    }
    // On Linux, detect Windows-style absolute paths like "C:\Windows"
    if path.len() >= 3 {
        let bytes = path.as_bytes();
        if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
            return true;
        }
    }
    false
}

/// Get the file extension in a platform-independent way.
pub fn get_extension(path: &str) -> Option<&str> {
    Path::new(path).extension().and_then(|e| e.to_str())
}

/// Get the file stem (name without extension).
pub fn get_stem(path: &str) -> Option<&str> {
    Path::new(path).file_stem().and_then(|s| s.to_str())
}

// ---------------------------------------------------------------------------
// 4. Shell Detection
// ---------------------------------------------------------------------------

/// Detect the current shell environment.
#[derive(Debug, Clone, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Unknown,
}

impl Shell {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            if std::env::var("PSModulePath").is_ok() {
                Shell::PowerShell
            } else {
                Shell::Cmd
            }
        } else {
            let shell = std::env::var("SHELL").unwrap_or_default();
            if shell.contains("fish") {
                Shell::Fish
            } else if shell.contains("zsh") {
                Shell::Zsh
            } else if shell.contains("bash") {
                Shell::Bash
            } else {
                Shell::Unknown
            }
        }
    }

    /// Get the appropriate shell comment prefix.
    pub fn comment_prefix(&self) -> &'static str {
        match self {
            Shell::PowerShell => "#",
            Shell::Cmd => "REM",
            _ => "#",
        }
    }

    /// Get the config file name for this shell.
    pub fn config_file(&self) -> Option<&'static str> {
        match self {
            Shell::Bash => Some(".bashrc"),
            Shell::Zsh => Some(".zshrc"),
            Shell::Fish => Some("config.fish"),
            Shell::PowerShell => Some("profile.ps1"),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Platform-Specific Execution
// ---------------------------------------------------------------------------

/// Build a command that opens a URL in the default browser.
pub fn open_url_command(url: &str) -> Vec<String> {
    let platform = Platform::current();
    match platform {
        Platform::MacOS => vec!["open".into(), url.into()],
        Platform::Linux => vec!["xdg-open".into(), url.into()],
        Platform::Windows => vec!["cmd".into(), "/c".into(), "start".into(), url.into()],
        _ => vec!["echo".into(), url.into()],
    }
}

/// Build a command that opens a file with the default application.
pub fn open_file_command(path: &str) -> Vec<String> {
    let platform = Platform::current();
    match platform {
        Platform::MacOS => vec!["open".into(), path.into()],
        Platform::Linux => vec!["xdg-open".into(), path.into()],
        Platform::Windows => vec!["cmd".into(), "/c".into(), "start".into(), path.into()],
        _ => vec!["echo".into(), path.into()],
    }
}

// ---------------------------------------------------------------------------
// 6. Environment Variable Helpers
// ---------------------------------------------------------------------------

/// Get an environment variable with a platform-specific default.
pub fn env_or_default(key: &str, unix_default: &str, windows_default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            windows_default.into()
        } else {
            unix_default.into()
        }
    })
}

/// Check if we're running in a CI environment.
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("TRAVIS").is_ok()
        || std::env::var("CIRCLECI").is_ok()
}

/// Check if running in a container (Docker, etc.)
pub fn is_container() -> bool {
    Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || std::env::var("container").is_ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        // We know we're running on Linux in this environment
        assert_eq!(platform, Platform::Linux);
    }

    #[test]
    fn test_platform_is_unix() {
        assert!(Platform::Linux.is_unix());
        assert!(Platform::MacOS.is_unix());
        assert!(!Platform::Windows.is_unix());
    }

    #[test]
    fn test_platform_path_separator() {
        assert_eq!(Platform::Linux.path_separator(), '/');
        assert_eq!(Platform::Windows.path_separator(), '\\');
    }

    #[test]
    fn test_platform_exe_extension() {
        assert_eq!(Platform::Linux.exe_extension(), "");
        assert_eq!(Platform::Windows.exe_extension(), ".exe");
    }

    #[test]
    fn test_platform_line_ending() {
        assert_eq!(Platform::Linux.line_ending(), "\n");
        assert_eq!(Platform::Windows.line_ending(), "\r\n");
    }

    #[test]
    fn test_app_directories_discover() {
        let dirs = AppDirectories::discover("test-app");
        assert!(dirs.home.exists() || dirs.home == PathBuf::from("/tmp"));
        assert!(dirs.config.ends_with("test-app"));
        assert!(dirs.data.ends_with("test-app"));
        assert!(dirs.cache.ends_with("test-app"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("/usr/local/bin")), "/usr/local/bin");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/documents");
        assert!(!expanded.to_string_lossy().starts_with('~'));
        assert!(expanded.ends_with("documents"));
    }

    #[test]
    fn test_expand_tilde_bare() {
        let expanded = expand_tilde("~");
        assert!(!expanded.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let path = expand_tilde("/absolute/path");
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_join_paths() {
        let path = join_paths("/base", &["sub", "file.txt"]);
        assert_eq!(path, PathBuf::from("/base/sub/file.txt"));
    }

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute("/usr/bin"));
        assert!(is_absolute("C:\\Windows"));
        assert!(!is_absolute("relative/path"));
        assert!(!is_absolute("./relative"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("file.txt"), Some("txt"));
        assert_eq!(get_extension("archive.tar.gz"), Some("gz"));
        assert_eq!(get_extension("noext"), None);
        assert_eq!(get_extension("/path/to/file.rs"), Some("rs"));
    }

    #[test]
    fn test_get_stem() {
        assert_eq!(get_stem("file.txt"), Some("file"));
        assert_eq!(get_stem("archive.tar.gz"), Some("archive.tar"));
        assert_eq!(get_stem("noext"), Some("noext"));
    }

    #[test]
    fn test_shell_comment_prefix() {
        assert_eq!(Shell::Bash.comment_prefix(), "#");
        assert_eq!(Shell::Zsh.comment_prefix(), "#");
        assert_eq!(Shell::Fish.comment_prefix(), "#");
        assert_eq!(Shell::PowerShell.comment_prefix(), "#");
        assert_eq!(Shell::Cmd.comment_prefix(), "REM");
    }

    #[test]
    fn test_shell_config_file() {
        assert_eq!(Shell::Bash.config_file(), Some(".bashrc"));
        assert_eq!(Shell::Zsh.config_file(), Some(".zshrc"));
        assert_eq!(Shell::Fish.config_file(), Some("config.fish"));
        assert_eq!(Shell::PowerShell.config_file(), Some("profile.ps1"));
        assert_eq!(Shell::Cmd.config_file(), None);
    }

    #[test]
    fn test_open_url_command_linux() {
        let cmd = open_url_command("https://example.com");
        assert!(cmd.contains(&"xdg-open".to_string()));
        assert!(cmd.contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_open_file_command_linux() {
        let cmd = open_file_command("/path/to/file.pdf");
        assert!(cmd.contains(&"xdg-open".to_string()));
    }

    #[test]
    fn test_env_or_default() {
        // Without the env var set, should return the platform default
        let val = env_or_default("NONEXISTENT_VAR_12345", "unix-val", "win-val");
        // On Linux, should get unix-val
        assert_eq!(val, "unix-val");
    }

    #[test]
    fn test_is_ci() {
        // In this test environment, CI might or might not be set
        // Just verify the function doesn't panic
        let _ = is_ci();
    }

    #[test]
    fn test_is_container() {
        // Just verify the function doesn't panic
        let _ = is_container();
    }

    #[test]
    fn test_dirs_home_not_empty() {
        let home = dirs_home();
        assert!(!home.as_os_str().is_empty());
    }

    #[test]
    fn test_platform_display() {
        let platform = Platform::current();
        let debug_str = format!("{platform:?}");
        assert!(!debug_str.is_empty());
    }
}
