# Parallel Proxy Testing Architecture

## Обзор

Архитектура параллельного тестирования прокси с визуализацией прогресса в реальном времени.

## 1. Concurrency Model

### Tokio Semaphore для ограничения параллелизма

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

/// Конфигурация параллельного тестирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelTestConfig {
    /// Максимальное количество одновременных тестов (default: 5)
    pub max_concurrent: usize,
    /// Таймаут на один тест в секундах (default: 10)
    pub timeout_secs: u64,
    /// Включить тест скорости загрузки
    pub test_download: bool,
    /// Включить тест скорости выгрузки
    pub test_upload: bool,
}

impl Default for ParallelTestConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_secs: 10,
            test_download: true,
            test_upload: false, // Опционально, занимает больше времени
        }
    }
}
```


### Реализация ProxyTester с Semaphore

```rust
// src-tauri/src/core/proxy_tester.rs

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Результат тестирования одного прокси
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTestResult {
    pub proxy_id: String,
    pub proxy_name: String,
    /// Статус теста
    pub status: TestStatus,
    /// Latency в миллисекундах (TCP handshake + TLS)
    pub latency_ms: Option<u32>,
    /// Скорость загрузки в Mbps
    pub download_speed_mbps: Option<f64>,
    /// Скорость выгрузки в Mbps  
    pub upload_speed_mbps: Option<f64>,
    /// Ошибка (если есть)
    pub error: Option<String>,
    /// Время начала теста
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Время завершения теста
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Pending,
    Testing,
    Success,
    Failed,
    Timeout,
    Cancelled,
}

/// Событие прогресса для Tauri events
#[derive(Debug, Clone, Serialize)]
pub struct ProxyTestProgress {
    pub proxy_id: String,
    pub proxy_name: String,
    pub status: TestStatus,
    pub current_phase: TestPhase,
    pub tested_count: usize,
    pub total_count: usize,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestPhase {
    Connecting,
    MeasuringLatency,
    TestingDownload,
    TestingUpload,
    Completed,
}
```


```rust
/// Параллельный тестер прокси
pub struct ParallelProxyTester {
    /// Семафор для ограничения параллелизма
    semaphore: Arc<Semaphore>,
    /// Конфигурация
    config: ParallelTestConfig,
    /// HTTP клиент
    client: reqwest::Client,
    /// Токен отмены
    cancel_token: CancellationToken,
}

impl ParallelProxyTester {
    pub fn new(config: ParallelTestConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();
        
        Self {
            semaphore,
            config,
            client,
            cancel_token: CancellationToken::new(),
        }
    }

    /// Тестирует все прокси параллельно с ограничением concurrency
    pub async fn test_all(
        &self,
        proxies: Vec<ProxyConfig>,
        progress_tx: mpsc::Sender<ProxyTestProgress>,
    ) -> Vec<ProxyTestResult> {
        let total = proxies.len();
        let tested = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        
        // Создаём futures для всех прокси
        let futures: Vec<_> = proxies
            .into_iter()
            .map(|proxy| {
                let semaphore = self.semaphore.clone();
                let client = self.client.clone();
                let config = self.config.clone();
                let cancel_token = self.cancel_token.clone();
                let progress_tx = progress_tx.clone();
                let tested = tested.clone();
                
                async move {
                    // Ждём разрешения от семафора
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Проверяем отмену
                    if cancel_token.is_cancelled() {
                        return ProxyTestResult {
                            proxy_id: proxy.id.clone(),
                            proxy_name: proxy.name.clone(),
                            status: TestStatus::Cancelled,
                            latency_ms: None,
                            download_speed_mbps: None,
                            upload_speed_mbps: None,
                            error: Some("Cancelled".to_string()),
                            started_at: chrono::Utc::now(),
                            completed_at: chrono::Utc::now(),
                        };
                    }
                    
                    // Отправляем прогресс: начало теста
                    let current = tested.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _ = progress_tx.send(ProxyTestProgress {
                        proxy_id: proxy.id.clone(),
                        proxy_name: proxy.name.clone(),
                        status: TestStatus::Testing,
                        current_phase: TestPhase::Connecting,
                        tested_count: current + 1,
                        total_count: total,
                        percent: (((current + 1) * 100) / total) as u8,
                    }).await;
                    
                    // Выполняем тест
                    let result = test_single_proxy(&client, &proxy, &config).await;
                    
                    // Отправляем финальный прогресс
                    let _ = progress_tx.send(ProxyTestProgress {
                        proxy_id: proxy.id.clone(),
                        proxy_name: proxy.name.clone(),
                        status: result.status.clone(),
                        current_phase: TestPhase::Completed,
                        tested_count: current + 1,
                        total_count: total,
                        percent: (((current + 1) * 100) / total) as u8,
                    }).await;
                    
                    result
                }
            })
            .collect();
        
        // Запускаем все futures параллельно
        futures::future::join_all(futures).await
    }

