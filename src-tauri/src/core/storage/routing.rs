//! Routing rules storage operations

use rusqlite::params;
use tracing::debug;

use crate::core::errors::{IsolateError, Result};
use super::types::RoutingRule;
use super::database::Storage;

impl Storage {
    /// Get all routing rules ordered by priority
    pub async fn get_routing_rules(&self) -> Result<Vec<RoutingRule>> {
        let conn = self.conn.clone();

        let rules = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                r#"
                SELECT id, name, enabled, source, source_value, action, proxy_id, priority
                FROM routing_rules
                ORDER BY priority ASC
                "#,
            )?;

            let rows = stmt.query_map([], |row| {
                let enabled_int: i32 = row.get(2)?;
                Ok(RoutingRule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    enabled: enabled_int != 0,
                    source: row.get(3)?,
                    source_value: row.get(4)?,
                    action: row.get(5)?,
                    proxy_id: row.get(6)?,
                    priority: row.get(7)?,
                })
            })?;

            let mut rules = Vec::new();
            for row in rows {
                rules.push(row?);
            }
            Ok::<_, rusqlite::Error>(rules)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(count = rules.len(), "Loaded routing rules");
        Ok(rules)
    }

    /// Add a new routing rule
    pub async fn add_routing_rule(&self, rule: &RoutingRule) -> Result<()> {
        let conn = self.conn.clone();
        let rule_id = rule.id.clone();
        let rule_name = rule.name.clone();
        let rule_enabled = rule.enabled;
        let rule_source = rule.source.clone();
        let rule_source_value = rule.source_value.clone();
        let rule_action = rule.action.clone();
        let rule_proxy_id = rule.proxy_id.clone();
        let rule_priority = rule.priority;

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT INTO routing_rules (id, name, enabled, source, source_value, action, proxy_id, priority, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))
                "#,
                params![
                    rule_id,
                    rule_name,
                    rule_enabled as i32,
                    rule_source,
                    rule_source_value,
                    rule_action,
                    rule_proxy_id,
                    rule_priority,
                ],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(id = %rule.id, name = %rule.name, "Routing rule added");
        Ok(())
    }

    /// Update an existing routing rule
    pub async fn update_routing_rule(&self, rule: &RoutingRule) -> Result<()> {
        let conn = self.conn.clone();
        let rule_id = rule.id.clone();
        let rule_name = rule.name.clone();
        let rule_enabled = rule.enabled;
        let rule_source = rule.source.clone();
        let rule_source_value = rule.source_value.clone();
        let rule_action = rule.action.clone();
        let rule_proxy_id = rule.proxy_id.clone();
        let rule_priority = rule.priority;
        let rule_id_for_err = rule.id.clone();

        let updated = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                UPDATE routing_rules
                SET name = ?2, enabled = ?3, source = ?4, source_value = ?5, 
                    action = ?6, proxy_id = ?7, priority = ?8, updated_at = datetime('now')
                WHERE id = ?1
                "#,
                params![
                    rule_id,
                    rule_name,
                    rule_enabled as i32,
                    rule_source,
                    rule_source_value,
                    rule_action,
                    rule_proxy_id,
                    rule_priority,
                ],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if updated == 0 {
            return Err(IsolateError::Storage(format!("Routing rule '{}' not found", rule_id_for_err)));
        }

        debug!(id = %rule.id, name = %rule.name, "Routing rule updated");
        Ok(())
    }

    /// Delete a routing rule
    pub async fn delete_routing_rule(&self, id: &str) -> Result<()> {
        let conn = self.conn.clone();
        let id_str = id.to_string();
        let id_for_err = id.to_string();

        let deleted = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM routing_rules WHERE id = ?1",
                params![id_str],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if deleted == 0 {
            return Err(IsolateError::Storage(format!("Routing rule '{}' not found", id_for_err)));
        }

        debug!(id = %id, "Routing rule deleted");
        Ok(())
    }

    /// Reorder routing rules by updating priorities
    pub async fn reorder_routing_rules(&self, rule_ids: &[String]) -> Result<()> {
        let conn = self.conn.clone();
        let rule_ids = rule_ids.to_vec();
        let count = rule_ids.len();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            for (index, id) in rule_ids.iter().enumerate() {
                conn.execute(
                    "UPDATE routing_rules SET priority = ?2, updated_at = datetime('now') WHERE id = ?1",
                    params![id, index as i32],
                )?;
            }
            Ok::<_, rusqlite::Error>(())
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(count = count, "Routing rules reordered");
        Ok(())
    }

    /// Toggle routing rule enabled state
    pub async fn toggle_routing_rule(&self, id: &str, enabled: bool) -> Result<()> {
        let conn = self.conn.clone();
        let id_str = id.to_string();
        let id_for_err = id.to_string();

        let updated = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "UPDATE routing_rules SET enabled = ?2, updated_at = datetime('now') WHERE id = ?1",
                params![id_str, enabled as i32],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if updated == 0 {
            return Err(IsolateError::Storage(format!("Routing rule '{}' not found", id_for_err)));
        }

        debug!(id = %id, enabled, "Routing rule toggled");
        Ok(())
    }
}
