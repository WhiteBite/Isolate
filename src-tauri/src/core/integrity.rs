//! Binary integrity verification module
//!
//! Provides SHA-256 hash verification for external binaries (winws, sing-box)
//! to ensure they haven't been tampered with.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};
use thiserror::Error;

/// Known binary hashes for verification
/// Format: filename -> expected SHA-256 hash
pub fn get_known_hashes() -> HashMap<&'static str, &'static str> {
    let hashes = HashMap::new();
    // TODO: Add actual hashes when binaries are finalized
    // hashes.insert("winws.exe", "expected_sha256_hash_here");
    // hashes.insert("sing-box.exe", "expected_sha256_hash_here");
    hashes
}

#[derive(Error, Debug)]
pub enum IntegrityError {
    #[error("Failed to open file: {0}")]
    FileOpen(#[from] std::io::Error),

    #[error("Binary not found: {0}")]
    BinaryNotFound(String),

    #[error("Hash mismatch for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("No known hash for binary: {0}")]
    UnknownBinary(String),
}

/// Calculates SHA-256 hash of a file
///
/// # Arguments
/// * `path` - Path to the file to hash
///
/// # Returns
/// * `Ok(String)` - Lowercase hex-encoded SHA-256 hash
/// * `Err(IntegrityError)` - If file cannot be read
///
/// # Example
/// ```ignore
/// let hash = calculate_sha256(Path::new("winws.exe"))?;
/// println!("SHA-256: {}", hash);
/// ```
pub fn calculate_sha256(path: &Path) -> Result<String, IntegrityError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Verifies a binary file against an expected SHA-256 hash
///
/// # Arguments
/// * `path` - Path to the binary file
/// * `expected_sha256` - Expected SHA-256 hash (lowercase hex)
///
/// # Returns
/// * `Ok(true)` - Hash matches
/// * `Ok(false)` - Hash does not match
/// * `Err(IntegrityError)` - If file cannot be read
///
/// # Example
/// ```ignore
/// let valid = verify_binary_hash(
///     Path::new("winws.exe"),
///     "a1b2c3d4..."
/// )?;
/// ```
pub fn verify_binary_hash(path: &Path, expected_sha256: &str) -> Result<bool, IntegrityError> {
    if !path.exists() {
        return Err(IntegrityError::BinaryNotFound(
            path.display().to_string(),
        ));
    }

    let actual_hash = calculate_sha256(path)?;
    let expected_lower = expected_sha256.to_lowercase();

    Ok(actual_hash == expected_lower)
}

/// Verification result for a single binary
#[derive(Debug, Clone)]
pub struct BinaryStatus {
    pub name: String,
    pub path: String,
    pub status: BinaryVerificationStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryVerificationStatus {
    /// Binary exists and hash matches
    Valid,
    /// Binary exists but hash doesn't match
    HashMismatch { expected: String, actual: String },
    /// Binary file not found
    Missing,
    /// No known hash to verify against
    Unverified,
    /// Error reading file
    ReadError(String),
}

/// Verifies all binaries in a directory against known hashes
///
/// # Arguments
/// * `binaries_dir` - Directory containing binary files
///
/// # Returns
/// * `Ok(Vec<String>)` - List of missing or corrupted binary names
/// * `Err(IntegrityError)` - If directory cannot be read
///
/// # Example
/// ```ignore
/// let problems = verify_all_binaries(Path::new("./binaries"))?;
/// if !problems.is_empty() {
///     println!("Problems found: {:?}", problems);
/// }
/// ```
pub fn verify_all_binaries(binaries_dir: &Path) -> Result<Vec<String>, IntegrityError> {
    let known_hashes = get_known_hashes();
    let mut problems = Vec::new();

    for (filename, expected_hash) in known_hashes.iter() {
        let binary_path = binaries_dir.join(filename);

        if !binary_path.exists() {
            problems.push(format!("{} (missing)", filename));
            continue;
        }

        match verify_binary_hash(&binary_path, expected_hash) {
            Ok(true) => {
                tracing::debug!("Binary {} verified successfully", filename);
            }
            Ok(false) => {
                problems.push(format!("{} (hash mismatch)", filename));
                tracing::warn!("Binary {} has invalid hash", filename);
            }
            Err(e) => {
                problems.push(format!("{} (error: {})", filename, e));
                tracing::error!("Failed to verify {}: {}", filename, e);
            }
        }
    }

    Ok(problems)
}

/// Detailed verification of all binaries with full status information
///
/// # Arguments
/// * `binaries_dir` - Directory containing binary files
///
/// # Returns
/// Vector of `BinaryStatus` for each known binary
pub fn verify_all_binaries_detailed(binaries_dir: &Path) -> Vec<BinaryStatus> {
    let known_hashes = get_known_hashes();
    let mut results = Vec::new();

    for (filename, expected_hash) in known_hashes.iter() {
        let binary_path = binaries_dir.join(filename);
        let path_str = binary_path.display().to_string();

        let status = if !binary_path.exists() {
            BinaryVerificationStatus::Missing
        } else {
            match calculate_sha256(&binary_path) {
                Ok(actual_hash) => {
                    if actual_hash == expected_hash.to_lowercase() {
                        BinaryVerificationStatus::Valid
                    } else {
                        BinaryVerificationStatus::HashMismatch {
                            expected: expected_hash.to_string(),
                            actual: actual_hash,
                        }
                    }
                }
                Err(e) => BinaryVerificationStatus::ReadError(e.to_string()),
            }
        };

        results.push(BinaryStatus {
            name: filename.to_string(),
            path: path_str,
            status,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_sha256() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let hash = calculate_sha256(temp_file.path()).unwrap();
        // SHA-256 of "test content"
        assert_eq!(
            hash,
            "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72"
        );
    }

    #[test]
    fn test_verify_binary_hash_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let result = verify_binary_hash(
            temp_file.path(),
            "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72",
        );
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_binary_hash_invalid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let result = verify_binary_hash(temp_file.path(), "invalid_hash");
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_binary_hash_missing_file() {
        let result = verify_binary_hash(Path::new("/nonexistent/file"), "somehash");
        assert!(matches!(result, Err(IntegrityError::BinaryNotFound(_))));
    }
}
