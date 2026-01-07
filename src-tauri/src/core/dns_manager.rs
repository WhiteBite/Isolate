//! DNS Manager - управление настройками DNS
//!
//! Модуль для управления DNS настройками приложения.
//! Поддерживает различные DNS серверы и кастомные адреса.

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::storage::Storage;

// ============================================================================
// DNS Settings Model
// ============================================================================

/// DNS сервер
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DnsServer {
    /// Системный DNS (по умолчанию)
    System,
    /// Cloudflare DNS (1.1.1.1)
    Cloudflare,
    /// Google DNS (8.8.8.8)
    Google,
    /// Quad9 DNS (9.9.9.9)
    Quad9,
    /// OpenDNS (208.67.222.222)
    OpenDns,
    /// AdGuard DNS (94.140.14.14)
    AdGuard,
    /// Кастомный DNS сервер
    Custom,
}

impl Default for DnsServer {
    fn default() -> Self {
        Self::System
    }
}

impl DnsServer {
    /// Возвращает IP адрес DNS сервера
    pub fn get_address(&self) -> Option<&'static str> {
        match self {
            Self::System => None,
            Self::Cloudflare => Some("1.1.1.1"),
            Self::Google => Some("8.8.8.8"),
            Self::Quad9 => Some("9.9.9.9"),
            Self::OpenDns => Some("208.67.222.222"),
            Self::AdGuard => Some("94.140.14.14"),
            Self::Custom => None, // Используется custom_address
        }
    }

    /// Возвращает альтернативный IP адрес DNS сервера
    pub fn get_secondary_address(&self) -> Option<&'static str> {
        match self {
            Self::System => None,
            Self::Cloudflare => Some("1.0.0.1"),
            Self::Google => Some("8.8.4.4"),
            Self::Quad9 => Some("149.112.112.112"),
            Self::OpenDns => Some("208.67.220.220"),
            Self::AdGuard => Some("94.140.15.15"),
            Self::Custom => None,
        }
    }
}

/// Настройки DNS
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsSettings {
    /// Выбранный DNS сервер
    pub server: DnsServer,
    /// Кастомный адрес DNS (используется когда server = Custom)
    pub custom_address: Option<String>,
    /// Включить DNS over HTTPS (DoH)
    #[serde(default)]
    pub doh_enabled: bool,
    /// URL для DoH (если включен)
    pub doh_url: Option<String>,
}

impl Default for DnsSettings {
    fn default() -> Self {
        Self {
            server: DnsServer::System,
            custom_address: None,
            doh_enabled: false,
            doh_url: None,
        }
    }
}

impl DnsSettings {
    /// Возвращает эффективный DNS адрес
    pub fn get_effective_address(&self) -> Option<String> {
        match &self.server {
            DnsServer::Custom => self.custom_address.clone(),
            server => server.get_address().map(String::from),
        }
    }

    /// Возвращает эффективный вторичный DNS адрес
    pub fn get_effective_secondary_address(&self) -> Option<String> {
        match &self.server {
            DnsServer::Custom => None, // Для кастомного DNS нет вторичного
            server => server.get_secondary_address().map(String::from),
        }
    }

