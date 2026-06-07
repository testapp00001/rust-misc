//! # Mozilla UniFFI Patterns
//!
//! UniFFI generates multi-language bindings from Rust code. It supports
//! Kotlin, Swift, Python, and Ruby from a single Rust implementation.
//!
//! ## Key Concepts:
//!
//! - UDL files define the interface
//! - `#[derive(uniffi::Record)]` for data structures
//! - `#[uniffi::export]` for functions and methods
//! - Automatic binding generation for each target language

/// Simulated UniFFI-compatible record (data structure).
/// In real UniFFI, this would use #[derive(uniffi::Record)].
#[derive(Debug, Clone, PartialEq)]
pub struct UserProfile {
    pub user_id: u64,
    pub username: String,
    pub email: String,
    pub is_active: bool,
}

impl UserProfile {
    pub fn new(user_id: u64, username: &str, email: &str) -> Self {
        Self {
            user_id,
            username: username.into(),
            email: email.into(),
            is_active: true,
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn display_name(&self) -> String {
        format!("{} ({})", self.username, self.email)
    }
}

/// UniFFI-compatible enum.
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            _ => None,
        }
    }

    pub fn is_idempotent(&self) -> bool {
        matches!(self, Self::Get | Self::Put | Self::Delete)
    }
}

/// UniFFI-compatible error type.
#[derive(Debug, Clone)]
pub enum AppError {
    NotFound { resource: String, id: u64 },
    Validation { field: String, message: String },
    Unauthorized,
    Internal { message: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { resource, id } => {
                write!(f, "{} with id {} not found", resource, id)
            }
            Self::Validation { field, message } => {
                write!(f, "Validation error on '{}': {}", field, message)
            }
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::Internal { message } => write!(f, "Internal error: {}", message),
        }
    }
}

impl std::error::Error for AppError {}

/// UniFFI-compatible interface (trait).
/// In real UniFFI, this would use #[uniffi::export].
pub trait DataStore: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn delete(&self, key: &str) -> bool;
    fn list_keys(&self) -> Vec<String>;
}

/// In-memory implementation of DataStore.
pub struct MemoryStore {
    data: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl DataStore for MemoryStore {
    fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) {
        self.data
            .lock()
            .unwrap()
            .insert(key.into(), value.into());
    }

    fn delete(&self, key: &str) -> bool {
        self.data.lock().unwrap().remove(key).is_some()
    }

    fn list_keys(&self) -> Vec<String> {
        self.data.lock().unwrap().keys().cloned().collect()
    }
}

/// UniFFI-compatible callback type.
pub trait ProgressCallback: Send {
    fn on_progress(&self, percentage: f64, message: &str);
    fn on_complete(&self, success: bool);
}

/// Service that uses callbacks.
pub struct DownloadService {
    store: Box<dyn DataStore>,
}

impl DownloadService {
    pub fn new(store: Box<dyn DataStore>) -> Self {
        Self { store }
    }

    pub fn download(
        &self,
        url: &str,
        callback: &dyn ProgressCallback,
    ) -> Result<Vec<u8>, AppError> {
        if url.is_empty() {
            return Err(AppError::Validation {
                field: "url".into(),
                message: "URL cannot be empty".into(),
            });
        }

        callback.on_progress(0.0, "Starting download");
        callback.on_progress(50.0, "Downloading...");
        callback.on_progress(100.0, "Complete");

        let data = format!("Downloaded content from {}", url).into_bytes();
        self.store.set(url, &data.len().to_string());

        callback.on_complete(true);
        Ok(data)
    }
}

/// Binding generator configuration.
#[derive(Debug, Clone)]
pub struct BindingConfig {
    pub language: TargetLanguage,
    pub output_dir: String,
    pub module_name: String,
    pub generate_docs: bool,
}

#[derive(Debug, Clone)]
pub enum TargetLanguage {
    Kotlin,
    Swift,
    Python,
    Ruby,
}

impl BindingConfig {
    pub fn new(language: TargetLanguage, output_dir: &str, module_name: &str) -> Self {
        Self {
            language,
            output_dir: output_dir.into(),
            module_name: module_name.into(),
            generate_docs: true,
        }
    }

    pub fn file_extension(&self) -> &str {
        match self.language {
            TargetLanguage::Kotlin => "kt",
            TargetLanguage::Swift => "swift",
            TargetLanguage::Python => "py",
            TargetLanguage::Ruby => "rb",
        }
    }

    pub fn output_path(&self) -> String {
        format!(
            "{}/{}.{}",
            self.output_dir,
            self.module_name,
            self.file_extension()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile() {
        let mut profile = UserProfile::new(1, "alice", "alice@example.com");
        assert!(profile.is_active);
        assert_eq!(profile.display_name(), "alice (alice@example.com)");

        profile.deactivate();
        assert!(!profile.is_active);
    }

    #[test]
    fn test_http_method() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::Post));
        assert!(HttpMethod::Get.is_idempotent());
        assert!(!HttpMethod::Post.is_idempotent());
    }

    #[test]
    fn test_http_method_from_str_invalid() {
        assert!(HttpMethod::from_str("invalid").is_none());
    }

    #[test]
    fn test_app_error_display() {
        let err = AppError::NotFound {
            resource: "User".into(),
            id: 42,
        };
        assert!(format!("{}", err).contains("42"));

        let err = AppError::Validation {
            field: "email".into(),
            message: "invalid format".into(),
        };
        assert!(format!("{}", err).contains("email"));
    }

    #[test]
    fn test_memory_store() {
        let store = MemoryStore::new();
        store.set("key1", "value1");
        assert_eq!(store.get("key1"), Some("value1".into()));
        assert!(store.delete("key1"));
        assert_eq!(store.get("key1"), None);
    }

    #[test]
    fn test_memory_store_list_keys() {
        let store = MemoryStore::new();
        store.set("a", "1");
        store.set("b", "2");

        let mut keys = store.list_keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_download_service() {
        let store = Box::new(MemoryStore::new());
        let service = DownloadService::new(store);

        struct TestCallback;
        impl ProgressCallback for TestCallback {
            fn on_progress(&self, _pct: f64, _msg: &str) {}
            fn on_complete(&self, _success: bool) {}
        }

        let result = service.download("https://example.com", &TestCallback);
        assert!(result.is_ok());
    }

    #[test]
    fn test_download_service_empty_url() {
        let store = Box::new(MemoryStore::new());
        let service = DownloadService::new(store);

        struct TestCallback;
        impl ProgressCallback for TestCallback {
            fn on_progress(&self, _pct: f64, _msg: &str) {}
            fn on_complete(&self, _success: bool) {}
        }

        let result = service.download("", &TestCallback);
        assert!(result.is_err());
    }

    #[test]
    fn test_binding_config() {
        let config = BindingConfig::new(TargetLanguage::Kotlin, "out", "mymodule");
        assert_eq!(config.file_extension(), "kt");
        assert_eq!(config.output_path(), "out/mymodule.kt");
    }

    #[test]
    fn test_binding_config_all_languages() {
        for (lang, ext) in [
            (TargetLanguage::Kotlin, "kt"),
            (TargetLanguage::Swift, "swift"),
            (TargetLanguage::Python, "py"),
            (TargetLanguage::Ruby, "rb"),
        ] {
            let config = BindingConfig::new(lang, "out", "mod");
            assert_eq!(config.file_extension(), ext);
        }
    }
}
