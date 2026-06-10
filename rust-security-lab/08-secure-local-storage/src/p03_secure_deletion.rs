//! # Lesson 03: Secure File Deletion
//!
//! ## The Problem: "Deleted" Files Aren't Really Deleted
//!
//! When you delete a file (e.g., `rm`, `unlink()`, `std::fs::remove_file()`), the
//! operating system merely marks the disk blocks as "available." The actual data
//! remains on disk until those blocks are overwritten by new data.
//!
//! This means:
//! - File recovery tools can restore "deleted" files
//! - Forensic analysis can read old data from disk
//! - SSD wear-leveling makes this even worse (data may persist in "spare" blocks)
//!
//! ## Secure Deletion Strategy
//!
//! 1. **Overwrite** the file contents with random data (multiple passes)
//! 2. **Flush** to disk with `fsync()` to ensure data is written
//! 3. **Rename** the file to a random name (obscures original filename)
//! 4. **Unlink** (delete) the file
//!
//! ## Attack Scenario: Data Recovery
//!
//! 1. User deletes a file containing sensitive data
//! 2. Attacker gains access to the disk (stolen laptop, forensics, cloud snapshot)
//! 3. Attacker uses recovery tools (PhotoRec, TestDisk) to restore the file
//! 4. Sensitive data is exposed
//!
//! ## Limitations
//!
//! - **SSDs**: Wear-leveling means overwriting a logical block may not overwrite
//!   the physical block. TRIM/discard helps but isn't guaranteed.
//! - **Journaling filesystems**: May keep copies of data in the journal.
//! - **Snapshots**: ZFS, btrfs, and cloud services keep historical versions.
//! - **Best practice**: Use full-disk encryption (Module 09) to make this moot.
//!
//! ## Defense: Full-Disk Encryption
//!
//! The strongest defense is full-disk encryption. If the disk is encrypted,
//! "deleted" data is cryptographically erased when the key is destroyed,
//! regardless of what's physically on disk.

use rand::Rng;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Exercise 1: Overwrite a file with random data.
///
/// This is the core of secure deletion — replace the file contents with
/// cryptographically random bytes before deleting.
///
/// # Arguments
/// * `path` - Path to the file to overwrite
/// * `passes` - Number of overwrite passes (more passes = more secure, but slower)
///
/// # Hints
/// - Open the file for writing: `File::create(path)` or `OpenOptions::new().write(true).open(path)`
/// - Get the file size: `fs::metadata(path)?.len()`
/// - For each pass, fill the file with random bytes
/// - Flush with `file.sync_data()` after each pass
/// - Use `rand::thread_rng()` for random bytes
///
/// # Security Note
/// On SSDs, overwriting may not reach the physical storage due to wear-leveling.
/// Full-disk encryption is the only reliable solution.
pub fn overwrite_file(path: &Path, passes: usize) -> io::Result<()> {
    todo!("Overwrite file contents with random data for N passes")
}

/// Exercise 2: Securely delete a file (overwrite + unlink).
///
/// Combines overwriting with random data and then removing the file.
///
/// # Arguments
/// * `path` - Path to the file to securely delete
/// * `passes` - Number of overwrite passes (typically 1-3; Gutmann's 35-pass is overkill)
///
/// # Hints
/// - Call `overwrite_file(path, passes)` first
/// - Then `fs::remove_file(path)`
/// - Return `io::Result<()>`
pub fn secure_delete(path: &Path, passes: usize) -> io::Result<()> {
    todo!("Securely delete file: overwrite with random data, then unlink")
}

/// Exercise 3: Securely delete a file and rename it first.
///
/// Rename the file to a random name before deletion. This obscures the
/// original filename from forensic analysis.
///
/// # Hints
/// - Generate a random filename in the same directory
/// - `fs::rename(path, &random_path)?`
/// - Then `secure_delete(&random_path, passes)`
pub fn secure_delete_with_rename(path: &Path, passes: usize) -> io::Result<()> {
    todo!("Rename file to random name, then securely delete")
}

/// Exercise 4: Overwrite a file with zeros (for quick sanitization).
///
/// Single-pass zero overwrite is fast and prevents casual recovery.
/// Not as secure as random data (patterns may be detectable), but
/// sufficient for non-adversarial scenarios.
///
/// # Hints
/// - Fill file with `0u8` bytes
/// - Single pass
/// - Sync and remove
pub fn zero_and_delete(path: &Path) -> io::Result<()> {
    todo!("Overwrite file with zeros and delete")
}

