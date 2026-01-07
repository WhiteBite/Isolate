//! Binary management module for Isolate
//!
//! Handles downloading, extracting, and verifying external binaries (winws, sing-box).
//! Provides progress reporting via callbacks for UI integration.
//!
//! NOTE: Some functions are prepared for future binary management features.

// Public API for binary management
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use futures::StreamExt;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::integrity::verify_binary_hash_async;
use crate::core::paths::get_binaries_dir;

/// Required binaries for Isolate to function
pub const REQUIRED_BINARIES: &[&str] = &[
    "winws.exe",
    "WinDivert64.sys",
    "WinDivert.dll",
    "sing-box.exe",
];

/// Binary source information for downloading
#[derive(Debug, Clone)]
pub struct BinarySource {
    /// Binary filename
    pub name: &'static str,
    /// Download URL (GitHub release asset)
    pub url: &'static str,
    /// Expected SHA-256 hash for verification
    pub sha256: &'static str,
    /// Whether this is an archive that needs extraction
    pub is_archive: bool,
    /// Files to extract from archive (if is_archive is true)
    pub extract_files: &'static [&'static str],
}

/// Binary sources configuration with verified SHA-256 hashes
/// 
/// SECURITY: These hashes are verified from official GitHub releases.
/// Update when upgrading binary versions.
/// 
/// ## Verification sources:
/// - Zapret: https://github.com/bol-van/zapret/releases (sha256sum.txt)
/// - Sing-box: https://github.com/SagerNet/sing-box/releases
pub const BINARY_SOURCES: &[BinarySource] = &[
    // Zapret/winws binaries (v72.6) - Latest as of 2026-01-06
    // Hashes from: https://github.com/bol-van/zapret/releases/download/v72.6/sha256sum.txt
    BinarySource {
        name: "zapret",
        url: "https://github.com/bol-van/zapret/releases/download/v72.6/zapret-v72.6-openwrt-embedded.tar.gz",
        // Note: Using embedded archive, individual file hashes verified below
        sha256: "",
        is_archive: true,
        extract_files: &["winws.exe", "WinDivert64.sys", "WinDivert.dll"],
    },
    // Sing-box binary (v1.10.0)
    // Archive hash computed from official release
    BinarySource {
        name: "sing-box",
        url: "https://github.com/SagerNet/sing-box/releases/download/v1.10.0/sing-box-1.10.0-windows-amd64.zip",
        sha256: "8ee3e6beaa94fb961b91c845446e3300cf0e995cb3995448da320ead88b8666b",
        is_archive: true,
        extract_files: &["sing-box.exe"],
    },
];

/// Individual binary file hashes for post-extraction verification
/// These are hashes of the extracted files, not the archives
/// 
/// ## How to compute hashes for extracted files:
/// 1. Extract the archive manually
/// 2. Run: `certutil -hashfile <file> SHA256` for each file
/// 3. Update the constants below
pub mod binary_hashes {
    /// SHA-256 hashes for zapret v72.6 binaries (extracted files)
    /// From: https://github.com/bol-van/zapret/releases/download/v72.6/sha256sum.txt
    pub mod zapret_v72_6 {
        pub const WINWS_EXE: &str = "21c5db984702de8b24d462ae3e64a1ef18937b4515d862a3dd9b70845944a595";
        pub const WINDIVERT64_SYS: &str = "8da085332782708d8767bcace5327a6ec7283c17cfb85e40b03cd2323a90ddc2";
        pub const WINDIVERT_DLL: &str = "c1e060ee19444a259b2162f8af0f3fe8c4428a1c6f694dce20de194ac8d7d9a2";
    }
    
    /// SHA-256 hashes for sing-box v1.10.0 (extracted files)
    pub mod singbox_v1_10_0 {
        pub const SING_BOX_EXE: &str = "0da10a2f1db4fc92dd8db9c301318db457073c23f51d7cc69507f3eda142c331";
    }
    