    /// Отменяет текущее тестирование
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}
```


```rust
/// Тестирует один прокси
async fn test_single_proxy(
    client: &reqwest::Client,
    proxy: &ProxyConfig,
    config: &ParallelTestConfig,
) -> ProxyTestResult {
    let started_at = chrono::Utc::now();
    
    // 1. Создаём прокси-клиент
    let proxy_client = match create_proxy_client(proxy, config.timeout_secs) {
        Ok(c) => c,
        Err(e) => {
            return ProxyTestResult {
                proxy_id: proxy.id.clone(),
                proxy_name: proxy.name.clone(),
                status: TestStatus::Failed,
                latency_ms: None,
                download_speed_mbps: None,
                upload_speed_mbps: None,
                error: Some(e.to_string()),
                started_at,
                completed_at: chrono::Utc::now(),
            };
        }
    };
    
    // 2. Измеряем latency (TCP + TLS handshake)
    let latency_start = Instant::now();
    let latency_result = proxy_client
        .head("https://www.google.com/generate_204")
        .send()
        .await;
    
    let latency_ms = match latency_result {
        Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 204 => {
            Some(latency_start.elapsed().as_millis() as u32)
        }
        Ok(resp) => {
            return ProxyTestResult {
                proxy_id: proxy.id.clone(),
                proxy_name: proxy.name.clone(),
                status: TestStatus::Failed,
                latency_ms: None,
                download_speed_mbps: None,
                upload_speed_mbps: None,
                error: Some(format!("Unexpected status: {}", resp.status())),
                started_at,
                completed_at: chrono::Utc::now(),
            };
        }
        Err(e) => {
            let status = if e.is_timeout() {
                TestStatus::Timeout
            } else {
                TestStatus::Failed
            };
            return ProxyTestResult {
                proxy_id: proxy.id.clone(),
                proxy_name: proxy.name.clone(),
                status,
                latency_ms: None,
                download_speed_mbps: None,
                upload_speed_mbps: None,
                error: Some(e.to_string()),
                started_at,
                completed_at: chrono::Utc::now(),
            };
        }
    };
    
    // 3. Тест скорости загрузки (опционально)
    let download_speed = if config.test_download {
        measure_download_speed(&proxy_client).await.ok()
    } else {
        None
    };
    
    // 4. Тест скорости выгрузки (опционально)
    let upload_speed = if config.test_upload {
        measure_upload_speed(&proxy_client).await.ok()
    } else {
        None
    };
    
    ProxyTestResult {
        proxy_id: proxy.id.clone(),
        proxy_name: proxy.name.clone(),
        status: TestStatus::Success,
        latency_ms,
        download_speed_mbps: download_speed,
        upload_speed_mbps: upload_speed,
        error: None,
        started_at,
        completed_at: chrono::Utc::now(),
    }
}
```


## 2. Tauri Events для Progress Updates

### Backend: Emit Events

```rust
// src-tauri/src/commands/proxy_testing.rs

use tauri::{AppHandle, Emitter, State, Window};
use tokio::sync::mpsc;

/// Запускает параллельное тестирование прокси
#[tauri::command]
pub async fn test_proxies_parallel(
    window: Window,
    state: State<'_, Arc<AppState>>,
    proxy_ids: Vec<String>,
    config: Option<ParallelTestConfig>,
) -> Result<Vec<ProxyTestResult>, IsolateError> {
    let config = config.unwrap_or_default();
    
    info!(
        proxy_count = proxy_ids.len(),
        max_concurrent = config.max_concurrent,
        "Starting parallel proxy test"
    );
    
    // Загружаем прокси из storage
    let mut proxies = Vec::new();
    for id in &proxy_ids {
        if let Ok(Some(proxy)) = state.storage.get_proxy(id).await {
            proxies.push(proxy);
        }
    }
    
    if proxies.is_empty() {
        return Err(IsolateError::Validation("No valid proxies to test".into()));
    }
    
    // Создаём канал для прогресса
    let (progress_tx, mut progress_rx) = mpsc::channel::<ProxyTestProgress>(100);
    
    // Спавним задачу для отправки событий в UI
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            // Emit progress event
            let _ = window_clone.emit("proxy-test:progress", &progress);
            
            // Emit individual result when completed
            if progress.current_phase == TestPhase::Completed {
                let _ = window_clone.emit("proxy-test:result", &progress);
            }
        }
    });
    
    // Создаём тестер и запускаем
    let tester = ParallelProxyTester::new(config);
    
    // Сохраняем тестер в state для возможности отмены
    {
        let mut current_tester = state.proxy_tester.write().await;
        *current_tester = Some(tester.clone());
    }
    
    let results = tester.test_all(proxies, progress_tx).await;
    
    // Очищаем тестер
    {
        let mut current_tester = state.proxy_tester.write().await;
        *current_tester = None;
    }
    
    // Emit completion event
    let _ = window.emit("proxy-test:complete", &results);
    
    // Сортируем по latency (лучшие первые)
    let mut sorted_results = results;
    sorted_results.sort_by(|a, b| {
        match (&a.latency_ms, &b.latency_ms) {
            (Some(a_lat), Some(b_lat)) => a_lat.cmp(b_lat),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    
    info!(
        results_count = sorted_results.len(),
        success_count = sorted_results.iter().filter(|r| r.status == TestStatus::Success).count(),
        "Parallel proxy test completed"
    );
    
    Ok(sorted_results)
}

/// Отменяет текущее тестирование
#[tauri::command]
pub async fn cancel_proxy_test(
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Cancelling proxy test");
    
    let tester_guard = state.proxy_tester.read().await;
    if let Some(ref tester) = *tester_guard {
        tester.cancel();
    }
    
    Ok(())
}
```


### Frontend: Listen to Events

```typescript
// src/lib/stores/proxy-test.ts

import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface ProxyTestProgress {
  proxy_id: string;
  proxy_name: string;
  status: 'pending' | 'testing' | 'success' | 'failed' | 'timeout' | 'cancelled';
  current_phase: 'connecting' | 'measuring_latency' | 'testing_download' | 'testing_upload' | 'completed';
  tested_count: number;
  total_count: number;
  percent: number;
}

export interface ProxyTestResult {
  proxy_id: string;
  proxy_name: string;
  status: 'pending' | 'testing' | 'success' | 'failed' | 'timeout' | 'cancelled';
  latency_ms: number | null;
  download_speed_mbps: number | null;
  upload_speed_mbps: number | null;
  error: string | null;
  started_at: string;
  completed_at: string;
}

export interface ParallelTestConfig {
  max_concurrent: number;
  timeout_secs: number;
  test_download: boolean;
  test_upload: boolean;
}

// Svelte 5 store для состояния тестирования
export function createProxyTestStore() {
  let testing = $state(false);
  let progress = $state<Map<string, ProxyTestProgress>>(new Map());
  let results = $state<ProxyTestResult[]>([]);
  let overallProgress = $state({ tested: 0, total: 0, percent: 0 });
  
  let unlistenProgress: (() => void) | null = null;
  let unlistenResult: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  
  async function startTest(proxyIds: string[], config?: Partial<ParallelTestConfig>) {
    testing = true;
    progress = new Map();
    results = [];
    overallProgress = { tested: 0, total: proxyIds.length, percent: 0 };
    
    // Инициализируем все прокси как pending
    for (const id of proxyIds) {
      progress.set(id, {
        proxy_id: id,
        proxy_name: '',
        status: 'pending',
        current_phase: 'connecting',
        tested_count: 0,
        total_count: proxyIds.length,
        percent: 0,
      });
    }
    
    // Подписываемся на события
    unlistenProgress = await listen<ProxyTestProgress>('proxy-test:progress', (event) => {
      const p = event.payload;
      progress.set(p.proxy_id, p);
      progress = new Map(progress); // Trigger reactivity
      overallProgress = { tested: p.tested_count, total: p.total_count, percent: p.percent };
    });
    
    unlistenResult = await listen<ProxyTestProgress>('proxy-test:result', (event) => {
      // Individual result received
    });
    
    unlistenComplete = await listen<ProxyTestResult[]>('proxy-test:complete', (event) => {
      results = event.payload;
      testing = false;
      cleanup();
    });
    
    // Запускаем тест
    try {
      await invoke('test_proxies_parallel', { 
        proxyIds, 
        config: config ?? null 
      });
    } catch (e) {
      testing = false;
      cleanup();
      throw e;
    }
  }
  
  async function cancel() {
    await invoke('cancel_proxy_test');
    testing = false;
    cleanup();
  }
  
  function cleanup() {
    unlistenProgress?.();
    unlistenResult?.();
    unlistenComplete?.();
    unlistenProgress = null;
    unlistenResult = null;
    unlistenComplete = null;
  }
  
  return {
    get testing() { return testing; },
    get progress() { return progress; },
    get results() { return results; },
    get overallProgress() { return overallProgress; },
    startTest,
    cancel,
  };
}
```


## 3. Модель данных ProxyTestResult

### Полная структура результата

```rust
/// Результат тестирования с метриками
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTestResult {
    // Идентификация
    pub proxy_id: String,
    pub proxy_name: String,
    pub protocol: ProxyProtocol,
    
    // Статус
    pub status: TestStatus,
    
    // Метрики производительности
    pub latency_ms: Option<u32>,           // TCP + TLS handshake
    pub download_speed_mbps: Option<f64>,  // Скорость загрузки
    pub upload_speed_mbps: Option<f64>,    // Скорость выгрузки
    pub jitter_ms: Option<u32>,            // Вариация latency
    pub packet_loss_percent: Option<f32>,  // Потеря пакетов (для UDP)
    
    // Информация о сервере
    pub server_location: Option<String>,   // Геолокация (если доступна)
    pub ip_address: Option<String>,        // IP выходного узла
    
    // Ошибки
    pub error: Option<String>,
    pub error_code: Option<String>,
    
    // Временные метки
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub test_duration_ms: u64,
}

/// Агрегированные результаты для UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTestSummary {
    pub total_tested: usize,
    pub successful: usize,
    pub failed: usize,
    pub timed_out: usize,
    pub cancelled: usize,
    
    pub avg_latency_ms: Option<u32>,
    pub min_latency_ms: Option<u32>,
    pub max_latency_ms: Option<u32>,
    
    pub avg_download_mbps: Option<f64>,
    pub avg_upload_mbps: Option<f64>,
    
    pub best_proxy_id: Option<String>,
    pub test_duration_ms: u64,
}

impl ProxyTestSummary {
    pub fn from_results(results: &[ProxyTestResult]) -> Self {
        let successful: Vec<_> = results.iter()
            .filter(|r| r.status == TestStatus::Success)
            .collect();
        
        let latencies: Vec<u32> = successful.iter()
            .filter_map(|r| r.latency_ms)
            .collect();
        
        let downloads: Vec<f64> = successful.iter()
            .filter_map(|r| r.download_speed_mbps)
            .collect();
        
        let uploads: Vec<f64> = successful.iter()
            .filter_map(|r| r.upload_speed_mbps)
            .collect();
        
        // Находим лучший прокси (минимальный latency)
        let best_proxy_id = successful.iter()
            .filter_map(|r| r.latency_ms.map(|l| (r.proxy_id.clone(), l)))
            .min_by_key(|(_, l)| *l)
            .map(|(id, _)| id);
        
        Self {
            total_tested: results.len(),
            successful: successful.len(),
            failed: results.iter().filter(|r| r.status == TestStatus::Failed).count(),
            timed_out: results.iter().filter(|r| r.status == TestStatus::Timeout).count(),
            cancelled: results.iter().filter(|r| r.status == TestStatus::Cancelled).count(),
            
            avg_latency_ms: if latencies.is_empty() { None } 
                else { Some(latencies.iter().sum::<u32>() / latencies.len() as u32) },
            min_latency_ms: latencies.iter().min().copied(),
            max_latency_ms: latencies.iter().max().copied(),
            
            avg_download_mbps: if downloads.is_empty() { None }
                else { Some(downloads.iter().sum::<f64>() / downloads.len() as f64) },
            avg_upload_mbps: if uploads.is_empty() { None }
                else { Some(uploads.iter().sum::<f64>() / uploads.len() as f64) },
            
            best_proxy_id,
            test_duration_ms: results.iter()
                .map(|r| r.test_duration_ms)
                .sum(),
        }
    }
}
```


## 4. UI Компоненты

### Progress Bars Component

```svelte
<!-- src/lib/components/proxies/ProxyTestProgress.svelte -->
<script lang="ts">
  import type { ProxyTestProgress, ProxyTestResult } from '$lib/stores/proxy-test';
  
  interface Props {
    progress: Map<string, ProxyTestProgress>;
    results: ProxyTestResult[];
    overallProgress: { tested: number; total: number; percent: number };
    oncancel?: () => void;
  }
  
  let { progress, results, overallProgress, oncancel }: Props = $props();
  
  // Сортируем по статусу: testing -> pending -> completed
  let sortedProgress = $derived(
    [...progress.values()].sort((a, b) => {
      const order = { testing: 0, pending: 1, success: 2, failed: 2, timeout: 2, cancelled: 2 };
      return (order[a.status] ?? 3) - (order[b.status] ?? 3);
    })
  );
  
  function getStatusColor(status: string): string {
    switch (status) {
      case 'testing': return 'bg-blue-500';
      case 'success': return 'bg-emerald-500';
      case 'failed': return 'bg-red-500';
      case 'timeout': return 'bg-amber-500';
      case 'cancelled': return 'bg-zinc-500';
      default: return 'bg-zinc-700';
    }
  }
  
  function getPhaseText(phase: string): string {
    switch (phase) {
      case 'connecting': return 'Connecting...';
      case 'measuring_latency': return 'Measuring latency...';
      case 'testing_download': return 'Testing download...';
      case 'testing_upload': return 'Testing upload...';
      case 'completed': return 'Completed';
      default: return 'Waiting...';
    }
  }
</script>

<div class="space-y-4">
  <!-- Overall Progress -->
  <div class="bg-zinc-900/50 rounded-xl p-4 border border-white/5">
    <div class="flex items-center justify-between mb-2">
      <span class="text-sm text-zinc-400">
        Testing {overallProgress.tested} / {overallProgress.total} proxies
      </span>
      <span class="text-sm font-medium text-white">{overallProgress.percent}%</span>
    </div>
    <div class="h-2 bg-zinc-800 rounded-full overflow-hidden">
      <div 
        class="h-full bg-gradient-to-r from-blue-500 to-emerald-500 transition-all duration-300"
        style="width: {overallProgress.percent}%"
      ></div>
    </div>
    {#if oncancel}
      <button
        onclick={oncancel}
        class="mt-3 px-3 py-1.5 text-xs bg-red-500/20 text-red-400 rounded-lg
               hover:bg-red-500/30 transition-colors"
      >
        Cancel
      </button>
    {/if}
  </div>
  
  <!-- Individual Progress Bars -->
  <div class="space-y-2 max-h-[400px] overflow-y-auto">
    {#each sortedProgress as item (item.proxy_id)}
      <div class="bg-zinc-900/30 rounded-lg p-3 border border-white/5">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-sm font-medium text-white truncate max-w-[200px]">
            {item.proxy_name || item.proxy_id}
          </span>
          <div class="flex items-center gap-2">
            {#if item.status === 'testing'}
              <span class="text-xs text-zinc-500">{getPhaseText(item.current_phase)}</span>
            {/if}
            <span class={`w-2 h-2 rounded-full ${getStatusColor(item.status)}`}></span>
          </div>
        </div>
        
        {#if item.status === 'testing'}
          <div class="h-1.5 bg-zinc-800 rounded-full overflow-hidden">
            <div class="h-full bg-blue-500 animate-pulse" style="width: 100%"></div>
          </div>
        {:else if item.status === 'success'}
          {@const result = results.find(r => r.proxy_id === item.proxy_id)}
          <div class="flex items-center gap-4 text-xs text-zinc-400">
            {#if result?.latency_ms}
              <span class="text-emerald-400">{result.latency_ms}ms</span>
            {/if}
            {#if result?.download_speed_mbps}
              <span>↓ {result.download_speed_mbps.toFixed(1)} Mbps</span>
            {/if}
            {#if result?.upload_speed_mbps}
              <span>↑ {result.upload_speed_mbps.toFixed(1)} Mbps</span>
            {/if}
          </div>
        {:else if item.status === 'failed' || item.status === 'timeout'}
          {@const result = results.find(r => r.proxy_id === item.proxy_id)}
          <span class="text-xs text-red-400">{result?.error || 'Failed'}</span>
        {/if}
      </div>
    {/each}
  </div>
</div>
```


### Results Table Component

```svelte
<!-- src/lib/components/proxies/ProxyTestResults.svelte -->
<script lang="ts">
  import type { ProxyTestResult, ProxyTestSummary } from '$lib/stores/proxy-test';
  
  interface Props {
    results: ProxyTestResult[];
    summary: ProxyTestSummary | null;
    sortBy?: 'latency' | 'download' | 'upload' | 'name';
    sortOrder?: 'asc' | 'desc';
    onsort?: (field: string) => void;
    onselect?: (proxyId: string) => void;
  }
  
  let { 
    results, 
    summary, 
    sortBy = 'latency', 
    sortOrder = 'asc',
    onsort,
    onselect 
  }: Props = $props();
  
  let sortedResults = $derived(
    [...results].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy) {
        case 'latency':
          const aLat = a.latency_ms ?? Infinity;
          const bLat = b.latency_ms ?? Infinity;
          comparison = aLat - bLat;
          break;
        case 'download':
          const aDown = a.download_speed_mbps ?? 0;
          const bDown = b.download_speed_mbps ?? 0;
          comparison = bDown - aDown; // Higher is better
          break;
        case 'upload':
          const aUp = a.upload_speed_mbps ?? 0;
          const bUp = b.upload_speed_mbps ?? 0;
          comparison = bUp - aUp;
          break;
        case 'name':
          comparison = a.proxy_name.localeCompare(b.proxy_name);
          break;
      }
      
      return sortOrder === 'asc' ? comparison : -comparison;
    })
  );
  
  function formatLatency(ms: number | null): string {
    if (ms === null) return '—';
    if (ms < 100) return `${ms}ms`;
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
  
  function formatSpeed(mbps: number | null): string {
    if (mbps === null) return '—';
    if (mbps < 1) return `${(mbps * 1000).toFixed(0)} Kbps`;
    return `${mbps.toFixed(1)} Mbps`;
  }
  
  function getLatencyColor(ms: number | null): string {
    if (ms === null) return 'text-zinc-500';
    if (ms < 100) return 'text-emerald-400';
    if (ms < 300) return 'text-amber-400';
    return 'text-red-400';
  }
</script>

<div class="space-y-4">
  <!-- Summary Card -->
  {#if summary}
    <div class="grid grid-cols-4 gap-3">
      <div class="bg-zinc-900/50 rounded-xl p-3 border border-white/5">
        <div class="text-2xl font-bold text-white">{summary.successful}</div>
        <div class="text-xs text-zinc-500">Successful</div>
      </div>
      <div class="bg-zinc-900/50 rounded-xl p-3 border border-white/5">
        <div class="text-2xl font-bold text-emerald-400">
          {summary.avg_latency_ms ? `${summary.avg_latency_ms}ms` : '—'}
        </div>
        <div class="text-xs text-zinc-500">Avg Latency</div>
      </div>
      <div class="bg-zinc-900/50 rounded-xl p-3 border border-white/5">
        <div class="text-2xl font-bold text-blue-400">
          {summary.avg_download_mbps ? `${summary.avg_download_mbps.toFixed(1)}` : '—'}
        </div>
        <div class="text-xs text-zinc-500">Avg Download (Mbps)</div>
      </div>
      <div class="bg-zinc-900/50 rounded-xl p-3 border border-white/5">
        <div class="text-2xl font-bold text-purple-400">
          {summary.avg_upload_mbps ? `${summary.avg_upload_mbps.toFixed(1)}` : '—'}
        </div>
        <div class="text-xs text-zinc-500">Avg Upload (Mbps)</div>
      </div>
    </div>
  {/if}
  
  <!-- Results Table -->
  <div class="bg-zinc-900/30 rounded-xl border border-white/5 overflow-hidden">
    <table class="w-full">
      <thead>
        <tr class="border-b border-white/5">
          <th class="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">
            <button onclick={() => onsort?.('name')} class="hover:text-white transition-colors">
              Proxy
            </button>
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-zinc-500 uppercase tracking-wider">
            Status
          </th>
          <th class="px-4 py-3 text-right text-xs font-medium text-zinc-500 uppercase tracking-wider">
            <button onclick={() => onsort?.('latency')} class="hover:text-white transition-colors">
              Latency
            </button>
          </th>
          <th class="px-4 py-3 text-right text-xs font-medium text-zinc-500 uppercase tracking-wider">
            <button onclick={() => onsort?.('download')} class="hover:text-white transition-colors">
              Download
            </button>
          </th>
          <th class="px-4 py-3 text-right text-xs font-medium text-zinc-500 uppercase tracking-wider">
            <button onclick={() => onsort?.('upload')} class="hover:text-white transition-colors">
              Upload
            </button>
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-white/5">
        {#each sortedResults as result, i (result.proxy_id)}
          <tr 
            class="hover:bg-white/5 cursor-pointer transition-colors"
            class:bg-emerald-500/5={summary?.best_proxy_id === result.proxy_id}
            onclick={() => onselect?.(result.proxy_id)}
          >
            <td class="px-4 py-3">
              <div class="flex items-center gap-2">
                {#if summary?.best_proxy_id === result.proxy_id}
                  <span class="text-emerald-400">★</span>
                {/if}
                <span class="text-sm font-medium text-white">{result.proxy_name}</span>
              </div>
            </td>
            <td class="px-4 py-3">
              {#if result.status === 'success'}
                <span class="px-2 py-0.5 text-xs bg-emerald-500/20 text-emerald-400 rounded">OK</span>
              {:else if result.status === 'failed'}
                <span class="px-2 py-0.5 text-xs bg-red-500/20 text-red-400 rounded">Failed</span>
              {:else if result.status === 'timeout'}
                <span class="px-2 py-0.5 text-xs bg-amber-500/20 text-amber-400 rounded">Timeout</span>
              {:else}
                <span class="px-2 py-0.5 text-xs bg-zinc-500/20 text-zinc-400 rounded">{result.status}</span>
              {/if}
            </td>
            <td class="px-4 py-3 text-right">
              <span class={`text-sm font-mono ${getLatencyColor(result.latency_ms)}`}>
                {formatLatency(result.latency_ms)}
              </span>
            </td>
            <td class="px-4 py-3 text-right">
              <span class="text-sm font-mono text-zinc-400">
                {formatSpeed(result.download_speed_mbps)}
              </span>
            </td>
            <td class="px-4 py-3 text-right">
              <span class="text-sm font-mono text-zinc-400">
                {formatSpeed(result.upload_speed_mbps)}
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
```


## 5. Cancellation Support

### Backend Cancellation

```rust
// Используем tokio_util::sync::CancellationToken

impl ParallelProxyTester {
    /// Создаёт новый тестер с токеном отмены
    pub fn new(config: ParallelTestConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build()
                .unwrap_or_default(),
            cancel_token: CancellationToken::new(),
        }
    }
    
    /// Отменяет все текущие тесты
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
    
    /// Проверяет, была ли запрошена отмена
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}

// В test_all() проверяем токен перед каждым тестом:
async move {
    // Ждём разрешения от семафора
    let _permit = semaphore.acquire().await.unwrap();
    
    // Проверяем отмену ПЕРЕД началом теста
    if cancel_token.is_cancelled() {
        return ProxyTestResult {
            status: TestStatus::Cancelled,
            error: Some("Test cancelled by user".to_string()),
            ..Default::default()
        };
    }
    
    // Также проверяем во время теста с помощью tokio::select!
    tokio::select! {
        result = test_single_proxy(&client, &proxy, &config) => result,
        _ = cancel_token.cancelled() => {
            ProxyTestResult {
                proxy_id: proxy.id.clone(),
                proxy_name: proxy.name.clone(),
                status: TestStatus::Cancelled,
                error: Some("Test cancelled".to_string()),
                ..Default::default()
            }
        }
    }
}
```

### Frontend Cancellation

```typescript
// В компоненте Network page
async function handleTestAllGateways() {
  if (testingAllGateways || gateways.length === 0) return;
  
  testingAllGateways = true;
  
  try {
    const store = createProxyTestStore();
    
    // Сохраняем функцию отмены
    cancelTestFn = () => store.cancel();
    
    await store.startTest(
      gateways.map(g => g.id),
      { max_concurrent: 5, test_download: true, test_upload: false }
    );
    
    // Обновляем gateways с результатами
    for (const result of store.results) {
      gateways = gateways.map(g => 
        g.id === result.proxy_id 
          ? { ...g, ping: result.latency_ms ?? undefined }
          : g
      );
    }
    
    // Сортируем по latency
    gateways = [...gateways].sort((a, b) => {
      if (a.ping === undefined && b.ping === undefined) return 0;
      if (a.ping === undefined) return 1;
      if (b.ping === undefined) return -1;
      return a.ping - b.ping;
    });
    
  } finally {
    testingAllGateways = false;
    cancelTestFn = null;
  }
}

function handleCancelTest() {
  cancelTestFn?.();
}
```

## 6. Speed Test Implementation

### Download Speed Test

```rust
/// Измеряет скорость загрузки через прокси
async fn measure_download_speed(client: &reqwest::Client) -> Result<f64, IsolateError> {
    // Используем Cloudflare speed test endpoint (100MB файл)
    // Или можно использовать собственный сервер
    let test_urls = [
        "https://speed.cloudflare.com/__down?bytes=10000000", // 10MB
        "https://proof.ovh.net/files/10Mb.dat",
    ];
    
    for url in &test_urls {
        let start = Instant::now();
        
        match client.get(*url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    continue;
                }
                
                // Читаем весь контент
                let bytes = response.bytes().await.map_err(|e| 
                    IsolateError::Network(e.to_string())
                )?;
                
                let duration = start.elapsed();
                let bytes_downloaded = bytes.len() as f64;
                let seconds = duration.as_secs_f64();
                
                // Конвертируем в Mbps
                let mbps = (bytes_downloaded * 8.0) / (seconds * 1_000_000.0);
                
                return Ok(mbps);
            }
            Err(_) => continue,
        }
    }
    
    Err(IsolateError::Network("All speed test servers failed".to_string()))
}

/// Измеряет скорость выгрузки через прокси
async fn measure_upload_speed(client: &reqwest::Client) -> Result<f64, IsolateError> {
    // Генерируем тестовые данные (1MB)
    let test_data = vec![0u8; 1_000_000];
    
    let start = Instant::now();
    
    // Используем httpbin или собственный endpoint
    let response = client
        .post("https://httpbin.org/post")
        .body(test_data.clone())
        .send()
        .await
        .map_err(|e| IsolateError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(IsolateError::Network(format!(
            "Upload test failed: {}", 
            response.status()
        )));
    }
    
    let duration = start.elapsed();
    let bytes_uploaded = test_data.len() as f64;
    let seconds = duration.as_secs_f64();
    
    // Конвертируем в Mbps
    let mbps = (bytes_uploaded * 8.0) / (seconds * 1_000_000.0);
    
    Ok(mbps)
}
```

## 7. Интеграция с существующим кодом

### Изменения в AppState

```rust
// src-tauri/src/state.rs

pub struct AppState {
    // ... existing fields ...
    
    /// Текущий тестер прокси (для отмены)
    pub proxy_tester: RwLock<Option<Arc<ParallelProxyTester>>>,
}
```

### Регистрация команд в lib.rs

```rust
// src-tauri/src/lib.rs

.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    
    // Proxy testing commands
    commands::test_proxies_parallel,
    commands::cancel_proxy_test,
])
```

### Использование существующего checker.rs

Можно расширить `EndpointChecker` для поддержки прокси:

```rust
impl EndpointChecker {
    /// Создаёт checker с прокси
    pub fn with_proxy(proxy_url: &str) -> Result<Self, IsolateError> {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| IsolateError::Network(e.to_string()))?;
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .proxy(proxy)
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| IsolateError::Network(e.to_string()))?;
        
        Ok(Self {
            client,
            retry_config: RetryConfig::network(),
        })
    }
}
```

## Диаграмма потока данных

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Svelte)                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │ Test Button │───▶│ invoke()     │───▶│ Event Listeners │    │
│  └─────────────┘    │ test_proxies │    │ proxy-test:*    │    │
│                     │ _parallel    │    └────────┬────────┘    │
│                     └──────────────┘             │              │
│                                                  ▼              │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              ProxyTestProgress Component                 │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │ Proxy 1 │ │ Proxy 2 │ │ Proxy 3 │ │ Proxy N │       │   │
│  │  │ ████░░░ │ │ ██████░ │ │ ░░░░░░░ │ │ ████████│       │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Backend (Rust/Tauri)                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  ParallelProxyTester                      │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │              Semaphore (max_concurrent=5)            │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  │                                                          │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │ Task 1  │ │ Task 2  │ │ Task 3  │ │ Task 4  │ ...   │   │
│  │  │ Testing │ │ Testing │ │ Waiting │ │ Waiting │       │   │
│  │  └────┬────┘ └────┬────┘ └─────────┘ └─────────┘       │   │
│  │       │           │                                     │   │
│  │       ▼           ▼                                     │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │              mpsc::channel (progress)            │   │   │
│  │  └──────────────────────┬──────────────────────────┘   │   │
│  └─────────────────────────┼────────────────────────────────┘   │
│                            │                                    │
│                            ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  window.emit("proxy-test:*")              │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```
