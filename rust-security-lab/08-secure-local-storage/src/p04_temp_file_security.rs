//! # Lesson 04: Temporary File Security
//!
//! ## The Problem with Default Temp Files
//!
//! Temporary files are often created in `/tmp` (Linux/macOS) or `%TEMP%` (Windows)
//! with **world-readable permissions**. This means:
//!
//! - Any user on the system can read your temp files
//! - Symlink attacks can redirect your writes to arbitrary locations
//! - Temp files may persist if your process crashes
//! - Race conditions between file creation and permission setting
//!
//! ## Attack Scenario: Temp File Race Condition
//!
//! 1. Your app creates `/tmp/myapp_tempfile` (world-readable by default)
//! 2. Attacker reads the file before you set restrictive permissions
//! 3. Sensitive data (passwords, keys, decrypted content) is exposed
//!
//! ## Secure Temp File Strategy
//!
//! 1. **Create temp file in a private directory** (0700 permissions)
//! 2. **Set restrictive permissions** immediately (0600 — owner read/write only)
//! 3. **Use unique names** to prevent collision/overwrite attacks
//! 4. **Auto-delete** on drop (RAII pattern)
//! 5. **Overwrite before deleting** if the file contained sensitive data
//!
//! ## Platform Differences
//!
//! | Platform | Default Temp Dir | Default Permissions |
//! |----------|-----------------|-------------------|
//! | Linux    | /tmp            | 1777 (sticky + world-readable) |
//! | macOS    | /tmp or /var/folders | 1777 |
//! | Windows  | %TEMP%          | User-specific (better) |
//!
//! ## Rust's `tempfile` Crate
//!
//! The `tempfile` crate provides RAII-based temp files that auto-delete on drop.
//! However, it doesn't set restrictive permissions by default — you must do this
//! yourself for security-sensitive applications.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Exercise 1: Create a secure temporary directory.
///
/// Create a directory with 0700 permissions (owner only: read, write, execute).
/// This prevents other users from listing or accessing files in the directory.
///
/// # Arguments
/// * `parent` - Parent directory (e.g., `std::env::temp_dir()`)
/// * `prefix` - Directory name prefix
///
/// # Hints
/// - Generate a unique name using random numbers or timestamp
/// - `fs::create_dir(&path)?`
/// - Set permissions: `fs::set_permissions(&path, Permissions::from_mode(0o700))`
/// - Return the path to the created directory
///
/// # Platform Note
/// `Permissions::from_mode()` is Unix-only. On Windows, use ACLs or the `winapi` crate.
pub fn create_secure_temp_dir(parent: &Path, prefix: &str) -> io::Result<PathBuf> {
    todo!("Create a temporary directory with 0700 permissions")
}

/// Exercise 2: Create a secure temporary file.
///
/// Create a file with 0600 permissions (owner read/write only).
/// The file is created in the given directory with a random name.
///
/// # Arguments
/// * `dir` - Directory to create the file in
/// * `prefix` - Filename prefix
///
/// # Hints
/// - Generate a unique filename: `format!("{}_{}", prefix, random_number)`
/// - Use `OpenOptions::new().create_new(true).write(true).open(&path)`
/// - Set permissions immediately after creation
/// - Return the file handle and path
pub fn create_secure_temp_file(dir: &Path, prefix: &str) -> io::Result<(File, PathBuf)> {
    todo!("Create a temporary file with 0600 permissions")
}

/// Exercise 3: Create a temp file, write data, and auto-delete.
///
/// This demonstrates the RAII pattern — the file is automatically deleted
/// when the `SecureTempFile` is dropped.
///
/// # Hints
/// - Create a struct `SecureTempFile` that holds the path
/// - Implement `Drop` to delete the file
/// - Provide a method to write data
pub struct SecureTempFile {
    path: PathBuf,
}

impl SecureTempFile {
    /// Create a new secure temp file in the system temp directory.
    pub fn new(prefix: &str) -> io::Result<Self> {
        todo!("Create SecureTempFile with secure permissions")
    }

    /// Write data to the temp file.
    pub fn write(&self, data: &[u8]) -> io::Result<()> {
        todo!("Write data to the secure temp file")
    }

    /// Read the contents of the temp file.
    pub fn read(&self) -> io::Result<Vec<u8>> {
        todo!("Read data from the secure temp file")
    }

    /// Get the path to the temp file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SecureTempFile {
    /// Automatically delete the temp file when dropped.
    fn drop(&mut self) {
        todo!("Delete the temp file on drop (RAII cleanup)")
    }
}

/// Exercise 4: Create a temp file that overwrites before deleting.
///
/// For files containing sensitive data, we should overwrite before deleting
/// (combining temp file management with secure deletion).
///
/// # Hints
/// - On drop, overwrite file contents with zeros
/// - Then truncate to zero length
/// - Then delete the file
pub struct SecureEphemeralFile {
    path: PathBuf,
}

impl SecureEphemeralFile {
    pub fn new(prefix: &str) -> io::Result<Self> {
        todo!("Create SecureEphemeralFile")
    }