    /// SHA-256 hashes for TLS/QUIC fingerprint .bin files
    /// These are used by winws for fake packet generation
    pub mod fingerprints {
        /// TLS ClientHello from www.google.com
        pub const TLS_CLIENTHELLO_WWW_GOOGLE_COM: &str = "936c2bee4cfb80aa3c426b2dcbcc834b3fbcd1adb17172959dc569c73a14275c";
        /// TLS ClientHello from 4pda.to (used in general_alt10 strategy)
        pub const TLS_CLIENTHELLO_4PDA_TO: &str = "eefeaf09dde8d69b1f176212541f63c68b314a33a335eced99a8a29f17254da8";
        /// TLS ClientHello from max.ru (used in general_alt11 strategy)
        pub const TLS_CLIENTHELLO_MAX_RU: &str = "e4a94cec50b3c048eb988a513ee28191e4d7544dd5f98a9bf94f37ee02d2568e";
        /// QUIC Initial from www.google.com
        pub const QUIC_INITIAL_WWW_GOOGLE_COM: &str = "f4589c57749f956bb30538197a521d7005f8b0a8723b4707e72405e51ddac50a";
    }
    
    /// Get expected hash for a binary file
    /// Returns None if hash is not configured (empty string)
    pub fn get_expected_hash(filename: &str) -> Option<&'static str> {
        let hash = match filename {
            "winws.exe" => zapret_v72_6::WINWS_EXE,
            "WinDivert64.sys" => zapret_v72_6::WINDIVERT64_SYS,
            "WinDivert.dll" => zapret_v72_6::WINDIVERT_DLL,
            "sing-box.exe" => singbox_v1_10_0::SING_BOX_EXE,
            // TLS/QUIC fingerprint files
            "tls_clienthello_www_google_com.bin" => fingerprints::TLS_CLIENTHELLO_WWW_GOOGLE_COM,
            "tls_clienthello_4pda_to.bin" => fingerprints::TLS_CLIENTHELLO_4PDA_TO,
            "tls_clienthello_max_ru.bin" => fingerprints::TLS_CLIENTHELLO_MAX_RU,
            "quic_initial_www_google_com.bin" => fingerprints::QUIC_INITIAL_WWW_GOOGLE_COM,
            _ => return None,
        };
        
        // Return None for empty/unconfigured hashes
        if hash.is_empty() {
            None
        } else {
            Some(hash)
        }
    }
}

/// Status of binary check
#[derive(Debug, Clone, serde::Serialize)]
pub struct BinaryCheckResult {
    /// List of missing binary names
    pub missing: Vec<String>,
    /// List of present binary names
    pub present: Vec<String>,
    /// Whether all required binaries are available
    pub all_present: bool,
}

/// Download progress information
#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    /// Name of the binary being downloaded
    pub binary_name: String,
    /// Bytes downloaded so far
    pub downloaded: u64,
    /// Total bytes to download (0 if unknown)
    pub total: u64,
    /// Progress percentage (0-100)
    pub percentage: u8,
    /// Current phase: "downloading", "extracting", "verifying"
    pub phase: String,
}

/// Check if all required binaries exist
///
/// # Returns
/// * `Ok(BinaryCheckResult)` - Result containing lists of missing and present binaries
pub async fn check_binaries() -> Result<BinaryCheckResult> {
    let binaries_dir = get_binaries_dir();
    let mut missing = Vec::new();
    let mut present = Vec::new();

    debug!("Checking binaries in: {}", binaries_dir.display());

    for binary in REQUIRED_BINARIES {
        let path = binaries_dir.join(binary);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            present.push(binary.to_string());
            debug!("Binary found: {}", binary);
        } else {
            missing.push(binary.to_string());
            debug!("Binary missing: {}", binary);
        }
    }

    let all_present = missing.is_empty();

    if all_present {
        info!("All required binaries present");
    } else {
        warn!("Missing binaries: {:?}", missing);
    }

    Ok(BinaryCheckResult {
        missing,
        present,
        all_present,
    })
}

/// Check if a specific binary exists
pub async fn binary_exists(name: &str) -> bool {
    let path = get_binaries_dir().join(name);
    tokio::fs::try_exists(&path).await.unwrap_or(false)
}

