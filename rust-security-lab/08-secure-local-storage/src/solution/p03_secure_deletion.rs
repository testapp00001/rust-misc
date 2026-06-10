//! # Lesson 03: Secure File Deletion (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Overwrite a file with random data for N passes.
///
/// Each pass fills the entire file with cryptographically random bytes
/// and flushes to disk with `sync_data()`. This makes forensic recovery
/// of the original data extremely difficult on traditional HDDs.
///
/// Note: On SSDs with wear-leveling, overwriting logical blocks may not
/// reach the physical blocks. Full-disk encryption is the only reliable
/// solution for SSDs.
pub fn overwrite_file(path: &Path, passes: usize) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len() as usize;

    for _ in 0..passes {
        let mut file = OpenOptions::new().write(true).open(path)?;
        let mut rng = rand::thread_rng();

        // Write random data in chunks to handle large files
        let chunk_size = 4096;
        let mut remaining = file_size;

        while remaining > 0 {
            let to_write = remaining.min(chunk_size);
            let chunk: Vec<u8> = (0..to_write).map(|_| rng.gen()).collect();
            file.write_all(&chunk)?;
            remaining -= to_write;
        }

        file.sync_data()?;
    }

    Ok(())
}

/// Securely delete a file: overwrite with random data, then unlink.
///
/// Combines overwriting (to prevent forensic recovery) with file deletion.
/// The number of passes controls the security level:
/// - 1 pass: Good for most use cases
/// - 3 passes: DoD 5220.22-M standard
/// - 7 passes: Gutmann's method (overkill for modern drives)
pub fn secure_delete(path: &Path, passes: usize) -> io::Result<()> {
    overwrite_file(path, passes)?;
    fs::remove_file(path)
}

/// Securely delete with rename: obscures the original filename.
///
/// Renaming to a random name before deletion prevents forensic tools
/// from recovering the original filename from directory entries.
pub fn secure_delete_with_rename(path: &Path, passes: usize) -> io::Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let random_name = format!("sd_{:016x}", rand::thread_rng().gen::<u64>());
    let random_path = parent.join(random_name);

    fs::rename(path, &random_path)?;
    secure_delete(&random_path, passes)
}

/// Overwrite file with zeros and delete.
///
/// Single-pass zero overwrite is fast and prevents casual recovery.
/// Not as secure as random data (magnetic force microscopy might detect
/// residual patterns), but sufficient for non-adversarial scenarios.
pub fn zero_and_delete(path: &Path) -> io::Result<()> {
    let file_size = fs::metadata(path)?.len() as usize;
    let mut file = OpenOptions::new().write(true).open(path)?;

    // Write zeros in chunks
    let chunk_size = 4096;
    let zero_chunk = vec![0u8; chunk_size];
    let mut remaining = file_size;

    while remaining > 0 {
        let to_write = remaining.min(chunk_size);
        file.write_all(&zero_chunk[..to_write])?;
        remaining -= to_write;
    }

    file.sync_data()?;
    fs::remove_file(path)
}

/// Verify that a file's contents have been overwritten.
///
/// Reads the file and checks that it no longer contains the original data.
/// Note: This is a heuristic check — it verifies the content changed,
/// not that it's cryptographically random.
pub fn verify_overwrite(path: &Path, original_data: &[u8]) -> io::Result<bool> {
    let current_data = fs::read(path)?;

    if current_data.len() != original_data.len() {
        return Ok(true); // Size changed — definitely overwritten
    }

    // Check if the content matches the original
    Ok(current_data != original_data)
}

/// Recursively secure-delete all files in a directory.
///
/// Walks the directory tree, securely deleting each file, then removes
/// the empty directories. This is useful for cleaning up temp directories
/// that may contain sensitive data.
pub fn secure_delete_dir(path: &Path, passes: usize) -> io::Result<()> {
    if !path.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Not a directory"));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            secure_delete_dir(&entry_path, passes)?;
        } else {
            secure_delete(&entry_path, passes)?;
        }
    }

    fs::remove_dir(path)
}

/// Shred a file: overwrite, truncate, rename, delete.
///
/// Each step makes forensic recovery progressively harder:
/// 1. Overwrite: Destroys file content
/// 2. Truncate: Destroys slack space
/// 3. Rename: Obscures original filename
/// 4. Delete: Removes directory entry
pub fn shred_file(path: &Path) -> io::Result<()> {
    // Step 1: Overwrite with random data
    overwrite_file(path, 1)?;

    // Step 2: Truncate to zero length (destroys slack space)
    let mut file = OpenOptions::new().write(true).open(path)?;
    file.set_len(0)?;
    file.sync_data()?;

    // Step 3: Rename to random name
    let parent = path.parent().unwrap_or(Path::new("."));
    let random_name = format!("shred_{:016x}", rand::thread_rng().gen::<u64>());
    let random_path = parent.join(random_name);
    fs::rename(path, &random_path)?;

    // Step 4: Delete
    fs::remove_file(&random_path)
}

/// Estimate the time required for secure deletion.
///
/// Calculates how long it would take to overwrite a file of given size
/// at the specified throughput. Useful for planning secure deletion of
/// large datasets.
pub fn estimate_deletion_time(file_size_bytes: u64, passes: usize, throughput_mbps: f64) -> f64 {
    let total_bytes = file_size_bytes as f64 * passes as f64;
    let throughput_bytes = throughput_mbps * 1_000_000.0;
    total_bytes / throughput_bytes
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
