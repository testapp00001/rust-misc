//! # Lesson 05: Secure Clipboard Handling (Reference Solution)
//!
//! See the exercise file for full documentation on secure clipboard handling.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Simulated clipboard for cross-platform testing.
#[derive(Debug, Clone)]
pub struct SecureClipboard {
    inner: Arc<Mutex<ClipboardInner>>,
}

#[derive(Debug)]
struct ClipboardInner {
    content: Vec<u8>,
    clear_at: Option<std::time::Instant>,
}

impl SecureClipboard {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClipboardInner {
                content: Vec::new(),
                clear_at: None,
            })),
        }
    }

    pub fn read(&self) -> Vec<u8> {
        let inner = self.inner.lock().unwrap();
        inner.content.clone()
    }

    pub fn write(&self, data: &[u8]) {
        let mut inner = self.inner.lock().unwrap();
        inner.content = data.to_vec();
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        for byte in inner.content.iter_mut() {
            *byte = 0;
        }
        inner.content.clear();
        inner.clear_at = None;
    }

    pub fn check_auto_clear(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(clear_at) = inner.clear_at {
            if std::time::Instant::now() >= clear_at {
                for byte in inner.content.iter_mut() {
                    *byte = 0;
                }
                inner.content.clear();
                inner.clear_at = None;
                return true;
            }
        }
        false
    }

    pub fn schedule_clear(&self, duration: Duration) {
        let mut inner = self.inner.lock().unwrap();
        inner.clear_at = Some(std::time::Instant::now() + duration);
    }
}

impl Default for SecureClipboard {
    fn default() -> Self {
        Self::new()
    }
}

/// A clipboard audit log entry.
#[derive(Debug, Clone)]
pub struct ClipboardEvent {
    pub entry_name: String,
    pub timestamp: String,
    pub clear_after_secs: u64,
    pub cleared: bool,
}

/// Copy a password to the clipboard with auto-clear.
pub fn copy_password_to_clipboard(
    clipboard: &SecureClipboard,
    entry_name: &str,
    password: &str,
    timeout_secs: u64,
) -> ClipboardEvent {
    clipboard.write(password.as_bytes());
    clipboard.schedule_clear(Duration::from_secs(timeout_secs));
    ClipboardEvent {
        entry_name: entry_name.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        clear_after_secs: timeout_secs,
        cleared: false,
    }
}

/// Check if the clipboard has expired and clear it.
pub fn check_and_clear_expired(clipboard: &SecureClipboard) -> bool {
    clipboard.check_auto_clear()
}

/// Manually clear the clipboard immediately.
pub fn immediate_clear(clipboard: &SecureClipboard) -> usize {
    let size = clipboard.read().len();
    clipboard.clear();
    size
}

/// Copy a password and wait for auto-clear (blocking).
pub fn copy_and_wait_for_clear(
    clipboard: &SecureClipboard,
    entry_name: &str,
    password: &str,
    timeout_secs: u64,
) -> bool {
    copy_password_to_clipboard(clipboard, entry_name, password, timeout_secs);
    thread::sleep(Duration::from_secs(timeout_secs));
    check_and_clear_expired(clipboard)
}

/// Get clipboard contents as a UTF-8 string.
pub fn get_clipboard_string(clipboard: &SecureClipboard) -> Option<String> {
    let data = clipboard.read();
    if data.is_empty() {
        return None;
    }
    String::from_utf8(data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_and_read() {
        let clip = SecureClipboard::new();
        clip.write(b"mypassword");
        assert_eq!(clip.read(), b"mypassword");
    }

    #[test]
    fn test_clear_clipboard() {
        let clip = SecureClipboard::new();
        clip.write(b"secret");
        clip.clear();
        assert!(clip.read().is_empty());
    }

    #[test]
    fn test_copy_password_to_clipboard() {
        let clip = SecureClipboard::new();
        let event = copy_password_to_clipboard(&clip, "GitHub", "s3cret", 30);
        assert_eq!(event.entry_name, "GitHub");
        assert_eq!(event.clear_after_secs, 30);
        assert!(!event.cleared);
        assert_eq!(clip.read(), b"s3cret");
    }

    #[test]
    fn test_immediate_clear() {
        let clip = SecureClipboard::new();
        clip.write(b"password123");
        let cleared = immediate_clear(&clip);
        assert_eq!(cleared, 11);
        assert!(clip.read().is_empty());
    }

    #[test]
    fn test_auto_clear_with_short_timeout() {
        let clip = SecureClipboard::new();
        clip.write(b"secret");
        clip.schedule_clear(Duration::from_millis(50));
        assert!(!clip.check_auto_clear());
        assert_eq!(clip.read(), b"secret");
        thread::sleep(Duration::from_millis(100));
        assert!(clip.check_auto_clear());
        assert!(clip.read().is_empty());
    }

    #[test]
    fn test_get_clipboard_string() {
        let clip = SecureClipboard::new();
        clip.write(b"hello");
        assert_eq!(get_clipboard_string(&clip), Some("hello".to_string()));
    }

    #[test]
    fn test_get_clipboard_string_empty() {
        let clip = SecureClipboard::new();
        assert_eq!(get_clipboard_string(&clip), None);
    }

    #[test]
    fn test_copy_and_wait_for_clear() {
        let clip = SecureClipboard::new();
        let cleared = copy_and_wait_for_clear(&clip, "Test", "pass", 1);
        assert!(cleared, "Auto-clear should have fired");
        assert!(clip.read().is_empty());
    }
}