/// Download a file from URL with progress reporting
///
/// # Arguments
/// * `url` - URL to download from
/// * `dest` - Destination file path
/// * `progress` - Callback function receiving (downloaded_bytes, total_bytes)
///
/// # Returns
/// * `Ok(())` - Download completed successfully
/// * `Err(IsolateError)` - Download failed
pub async fn download_file<F>(url: &str, dest: &Path, progress: F) -> Result<()>
where
    F: Fn(u64, u64) + Send + 'static,
{
    info!("Downloading: {} -> {}", url, dest.display());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 min timeout
        .build()
        .map_err(|e| IsolateError::Network(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| IsolateError::Network(format!("Failed to start download: {}", e)))?;

    if !response.status().is_success() {
        return Err(IsolateError::Network(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);
    debug!("Download size: {} bytes", total_size);

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(dest).await?;
    let mut downloaded: u64 = 0;

    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| IsolateError::Network(format!("Download interrupted: {}", e)))?;

        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        progress(downloaded, total_size);
    }

    file.flush().await?;

    info!("Download complete: {} bytes", downloaded);
    Ok(())
}

/// Extract a ZIP archive to destination directory (sync version for internal use)
///
/// # Arguments
/// * `archive` - Path to the ZIP archive
/// * `dest_dir` - Directory to extract files to
/// * `files_to_extract` - Optional list of specific files to extract (None = extract all)
///
/// # Returns
/// * `Ok(Vec<String>)` - List of extracted file names
/// * `Err(IsolateError)` - Extraction failed
fn extract_zip_sync(
    archive: &Path,
    dest_dir: &Path,
    files_to_extract: Option<&[&str]>,
) -> Result<Vec<String>> {
    info!(
        "Extracting: {} -> {}",
        archive.display(),
        dest_dir.display()
    );

    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| IsolateError::Process(format!("Failed to open ZIP archive: {}", e)))?;

    std::fs::create_dir_all(dest_dir)?;

    let mut extracted = Vec::new();

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| IsolateError::Process(format!("Failed to read ZIP entry: {}", e)))?;

        let entry_name = entry.name().to_string();

        // Skip directories
        if entry.is_dir() {
            continue;
        }

        // Get just the filename (ignore directory structure in archive)
        let file_name = Path::new(&entry_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&entry_name);

        // Check if we should extract this file
        let should_extract = match files_to_extract {
            Some(files) => files.contains(&file_name),
            None => true,
        };

        if !should_extract {
            continue;
        }

        let dest_path = dest_dir.join(file_name);
        debug!("Extracting: {} -> {}", entry_name, dest_path.display());

        let mut outfile = std::fs::File::create(&dest_path)?;
        std::io::copy(&mut entry, &mut outfile)?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if file_name.ends_with(".exe") || !file_name.contains('.') {
                let mut perms = outfile.metadata()?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest_path, perms)?;
            }
        }

        extracted.push(file_name.to_string());
    }

    info!("Extracted {} files: {:?}", extracted.len(), extracted);
    Ok(extracted)
}

/// Extract a ZIP archive to destination directory (async version)
///
/// Uses spawn_blocking to avoid blocking the async runtime since
/// the zip crate doesn't support async I/O.
///
/// # Arguments
/// * `archive` - Path to the ZIP archive
/// * `dest_dir` - Directory to extract files to
/// * `files_to_extract` - Optional list of specific files to extract (None = extract all)
///
/// # Returns
/// * `Ok(Vec<String>)` - List of extracted file names
/// * `Err(IsolateError)` - Extraction failed
pub async fn extract_zip(
    archive: &Path,
    dest_dir: &Path,
    files_to_extract: Option<Vec<&str>>,
) -> Result<Vec<String>> {
    let archive = archive.to_path_buf();
    let dest_dir = dest_dir.to_path_buf();
    let files: Option<Vec<String>> = files_to_extract.map(|f| f.into_iter().map(|s| s.to_string()).collect());
    
    tokio::task::spawn_blocking(move || {
        let files_ref: Option<Vec<&str>> = files.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
        extract_zip_sync(&archive, &dest_dir, files_ref.as_deref())
    })
    .await
    .map_err(|e| IsolateError::Process(format!("Extract task failed: {}", e)))?
}

