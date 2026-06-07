/// Problem: File I/O
///
/// Master Rust's file I/O operations.
///
/// Key Concepts:
/// - Reading files
/// - Writing files
/// - Appending to files
/// - File metadata
/// - Directory operations

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};
use std::path::Path;

/// Problem 1: Read file to string
/// Read entire file to string
pub fn read_to_string(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Problem 2: Write string to file
/// Write string to file
pub fn write_to_file(path: &str, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

/// Problem 3: Append to file
/// Append content to file
pub fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Problem 4: Read file line by line
/// Read file line by line
pub fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(lines)
}

/// Problem 5: Write file with buffered writer
/// Use buffered writer
pub fn write_buffered(path: &str, content: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(content.as_bytes())?;
    writer.flush()?;
    Ok(())
}

/// Problem 6: Read file metadata
/// Get file metadata
pub fn file_metadata(path: &str) -> io::Result<(u64, bool)> {
    let metadata = fs::metadata(path)?;
    Ok((metadata.len(), metadata.is_dir()))
}

/// Problem 7: Check if file exists
/// Check file existence
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Problem 8: Create directory
/// Create directory
pub fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
}

/// Problem 9: List directory contents
/// List files in directory
pub fn list_directory(path: &str) -> io::Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        entries.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(entries)
}

/// Problem 10: Copy file
/// Copy file
pub fn copy_file(from: &str, to: &str) -> io::Result<u64> {
    fs::copy(from, to)
}

/// Problem 11: Delete file
/// Delete file
pub fn delete_file(path: &str) -> io::Result<()> {
    fs::remove_file(path)
}

/// Problem 12: Delete directory
/// Delete directory
pub fn delete_directory(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
}

/// Problem 13: Read file bytes
/// Read file as bytes
pub fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// Problem 14: Write bytes to file
/// Write bytes to file
pub fn write_bytes(path: &str, data: &[u8]) -> io::Result<()> {
    fs::write(path, data)
}

/// Problem 15: Read file with custom reader
/// Use custom reader
pub fn read_with_custom_reader(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_to_string() {
        let path = "test_read.txt";
        fs::write(path, "Hello, World!").unwrap();
        assert_eq!(read_to_string(path).unwrap(), "Hello, World!");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_write_to_file() {
        let path = "test_write.txt";
        write_to_file(path, "Hello, World!").unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "Hello, World!");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_append_to_file() {
        let path = "test_append.txt";
        fs::write(path, "Hello").unwrap();
        append_to_file(path, ", World!").unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "Hello, World!");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_read_lines() {
        let path = "test_lines.txt";
        fs::write(path, "line1\nline2\nline3").unwrap();
        assert_eq!(read_lines(path).unwrap(), vec!["line1", "line2", "line3"]);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_write_buffered() {
        let path = "test_buffered.txt";
        write_buffered(path, "Hello, World!").unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "Hello, World!");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_file_metadata() {
        let path = "test_metadata.txt";
        fs::write(path, "Hello").unwrap();
        let (size, is_dir) = file_metadata(path).unwrap();
        assert_eq!(size, 5);
        assert!(!is_dir);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_file_exists() {
        let path = "test_exists.txt";
        assert!(!file_exists(path));
        fs::write(path, "Hello").unwrap();
        assert!(file_exists(path));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_create_directory() {
        let path = "test_dir";
        create_directory(path).unwrap();
        assert!(Path::new(path).exists());
        fs::remove_dir(path).unwrap();
    }

    #[test]
    fn test_list_directory() {
        let path = "test_list_dir";
        fs::create_dir(path).unwrap();
        fs::write(format!("{}/file1.txt", path), "content1").unwrap();
        fs::write(format!("{}/file2.txt", path), "content2").unwrap();
        let entries = list_directory(path).unwrap();
        assert_eq!(entries.len(), 2);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_copy_file() {
        let from = "test_copy_from.txt";
        let to = "test_copy_to.txt";
        fs::write(from, "Hello").unwrap();
        copy_file(from, to).unwrap();
        assert_eq!(fs::read_to_string(to).unwrap(), "Hello");
        fs::remove_file(from).unwrap();
        fs::remove_file(to).unwrap();
    }

    #[test]
    fn test_delete_file() {
        let path = "test_delete.txt";
        fs::write(path, "Hello").unwrap();
        delete_file(path).unwrap();
        assert!(!Path::new(path).exists());
    }

    #[test]
    fn test_delete_directory() {
        let path = "test_delete_dir";
        fs::create_dir(path).unwrap();
        delete_directory(path).unwrap();
        assert!(!Path::new(path).exists());
    }

    #[test]
    fn test_read_bytes() {
        let path = "test_bytes.txt";
        fs::write(path, "Hello").unwrap();
        assert_eq!(read_bytes(path).unwrap(), b"Hello");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_write_bytes() {
        let path = "test_write_bytes.txt";
        write_bytes(path, b"Hello").unwrap();
        assert_eq!(fs::read(path).unwrap(), b"Hello");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_read_with_custom_reader() {
        let path = "test_custom_reader.txt";
        fs::write(path, "Hello, World!").unwrap();
        assert_eq!(read_with_custom_reader(path).unwrap(), "Hello, World!");
        fs::remove_file(path).unwrap();
    }
}
