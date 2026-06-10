//! # Lesson 05: Secure Clipboard Handling
//!
//! ## Copying Passwords Safely
//!
//! Users need to copy passwords from the vault to paste into login forms.
//! But the clipboard is a security risk: any application can read it, and
//! clipboard history can persist passwords indefinitely.
//!
//! ## Threat Model
//!
//! - **Clipboard sniffing**: Malware reads clipboard contents
//! - **Clipboard history**: OS/DE stores clipboard history with passwords
//! - **Forgetful users**: Password remains in clipboard after use
//!
//! ## Mitigations
//!
//! 1. **Auto-clear**: Overwrite clipboard after N seconds (default: 30)
//! 2. **Single-paste mode**: Clear clipboard after first paste (if supported)
//! 3. **Minimal exposure**: Copy only when explicitly requested
//!
//! ## Cross-Platform Reality
//!
//! Direct clipboard access requires platform-specific code or crates like
//! `arboard` or `cli-clipboard`. For this exercise, we simulate clipboard
//! operations with a thread-safe in-memory store.
//!
//! ## Implementation Pattern
//!
//! ```text
//! 1. User requests copy of "GitHub" password
//! 2. Copy password to clipboard
//! 3. Start a 30-second timer
//! 4. After 30 seconds, overwrite clipboard with zeros
//! 5. Log the event (without the password)
//! ```

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Utc;

/// Simulated clipboard for cross-platform testing.
/// In production, this would use the OS clipboard API.
#[derive(Debug, Clone)]
pub struct SecureClipboard {
    inner: Arc<Mutex<ClipboardInner>>,
}

#[derive(Debug)]
struct ClipboardInner {
    content: Vec<u8>,
    /// If set, clipboard should be cleared at this time
    clear_at: Option<std::time::Instant>,
}

impl SecureClipboard {
    /// Create a new secure clipboard.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClipboardInner {
                content: Vec::new(),
                clear_at: None,
            })),
        }
    }

    /// Read current clipboard content.
    pub fn read(&self) -> Vec<u8> {
        let inner = self.inner.lock().unwrap();
        inner.content.clone()
    }

    /// Write content to clipboard.
    pub fn write(&self, data: &[u8]) {
        let mut inner = self.inner.lock().unwrap();
        inner.content = data.to_vec();
    }

    /// Clear the clipboard (overwrite with zeros, then empty).
    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        for byte in inner.content.iter_mut() {
            *byte = 0;
        }
        inner.content.clear();
        inner.clear_at = None;
    }

    /// Check if the auto-clear timer has expired and clear if so.
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

    /// Schedule auto-clear after the given duration.
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
    /// What was copied (entry name, not the password)
    pub entry_name: String,
    /// When it was copied
    pub timestamp: String,
    /// Duration before auto-clear
    pub clear_after_secs: u64,
    /// Whether the clipboard was cleared
    pub cleared: bool,
}

/// Exercise 1: Copy a password to the clipboard with auto-clear.
///
/// 1. Write the password to the clipboard
/// 2. Schedule auto-clear after `timeout_secs` seconds
/// 3. Return a ClipboardEvent for the audit log
///
/// Hints:
/// - `clipboard.write(password.as_bytes())`
/// - `clipboard.schedule_clear(Duration::from_secs(timeout_secs))`
/// - Create a ClipboardEvent with `cleared: false`
pub fn copy_password_to_clipboard(
    clipboard: &SecureClipboard,
    entry_name: &str,
    password: &str,
    timeout_secs: u64,
) -> ClipboardEvent {
    todo!("Copy password to clipboard with auto-clear")
}

/// Exercise 2: Check if the clipboard has expired and clear it.
///
/// Returns true if the clipboard was cleared.
///
/// Hints:
/// - `clipboard.check_auto_clear()`
pub fn check_and_clear_expired(clipboard: &SecureClipboard) -> bool {
    todo!("Check if clipboard auto-clear has expired")
}

/// Exercise 3: Manually clear the clipboard immediately.
///
/// Also updates the event to reflect the clearing.
///
/// Hints:
/// - `clipboard.clear()`
/// - Return the number of bytes cleared
pub fn immediate_clear(clipboard: &SecureClipboard) -> usize {
    todo!("Immediately clear the clipboard")
}

/// Exercise 4: Copy a password and wait for auto-clear (blocking).
///
/// This simulates the real-world flow where the user copies a password,
/// pastes it, and the system auto-clears after the timeout.
///
/// Hints:
/// - Copy the password
/// - `thread::sleep(Duration::from_secs(timeout_secs))`
/// - Check and clear
/// - Return whether the auto-clear fired
pub fn copy_and_wait_for_clear(
    clipboard: &SecureClipboard,
    entry_name: &str,
    password: &str,
    timeout_secs: u64,
) -> bool {
    todo!("Copy and block until auto-clear")
}

/// Exercise 5: Get clipboard contents as a UTF-8 string.
///
/// Returns None if the clipboard is empty or contains invalid UTF-8.
///
/// Hints:
/// - `clipboard.read()`
/// - `String::from_utf8(data).ok()`
pub fn get_clipboard_string(clipboard: &SecureClipboard) -> Option<String> {
    todo!("Read clipboard as string")
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
        assert_eq!(cleared, 11); // "password123" = 11 bytes
        assert!(clip.read().is_empty());
    }

    #[test]
    fn test_auto_clear_with_short_timeout() {
        let clip = SecureClipboard::new();
        clip.write(b"secret");
        clip.schedule_clear(Duration::from_millis(50));
        // Not yet expired
        assert!(!clip.check_auto_clear());
        assert_eq!(clip.read(), b"secret");
        // Wait for expiration
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