/// Download and extract a binary source with hash verification
///
/// # Arguments
/// * `source` - Binary source configuration
/// * `progress` - Progress callback receiving (binary_name, downloaded, total)
///
/// # Returns
/// * `Ok(())` - Binary downloaded, verified and extracted successfully
/// * `Err(IsolateError)` - Download, verification or extraction failed
///
/// # Security
/// This function performs two levels of hash verification:
/// 1. Archive hash verification after download (if sha256 is provided)
/// 2. Individual file hash verification after extraction
pub async fn download_binary_source<F>(source: &BinarySource, progress: F) -> Result<()>
where
    F: Fn(&str, u64, u64) + Send + Clone + 'static,
{
    let binaries_dir = get_binaries_dir();
    let temp_dir = binaries_dir.join("temp");
    fs::create_dir_all(&temp_dir).await?;

    let archive_name = format!("{}.zip", source.name);
    let archive_path = temp_dir.join(&archive_name);

    // Download
    let source_name = source.name.to_string();
    let progress_clone = progress.clone();
    download_file(source.url, &archive_path, move |downloaded, total| {
        progress_clone(&source_name, downloaded, total);
    })
    .await?;

    // Verify archive hash BEFORE extraction (critical security check)
    if !source.sha256.is_empty() {
        info!("Verifying archive hash for: {}", source.name);
        match verify_binary_hash_async(&archive_path, source.sha256).await {
            Ok(true) => {
                info!("Archive hash verified for: {}", source.name);
            }
            Ok(false) => {
                // Delete the corrupted/tampered archive
                let _ = fs::remove_file(&archive_path).await;
                error!("Archive hash mismatch for: {} - file may be corrupted or tampered", source.name);
                return Err(IsolateError::Process(format!(
                    "Security error: Archive hash verification failed for {}. The downloaded file may be corrupted or tampered with.",
                    source.name
                )));
            }
            Err(e) => {
                let _ = fs::remove_file(&archive_path).await;
                error!("Failed to verify archive hash for {}: {}", source.name, e);
                return Err(IsolateError::Process(format!(
                    "Failed to verify archive integrity for {}: {}",
                    source.name, e
                )));
            }
        }
    } else {
        warn!("No hash configured for archive: {} - skipping archive verification", source.name);
    }

    // Extract
    if source.is_archive {
        let files_to_extract: Vec<&str> = source.extract_files.to_vec();
        let extracted = extract_zip(
            &archive_path,
            &binaries_dir,
            Some(files_to_extract),
        ).await?;

        // Verify all expected files were extracted
        for expected in source.extract_files {
            if !extracted.iter().any(|f| f == *expected) {
                warn!("Expected file not found in archive: {}", expected);
            }
        }
        
        // Verify individual file hashes after extraction
        for file_name in source.extract_files {
            let file_path = binaries_dir.join(file_name);
            if tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
                if let Some(expected_hash) = binary_hashes::get_expected_hash(file_name) {
                    match verify_binary_hash_async(&file_path, expected_hash).await {
                        Ok(true) => {
                            info!("File hash verified: {}", file_name);
                        }
                        Ok(false) => {
                            // Delete the corrupted/tampered file
                            let _ = fs::remove_file(&file_path).await;
                            error!("File hash mismatch for: {} - file may be corrupted or tampered", file_name);
                            return Err(IsolateError::Process(format!(
                                "Security error: Hash verification failed for {}. The file may be corrupted or tampered with.",
                                file_name
                            )));
                        }
                        Err(e) => {
                            warn!("Could not verify hash for {}: {}", file_name, e);
                        }
                    }
                } else {
                    warn!("No hash configured for file: {} - skipping file verification", file_name);
                }
            }
        }
    }

    // Cleanup temp file
    if let Err(e) = fs::remove_file(&archive_path).await {
        warn!("Failed to cleanup temp file: {}", e);
    }

    Ok(())
}

/// Download all missing binaries with progress reporting
///
/// # Arguments
/// * `progress` - Callback receiving DownloadProgress updates
///
/// # Returns
/// * `Ok(())` - All binaries downloaded successfully
/// * `Err(IsolateError)` - Download failed
pub async fn ensure_binaries<F>(progress: F) -> Result<()>
where
    F: Fn(DownloadProgress) + Send + Clone + 'static,
{
    let check_result = check_binaries().await?;

    if check_result.all_present {
        info!("All binaries already present, skipping download");
        return Ok(());
    }

    info!("Missing binaries: {:?}", check_result.missing);

    // Determine which sources need to be downloaded
    for source in BINARY_SOURCES {
        let needs_download = source
            .extract_files
            .iter()
            .any(|f| check_result.missing.contains(&f.to_string()));

        if !needs_download {
            debug!("Skipping {}: all files present", source.name);
            continue;
        }

        info!("Downloading source: {}", source.name);

        let progress_clone = progress.clone();
        let source_name = source.name.to_string();

        download_binary_source(source, move |name, downloaded, total| {
            let percentage = if total > 0 {
                ((downloaded as f64 / total as f64) * 100.0) as u8
            } else {
                0
            };

            progress_clone(DownloadProgress {
                binary_name: name.to_string(),
                downloaded,
                total,
                percentage,
                phase: "downloading".to_string(),
            });
        })
        .await?;

        // Report extraction phase
        progress(DownloadProgress {
            binary_name: source_name.clone(),
            downloaded: 0,
            total: 0,
            percentage: 100,
            phase: "extracting".to_string(),
        });
    }

    // Final verification
    let final_check = check_binaries().await?;
    if !final_check.all_present {
        return Err(IsolateError::Process(format!(
            "Some binaries still missing after download: {:?}",
            final_check.missing
        )));
    }

    info!("All binaries downloaded and verified");
    Ok(())
}

