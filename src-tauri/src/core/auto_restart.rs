//! Auto-restart mechanism for failed strategies
//!
//! Monitors strategy health and automatically restarts on failure.

#![allow(dead_code)] // Public auto-restart API

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{info, warn, error};

use crate::core::strategy_engine::SharedStrategyEngine;
use crate::core::models::Strategy;

/// Configuration for auto-restart
#[derive(Debug, Clone)]
pub struct AutoRestartConfig {
    /// Maximum restart attempts before giving up
    pub max_retries: u32,
    /// Delay between restart attempts
    pub retry_delay: Duration,
    /// Cooldown after max retries reached
    pub cooldown: Duration,
    /// Whether auto-restart is enabled
    pub enabled: bool,
}

impl Default for AutoRestartConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(5),
            cooldown: Duration::from_secs(60),
            enabled: true,
        }
    }
}

/// Auto-restart manager
pub struct AutoRestartManager {
    engine: SharedStrategyEngine,
    config: RwLock<AutoRestartConfig>,
    restart_count: AtomicU32,
    running: AtomicBool,
    last_strategy: RwLock<Option<Strategy>>,
}

impl AutoRestartManager {
    pub fn new(engine: SharedStrategyEngine) -> Self {
        Self {
            engine,
            config: RwLock::new(AutoRestartConfig::default()),
            restart_count: AtomicU32::new(0),
            running: AtomicBool::new(false),
            last_strategy: RwLock::new(None),
        }
    }
    
    /// Set the strategy to auto-restart
    pub async fn set_strategy(&self, strategy: Strategy) {
        let mut last = self.last_strategy.write().await;
        *last = Some(strategy);
        self.restart_count.store(0, Ordering::SeqCst);
    }
    
    /// Clear the auto-restart strategy
    pub async fn clear_strategy(&self) {
        let mut last = self.last_strategy.write().await;
        *last = None;
    }
    
    /// Check if auto-restart is enabled
    pub async fn is_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.enabled
    }
    
    /// Check if manager is currently running restart loop
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Attempt to restart the last strategy
    pub async fn try_restart(&self) -> Result<bool, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Ok(false);
        }
        
        let strategy = {
            let last = self.last_strategy.read().await;
            match last.clone() {
                Some(s) => s,
                None => return Ok(false),
            }
        };
        
        let count = self.restart_count.fetch_add(1, Ordering::SeqCst);
        
        if count >= config.max_retries {
            warn!(
                strategy_id = %strategy.id,
                attempts = count,
                "Max restart attempts reached, entering cooldown"
            );
            
            // Reset after cooldown
            let cooldown = config.cooldown;
            drop(config);
            
            let restart_count = Arc::new(AtomicU32::new(0));
            let restart_count_clone = restart_count.clone();
            
            tokio::spawn(async move {
                sleep(cooldown).await;
                restart_count_clone.store(0, Ordering::SeqCst);
            });
            
            return Ok(false);
        }
        
        info!(
            strategy_id = %strategy.id,
            attempt = count + 1,
            max = config.max_retries,
            "Auto-restarting strategy"
        );
        
        let retry_delay = config.retry_delay;
        drop(config);
        
        sleep(retry_delay).await;
        
        match self.engine.start_global(&strategy).await {
            Ok(()) => {
                info!(strategy_id = %strategy.id, "Strategy restarted successfully");
                Ok(true)
            }
            Err(e) => {
                error!(strategy_id = %strategy.id, error = %e, "Failed to restart strategy");
                Err(e.to_string())
            }
        }
    }
    
    /// Update configuration
    pub async fn set_config(&self, config: AutoRestartConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> AutoRestartConfig {
        let cfg = self.config.read().await;
        cfg.clone()
    }
    
    /// Get current restart count
    pub fn restart_count(&self) -> u32 {
        self.restart_count.load(Ordering::SeqCst)
    }
    
    /// Reset restart counter
    pub fn reset_count(&self) {
        self.restart_count.store(0, Ordering::SeqCst);
    }
    
    /// Get the last strategy set for auto-restart
    pub async fn get_last_strategy(&self) -> Option<Strategy> {
        let last = self.last_strategy.read().await;
        last.clone()
    }
}

pub type SharedAutoRestartManager = Arc<AutoRestartManager>;

/// Creates a shared auto-restart manager
pub fn create_auto_restart_manager(engine: SharedStrategyEngine) -> SharedAutoRestartManager {
    Arc::new(AutoRestartManager::new(engine))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::strategy_engine::create_engine;
    
    #[test]
    fn test_default_config() {
        let config = AutoRestartConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.enabled);
        assert_eq!(config.retry_delay, Duration::from_secs(5));
        assert_eq!(config.cooldown, Duration::from_secs(60));
    }
    
    #[test]
    fn test_restart_count() {
        let engine = create_engine();
        let manager = AutoRestartManager::new(engine);
        
        assert_eq!(manager.restart_count(), 0);
        manager.restart_count.fetch_add(1, Ordering::SeqCst);
        assert_eq!(manager.restart_count(), 1);
        manager.reset_count();
        assert_eq!(manager.restart_count(), 0);
    }
    
    #[tokio::test]
    async fn test_set_and_clear_strategy() {
        let engine = create_engine();
        let manager = AutoRestartManager::new(engine);
        
        // Initially no strategy
        assert!(manager.get_last_strategy().await.is_none());
        
        // Clear on empty should work
        manager.clear_strategy().await;
        assert!(manager.get_last_strategy().await.is_none());
    }
    
    #[tokio::test]
    async fn test_config_update() {
        let engine = create_engine();
        let manager = AutoRestartManager::new(engine);
        
        let new_config = AutoRestartConfig {
            max_retries: 5,
            retry_delay: Duration::from_secs(10),
            cooldown: Duration::from_secs(120),
            enabled: false,
        };
        
        manager.set_config(new_config.clone()).await;
        
        let config = manager.get_config().await;
        assert_eq!(config.max_retries, 5);
        assert!(!config.enabled);
    }
    
    #[tokio::test]
    async fn test_try_restart_disabled() {
        let engine = create_engine();
        let manager = AutoRestartManager::new(engine);
        
        // Disable auto-restart
        let config = AutoRestartConfig {
            enabled: false,
            ..Default::default()
        };
        manager.set_config(config).await;
        
        // Should return Ok(false) when disabled
        let result = manager.try_restart().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
    
    #[tokio::test]
    async fn test_try_restart_no_strategy() {
        let engine = create_engine();
        let manager = AutoRestartManager::new(engine);
        
        // No strategy set, should return Ok(false)
        let result = manager.try_restart().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