    pub fn write(&self, data: &[u8]) -> io::Result<()> {
        todo!("Write data")
    }

    pub fn read(&self) -> io::Result<Vec<u8>> {
        todo!("Read data")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SecureEphemeralFile {
    fn drop(&mut self) {
        todo!("Overwrite with zeros, then delete")
    }
}

/// Exercise 5: Validate that a path is safe for temp file creation.
///
/// Check that:
/// - The parent directory exists
/// - The parent directory has restrictive permissions (not world-writable)
/// - The path doesn't contain symlinks (to prevent symlink attacks)
///
/// # Hints
/// - `fs::metadata(parent)?.permissions().mode()` to check permissions
/// - Check that "other write" bit (0o002) is not set
/// - Use `fs::symlink_metadata(path)` to check without following symlinks
pub fn validate_temp_dir_safety(path: &Path) -> io::Result<bool> {
    todo!("Validate that a temp directory is safe to use")
}

/// Exercise 6: Get a secure temp directory path, creating it if necessary.
///
/// This function should:
/// 1. Check if a secure temp directory exists (e.g., `/tmp/.myapp_secure_XXXX`)
/// 2. If not, create it with 0700 permissions
/// 3. Return the path
///
/// # Hints
/// - Use a consistent naming scheme so the same directory is reused
/// - The directory should be owned by the current user
pub fn get_or_create_secure_temp_dir(app_name: &str) -> io::Result<PathBuf> {
    todo!("Get or create a secure temp directory for the application")
}

/// Exercise 7: Clean up old temp files (older than a given duration).
///
/// Temporary files may persist if the process crashes. This function
/// cleans up files older than the given age.
///
/// # Arguments
/// * `dir` - The temp directory to clean
/// * `max_age_secs` - Maximum age in seconds
///
/// # Hints
/// - Use `fs::metadata(path)?.modified()` to get modification time
/// - Compare with `SystemTime::now() - Duration::from_secs(max_age_secs)`
/// - Securely delete old files
pub fn cleanup_old_temp_files(dir: &Path, max_age_secs: u64) -> io::Result<usize> {
    todo!("Clean up temp files older than the given duration")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secure_temp_dir() {
        let parent = std::env::temp_dir();
        let result = create_secure_temp_dir(&parent, "test_secure");
        assert!(result.is_ok(), "Should create secure temp dir");

        let dir = result.unwrap();
        assert!(dir.exists(), "Directory should exist");

        // Check permissions (Unix only)
        #[cfg(unix)]
        {
            let mode = fs::metadata(&dir).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o700, "Should have 0700 permissions");
        }

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_create_secure_temp_file() {
        let dir = std::env::temp_dir().join("secure_temp_test");
        fs::create_dir_all(&dir).unwrap();

        let result = create_secure_temp_file(&dir, "test");
        assert!(result.is_ok());

        let (file, path) = result.unwrap();
        assert!(path.exists(), "File should exist");

        #[cfg(unix)]
        {
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "Should have 0600 permissions");
        }

        drop(file);
        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_secure_temp_file_raii() {
        let file = SecureTempFile::new("raii_test").unwrap();
        let path = file.path().to_path_buf();
        assert!(path.exists(), "File should exist while SecureTempFile is alive");

        drop(file);
        assert!(!path.exists(), "File should be deleted after drop");
    }

    #[test]
    fn test_secure_temp_file_write_read() {
        let file = SecureTempFile::new("write_read_test").unwrap();
        let data = b"temporary sensitive data";

        file.write(data).unwrap();
        let read_back = file.read().unwrap();
        assert_eq!(read_back, data);

        drop(file); // Cleanup
    }

    #[test]
    fn test_secure_ephemeral_file() {
        let file = SecureEphemeralFile::new("ephemeral_test").unwrap();
        let path = file.path().to_path_buf();

        file.write(b"sensitive data").unwrap();
        assert!(path.exists());

        drop(file);
        assert!(!path.exists(), "Ephemeral file should be deleted after drop");
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_temp_dir_safety() {
        // System temp dir should be safe
        let result = validate_temp_dir_safety(&std::env::temp_dir());
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_or_create_secure_temp_dir() {
        let dir = get_or_create_secure_temp_dir("test_app").unwrap();
        assert!(dir.exists(), "Secure temp dir should exist");

        #[cfg(unix)]
        {
            let mode = fs::metadata(&dir).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o700);
        }

        // Calling again should return the same directory
        let dir2 = get_or_create_secure_temp_dir("test_app").unwrap();
        assert_eq!(dir, dir2, "Should return the same directory on second call");

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_cleanup_old_temp_files() {
        let dir = std::env::temp_dir().join("cleanup_test");
        fs::create_dir_all(&dir).unwrap();

        // Create some test files
        for i in 0..3 {
            let path = dir.join(format!("old_file_{}", i));
            fs::write(&path, b"old data").unwrap();
        }

        // With max_age of 0, all files should be cleaned up
        let cleaned = cleanup_old_temp_files(&dir, 0).unwrap();
        assert_eq!(cleaned, 3, "Should clean up all 3 files");

        fs::remove_dir(&dir).ok();
    }
}