/// Get the path to a specific binary
pub fn get_binary_path(name: &str) -> PathBuf {
    get_binaries_dir().join(name)
}

/// Delete all downloaded binaries (for cleanup/reinstall)
pub async fn cleanup_binaries() -> Result<()> {
    let binaries_dir = get_binaries_dir();

    for binary in REQUIRED_BINARIES {
        let path = binaries_dir.join(binary);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await?;
            info!("Removed: {}", binary);
        }
    }

    // Also cleanup temp directory
    let temp_dir = binaries_dir.join("temp");
    if tokio::fs::try_exists(&temp_dir).await.unwrap_or(false) {
        fs::remove_dir_all(&temp_dir).await?;
    }

    Ok(())
}

/// Compute and print SHA-256 hashes for all existing binaries
/// 
/// This is a utility function for developers to generate hash values
/// that should be added to the `binary_hashes` module.
/// 
/// # Returns
/// * `Ok(Vec<(String, String)>)` - List of (filename, hash) pairs
/// 
/// # Example output:
/// ```text
/// winws.exe: abc123...
/// WinDivert64.sys: def456...
/// ```
pub async fn compute_binary_hashes() -> Result<Vec<(String, String)>> {
    use crate::core::integrity::calculate_sha256_async;
    
    let binaries_dir = get_binaries_dir();
    let mut hashes = Vec::new();
    
    for binary in REQUIRED_BINARIES {
        let path = binaries_dir.join(binary);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            match calculate_sha256_async(&path).await {
                Ok(hash) => {
                    info!("{}: {}", binary, hash);
                    hashes.push((binary.to_string(), hash));
                }
                Err(e) => {
                    warn!("Failed to compute hash for {}: {}", binary, e);
                }
            }
        } else {
            debug!("Binary not found, skipping: {}", binary);
        }
    }
    
    Ok(hashes)
}

/// Verify all existing binaries against configured hashes
/// 
/// # Returns
/// * `Ok(BinaryVerificationResult)` - Verification results for each binary
pub async fn verify_all_binary_hashes() -> Result<BinaryVerificationResult> {
    use crate::core::integrity::calculate_sha256_async;
    
    let binaries_dir = get_binaries_dir();
    let mut verified = Vec::new();
    let mut failed = Vec::new();
    let mut missing = Vec::new();
    let mut unconfigured = Vec::new();
    
    for binary in REQUIRED_BINARIES {
        let path = binaries_dir.join(binary);
        
        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            missing.push(binary.to_string());
            continue;
        }
        
        match binary_hashes::get_expected_hash(binary) {
            Some(expected_hash) => {
                match calculate_sha256_async(&path).await {
                    Ok(actual_hash) => {
                        if actual_hash == expected_hash.to_lowercase() {
                            verified.push(binary.to_string());
                        } else {
                            failed.push(BinaryHashFailure {
                                name: binary.to_string(),
                                expected: expected_hash.to_string(),
                                actual: actual_hash,
                            });
                        }
                    }
                    Err(e) => {
                        warn!("Failed to compute hash for {}: {}", binary, e);
                        failed.push(BinaryHashFailure {
                            name: binary.to_string(),
                            expected: expected_hash.to_string(),
                            actual: format!("error: {}", e),
                        });
                    }
                }
            }
            None => {
                unconfigured.push(binary.to_string());
            }
        }
    }
    
    let all_verified = failed.is_empty() && missing.is_empty();
    
    Ok(BinaryVerificationResult {
        verified,
        failed,
        missing,
        unconfigured,
        all_verified,
    })
}

/// Result of binary hash verification
#[derive(Debug, Clone, serde::Serialize)]
pub struct BinaryVerificationResult {
    /// Binaries that passed hash verification
    pub verified: Vec<String>,
    /// Binaries that failed hash verification
    pub failed: Vec<BinaryHashFailure>,
    /// Binaries that are missing
    pub missing: Vec<String>,
    /// Binaries without configured hashes
    pub unconfigured: Vec<String>,
    /// Whether all present binaries passed verification
    pub all_verified: bool,
}

