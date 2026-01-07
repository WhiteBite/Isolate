//! Database schema and migrations

use rusqlite::Connection;

/// Initialize database schema
pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Настройки приложения
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Кэш стратегий по окружению
        CREATE TABLE IF NOT EXISTS strategy_cache (
            env_key TEXT PRIMARY KEY,
            strategy_id TEXT NOT NULL,
            score REAL NOT NULL,
            timestamp INTEGER NOT NULL
        );

        -- История тестов
        CREATE TABLE IF NOT EXISTS test_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            env_key TEXT NOT NULL,
            strategy_id TEXT NOT NULL,
            success INTEGER NOT NULL,
            score REAL NOT NULL,
            latency_ms REAL NOT NULL,
            timestamp INTEGER NOT NULL
        );

        -- Domain routing rules
        CREATE TABLE IF NOT EXISTS domain_routes (
            domain TEXT PRIMARY KEY,
            proxy_id TEXT NOT NULL
        );

        -- App routing rules
        CREATE TABLE IF NOT EXISTS app_routes (
            app_path TEXT PRIMARY KEY,
            app_name TEXT NOT NULL,
            proxy_id TEXT NOT NULL
        );

        -- Subscriptions table (proxy subscription URLs)
        CREATE TABLE IF NOT EXISTS subscriptions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            update_interval INTEGER NOT NULL DEFAULT 86400,
            last_updated TEXT,
            last_error TEXT,
            proxy_count INTEGER NOT NULL DEFAULT 0,
            auto_update INTEGER NOT NULL DEFAULT 1,
            user_agent TEXT,
            format TEXT NOT NULL DEFAULT 'auto',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Proxies table
        CREATE TABLE IF NOT EXISTS proxies (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            protocol TEXT NOT NULL,
            server TEXT NOT NULL,
            port INTEGER NOT NULL,
            username TEXT,
            password TEXT,
            uuid TEXT,
            tls INTEGER NOT NULL DEFAULT 0,
            sni TEXT,
            transport TEXT,
            custom_fields TEXT NOT NULL DEFAULT '{}',
            active INTEGER NOT NULL DEFAULT 0,
            subscription_id TEXT REFERENCES subscriptions(id) ON DELETE SET NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Learned strategies (Orchestra)
        CREATE TABLE IF NOT EXISTS learned_strategies (
            domain TEXT PRIMARY KEY,
            strategy_id TEXT NOT NULL,
            successes INTEGER DEFAULT 0,
            failures INTEGER DEFAULT 0,
            locked_at TEXT,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Routing rules (high-level abstraction)
        CREATE TABLE IF NOT EXISTS routing_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            source TEXT NOT NULL,
            source_value TEXT,
            action TEXT NOT NULL,
            proxy_id TEXT,
            priority INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Strategy history (success/failure tracking per strategy)
        CREATE TABLE IF NOT EXISTS strategy_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id TEXT NOT NULL,
            service_id TEXT NOT NULL,
            success INTEGER NOT NULL,
            latency_ms REAL,
            timestamp INTEGER NOT NULL DEFAULT (unixepoch())
        );

        -- Индексы
        CREATE INDEX IF NOT EXISTS idx_test_history_strategy
            ON test_history(strategy_id, timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_strategy_cache_timestamp
            ON strategy_cache(timestamp);
        CREATE INDEX IF NOT EXISTS idx_domain_routes_proxy
            ON domain_routes(proxy_id);
        CREATE INDEX IF NOT EXISTS idx_app_routes_proxy
            ON app_routes(proxy_id);
        CREATE INDEX IF NOT EXISTS idx_proxies_active
            ON proxies(active);
        CREATE INDEX IF NOT EXISTS idx_proxies_subscription
            ON proxies(subscription_id);
        CREATE INDEX IF NOT EXISTS idx_subscriptions_auto_update
            ON subscriptions(auto_update, update_interval);
        CREATE INDEX IF NOT EXISTS idx_learned_strategies_strategy
            ON learned_strategies(strategy_id);
        CREATE INDEX IF NOT EXISTS idx_routing_rules_priority
            ON routing_rules(priority);
        CREATE INDEX IF NOT EXISTS idx_routing_rules_enabled
            ON routing_rules(enabled);
        CREATE INDEX IF NOT EXISTS idx_strategy_history_strategy
            ON strategy_history(strategy_id, timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_strategy_history_service
            ON strategy_history(service_id, timestamp DESC);

        -- Blocked strategies (default + user blacklist)
        CREATE TABLE IF NOT EXISTS blocked_strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL,
            strategy_id TEXT NOT NULL,
            is_user INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(domain, strategy_id)
        );

        -- Locked strategies (per protocol)
        CREATE TABLE IF NOT EXISTS locked_strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL,
            strategy_id TEXT NOT NULL,
            protocol TEXT NOT NULL DEFAULT 'tls',
            locked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(domain, protocol)
        );

        -- Индексы для blocked/locked
        CREATE INDEX IF NOT EXISTS idx_blocked_strategies_domain
            ON blocked_strategies(domain);
        CREATE INDEX IF NOT EXISTS idx_blocked_strategies_strategy
            ON blocked_strategies(strategy_id);
        CREATE INDEX IF NOT EXISTS idx_locked_strategies_domain
            ON locked_strategies(domain);
        CREATE INDEX IF NOT EXISTS idx_locked_strategies_protocol
            ON locked_strategies(protocol);

        -- Strategy history v2 (domain-based success/failure tracking for managers)
        CREATE TABLE IF NOT EXISTS strategy_history_v2 (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT NOT NULL,
            strategy_id TEXT NOT NULL,
            successes INTEGER NOT NULL DEFAULT 0,
            failures INTEGER NOT NULL DEFAULT 0,
            last_success TEXT,
            last_failure TEXT,
            UNIQUE(domain, strategy_id)
        );

        CREATE INDEX IF NOT EXISTS idx_strategy_history_v2_domain
            ON strategy_history_v2(domain);
        CREATE INDEX IF NOT EXISTS idx_strategy_history_v2_strategy
            ON strategy_history_v2(strategy_id);

        -- Service health history (tracking service availability over time)
        CREATE TABLE IF NOT EXISTS service_health_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            service_id TEXT NOT NULL,
            timestamp INTEGER NOT NULL DEFAULT (unixepoch()),
            accessible INTEGER NOT NULL,
            latency_ms INTEGER,
            error TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_service_health_history_service
            ON service_health_history(service_id, timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_service_health_history_timestamp
            ON service_health_history(timestamp);
        "#,
    )
}
