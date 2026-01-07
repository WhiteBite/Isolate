//! Binary integrity verification module
//!
//! Provides SHA-256 hash verification for external binaries (winws, sing-box)
//! to ensure they haven't been tampered with.
//!
//! NOTE: Some functions and types are prepared for future integrity features.

// Public API for binary integrity verification
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::core::binaries;

/// Known binary hashes for verification
/// 
/// Uses binaries::binary_hashes as the single source of truth.
/// This ensures consistency between download verification and runtime verification.
pub fn get_known_hashes() -> HashMap<&'static str, &'static str> {
    let mut hashes = HashMap::new();
    
    // Core executables from binaries.rs
    if let Some(hash) = binaries::binary_hashes::get_expected_hash("winws.exe") {
        hashes.insert("winws.exe", hash);
    }
    if let Some(hash) = binaries::binary_hashes::get_expected_hash("sing-box.exe") {
        hashes.insert("sing-box.exe", hash);
    }
    if let Some(hash) = binaries::binary_hashes::get_expected_hash("WinDivert64.sys") {
        hashes.insert("WinDivert64.sys", hash);
    }
    if let Some(hash) = binaries::binary_hashes::get_expected_hash("WinDivert.dll") {
        hashes.insert("WinDivert.dll", hash);
    }
    
    // Additional blobs (keep hardcoded as they're not in binaries.rs)
    // Cygwin runtime (required for winws)
    hashes.insert("cygwin1.dll", "103104a52e5293ce418944725df19e2bf81ad9269b9a120d71d39028e821499b");
    
    // TLS ClientHello blobs for DPI bypass
    hashes.insert("tls_clienthello_www_google_com.bin", "936c2bee4cfb80aa3c426b2dcbcc834b3fbcd1adb17172959dc569c73a14275c");
    hashes.insert("tls_clienthello_4pda_to.bin", "eefeaf09dde8d69b1f176212541f63c68b314a33a335eced99a8a29f17254da8");
    hashes.insert("tls_clienthello_max_ru.bin", "e4a94cec50b3c048eb988a513ee28191e4d7544dd5f98a9bf94f37ee02d2568e");
    
    // QUIC Initial packet blob
    hashes.insert("quic_initial_www_google_com.bin", "f4589c57749f956bb30538197a521d7005f8b0a8723b4707e72405e51ddac50a");
    
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

/// Async version of calculate_sha256 that uses spawn_blocking
/// to avoid blocking the async runtime during CPU-intensive hashing.
///
/// # Arguments
/// * `path` - Path to the file to hash
///
/// # Returns
/// * `Ok(String)` - Lowercase hex-encoded SHA-256 hash
/// * `Err(IntegrityError)` - If file cannot be read
pub async fn calculate_sha256_async(path: &Path) -> Result<String, IntegrityError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || calculate_sha256(&path))
        .await
        .map_err(|e| IntegrityError::FileOpen(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Hash task failed: {}", e)
        )))?
}

/// Async version of verify_binary_hash that uses spawn_blocking
/// to avoid blocking the async runtime.
///
/// # Arguments
/// * `path` - Path to the binary file
/// * `expected_sha256` - Expected SHA-256 hash (lowercase hex)
///
/// # Returns
/// * `Ok(true)` - Hash matches
/// * `Ok(false)` - Hash does not match
/// * `Err(IntegrityError)` - If file cannot be read
pub async fn verify_binary_hash_async(path: &Path, expected_sha256: &str) -> Result<bool, IntegrityError> {
    let path = path.to_path_buf();
    let expected = expected_sha256.to_string();
    tokio::task::spawn_blocking(move || verify_binary_hash(&path, &expected))
        .await
        .map_err(|e| IntegrityError::FileOpen(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Verify task failed: {}", e)
        )))?
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

// ============================================================================
// Startup Verification
// ============================================================================

/// Result of startup verification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StartupVerificationResult {
    /// Total number of binaries checked
    pub total_checked: usize,
    /// Number of binaries that passed verification
    pub verified: usize,
    /// Number of binaries with hash mismatch (potential tampering)
    pub tampered: Vec<String>,
    /// Number of missing binaries
    pub missing: Vec<String>,
    /// Number of binaries without known hashes
    pub unverified: Vec<String>,
    /// Whether all critical binaries are valid
    pub is_safe: bool,
}

/// Critical binaries that MUST pass verification for the app to run safely
const CRITICAL_BINARIES: &[&str] = &[
    "winws.exe",
    "sing-box.exe",
    "WinDivert64.sys",
    "WinDivert.dll",
];

