//! # Lesson 05: Unix Filesystem Permissions for Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

/// Set file permissions to 0600 (owner read/write only).
///
/// This is the standard permission for files containing secrets.
/// Group and other users get no access whatsoever.
pub fn set_private_file(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, Permissions::from_mode(0o600))
}

/// Set directory permissions to 0700 (owner read/write/execute).
///
/// Execute permission on a directory means "can enter/list the directory."
/// 0700 means only the owner can access the directory contents.
pub fn set_private_dir(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, Permissions::from_mode(0o700))
}

/// Check if a file has secure (0600) permissions.
///
/// A file is "secure" if it has no group or other permissions.
/// We check that `mode & 0o077 == 0` (no group/other bits set)
/// and that owner has read+write (`mode & 0o600 != 0`).
pub fn has_secure_permissions(path: &Path) -> io::Result<bool> {
    let mode = fs::metadata(path)?.permissions().mode();

    // No group or other permissions
    let no_group_other = (mode & 0o077) == 0;

    // Owner has read + write
    let owner_rw = (mode & 0o600) != 0;

    Ok(no_group_other && owner_rw)
}

/// Check if a directory has secure (0700) permissions.
///
/// Similar to file check but requires execute permission (needed to enter).
pub fn has_secure_dir_permissions(path: &Path) -> io::Result<bool> {
    let mode = fs::metadata(path)?.permissions().mode();

    let no_group_other = (mode & 0o077) == 0;
    let owner_rwx = (mode & 0o700) != 0;

    Ok(no_group_other && owner_rwx)
}

/// Create a file with secure permissions atomically.
///
/// Uses `OpenOptions::mode()` to set permissions at creation time,
/// preventing any window where the file exists with wrong permissions.
/// `create_new(true)` fails if the file already exists, preventing
/// TOCTOU race conditions.
pub fn create_secure_file(path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)?;

    file.write_all(content)?;
    file.sync_data()
}

/// Repair insecure file permissions.
///
/// Strips group and other permissions while preserving owner permissions.
/// This is useful for fixing files that were created with overly permissive
/// settings (e.g., by a process with a permissive umask).
pub fn repair_permissions(path: &Path) -> io::Result<()> {
    let current_mode = fs::metadata(path)?.permissions().mode();

    // Keep only owner permission bits
    let repaired_mode = current_mode & 0o700;

    // Ensure owner has at least read+write
    let final_mode = repaired_mode | 0o600;

    fs::set_permissions(path, Permissions::from_mode(final_mode))
}

/// Find files with insecure permissions in a directory.
///
/// Returns paths of files that have any group or other permissions set.
/// This is useful for auditing a directory for security compliance.
pub fn find_insecure_files(dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    let mut insecure = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let mode = fs::metadata(&path)?.permissions().mode();
            if mode & 0o077 != 0 {
                insecure.push(path);
            }
        }
    }

    Ok(insecure)
}

/// Convert octal permission mode to a human-readable string.
///
/// Maps each permission bit to its character:
/// - Read (4) → 'r'
/// - Write (2) → 'w'
/// - Execute (1) → 'x'
/// - Not set (0) → '-'
pub fn permission_string(mode: u32) -> String {
    let mut result = String::with_capacity(9);

    // Owner permissions
    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Other permissions
    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
