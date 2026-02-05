//! File I/O utilities for configuration files
//!
//! This module provides secure file operations including:
//! - File locking to prevent race conditions
//! - Atomic writes to prevent corruption
//! - Path validation to prevent traversal attacks

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use fs2::FileExt;

/// Timeout for acquiring file locks (in seconds)
const FILE_LOCK_TIMEOUT_SECS: u64 = 10;

/// Execute a function while holding an exclusive lock on a file.
/// Creates a .lock file adjacent to the target file to coordinate access.
/// Returns the result of the function, or an error if the lock cannot be acquired.
pub fn with_file_lock<T, F>(path: &Path, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    // Create lock file path
    let lock_path = path.with_extension("lock");

    // Ensure parent directory exists for lock file
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create lock file directory: {}", e))?;
    }

    // Open or create the lock file
    let lock_file = File::create(&lock_path)
        .map_err(|e| format!("Failed to create lock file: {}", e))?;

    // Try to acquire exclusive lock with timeout
    let start = std::time::Instant::now();
    loop {
        match lock_file.try_lock_exclusive() {
            Ok(_) => break,
            Err(_) if start.elapsed().as_secs() >= FILE_LOCK_TIMEOUT_SECS => {
                return Err(format!(
                    "Timeout waiting for file lock on {}. Another process may be modifying this file.",
                    path.display()
                ));
            }
            Err(_) => {
                // Wait a bit before retrying
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    // Execute the function while holding the lock
    let result = f();

    // Lock is automatically released when lock_file is dropped
    // Attempt to clean up lock file (ignore errors - it's just cleanup)
    let _ = fs::remove_file(&lock_path);

    result
}

/// Validate a path to prevent access to sensitive files
///
/// This function ensures that:
/// 1. The path doesn't contain directory traversal sequences (..)
/// 2. The path doesn't point to sensitive system files (SSH keys, credentials, etc.)
///
/// Note: Config paths come from trusted sources (database, hard-coded client defaults),
/// so we don't restrict to specific directories. The traversal and sensitive file
/// checks provide sufficient protection.
pub fn validate_path_security(path: &Path) -> Result<PathBuf, String> {
    // Check for directory traversal sequences
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err("Path contains directory traversal sequences".to_string());
    }

    // Canonicalize if the path exists to resolve symlinks
    let canonical = if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?
    } else if let Some(parent) = path.parent() {
        // If parent exists, canonicalize it and append the filename
        if parent.exists() {
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("Failed to resolve parent path: {}", e))?;
            canonical_parent.join(path.file_name().unwrap_or_default())
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    };

    // For sensitive pattern checking, use string comparison
    // Normalize separators for cross-platform matching
    let canonical_str = canonical.to_string_lossy().replace('\\', "/");

    // Additional safety: block access to sensitive system files even within home
    let sensitive_patterns = [
        ".ssh/",
        ".gnupg/",
        ".aws/credentials",
        ".netrc",
        ".npmrc",  // May contain tokens
        ".pypirc", // May contain tokens
    ];

    for pattern in sensitive_patterns {
        if canonical_str.contains(pattern) {
            return Err(format!(
                "Access to sensitive path pattern '{}' is not allowed",
                pattern
            ));
        }
    }

    // If the original path exists, return the canonical version
    // Otherwise return the original (for new file creation)
    if path.exists() {
        Ok(canonical)
    } else {
        Ok(path.to_path_buf())
    }
}

/// Write content to a file atomically
///
/// This prevents file corruption if the application crashes during write:
/// 1. Write to a temporary file in the same directory
/// 2. Sync the file to ensure data is flushed to disk
/// 3. Rename the temp file to the target (atomic on most filesystems)
pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent()
        .ok_or_else(|| "Invalid path: no parent directory".to_string())?;

    // Create temp file in same directory (ensures same filesystem for atomic rename)
    let temp_path = parent.join(format!(
        ".{}.tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config"),
        std::process::id()
    ));

    // Write to temp file
    let mut file = File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;

    // Sync to disk before rename
    file.sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;

    // Atomic rename
    fs::rename(&temp_path, path)
        .map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            format!("Failed to rename temp file: {}", e)
        })?;

    Ok(())
}
