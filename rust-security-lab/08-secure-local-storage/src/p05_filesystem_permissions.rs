//! # Lesson 05: Unix Filesystem Permissions for Security
//!
//! ## Understanding Unix Permissions
//!
//! Unix file permissions control who can read, write, and execute files.
//! Permissions are expressed in octal notation:
//!
//! ```text
//! Permission  Octal  Binary  Meaning
//! ---------   -----  ------  -------
//! rwx------   0700   111 000 000  Owner: read+write+execute
//! rw-------   0600   110 000 000  Owner: read+write (for files)
//! rwxr-x---   0750   111 101 000  Owner: rwx, Group: r-x
//! rw-r-----   0640   110 100 000  Owner: rw, Group: r--
//! rwxr-xr-x   0755   111 101 101  Owner: rwx, Group/Others: r-x
//! rw-r--r--   0644   110 100 100  Owner: rw, Group/Others: r--
//! ---------   0000   000 000 000  No permissions
//! ```
//!
//! ## Security Best Practices
//!
//! | File Type | Recommended Permission | Reason |
//! |-----------|----------------------|--------|
//! | Private keys | 0600 | Only owner should read |
//! | Config files (secrets) | 0600 | Contains sensitive data |
//! | Executables | 0755 | Everyone can run, only owner can modify |
//! | Home directory | 0700 | Prevent other users from listing |
//! | SSH directory | 0700 | Strict access control |
//!
//! ## Attack Scenario: Overly Permissive Files
//!
//! 1. Application writes API keys to a config file with 0644 permissions
//! 2. Any user on the system can read the file: `cat /etc/myapp/config`
//! 3. Attacker reads the API keys and gains unauthorized access
//!
//! ## Defense: Principle of Least Privilege
//!
//! Grant the minimum permissions necessary. A file that only the owner
//! needs to read should be 0600, not 0644.

use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Exercise 1: Set file permissions to 0600 (owner read/write only).
///
/// This is the standard permission for files containing secrets.
///
/// # Hints
/// - `fs::set_permissions(path, Permissions::from_mode(0o600))`
/// - Returns `io::Result<()>`
pub fn set_private_file(path: &Path) -> io::Result<()> {
    todo!("Set file permissions to 0600")
}

/// Exercise 2: Set directory permissions to 0700 (owner only).
///
/// This is the standard permission for private directories (e.g., ~/.ssh).
///
/// # Hints
/// - `fs::set_permissions(path, Permissions::from_mode(0o700))`
pub fn set_private_dir(path: &Path) -> io::Result<()> {
    todo!("Set directory permissions to 0700")
}

/// Exercise 3: Check if a file has secure permissions.
///
/// A file is "secure" if:
/// - Owner has read+write (0600)
/// - Group has NO permissions
/// - Others have NO permissions
///
/// # Hints
/// - `fs::metadata(path)?.permissions().mode()`
/// - Check that `mode & 0o077 == 0` (no group/other permissions)
/// - Check that `mode & 0o700` includes read and write
pub fn has_secure_permissions(path: &Path) -> io::Result<bool> {
    todo!("Check if file has secure (0600) permissions")
}

/// Exercise 4: Check if a directory has secure permissions.
///
/// A directory is "secure" if:
/// - Owner has read+write+execute (0700)
/// - Group has NO permissions
/// - Others have NO permissions
///
/// # Hints
/// - Similar to file check but for directories
/// - Execute permission on directories means "can enter/list"
pub fn has_secure_dir_permissions(path: &Path) -> io::Result<bool> {
    todo!("Check if directory has secure (0700) permissions")
}

/// Exercise 5: Create a file with secure permissions atomically.
///
/// The file must be created with 0600 permissions from the start — there
/// should be no window where the file exists with more permissive settings.
///
/// # Hints
/// - Use `OpenOptions::new().create_new(true).write(true).mode(0o600).open(path)`
/// - `create_new(true)` fails if the file already exists (prevents TOCTOU)
/// - `.mode()` is Unix-only
pub fn create_secure_file(path: &Path, content: &[u8]) -> io::Result<()> {
    todo!("Create a file with 0600 permissions atomically")
}