/// Details of a hash verification failure
#[derive(Debug, Clone, serde::Serialize)]
pub struct BinaryHashFailure {
    pub name: String,
    pub expected: String,
    pub actual: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_binaries() {
        let result = check_binaries().await.unwrap();
        // In test environment, binaries are likely missing
        assert!(result.missing.len() + result.present.len() == REQUIRED_BINARIES.len());
    }

    #[tokio::test]
    async fn test_binary_exists_missing() {
        // Test that binary_exists returns false for non-existent binary
        let exists = binary_exists("nonexistent_binary_12345.exe").await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_check_binaries_result_structure() {
        let result = check_binaries().await.unwrap();
        
        // Verify result structure is consistent
        assert_eq!(
            result.missing.len() + result.present.len(),
            REQUIRED_BINARIES.len(),
            "Total of missing + present should equal REQUIRED_BINARIES count"
        );
        
        // all_present should be true only if missing is empty
        assert_eq!(result.all_present, result.missing.is_empty());
    }

    #[test]
    fn test_binary_sources_configured() {
        // Ensure all required binaries are covered by sources
        for binary in REQUIRED_BINARIES {
            let covered = BINARY_SOURCES
                .iter()
                .any(|s| s.extract_files.contains(binary));
            assert!(covered, "Binary {} not covered by any source", binary);
        }
    }

    #[test]
    fn test_get_binary_path() {
        let path = get_binary_path("winws.exe");
        assert!(path.ends_with("winws.exe"));
    }

    #[test]
    fn test_get_binary_path_preserves_filename() {
        for binary in REQUIRED_BINARIES {
            let path = get_binary_path(binary);
            assert!(
                path.file_name().and_then(|n| n.to_str()) == Some(*binary),
                "Path should end with the binary name"
            );
        }
    }

    #[test]
    fn test_binary_hashes_get_expected_hash() {
        // Test known binaries have hashes
        assert!(binary_hashes::get_expected_hash("winws.exe").is_some());
        assert!(binary_hashes::get_expected_hash("WinDivert64.sys").is_some());
        assert!(binary_hashes::get_expected_hash("WinDivert.dll").is_some());
        assert!(binary_hashes::get_expected_hash("sing-box.exe").is_some());
        
        // Test TLS/QUIC fingerprint files have hashes
        assert!(binary_hashes::get_expected_hash("tls_clienthello_www_google_com.bin").is_some());
        assert!(binary_hashes::get_expected_hash("tls_clienthello_4pda_to.bin").is_some());
        assert!(binary_hashes::get_expected_hash("tls_clienthello_max_ru.bin").is_some());
        assert!(binary_hashes::get_expected_hash("quic_initial_www_google_com.bin").is_some());
        
        // Test unknown binary returns None
        assert!(binary_hashes::get_expected_hash("unknown.exe").is_none());
    }

    #[test]
    fn test_binary_check_result_serialization() {
        let result = BinaryCheckResult {
            missing: vec!["test.exe".to_string()],
            present: vec!["other.exe".to_string()],
            all_present: false,
        };
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("missing"));
        assert!(json.contains("present"));
        assert!(json.contains("all_present"));
    }

    #[test]
    fn test_download_progress_serialization() {
        let progress = DownloadProgress {
            binary_name: "test.exe".to_string(),
            downloaded: 1024,
            total: 2048,
            percentage: 50,
            phase: "downloading".to_string(),
        };
        
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("binary_name"));
        assert!(json.contains("downloaded"));
        assert!(json.contains("percentage"));
        assert!(json.contains("phase"));
    }

    #[test]
    fn test_binary_verification_result_serialization() {
        let result = BinaryVerificationResult {
            verified: vec!["a.exe".to_string()],
            failed: vec![BinaryHashFailure {
                name: "b.exe".to_string(),
                expected: "abc".to_string(),
                actual: "def".to_string(),
            }],
            missing: vec!["c.exe".to_string()],
            unconfigured: vec!["d.exe".to_string()],
            all_verified: false,
        };
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("verified"));
        assert!(json.contains("failed"));
        assert!(json.contains("missing"));
        assert!(json.contains("unconfigured"));
    }

    #[test]
    fn test_required_binaries_not_empty() {
        assert!(!REQUIRED_BINARIES.is_empty());
    }

    #[test]
    fn test_binary_sources_not_empty() {
        assert!(!BINARY_SOURCES.is_empty());
    }

    #[test]
    fn test_binary_source_has_valid_url() {
        for source in BINARY_SOURCES {
            assert!(
                source.url.starts_with("https://"),
                "Source {} should have HTTPS URL",
                source.name
            );
        }
    }
}
