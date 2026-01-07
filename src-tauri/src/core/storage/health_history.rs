//! Service Health History - хранение истории проверок сервисов
//!
//! Записывает результаты проверок сервисов и предоставляет статистику.

use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::core::errors::{IsolateError, Result};

/// Максимальное время хранения истории (7 дней в секундах)
const MAX_HISTORY_AGE_SECS: i64 = 7 * 24 * 60 * 60;

/// Запись истории проверки сервиса
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceHealthRecord {
    pub id: i64,
    pub service_id: String,
    pub timestamp: i64,
    pub accessible: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// Статистика здоровья сервиса
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceHealthStats {
    pub service_id: String,
    pub total_checks: u32,
    pub successful_checks: u32,
    pub failed_checks: u32,
    pub uptime_percent: f64,
    pub avg_latency_ms: Option<f64>,
    pub min_latency_ms: Option<u64>,
    pub max_latency_ms: Option<u64>,
    pub last_check_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub last_failure_at: Option<i64>,
}

/// Расширение Storage для работы с историей здоровья сервисов
impl super::Storage {
    /// Записывает результат проверки сервиса
    pub async fn record_service_check(
        &self,
        service_id: &str,
        accessible: bool,
        latency_ms: Option<u64>,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let service_id_owned = service_id.to_string();
        let error = error.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT INTO service_health_history (service_id, timestamp, accessible, latency_ms, error)
                VALUES (?1, unixepoch(), ?2, ?3, ?4)
                "#,
                params![service_id_owned, accessible, latency_ms, error],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(
            service_id = %service_id,
            accessible = accessible,
            latency_ms = ?latency_ms,
            "Recorded service health check"
        );

        Ok(())
    }

    /// Получает историю проверок сервиса за указанное количество часов
    pub async fn get_service_history(
        &self,
        service_id: &str,
        hours: u32,
    ) -> Result<Vec<ServiceHealthRecord>> {
        let conn = self.conn.clone();
        let service_id_owned = service_id.to_string();
        let since_timestamp = chrono::Utc::now().timestamp() - (hours as i64 * 3600);

        let records = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, service_id, timestamp, accessible, latency_ms, error
                FROM service_health_history
                WHERE service_id = ?1 AND timestamp >= ?2
                ORDER BY timestamp ASC
                "#,
            )?;

            let rows = stmt.query_map(params![service_id_owned, since_timestamp], |row| {
                Ok(ServiceHealthRecord {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    timestamp: row.get(2)?,
                    accessible: row.get(3)?,
                    latency_ms: row.get(4)?,
                    error: row.get(5)?,
                })
            })?;

            let mut records = Vec::new();
            for row in rows {
                records.push(row?);
            }
            Ok::<_, rusqlite::Error>(records)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(
            service_id = %service_id,
            hours = hours,
            count = records.len(),
            "Retrieved service health history"
        );

        Ok(records)
    }

    /// Получает статистику здоровья сервиса за указанное количество часов
    pub async fn get_service_health_stats(
        &self,
        service_id: &str,
        hours: u32,
    ) -> Result<ServiceHealthStats> {
        let conn = self.conn.clone();
        let service_id_clone = service_id.to_string();
        let since_timestamp = chrono::Utc::now().timestamp() - (hours as i64 * 3600);

        let stats = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();

            // Получаем агрегированную статистику
            let (total, successful, avg_latency, min_latency, max_latency): (
                u32,
                u32,
                Option<f64>,
                Option<u64>,
                Option<u64>,
            ) = conn.query_row(
                r#"
                SELECT 
                    COUNT(*) as total,
                    SUM(CASE WHEN accessible = 1 THEN 1 ELSE 0 END) as successful,
                    AVG(CASE WHEN accessible = 1 THEN latency_ms ELSE NULL END) as avg_latency,
                    MIN(CASE WHEN accessible = 1 THEN latency_ms ELSE NULL END) as min_latency,
                    MAX(CASE WHEN accessible = 1 THEN latency_ms ELSE NULL END) as max_latency
                FROM service_health_history
                WHERE service_id = ?1 AND timestamp >= ?2
                "#,
                params![service_id_clone, since_timestamp],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )?;

            // Получаем временные метки последних событий
            let last_check_at: Option<i64> = conn
                .query_row(
                    r#"
                    SELECT MAX(timestamp) FROM service_health_history
                    WHERE service_id = ?1 AND timestamp >= ?2
                    "#,
                    params![service_id_clone, since_timestamp],
                    |row| row.get(0),
                )
                .ok();

            let last_success_at: Option<i64> = conn
                .query_row(
                    r#"
                    SELECT MAX(timestamp) FROM service_health_history
                    WHERE service_id = ?1 AND timestamp >= ?2 AND accessible = 1
                    "#,
                    params![service_id_clone, since_timestamp],
                    |row| row.get(0),
                )
                .ok();

            let last_failure_at: Option<i64> = conn
                .query_row(
                    r#"
                    SELECT MAX(timestamp) FROM service_health_history
                    WHERE service_id = ?1 AND timestamp >= ?2 AND accessible = 0
                    "#,
                    params![service_id_clone, since_timestamp],
                    |row| row.get(0),
                )
                .ok();

            let uptime_percent = if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            Ok::<_, rusqlite::Error>(ServiceHealthStats {
                service_id: service_id_clone,
                total_checks: total,
                successful_checks: successful,
                failed_checks: total - successful,
                uptime_percent,
                avg_latency_ms: avg_latency,
                min_latency_ms: min_latency,
                max_latency_ms: max_latency,
                last_check_at,
                last_success_at,
                last_failure_at,
            })
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(
            service_id = %service_id,
            hours = hours,
            uptime = stats.uptime_percent,
            "Retrieved service health stats"
        );

        Ok(stats)
    }

    /// Удаляет старые записи истории (старше 7 дней)
    pub async fn cleanup_old_health_history(&self) -> Result<u64> {
        let conn = self.conn.clone();
        let cutoff_timestamp = chrono::Utc::now().timestamp() - MAX_HISTORY_AGE_SECS;

        let deleted = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM service_health_history WHERE timestamp < ?1",
                params![cutoff_timestamp],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if deleted > 0 {
            info!(deleted = deleted, "Cleaned up old health history records");
        }

        Ok(deleted as u64)
    }

    /// Получает историю для всех сервисов за указанное количество часов
    pub async fn get_all_services_history(
        &self,
        hours: u32,
    ) -> Result<std::collections::HashMap<String, Vec<ServiceHealthRecord>>> {
        let conn = self.conn.clone();
        let since_timestamp = chrono::Utc::now().timestamp() - (hours as i64 * 3600);

        let records = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, service_id, timestamp, accessible, latency_ms, error
                FROM service_health_history
                WHERE timestamp >= ?1
                ORDER BY service_id, timestamp ASC
                "#,
            )?;

            let rows = stmt.query_map(params![since_timestamp], |row| {
                Ok(ServiceHealthRecord {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    timestamp: row.get(2)?,
                    accessible: row.get(3)?,
                    latency_ms: row.get(4)?,
                    error: row.get(5)?,
                })
            })?;

            let mut map: std::collections::HashMap<String, Vec<ServiceHealthRecord>> =
                std::collections::HashMap::new();
            for row in rows {
                let record = row?;
                map.entry(record.service_id.clone())
                    .or_default()
                    .push(record);
            }
            Ok::<_, rusqlite::Error>(map)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_storage() -> super::super::Storage {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        super::super::Storage::open(&path).await.unwrap()
    }

    #[tokio::test]
    async fn test_record_and_get_history() {
        let storage = create_test_storage().await;

        // Record some checks
        storage
            .record_service_check("youtube", true, Some(150), None)
            .await
            .unwrap();
        storage
            .record_service_check("youtube", true, Some(120), None)
            .await
            .unwrap();
        storage
            .record_service_check("youtube", false, None, Some("Connection refused"))
            .await
            .unwrap();

        // Get history
        let history = storage.get_service_history("youtube", 1).await.unwrap();
        assert_eq!(history.len(), 3);
        assert!(history[0].accessible);
        assert!(history[1].accessible);
        assert!(!history[2].accessible);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let storage = create_test_storage().await;

        // Record checks
        storage
            .record_service_check("discord", true, Some(100), None)
            .await
            .unwrap();
        storage
            .record_service_check("discord", true, Some(200), None)
            .await
            .unwrap();
        storage
            .record_service_check("discord", false, None, Some("Timeout"))
            .await
            .unwrap();

        // Get stats
        let stats = storage.get_service_health_stats("discord", 1).await.unwrap();
        assert_eq!(stats.total_checks, 3);
        assert_eq!(stats.successful_checks, 2);
        assert_eq!(stats.failed_checks, 1);
        assert!((stats.uptime_percent - 66.67).abs() < 1.0);
        assert_eq!(stats.avg_latency_ms, Some(150.0));
        assert_eq!(stats.min_latency_ms, Some(100));
        assert_eq!(stats.max_latency_ms, Some(200));
    }
}
