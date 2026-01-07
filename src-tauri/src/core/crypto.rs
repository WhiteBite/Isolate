//! Cryptographic utilities for Isolate
//!
//! Provides secure encryption/decryption using Windows DPAPI (Data Protection API).
//! DPAPI encrypts data using the current user's credentials, ensuring that only
//! the same user on the same machine can decrypt the data.

use crate::core::errors::{IsolateError, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use tracing::{debug, error};

/// Prefix for encrypted data to identify DPAPI-encrypted strings
const DPAPI_PREFIX: &str = "DPAPI:";

/// Encrypts sensitive data using Windows DPAPI
///
/// # Arguments
/// * `plaintext` - The data to encrypt
///
/// # Returns
/// * `Ok(String)` - Base64-encoded encrypted data with DPAPI prefix
/// * `Err(IsolateError)` - If encryption fails
///
/// # Security
/// - Data is encrypted using the current Windows user's credentials
/// - Only the same user on the same machine can decrypt the data
/// - Uses Windows CryptProtectData internally
#[cfg(windows)]
pub fn encrypt_dpapi(plaintext: &str) -> Result<String> {
    use std::ptr;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPT_INTEGER_BLOB,
    };
    use windows_sys::Win32::Foundation::{GetLastError, LocalFree};

    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let mut plaintext_bytes = plaintext.as_bytes().to_vec();
    
    let data_in = CRYPT_INTEGER_BLOB {
        cbData: plaintext_bytes.len() as u32,
        pbData: plaintext_bytes.as_mut_ptr(),
    };
    
    let mut data_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    let result = unsafe {
        CryptProtectData(
            &data_in,
            ptr::null(),      // szDataDescr - optional description
            ptr::null(),      // pOptionalEntropy - additional entropy (not used)
            ptr::null_mut(),  // pvReserved
            ptr::null(),      // pPromptStruct - no UI prompt
            0,                // dwFlags
            &mut data_out,
        )
    };

    if result == 0 {
        let error_code = unsafe { GetLastError() };
        error!("CryptProtectData failed with error code: {}", error_code);
        return Err(IsolateError::Storage(format!(
            "Failed to encrypt data: Windows error {}",
            error_code
        )));
    }

    // Copy encrypted data and free Windows-allocated memory
    let encrypted_bytes = unsafe {
        let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
        let bytes = slice.to_vec();
        LocalFree(data_out.pbData as *mut _);
        bytes
    };

    // Clear plaintext from memory
    plaintext_bytes.iter_mut().for_each(|b| *b = 0);

    let encoded = BASE64.encode(&encrypted_bytes);
    debug!("Data encrypted successfully ({} bytes)", encrypted_bytes.len());
    
    Ok(format!("{}{}", DPAPI_PREFIX, encoded))
}

/// Decrypts data that was encrypted with Windows DPAPI
///
/// # Arguments
/// * `encrypted` - Base64-encoded encrypted data with DPAPI prefix
///
/// # Returns
/// * `Ok(String)` - Decrypted plaintext
/// * `Err(IsolateError)` - If decryption fails
///
/// # Security
/// - Only the same Windows user who encrypted the data can decrypt it
/// - Uses Windows CryptUnprotectData internally
#[cfg(windows)]
pub fn decrypt_dpapi(encrypted: &str) -> Result<String> {
    use std::ptr;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPT_INTEGER_BLOB,
    };
    use windows_sys::Win32::Foundation::{GetLastError, LocalFree};

    if encrypted.is_empty() {
        return Ok(String::new());
    }

    // Check for DPAPI prefix
    let encoded = if encrypted.starts_with(DPAPI_PREFIX) {
        &encrypted[DPAPI_PREFIX.len()..]
    } else {
        // Not encrypted with DPAPI, return as-is (for backward compatibility)
        debug!("Data not DPAPI-encrypted, returning as-is");
        return Ok(encrypted.to_string());
    };

    let mut encrypted_bytes = BASE64.decode(encoded).map_err(|e| {
        IsolateError::Storage(format!("Failed to decode encrypted data: {}", e))
    })?;

    let data_in = CRYPT_INTEGER_BLOB {
        cbData: encrypted_bytes.len() as u32,
        pbData: encrypted_bytes.as_mut_ptr(),
    };

    let mut data_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    let result = unsafe {
        CryptUnprotectData(
            &data_in,
            ptr::null_mut(),  // ppszDataDescr
            ptr::null(),      // pOptionalEntropy
            ptr::null_mut(),  // pvReserved
            ptr::null(),      // pPromptStruct
            0,                // dwFlags
            &mut data_out,
        )
    };

    if result == 0 {
        let error_code = unsafe { GetLastError() };
        error!("CryptUnprotectData failed with error code: {}", error_code);
        return Err(IsolateError::Storage(format!(
            "Failed to decrypt data: Windows error {}",
            error_code
        )));
    }

    // Copy decrypted data and free Windows-allocated memory
    let decrypted_bytes = unsafe {
        let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
        let bytes = slice.to_vec();
        
        // Zero out the Windows-allocated buffer before freeing
        std::ptr::write_bytes(data_out.pbData, 0, data_out.cbData as usize);
        LocalFree(data_out.pbData as *mut _);
        bytes
    };

    // Clear encrypted bytes from memory
    encrypted_bytes.iter_mut().for_each(|b| *b = 0);

    let plaintext = String::from_utf8(decrypted_bytes).map_err(|e| {
        IsolateError::Storage(format!("Decrypted data is not valid UTF-8: {}", e))
    })?;

    debug!("Data decrypted successfully");
    Ok(plaintext)
}