/// Exercise 6: Repair insecure file permissions.
///
/// If a file has overly permissive permissions, fix them:
/// - Remove group and other permissions
/// - Ensure owner has read+write
///
/// # Hints
/// - Read current permissions
/// - Mask with 0o700 to keep only owner bits
/// - Set the repaired permissions
pub fn repair_permissions(path: &Path) -> io::Result<()> {
    todo!("Fix overly permissive file permissions")
}

/// Exercise 7: List files with insecure permissions in a directory.
///
/// Scan a directory and return paths of files that have group or other
/// read permissions (i.e., permissions beyond 0600 for files).
///
/// # Hints
/// - Use `fs::read_dir(dir)` to list files
/// - For each file, check if `mode & 0o077 != 0`
/// - Return a Vec of insecure file paths
pub fn find_insecure_files(dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    todo!("Find files with insecure permissions in a directory")
}

/// Exercise 8: Get a human-readable permission string.
///
/// Convert octal permissions to a string like "rw-------" or "rwxr-xr-x".
///
/// # Hints
/// - Parse each octal digit (owner, group, other)
/// - Each digit: read=4, write=2, execute=1
/// - Map to characters: r, w, x (or - if not set)
pub fn permission_string(mode: u32) -> String {
    todo!("Convert octal mode to readable string like 'rw-------'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_set_private_file() {
        let dir = std::env::temp_dir().join("perm_test_1");
        fs::create_dir_all(&dir).unwrap();
        let path = create_test_file(&dir, "secret.txt", b"secret");

        set_private_file(&path).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "Should be 0600");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_set_private_dir() {
        let dir = std::env::temp_dir().join("perm_test_2");
        fs::create_dir_all(&dir).unwrap();

        set_private_dir(&dir).unwrap();

        let mode = fs::metadata(&dir).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700, "Should be 0700");

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_has_secure_permissions_true() {
        let dir = std::env::temp_dir().join("perm_test_3");
        fs::create_dir_all(&dir).unwrap();
        let path = create_test_file(&dir, "secure.txt", b"data");

        fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
        assert!(has_secure_permissions(&path).unwrap());

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_has_secure_permissions_false() {
        let dir = std::env::temp_dir().join("perm_test_4");
        fs::create_dir_all(&dir).unwrap();
        let path = create_test_file(&dir, "insecure.txt", b"data");

        fs::set_permissions(&path, Permissions::from_mode(0o644)).unwrap();
        assert!(!has_secure_permissions(&path).unwrap());

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_create_secure_file() {
        let dir = std::env::temp_dir().join("perm_test_5");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("atomic.txt");

        create_secure_file(&path, b"secure content").unwrap();

        assert!(path.exists());
        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"secure content");

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_repair_permissions() {
        let dir = std::env::temp_dir().join("perm_test_6");
        fs::create_dir_all(&dir).unwrap();
        let path = create_test_file(&dir, "needs_repair.txt", b"data");

        // Set overly permissive
        fs::set_permissions(&path, Permissions::from_mode(0o666)).unwrap();

        repair_permissions(&path).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "Group/other permissions should be removed");
        assert!(mode & 0o600 != 0, "Owner should have read+write");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_find_insecure_files() {
        let dir = std::env::temp_dir().join("perm_test_7");
        fs::create_dir_all(&dir).unwrap();

        let secure = create_test_file(&dir, "secure.txt", b"ok");
        let insecure = create_test_file(&dir, "insecure.txt", b"bad");

        fs::set_permissions(&secure, Permissions::from_mode(0o600)).unwrap();
        fs::set_permissions(&insecure, Permissions::from_mode(0o644)).unwrap();

        let insecure_files = find_insecure_files(&dir).unwrap();
        assert_eq!(insecure_files.len(), 1, "Should find 1 insecure file");
        assert_eq!(insecure_files[0], insecure);

        fs::remove_file(&secure).ok();
        fs::remove_file(&insecure).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_permission_string() {
        assert_eq!(permission_string(0o600), "rw-------");
        assert_eq!(permission_string(0o644), "rw-r--r--");
        assert_eq!(permission_string(0o755), "rwxr-xr-x");
        assert_eq!(permission_string(0o700), "rwx------");
        assert_eq!(permission_string(0o000), "---------");
    }
}
