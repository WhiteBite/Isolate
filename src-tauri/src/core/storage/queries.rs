//! SQL query implementations for Storage
//!
//! This module contains implementations for proxy, routing, and learned strategies queries.

use rusqlite::{params, OptionalExtension};
use tracing::{debug, error, info};

use crate::core::crypto::encrypt_password;
use crate::core::errors::{IsolateError, Result};
use crate::core::models::{AppRoute, DomainRoute, ProxyConfig};
use super::types::{LearnedStrategy, TestHistoryEntry, ProxyConfigRow};
use super::database::Storage;

impl Storage {
    // ========================================================================
    // Proxy Management
    // ========================================================================

    /// Save a proxy configuration
    /// 
    /// # Security
    /// Password is encrypted using Windows DPAPI before storing in the database.
    pub async fn save_proxy(&self, proxy: &ProxyConfig) -> Result<()> {
        let conn = self.conn.clone();

        let protocol = serde_json::to_string(&proxy.protocol)?;
        let custom_fields = serde_json::to_string(&proxy.custom_fields)?;
        
        // Encrypt password before storing
        let encrypted_password = match &proxy.password {
            Some(pwd) if !pwd.is_empty() => {
                match encrypt_password(pwd) {
                    Ok(encrypted) => Some(encrypted),
                    Err(e) => {
                        error!("[SECURITY] Failed to encrypt password: {}. Refusing to store plaintext.", e);
                        return Err(IsolateError::other(format!(
                            "Cannot save proxy: password encryption failed. Please check Windows DPAPI availability."
                        )));
                    }
                }
            }
            other => other.clone(),
        };

        let proxy_id = proxy.id.clone();
        let proxy_name = proxy.name.clone();
        let proxy_server = proxy.server.clone();
        let proxy_port = proxy.port;
        let proxy_username = proxy.username.clone();
        let proxy_uuid = proxy.uuid.clone();
        let proxy_tls = proxy.tls;
        let proxy_sni = proxy.sni.clone();
        let proxy_transport = proxy.transport.clone();
        let proxy_active = proxy.active;

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT INTO proxies (id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'), datetime('now'))
                "#,
                params![
                    proxy_id,
                    proxy_name,
                    protocol.trim_matches('"'),
                    proxy_server,
                    proxy_port,
                    proxy_username,
                    encrypted_password,
                    proxy_uuid,
                    proxy_tls as i32,
                    proxy_sni,
                    proxy_transport,
                    custom_fields,
                    proxy_active as i32,
                ],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(id = %proxy.id, name = %proxy.name, "Proxy saved (password encrypted)");
        Ok(())
    }

    /// Get a proxy by ID
    pub async fn get_proxy(&self, id: &str) -> Result<Option<ProxyConfig>> {
        let conn = self.conn.clone();
        let id = id.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                r#"
                SELECT id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active
                FROM proxies
                WHERE id = ?1
                "#,
                params![id],
                |row| {
                    let protocol_str: String = row.get(2)?;
                    let custom_fields_str: String = row.get(11)?;
                    let tls_int: i32 = row.get(8)?;
                    let active_int: i32 = row.get(12)?;

                    Ok(ProxyConfigRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: protocol_str,
                        server: row.get(3)?,
                        port: row.get(4)?,
                        username: row.get(5)?,
                        password: row.get(6)?,
                        uuid: row.get(7)?,
                        tls: tls_int != 0,
                        sni: row.get(9)?,
                        transport: row.get(10)?,
                        custom_fields: custom_fields_str,
                        active: active_int != 0,
                    })
                },
            )
            .optional()
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        match result {
            Some(row) => Ok(Some(row.into_proxy_config()?)),
            None => Ok(None),
        }
    }

    /// Get all proxies
    pub async fn get_all_proxies(&self) -> Result<Vec<ProxyConfig>> {
        let conn = self.conn.clone();

        let rows = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active
                FROM proxies
                ORDER BY created_at DESC
                "#,
            )?;

            let rows = stmt.query_map([], |row| {
                let protocol_str: String = row.get(2)?;
                let custom_fields_str: String = row.get(11)?;
                let tls_int: i32 = row.get(8)?;
                let active_int: i32 = row.get(12)?;

                Ok(ProxyConfigRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    protocol: protocol_str,
                    server: row.get(3)?,
                    port: row.get(4)?,
                    username: row.get(5)?,
                    password: row.get(6)?,
                    uuid: row.get(7)?,
                    tls: tls_int != 0,
                    sni: row.get(9)?,
                    transport: row.get(10)?,
                    custom_fields: custom_fields_str,
                    active: active_int != 0,
                })
            })?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok::<_, rusqlite::Error>(result)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        let mut proxies = Vec::new();
        for row in rows {
            proxies.push(row.into_proxy_config()?);
        }

        Ok(proxies)
    }

    /// Delete a proxy by ID
    pub async fn delete_proxy(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id_str = id.to_string();
        let id_for_log = id.to_string();

        let deleted = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM proxies WHERE id = ?1", params![id_str])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if deleted == 0 {
            return Err(IsolateError::Storage(format!("Proxy '{}' not found", id)));
        }

        debug!(id = %id_for_log, "Proxy deleted");
        Ok(())
    }

    /// Update an existing proxy
    pub async fn update_proxy(&self, proxy: &ProxyConfig) -> Result<()> {
        let conn = self.conn.clone();

        let protocol = serde_json::to_string(&proxy.protocol)?;
        let custom_fields = serde_json::to_string(&proxy.custom_fields)?;
        
        // Encrypt password before storing
        let encrypted_password = match &proxy.password {
            Some(pwd) if !pwd.is_empty() => {
                match encrypt_password(pwd) {
                    Ok(encrypted) => Some(encrypted),
                    Err(e) => {
                        error!("[SECURITY] Failed to encrypt password: {}. Refusing to store plaintext.", e);
                        return Err(IsolateError::other(format!(
                            "Cannot update proxy: password encryption failed."
                        )));
                    }
                }
            }
            other => other.clone(),
        };

        let proxy_id = proxy.id.clone();
        let proxy_name = proxy.name.clone();
        let proxy_server = proxy.server.clone();
        let proxy_port = proxy.port;
        let proxy_username = proxy.username.clone();
        let proxy_uuid = proxy.uuid.clone();
        let proxy_tls = proxy.tls;
        let proxy_sni = proxy.sni.clone();
        let proxy_transport = proxy.transport.clone();
        let proxy_active = proxy.active;
        let proxy_id_for_err = proxy.id.clone();

        let updated = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                UPDATE proxies
                SET name = ?2, protocol = ?3, server = ?4, port = ?5, username = ?6, password = ?7,
                    uuid = ?8, tls = ?9, sni = ?10, transport = ?11, custom_fields = ?12, active = ?13,
                    updated_at = datetime('now')
                WHERE id = ?1
                "#,
                params![
                    proxy_id,
                    proxy_name,
                    protocol.trim_matches('"'),
                    proxy_server,
                    proxy_port,
                    proxy_username,
                    encrypted_password,
                    proxy_uuid,
                    proxy_tls as i32,
                    proxy_sni,
                    proxy_transport,
                    custom_fields,
                    proxy_active as i32,
                ],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if updated == 0 {
            return Err(IsolateError::Storage(format!("Proxy '{}' not found", proxy_id_for_err)));
        }

        debug!(id = %proxy.id, name = %proxy.name, "Proxy updated");
        Ok(())
    }

    /// Set proxy active state
    pub async fn set_proxy_active(&self, id: &str, active: bool) -> Result<()> {
        let conn = self.conn.clone();
        let id_str = id.to_string();
        let id_for_err = id.to_string();

        let updated = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            
            // If activating, deactivate all other proxies first
            if active {
                conn.execute("UPDATE proxies SET active = 0", [])?;
            }

            conn.execute(
                "UPDATE proxies SET active = ?2, updated_at = datetime('now') WHERE id = ?1",
                params![id_str, active as i32],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if updated == 0 {
            return Err(IsolateError::Storage(format!("Proxy '{}' not found", id_for_err)));
        }

        debug!(id = %id, active, "Proxy active state updated");
        Ok(())
    }

    /// Get the currently active proxy
    pub async fn get_active_proxy(&self) -> Result<Option<ProxyConfig>> {
        let conn = self.conn.clone();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                r#"
                SELECT id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active
                FROM proxies
                WHERE active = 1
                LIMIT 1
                "#,
                [],
                |row| {
                    let protocol_str: String = row.get(2)?;
                    let custom_fields_str: String = row.get(11)?;
                    let tls_int: i32 = row.get(8)?;
                    let active_int: i32 = row.get(12)?;

                    Ok(ProxyConfigRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: protocol_str,
                        server: row.get(3)?,
                        port: row.get(4)?,
                        username: row.get(5)?,
                        password: row.get(6)?,
                        uuid: row.get(7)?,
                        tls: tls_int != 0,
                        sni: row.get(9)?,
                        transport: row.get(10)?,
                        custom_fields: custom_fields_str,
                        active: active_int != 0,
                    })
                },
            )
            .optional()
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        match result {
            Some(row) => Ok(Some(row.into_proxy_config()?)),
            None => Ok(None),
        }
    }

    // ========================================================================
    // Domain Routes
    // ========================================================================

    /// Save a domain route
    pub async fn save_domain_route(&self, route: &DomainRoute) -> Result<()> {
        let conn = self.conn.clone();
        let domain = route.domain.clone();
        let proxy_id = route.proxy_id.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "INSERT OR REPLACE INTO domain_routes (domain, proxy_id) VALUES (?1, ?2)",
                params![domain, proxy_id],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(domain = %route.domain, proxy_id = %route.proxy_id, "Domain route saved");
        Ok(())
    }

    /// Delete a domain route
    pub async fn delete_domain_route(&self, domain: &str) -> Result<()> {
        let conn = self.conn.clone();
        let domain_for_log = domain.to_string();
        let domain = domain.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM domain_routes WHERE domain = ?1",
                params![domain],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(domain = %domain_for_log, "Domain route deleted");
        Ok(())
    }

    /// Get all domain routes
    pub async fn get_domain_routes(&self) -> Result<Vec<DomainRoute>> {
        let conn = self.conn.clone();

        let routes = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare("SELECT domain, proxy_id FROM domain_routes")?;
            let rows = stmt.query_map([], |row| {
                Ok(DomainRoute {
                    domain: row.get(0)?,
                    proxy_id: row.get(1)?,
                })
            })?;

            let mut routes = Vec::new();
            for row in rows {
                routes.push(row?);
            }
            Ok::<_, rusqlite::Error>(routes)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(routes)
    }

    // ========================================================================
    // App Routes
    // ========================================================================

    /// Save an app route
    pub async fn save_app_route(&self, route: &AppRoute) -> Result<()> {
        let conn = self.conn.clone();
        let app_path = route.app_path.clone();
        let app_name = route.app_name.clone();
        let proxy_id = route.proxy_id.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "INSERT OR REPLACE INTO app_routes (app_path, app_name, proxy_id) VALUES (?1, ?2, ?3)",
                params![app_path, app_name, proxy_id],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(app_path = %route.app_path, "App route saved");
        Ok(())
    }

    /// Delete an app route
    pub async fn delete_app_route(&self, app_path: &str) -> Result<()> {
        let conn = self.conn.clone();
        let app_path_for_log = app_path.to_string();
        let app_path = app_path.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM app_routes WHERE app_path = ?1",
                params![app_path],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(app_path = %app_path_for_log, "App route deleted");
        Ok(())
    }

    /// Get all app routes
    pub async fn get_app_routes(&self) -> Result<Vec<AppRoute>> {
        let conn = self.conn.clone();

        let routes = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare("SELECT app_path, app_name, proxy_id FROM app_routes")?;
            let rows = stmt.query_map([], |row| {
                Ok(AppRoute {
                    app_path: row.get(0)?,
                    app_name: row.get(1)?,
                    proxy_id: row.get(2)?,
                })
            })?;

            let mut routes = Vec::new();
            for row in rows {
                routes.push(row?);
            }
            Ok::<_, rusqlite::Error>(routes)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(routes)
    }
}

