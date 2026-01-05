//! Binary management module for Isolate
//!
//! Handles downloading, extracting, and verifying external binaries (winws, sing-box).
//! Provides progress reporting via callbacks for UI integration.

use std::path::{Path, PathBuf};

use futures::StreamExt;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::integrity::verify_binary_hash;
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

/// Binary sources configuration
/// TODO: Update with actual GitHub release URLs and hashes
pub const BINARY_SOURCES: &[BinarySource] = &[
    // Zapret/winws binaries
    BinarySource {
        name: "zapret",
        url: "https://github.com/bol-van/zapret/releases/download/v70.6/zapret-win-bundle-v70.6.zip",
        sha256: "", // TODO: Add actual hash
        is_archive: true,
        extract_files: &["winws.exe", "WinDivert64.sys", "WinDivert.dll"],
    },
    // Sing-box binary
    BinarySource {
        name: "sing-box",
        url: "https://github.com/SagerNet/sing-box/releases/download/v1.10.0/sing-box-1.10.0-windows-amd64.zip",
        sha256: "", // TODO: Add actual hash
        is_archive: true,
        extract_files: &["sing-box.exe"],
    },
];

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
        if path.exists() {
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
    path.exists()
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

/// Extract a ZIP archive to destination directory
///
/// # Arguments
/// * `archive` - Path to the ZIP archive
/// * `dest_dir` - Directory to extract files to
/// * `files_to_extract` - Optional list of specific files to extract (None = extract all)
///
/// # Returns
/// * `Ok(Vec<String>)` - List of extracted file names
/// * `Err(IsolateError)` - Extraction failed
pub fn extract_zip(
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

/// Download and extract a binary source
///
/// # Arguments
/// * `source` - Binary source configuration
/// * `progress` - Progress callback receiving (binary_name, downloaded, total)
///
/// # Returns
/// * `Ok(())` - Binary downloaded and extracted successfully
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

    // Extract
    if source.is_archive {
        let files_to_extract: Vec<&str> = source.extract_files.to_vec();
        let extracted = extract_zip(
            &archive_path,
            &binaries_dir,
            Some(&files_to_extract),
        )?;

        // Verify all expected files were extracted
        for expected in source.extract_files {
            if !extracted.iter().any(|f| f == *expected) {
                warn!("Expected file not found in archive: {}", expected);
            }
        }
    }

    // Cleanup temp file
    if let Err(e) = fs::remove_file(&archive_path).await {
        warn!("Failed to cleanup temp file: {}", e);
    }

    // Verify hash if provided
    if !source.sha256.is_empty() {
        for file_name in source.extract_files {
            let file_path = binaries_dir.join(file_name);
            if file_path.exists() {
                match verify_binary_hash(&file_path, source.sha256) {
                    Ok(true) => {
                        info!("Hash verified for: {}", file_name);
                    }
                    Ok(false) => {
                        error!("Hash mismatch for: {}", file_name);
                        return Err(IsolateError::Process(format!(
                            "Hash verification failed for {}",
                            file_name
                        )));
                    }
                    Err(e) => {
                        warn!("Could not verify hash for {}: {}", file_name, e);
                    }
                }
            }
        }
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
        if path.exists() {
            fs::remove_file(&path).await?;
            info!("Removed: {}", binary);
        }
    }

    // Also cleanup temp directory
    let temp_dir = binaries_dir.join("temp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).await?;
    }

    Ok(())
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
}