/// Verify all binaries at application startup
///
/// This function should be called during app initialization to ensure
/// all binaries are present and haven't been tampered with.
///
/// # Arguments
/// * `binaries_dir` - Directory containing binary files
///
/// # Returns
/// * `StartupVerificationResult` - Detailed verification results
///
/// # Security
/// - Logs warnings for any hash mismatches (potential tampering)
/// - Returns `is_safe = false` if any critical binary fails verification
/// - Non-critical binaries (blobs) can be missing without blocking startup
///
/// # Example
/// ```ignore
/// let result = verify_on_startup(&get_binaries_dir());
/// if !result.is_safe {
///     error!("Binary integrity check failed!");
///     // Show warning to user or exit
/// }
/// ```
pub fn verify_on_startup(binaries_dir: &Path) -> StartupVerificationResult {
    use tracing::{debug, error, info, warn};
    
    info!(
        binaries_dir = %binaries_dir.display(),
        "Starting binary integrity verification..."
    );
    
    let statuses = verify_all_binaries_detailed(binaries_dir);
    let known_hashes = get_known_hashes();
    
    let mut verified = Vec::new();
    let mut tampered = Vec::new();
    let mut missing = Vec::new();
    let mut unverified = Vec::new();
    
    for status in &statuses {
        match &status.status {
            BinaryVerificationStatus::Valid => {
                // Log verified binaries with their hash for audit trail
                let hash = known_hashes.get(status.name.as_str()).unwrap_or(&"unknown");
                info!(
                    binary = %status.name,
                    hash = %hash,
                    status = "OK",
                    "Binary verified"
                );
                verified.push(status.name.clone());
            }
            BinaryVerificationStatus::HashMismatch { expected, actual } => {
                error!(
                    binary = %status.name,
                    expected_hash = %expected,
                    actual_hash = %actual,
                    status = "FAIL",
                    "SECURITY WARNING: Binary hash mismatch - possible tampering!"
                );
                tampered.push(status.name.clone());
            }
            BinaryVerificationStatus::Missing => {
                if CRITICAL_BINARIES.contains(&status.name.as_str()) {
                    warn!(
                        binary = %status.name,
                        status = "MISSING",
                        critical = true,
                        "Critical binary not found"
                    );
                } else {
                    debug!(
                        binary = %status.name,
                        status = "MISSING",
                        critical = false,
                        "Optional binary not found"
                    );
                }
                missing.push(status.name.clone());
            }
            BinaryVerificationStatus::Unverified => {
                debug!(
                    binary = %status.name,
                    status = "UNVERIFIED",
                    "Binary has no known hash to verify against"
                );
                unverified.push(status.name.clone());
            }
            BinaryVerificationStatus::ReadError(e) => {
                error!(
                    binary = %status.name,
                    error = %e,
                    status = "ERROR",
                    "Failed to read binary file"
                );
                tampered.push(status.name.clone()); // Treat read errors as suspicious
            }
        }
    }
    
    // Check if all critical binaries are safe
    let critical_tampered: Vec<_> = tampered.iter()
        .filter(|name| CRITICAL_BINARIES.contains(&name.as_str()))
        .collect();
    
    let critical_missing: Vec<_> = missing.iter()
        .filter(|name| CRITICAL_BINARIES.contains(&name.as_str()))
        .collect();
    
    let is_safe = critical_tampered.is_empty();
    
    if !is_safe {
        error!(
            "SECURITY ALERT: {} critical binaries may have been tampered with: {:?}",
            critical_tampered.len(),
            critical_tampered
        );
    }
    
    if !critical_missing.is_empty() {
        warn!(
            "Missing {} critical binaries: {:?}",
            critical_missing.len(),
            critical_missing
        );
    }
    
    let result = StartupVerificationResult {
        total_checked: statuses.len(),
        verified: verified.len(),
        tampered,
        missing,
        unverified,
        is_safe,
    };
    
    // Log final summary
    if is_safe {
        info!(
            verified = result.verified,
            total = result.total_checked,
            missing_count = result.missing.len(),
            "Binary integrity verification completed successfully"
        );
    } else {
        error!(
            verified = result.verified,
            total = result.total_checked,
            tampered_count = result.tampered.len(),
            tampered_binaries = ?result.tampered,
            missing_count = result.missing.len(),
            "Binary integrity verification FAILED - security risk detected"
        );
    }
    
    result
}

/// Async version of verify_on_startup
///
/// Uses spawn_blocking to avoid blocking the async runtime.
pub async fn verify_on_startup_async(binaries_dir: &Path) -> StartupVerificationResult {
    let binaries_dir = binaries_dir.to_path_buf();
    tokio::task::spawn_blocking(move || verify_on_startup(&binaries_dir))
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Verification task panicked: {}", e);
            StartupVerificationResult {
                total_checked: 0,
                verified: 0,
                tampered: vec!["verification_failed".to_string()],
                missing: vec![],
                unverified: vec![],
                is_safe: false,
            }
        })
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
