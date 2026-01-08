// Real-time traffic monitoring store with Svelte 5 runes

export interface TrafficPoint {
  timestamp: number;
  download: number;
  upload: number;
}

class TrafficMonitorStore {
  // State
  isMonitoring = $state(false);
  currentDownload = $state(0);
  currentUpload = $state(0);
  history = $state<TrafficPoint[]>([]);
  
  // Session totals
  totalDownload = $state(0);
  totalUpload = $state(0);
  
  private intervalId: number | null = null;
  
  // Derived values
  get downloadSpeed() {
    return this.currentDownload;
  }
  
  get uploadSpeed() {
    return this.currentUpload;
  }
  
  get historyLength() {
    return this.history.length;
  }
  
  /**
   * Start monitoring traffic
   * Currently uses simulated data - will be replaced with real Tauri events
   */
  start() {
    if (this.isMonitoring) return;
    this.isMonitoring = true;
    
    // Simulate traffic data (replace with real Tauri events later)
    this.intervalId = setInterval(() => {
      // Simulate realistic traffic patterns
      const baseDownload = 50000 + Math.random() * 200000; // 50-250 KB/s
      const baseUpload = 10000 + Math.random() * 50000;    // 10-60 KB/s
      
      // Add some variance
      const download = Math.round(baseDownload + (Math.random() - 0.5) * 30000);
      const upload = Math.round(baseUpload + (Math.random() - 0.5) * 15000);
      
      this.addPoint(download, upload);
    }, 1000) as unknown as number;
  }
  
  /**
   * Stop monitoring traffic
   */
  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
    this.isMonitoring = false;
  }
  
  /**
   * Add a traffic data point
   */
  addPoint(download: number, upload: number) {
    this.currentDownload = download;
    this.currentUpload = upload;
    this.totalDownload += download;
    this.totalUpload += upload;
    
    // Keep last 60 points (1 minute at 1 point/sec)
    this.history = [...this.history.slice(-59), {
      timestamp: Date.now(),
      download,
      upload
    }];
  }
  
  /**
   * Reset all stats
   */
  reset() {
    this.stop();
    this.currentDownload = 0;
    this.currentUpload = 0;
    this.totalDownload = 0;
    this.totalUpload = 0;
    this.history = [];
  }
  
  /**
   * Format bytes to human readable string
   */
  static formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  }
  
  /**
   * Format speed to human readable string
   */
  static formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec < 1024) return `${bytesPerSec} B/s`;
    if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
    return `${(bytesPerSec / 1024 / 1024).toFixed(1)} MB/s`;
  }
}

export const trafficMonitor = new TrafficMonitorStore();