impl Storage {
    // ========================================================================
    // Test History
    // ========================================================================

    /// Сохраняет результат теста стратегии
    pub async fn save_test_result(
        &self,
        env_key: &str,
        strategy_id: &str,
        success: bool,
        score: f64,
        latency_ms: f64,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let env_key = env_key.to_string();
        let strategy_id = strategy_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT INTO test_history (env_key, strategy_id, success, score, latency_ms, timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, unixepoch())
                "#,
                params![env_key, strategy_id, success, score, latency_ms],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(())
    }

    /// Получает историю тестов для стратегии
    pub async fn get_test_history(
        &self,
        strategy_id: &str,
        limit: u32,
    ) -> Result<Vec<TestHistoryEntry>> {
        let conn = self.conn.clone();
        let strategy_id = strategy_id.to_string();

        let history = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT env_key, success, score, latency_ms, timestamp
                FROM test_history
                WHERE strategy_id = ?1
                ORDER BY timestamp DESC
                LIMIT ?2
                "#,
            )?;

            let rows = stmt.query_map(params![strategy_id, limit], |row| {
                Ok(TestHistoryEntry {
                    env_key: row.get(0)?,
                    success: row.get(1)?,
                    score: row.get(2)?,
                    latency_ms: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })?;

            let mut history = Vec::new();
            for row in rows {
                history.push(row?);
            }
            Ok::<_, rusqlite::Error>(history)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(history)
    }

    // ========================================================================
    // Learned Strategies (Orchestra)
    // ========================================================================

    /// Сохраняет обученную стратегию для домена
    pub async fn save_learned_strategy(
        &self,
        domain: &str,
        strategy_id: &str,
        successes: u32,
        failures: u32,
        locked_at: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let domain_for_log = domain.to_string();
        let strategy_id_for_log = strategy_id.to_string();
        let domain = domain.to_string();
        let strategy_id = strategy_id.to_string();
        let locked_at = locked_at.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT OR REPLACE INTO learned_strategies (domain, strategy_id, successes, failures, locked_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
                "#,
                params![domain, strategy_id, successes, failures, locked_at],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(domain = domain_for_log, strategy_id = strategy_id_for_log, "Learned strategy saved");
        Ok(())
    }

    /// Получает все обученные стратегии
    pub async fn get_learned_strategies(&self) -> Result<std::collections::HashMap<String, LearnedStrategy>> {
        let conn = self.conn.clone();

        let strategies = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT domain, strategy_id, successes, failures, locked_at, updated_at
                FROM learned_strategies
                "#,
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(LearnedStrategy {
                    domain: row.get(0)?,
                    strategy_id: row.get(1)?,
                    successes: row.get::<_, i32>(2)? as u32,
                    failures: row.get::<_, i32>(3)? as u32,
                    locked_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;

            let mut strategies = std::collections::HashMap::new();
            for row in rows {
                let strategy = row?;
                strategies.insert(strategy.domain.clone(), strategy);
            }
            Ok::<_, rusqlite::Error>(strategies)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(count = strategies.len(), "Loaded learned strategies");
        Ok(strategies)
    }

    /// Получает обученную стратегию для конкретного домена
    pub async fn get_learned_strategy(&self, domain: &str) -> Result<Option<LearnedStrategy>> {
        let conn = self.conn.clone();
        let domain = domain.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                r#"
                SELECT domain, strategy_id, successes, failures, locked_at, updated_at
                FROM learned_strategies
                WHERE domain = ?1
                "#,
                params![domain],
                |row| {
                    Ok(LearnedStrategy {
                        domain: row.get(0)?,
                        strategy_id: row.get(1)?,
                        successes: row.get::<_, i32>(2)? as u32,
                        failures: row.get::<_, i32>(3)? as u32,
                        locked_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(result)
    }

    /// Удаляет обученную стратегию для домена
    pub async fn delete_learned_strategy(&self, domain: &str) -> Result<()> {
        let conn = self.conn.clone();
        let domain_for_log = domain.to_string();
        let domain = domain.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM learned_strategies WHERE domain = ?1",
                params![domain],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(domain = domain_for_log, "Learned strategy deleted");
        Ok(())
    }

    /// Очищает все обученные стратегии
    pub async fn clear_learned_strategies(&self) -> Result<()> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM learned_strategies", [])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!("All learned strategies cleared");
        Ok(())
    }

    // ========================================================================
    // Strategy History (Success/Failure Tracking)
    // ========================================================================

    /// Записывает результат выполнения стратегии
    pub async fn record_strategy_result(
        &self,
        strategy_id: &str,
        service_id: &str,
        success: bool,
        latency_ms: Option<f64>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let strategy_id_owned = strategy_id.to_string();
        let service_id_owned = service_id.to_string();
        let strategy_id_log = strategy_id.to_string();
        let service_id_log = service_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT INTO strategy_history (strategy_id, service_id, success, latency_ms, timestamp)
                VALUES (?1, ?2, ?3, ?4, unixepoch())
                "#,
                params![strategy_id_owned, service_id_owned, success as i32, latency_ms],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(strategy_id = %strategy_id_log, service_id = %service_id_log, success, "Strategy result recorded");
        Ok(())
    }

    /// Получает историю результатов для стратегии
    pub async fn get_strategy_history(
        &self,
        strategy_id: &str,
        limit: u32,
    ) -> Result<Vec<super::types::StrategyHistoryEntry>> {
        let conn = self.conn.clone();
        let strategy_id = strategy_id.to_string();

        let history = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, strategy_id, service_id, success, latency_ms, timestamp
                FROM strategy_history
                WHERE strategy_id = ?1
                ORDER BY timestamp DESC
                LIMIT ?2
                "#,
            )?;

            let rows = stmt.query_map(params![strategy_id, limit], |row| {
                let success_int: i32 = row.get(3)?;
                Ok(super::types::StrategyHistoryEntry {
                    id: row.get(0)?,
                    strategy_id: row.get(1)?,
                    service_id: row.get(2)?,
                    success: success_int != 0,
                    latency_ms: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })?;

            let mut history = Vec::new();
            for row in rows {
                history.push(row?);
            }
            Ok::<_, rusqlite::Error>(history)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(history)
    }

    /// Получает агрегированную статистику для стратегии
    pub async fn get_strategy_stats(
        &self,
        strategy_id: &str,
    ) -> Result<super::types::StrategyStats> {
        let conn = self.conn.clone();
        let strategy_id_owned = strategy_id.to_string();
        let strategy_id_for_result = strategy_id.to_string();

        let stats = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                r#"
                SELECT 
                    COUNT(*) as total_tests,
                    SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successes,
                    SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) as failures,
                    AVG(CASE WHEN latency_ms IS NOT NULL THEN latency_ms END) as avg_latency,
                    MAX(timestamp) as last_test_at
                FROM strategy_history
                WHERE strategy_id = ?1
                "#,
                params![strategy_id_owned],
                |row| {
                    let total_tests: i32 = row.get(0)?;
                    let successes: i32 = row.get(1).unwrap_or(0);
                    let failures: i32 = row.get(2).unwrap_or(0);
                    let avg_latency: Option<f64> = row.get(3)?;
                    let last_test_at: Option<i64> = row.get(4)?;
                    
                    Ok((total_tests, successes, failures, avg_latency, last_test_at))
                },
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        let (total_tests, successes, failures, avg_latency, last_test_at) = stats;
        let success_rate = if total_tests > 0 {
            (successes as f64) / (total_tests as f64)
        } else {
            0.0
        };

        Ok(super::types::StrategyStats {
            strategy_id: strategy_id_for_result,
            total_tests: total_tests as u32,
            successes: successes as u32,
            failures: failures as u32,
            success_rate,
            avg_latency_ms: avg_latency,
            last_test_at,
        })
    }

    /// Получает статистику для всех стратегий
    pub async fn get_all_strategy_stats(&self) -> Result<Vec<super::types::StrategyStats>> {
        let conn = self.conn.clone();

        let stats = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT 
                    strategy_id,
                    COUNT(*) as total_tests,
                    SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successes,
                    SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) as failures,
                    AVG(CASE WHEN latency_ms IS NOT NULL THEN latency_ms END) as avg_latency,
                    MAX(timestamp) as last_test_at
                FROM strategy_history
                GROUP BY strategy_id
                ORDER BY total_tests DESC
                "#,
            )?;

            let rows = stmt.query_map([], |row| {
                let strategy_id: String = row.get(0)?;
                let total_tests: i32 = row.get(1)?;
                let successes: i32 = row.get(2).unwrap_or(0);
                let failures: i32 = row.get(3).unwrap_or(0);
                let avg_latency: Option<f64> = row.get(4)?;
                let last_test_at: Option<i64> = row.get(5)?;
                
                let success_rate = if total_tests > 0 {
                    (successes as f64) / (total_tests as f64)
                } else {
                    0.0
                };

                Ok(super::types::StrategyStats {
                    strategy_id,
                    total_tests: total_tests as u32,
                    successes: successes as u32,
                    failures: failures as u32,
                    success_rate,
                    avg_latency_ms: avg_latency,
                    last_test_at,
                })
            })?;

            let mut stats = Vec::new();
            for row in rows {
                stats.push(row?);
            }
            Ok::<_, rusqlite::Error>(stats)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(stats)
    }

    /// Очищает историю для конкретной стратегии
    pub async fn clear_strategy_history(&self, strategy_id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let strategy_id_owned = strategy_id.to_string();
        let strategy_id_log = strategy_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM strategy_history WHERE strategy_id = ?1",
                params![strategy_id_owned],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!(strategy_id = %strategy_id_log, "Strategy history cleared");
        Ok(())
    }

    /// Очищает всю историю стратегий
    pub async fn clear_all_strategy_history(&self) -> Result<()> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM strategy_history", [])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!("All strategy history cleared");
        Ok(())
    }
}