/// Exercise 5: Verify that a file's contents are overwritten.
///
/// Read the file after overwriting and check that it no longer contains
/// the original data.
///
/// # Hints
/// - Store the original hash/data before overwriting
/// - After overwriting, read the file and compare
/// - The contents should be random data, not the original
pub fn verify_overwrite(path: &Path, original_data: &[u8]) -> io::Result<bool> {
    todo!("Verify that file contents have been overwritten")
}

/// Exercise 6: Securely delete a directory and all its contents.
///
/// Recursively secure-delete all files in a directory, then remove the directory.
///
/// # Hints
/// - Use `fs::read_dir(path)` to list directory contents
/// - For each entry, check if it's a file or directory
/// - Securely delete files, recursively handle subdirectories
/// - Finally remove the empty directory
pub fn secure_delete_dir(path: &Path, passes: usize) -> io::Result<()> {
    todo!("Recursively secure-delete all files in a directory")
}

/// Exercise 7: Create a "shredded" file that looks like it was never there.
///
/// This function:
/// 1. Overwrites the file with random data
/// 2. Truncates the file to zero length
/// 3. Renames to a random name
/// 4. Removes the file
///
/// # Hints
/// - Use `file.set_len(0)` to truncate
/// - Each step makes forensic recovery harder
pub fn shred_file(path: &Path) -> io::Result<()> {
    todo!("Shred a file: overwrite, truncate, rename, delete")
}

/// Exercise 8: Estimate the time required for secure deletion.
///
/// Calculate how long it would take to overwrite a file of given size
/// at a given throughput (MB/s).
///
/// # Hints
/// - `time_seconds = (file_size_bytes * passes) / (throughput_mbps * 1_000_000)`
/// - Return as `f64`
pub fn estimate_deletion_time(file_size_bytes: u64, passes: usize, throughput_mbps: f64) -> f64 {
    todo!("Estimate secure deletion time in seconds")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_file(path: &Path, content: &[u8]) {
        let mut file = File::create(path).unwrap();
        file.write_all(content).unwrap();
        file.sync_data().unwrap();
    }

    #[test]
    fn test_overwrite_file() {
        let dir = std::env::temp_dir().join("secure_del_test_1");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        create_test_file(&path, b"sensitive data here");

        let result = overwrite_file(&path, 1);
        assert!(result.is_ok(), "Overwrite should succeed");

        // File should still exist but content should differ
        let content = fs::read(&path).unwrap();
        assert_ne!(content, b"sensitive data here", "Content should be overwritten");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_secure_delete() {
        let dir = std::env::temp_dir().join("secure_del_test_2");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("to_delete.txt");
        create_test_file(&path, b"delete me securely");

        let result = secure_delete(&path, 1);
        assert!(result.is_ok(), "Secure delete should succeed");
        assert!(!path.exists(), "File should be deleted");

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_secure_delete_with_rename() {
        let dir = std::env::temp_dir().join("secure_del_test_3");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("secret.txt");
        create_test_file(&path, b"rename and delete");

        let result = secure_delete_with_rename(&path, 1);
        assert!(result.is_ok());
        assert!(!path.exists(), "Original file should be gone");

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_zero_and_delete() {
        let dir = std::env::temp_dir().join("secure_del_test_4");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("zero.txt");
        create_test_file(&path, b"zero this out");

        let result = zero_and_delete(&path);
        assert!(result.is_ok());
        assert!(!path.exists(), "File should be deleted after zeroing");

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_verify_overwrite() {
        let dir = std::env::temp_dir().join("secure_del_test_5");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("verify.txt");
        let original = b"original content for verification";
        create_test_file(&path, original);

        overwrite_file(&path, 1).unwrap();

        let is_overwritten = verify_overwrite(&path, original).unwrap();
        assert!(is_overwritten, "Original content should not be present after overwrite");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_shred_file() {
        let dir = std::env::temp_dir().join("secure_del_test_6");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shred.txt");
        create_test_file(&path, b"shred this file");

        let result = shred_file(&path);
        assert!(result.is_ok());
        assert!(!path.exists(), "Shredded file should be deleted");

        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_estimate_deletion_time() {
        // 1GB file, 1 pass, 100 MB/s
        let time = estimate_deletion_time(1_000_000_000, 1, 100.0);
        assert!((time - 10.0).abs() < 0.01, "1GB at 100MB/s should take ~10 seconds");

        // 1GB file, 3 passes, 100 MB/s
        let time = estimate_deletion_time(1_000_000_000, 3, 100.0);
        assert!((time - 30.0).abs() < 0.01, "3 passes should take ~30 seconds");
    }

    #[test]
    fn test_secure_delete_nonexistent() {
        let path = Path::new("/tmp/nonexistent_file_that_does_not_exist_12345");
        let result = secure_delete(path, 1);
        assert!(result.is_err(), "Deleting nonexistent file should fail");
    }
}