/// Checks if a string is DPAPI-encrypted
pub fn is_dpapi_encrypted(data: &str) -> bool {
    data.starts_with(DPAPI_PREFIX)
}

/// Encrypts a password if it's not already encrypted
///
/// # Arguments
/// * `password` - Password to encrypt (may already be encrypted)
///
/// # Returns
/// * `Ok(String)` - Encrypted password
#[cfg(windows)]
pub fn encrypt_password(password: &str) -> Result<String> {
    if password.is_empty() || is_dpapi_encrypted(password) {
        return Ok(password.to_string());
    }
    encrypt_dpapi(password)
}

/// Decrypts a password if it's encrypted
///
/// # Arguments
/// * `password` - Password to decrypt (may be plaintext for backward compatibility)
///
/// # Returns
/// * `Ok(String)` - Decrypted password
#[cfg(windows)]
pub fn decrypt_password(password: &str) -> Result<String> {
    if password.is_empty() {
        return Ok(String::new());
    }
    decrypt_dpapi(password)
}

// Non-Windows stubs for compilation
#[cfg(not(windows))]
pub fn encrypt_dpapi(plaintext: &str) -> Result<String> {
    // On non-Windows, just return the plaintext (no encryption)
    Ok(plaintext.to_string())
}

#[cfg(not(windows))]
pub fn decrypt_dpapi(encrypted: &str) -> Result<String> {
    // On non-Windows, just return as-is
    Ok(encrypted.to_string())
}

#[cfg(not(windows))]
pub fn encrypt_password(password: &str) -> Result<String> {
    Ok(password.to_string())
}

#[cfg(not(windows))]
pub fn decrypt_password(password: &str) -> Result<String> {
    Ok(password.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dpapi_encrypted() {
        assert!(is_dpapi_encrypted("DPAPI:abc123"));
        assert!(!is_dpapi_encrypted("plaintext"));
        assert!(!is_dpapi_encrypted(""));
    }

    #[test]
    fn test_empty_string() {
        let encrypted = encrypt_dpapi("").unwrap();
        assert_eq!(encrypted, "");
        
        let decrypted = decrypt_dpapi("").unwrap();
        assert_eq!(decrypted, "");
    }

    #[cfg(windows)]
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "my_secret_password_123!@#";
        let encrypted = encrypt_dpapi(original).unwrap();
        
        // Should be prefixed
        assert!(encrypted.starts_with(DPAPI_PREFIX));
        
        // Should be different from original
        assert_ne!(encrypted, original);
        
        // Should decrypt back to original
        let decrypted = decrypt_dpapi(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[cfg(windows)]
    #[test]
    fn test_encrypt_password_idempotent() {
        let password = "test_password";
        
        // First encryption
        let encrypted1 = encrypt_password(password).unwrap();
        assert!(is_dpapi_encrypted(&encrypted1));
        
        // Second encryption should return same (already encrypted)
        let encrypted2 = encrypt_password(&encrypted1).unwrap();
        assert_eq!(encrypted1, encrypted2);
    }

    #[test]
    fn test_decrypt_plaintext_backward_compat() {
        // Plaintext without DPAPI prefix should be returned as-is
        let plaintext = "old_unencrypted_password";
        let result = decrypt_dpapi(plaintext).unwrap();
        assert_eq!(result, plaintext);
    }
}
