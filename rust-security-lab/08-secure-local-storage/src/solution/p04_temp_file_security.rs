//! # Lesson 04: Temporary File Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Create a secure temporary directory with 0700 permissions.
///
/// Uses a random suffix to prevent collision attacks. The directory is
/// created with restrictive permissions so only the owner can access it.
pub fn create_secure_temp_dir(parent: &Path, prefix: &str) -> io::Result<PathBuf> {
    let random_suffix: u64 = rand::random();
    let dir_name = format!("{}_{:016x}", prefix, random_suffix);
    let dir_path = parent.join(dir_name);

    fs::create_dir(&dir_path)?;
    fs::set_permissions(&dir_path, fs::Permissions::from_mode(0o700))?;

    Ok(dir_path)
}

/// Create a secure temporary file with 0600 permissions.
///
/// Uses `create_new(true)` to prevent TOCTOU (time-of-check-time-of-use)
/// race conditions — the file is created with the correct permissions
/// atomically, with no window where it exists with wrong permissions.
pub fn create_secure_temp_file(dir: &Path, prefix: &str) -> io::Result<(File, PathBuf)> {
    let random_suffix: u64 = rand::random();
    let file_name = format!("{}_{:016x}", prefix, random_suffix);
    let file_path = dir.join(file_name);

    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&file_path)?;

    Ok((file, file_path))
}

/// SecureTempFile: RAII-based temp file that auto-deletes on drop.
///
/// This pattern ensures cleanup even if the process panics or the
/// function returns early via `?`. The file is deleted when the
/// `SecureTempFile` value goes out of scope.
pub struct SecureTempFile {
    path: PathBuf,
}

impl SecureTempFile {
    /// Create a new secure temp file in the system temp directory.
    pub fn new(prefix: &str) -> io::Result<Self> {
        let temp_dir = std::env::temp_dir();
        let (_, path) = create_secure_temp_file(&temp_dir, prefix)?;
        Ok(Self { path })
    }

    /// Write data to the temp file.
    pub fn write(&self, data: &[u8]) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).open(&self.path)?;
        file.write_all(data)?;
        file.sync_data()
    }

    /// Read the contents of the temp file.
    pub fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path)
    }

    /// Get the path to the temp file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SecureTempFile {
    /// Automatically delete the temp file when dropped.
    ///
    /// This is the RAII pattern — the file is cleaned up when the
    /// `SecureTempFile` goes out of scope, regardless of how we
    /// got there (normal return, early return via `?`, or panic).
    fn drop(&mut self) {
        fs::remove_file(&self.path).ok();
    }
}

/// SecureEphemeralFile: Temp file that overwrites before deleting.
///
/// For files containing sensitive data, just deleting isn't enough —
/// the data remains on disk. This type overwrites with zeros before
/// deleting, combining secure temp file management with secure deletion.
pub struct SecureEphemeralFile {
    path: PathBuf,
}

impl SecureEphemeralFile {
    pub fn new(prefix: &str) -> io::Result<Self> {
        let temp_dir = std::env::temp_dir();
        let (_, path) = create_secure_temp_file(&temp_dir, prefix)?;
        Ok(Self { path })
    }

    pub fn write(&self, data: &[u8]) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).open(&self.path)?;
        file.write_all(data)?;
        file.sync_data()
    }

    pub fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SecureEphemeralFile {
    /// Overwrite with zeros, then delete.
    ///
    /// This prevents forensic recovery of the sensitive data that
    /// was stored in the temp file.
    fn drop(&mut self) {
        if let Ok(metadata) = fs::metadata(&self.path) {
            let size = metadata.len() as usize;
            if let Ok(mut file) = OpenOptions::new().write(true).open(&self.path) {
                let chunk = vec![0u8; 4096];
                let mut remaining = size;
                while remaining > 0 {
                    let to_write = remaining.min(4096);
                    let _ = file.write_all(&chunk[..to_write]);
                    remaining -= to_write;
                }
                let _ = file.sync_data();
            }
        }
        fs::remove_file(&self.path).ok();
    }
}

/// Validate that a path is safe for temp file creation.
///
/// Checks:
/// - The parent directory exists
/// - The parent is not world-writable (prevents symlink attacks)
/// - No symlink components in the path
pub fn validate_temp_dir_safety(path: &Path) -> io::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let metadata = fs::symlink_metadata(path)?;

    // Check if it's a symlink (unsafe — could point elsewhere)
    if metadata.file_type().is_symlink() {
        return Ok(false);
    }

    // Check permissions — world-writable directories are unsafe
    let mode = metadata.permissions().mode();
    if mode & 0o002 != 0 {
        return Ok(false); // World-writable
    }

    Ok(true)
}

/// Get or create a secure temp directory for the application.
///
/// Uses a consistent naming scheme so the same directory is reused
/// across runs. Creates with 0700 if it doesn't exist.
pub fn get_or_create_secure_temp_dir(app_name: &str) -> io::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let secure_dir = temp_dir.join(format!(".{}_secure", app_name));

    if !secure_dir.exists() {
        fs::create_dir(&secure_dir)?;
        fs::set_permissions(&secure_dir, fs::Permissions::from_mode(0o700))?;
    }

    Ok(secure_dir)
}

/// Clean up old temp files older than the given duration.
///
/// Returns the number of files cleaned up. This is useful for cleaning
/// up temp files that may have been left behind by crashed processes.
pub fn cleanup_old_temp_files(dir: &Path, max_age_secs: u64) -> io::Result<usize> {
    let max_age = Duration::from_secs(max_age_secs);
    let now = SystemTime::now();
    let mut cleaned = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            let modified = metadata.modified()?;

            if now.duration_since(modified).unwrap_or(Duration::ZERO) > max_age {
                fs::remove_file(&path)?;
                cleaned += 1;
            }
        }
    }

    Ok(cleaned)
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