    /// Проверяет валидность настроек
    pub fn validate(&self) -> Result<()> {
        // Если выбран Custom, должен быть указан адрес
        if self.server == DnsServer::Custom {
            match &self.custom_address {
                Some(addr) if !addr.is_empty() => {
                    // Проверяем что это валидный IP адрес
                    if addr.parse::<std::net::IpAddr>().is_err() {
                        return Err(IsolateError::Validation(format!(
                            "Invalid custom DNS address: {}",
                            addr
                        )));
                    }
                }
                _ => {
                    return Err(IsolateError::Validation(
                        "Custom DNS address is required when using Custom server".into(),
                    ));
                }
            }
        }

        // Если включен DoH, должен быть указан URL
        if self.doh_enabled {
            match &self.doh_url {
                Some(url) if !url.is_empty() => {
                    // Базовая проверка URL
                    if !url.starts_with("https://") {
                        return Err(IsolateError::Validation(
                            "DoH URL must start with https://".into(),
                        ));
                    }
                }
                _ => {
                    return Err(IsolateError::Validation(
                        "DoH URL is required when DoH is enabled".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// Storage Key
// ============================================================================

const DNS_SETTINGS_KEY: &str = "dns_settings";

// ============================================================================
// DNS Manager Functions
// ============================================================================

/// Получает текущие настройки DNS из storage
pub async fn get_dns_settings(storage: &Storage) -> Result<DnsSettings> {
    debug!("Getting DNS settings from storage");

    let settings: Option<DnsSettings> = storage.get_setting(DNS_SETTINGS_KEY).await?;

    Ok(settings.unwrap_or_default())
}

/// Сохраняет настройки DNS в storage
pub async fn set_dns_settings(storage: &Storage, settings: &DnsSettings) -> Result<()> {
    // Валидируем настройки перед сохранением
    settings.validate()?;

    info!(
        server = ?settings.server,
        custom_address = ?settings.custom_address,
        doh_enabled = settings.doh_enabled,
        "Saving DNS settings"
    );

    storage.set_setting(DNS_SETTINGS_KEY, settings).await?;

    debug!("DNS settings saved successfully");
    Ok(())
}

/// Устанавливает DNS сервер
pub async fn set_dns_server(
    storage: &Storage,
    server: DnsServer,
    custom_address: Option<String>,
) -> Result<()> {
    let mut settings = get_dns_settings(storage).await?;

    settings.server = server;
    settings.custom_address = custom_address;

    set_dns_settings(storage, &settings).await
}

/// Сбрасывает настройки DNS на системные
pub async fn reset_dns_settings(storage: &Storage) -> Result<()> {
    info!("Resetting DNS settings to system defaults");

    let default_settings = DnsSettings::default();
    storage.set_setting(DNS_SETTINGS_KEY, &default_settings).await?;

    Ok(())
}

// ============================================================================
// Windows DNS Configuration
// ============================================================================

/// Применяет настройки DNS к системе Windows
///
/// ВАЖНО: Эта функция требует прав администратора!
///
/// Использует netsh для изменения DNS на активных сетевых интерфейсах.
pub async fn apply_dns_to_system(settings: &DnsSettings) -> Result<()> {
    use tokio::process::Command;
    
    info!(server = ?settings.server, "Applying DNS settings to system");

    // Get effective DNS address
    let primary_address = match settings.get_effective_address() {
        Some(addr) => addr,
        None => {
            // System DNS - restore to DHCP
            return restore_system_dns().await;
        }
    };

    let secondary_address = settings.get_effective_secondary_address();

    // Get active network interfaces
    let interfaces = get_active_network_interfaces().await?;
    
    if interfaces.is_empty() {
        warn!("No active network interfaces found");
        return Ok(());
    }

    for interface in &interfaces {
        info!(interface = %interface, primary = %primary_address, "Setting DNS for interface");

        // Set primary DNS
        let output = Command::new("netsh")
            .args([
                "interface", "ip", "set", "dns",
                interface,
                "static",
                &primary_address,
                "primary"
            ])
            .output()
            .await
            .map_err(|e| IsolateError::Process(format!("Failed to run netsh: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                interface = %interface,
                error = %stderr,
                "Failed to set primary DNS (may require admin)"
            );
            // Continue with other interfaces
            continue;
        }

        // Set secondary DNS if available
        if let Some(ref secondary) = secondary_address {
            let output = Command::new("netsh")
                .args([
                    "interface", "ip", "add", "dns",
                    interface,
                    secondary,
                    "index=2"
                ])
                .output()
                .await
                .map_err(|e| IsolateError::Process(format!("Failed to run netsh: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                debug!(
                    interface = %interface,
                    error = %stderr,
                    "Failed to set secondary DNS"
                );
            }
        }

        info!(interface = %interface, "DNS configured successfully");
    }

    Ok(())
}

/// Восстанавливает системные DNS настройки (DHCP)
pub async fn restore_system_dns() -> Result<()> {
    use tokio::process::Command;
    
    info!("Restoring system DNS to DHCP");

    let interfaces = get_active_network_interfaces().await?;

    for interface in &interfaces {
        let output = Command::new("netsh")
            .args([
                "interface", "ip", "set", "dns",
                interface,
                "dhcp"
            ])
            .output()
            .await
            .map_err(|e| IsolateError::Process(format!("Failed to run netsh: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                interface = %interface,
                error = %stderr,
                "Failed to restore DNS to DHCP"
            );
        } else {
            info!(interface = %interface, "DNS restored to DHCP");
        }
    }

    Ok(())
}

/// Получает список активных сетевых интерфейсов
async fn get_active_network_interfaces() -> Result<Vec<String>> {
    use tokio::process::Command;
    
    // Use PowerShell to get active interfaces
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | Select-Object -ExpandProperty Name"
        ])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to get network interfaces: {}", e)))?;

    if !output.status.success() {
        // Fallback to common interface names
        debug!("PowerShell failed, using fallback interface names");
        return Ok(vec!["Ethernet".to_string(), "Wi-Fi".to_string()]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let interfaces: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    debug!(interfaces = ?interfaces, "Found active network interfaces");
    Ok(interfaces)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_server_addresses() {
        assert_eq!(DnsServer::Cloudflare.get_address(), Some("1.1.1.1"));
        assert_eq!(DnsServer::Google.get_address(), Some("8.8.8.8"));
        assert_eq!(DnsServer::System.get_address(), None);
        assert_eq!(DnsServer::Custom.get_address(), None);
    }

    #[test]
    fn test_dns_settings_validation() {
        // Valid system settings
        let settings = DnsSettings::default();
        assert!(settings.validate().is_ok());

        // Invalid custom without address
        let settings = DnsSettings {
            server: DnsServer::Custom,
            custom_address: None,
            ..Default::default()
        };
        assert!(settings.validate().is_err());

        // Valid custom with address
        let settings = DnsSettings {
            server: DnsServer::Custom,
            custom_address: Some("1.2.3.4".into()),
            ..Default::default()
        };
        assert!(settings.validate().is_ok());

        // Invalid custom with bad address
        let settings = DnsSettings {
            server: DnsServer::Custom,
            custom_address: Some("not-an-ip".into()),
            ..Default::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_effective_address() {
        let settings = DnsSettings {
            server: DnsServer::Cloudflare,
            custom_address: None,
            ..Default::default()
        };
        assert_eq!(settings.get_effective_address(), Some("1.1.1.1".into()));

        let settings = DnsSettings {
            server: DnsServer::Custom,
            custom_address: Some("9.9.9.9".into()),
            ..Default::default()
        };
        assert_eq!(settings.get_effective_address(), Some("9.9.9.9".into()));
    }
}
