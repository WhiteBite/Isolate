//! Process Resource Limits module
//!
//! Provides functionality to limit memory and CPU usage for spawned processes
//! (winws, sing-box) using Windows Job Objects API.
//!
//! This module is Windows-only and uses Job Objects to enforce:
//! - Memory limits (working set and commit limits)
//! - CPU affinity (which CPU cores the process can use)
//! - Process priority
//!
//! # Example
//! ```ignore
//! use isolate_lib::core::resource_limits::{ResourceLimits, ProcessLimiter};
//!
//! let limits = ResourceLimits::default()
//!     .with_max_memory_mb(256)
//!     .with_priority(ProcessPriority::BelowNormal);
//!
//! let limiter = ProcessLimiter::new()?;
//! limiter.apply_limits(process_id, &limits)?;
//! ```

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::errors::{IsolateError, Result};

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPriority {
    /// Idle priority - lowest
    Idle,
    /// Below normal priority
    BelowNormal,
    /// Normal priority (default)
    #[default]
    Normal,
    /// Above normal priority
    AboveNormal,
    /// High priority
    High,
    /// Realtime priority - highest (requires admin)
    Realtime,
}

impl ProcessPriority {
    /// Convert to Windows priority class constant
    #[cfg(windows)]
    pub fn to_windows_priority(&self) -> u32 {
        use windows_sys::Win32::System::Threading::*;
        match self {
            ProcessPriority::Idle => IDLE_PRIORITY_CLASS,
            ProcessPriority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            ProcessPriority::Normal => NORMAL_PRIORITY_CLASS,
            ProcessPriority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
            ProcessPriority::High => HIGH_PRIORITY_CLASS,
            ProcessPriority::Realtime => REALTIME_PRIORITY_CLASS,
        }
    }
}

/// Resource limits configuration for a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in megabytes (0 = unlimited)
    pub max_memory_mb: u64,
    
    /// CPU affinity mask (0 = all CPUs)
    /// Each bit represents a CPU core (bit 0 = CPU 0, bit 1 = CPU 1, etc.)
    pub cpu_affinity: u64,
    
    /// Process priority
    pub priority: ProcessPriority,
    
    /// Maximum working set size in MB (0 = unlimited)
    /// Working set is the amount of physical memory the process can use
    pub max_working_set_mb: u64,
    
    /// Minimum working set size in MB (0 = system default)
    pub min_working_set_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 0,        // Unlimited
            cpu_affinity: 0,         // All CPUs
            priority: ProcessPriority::Normal,
            max_working_set_mb: 0,   // Unlimited
            min_working_set_mb: 0,   // System default
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits with defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set maximum memory limit in MB
    pub fn with_max_memory_mb(mut self, mb: u64) -> Self {
        self.max_memory_mb = mb;
        self
    }
    
    /// Set CPU affinity mask
    pub fn with_cpu_affinity(mut self, mask: u64) -> Self {
        self.cpu_affinity = mask;
        self
    }
    
    /// Set process priority
    pub fn with_priority(mut self, priority: ProcessPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set working set limits in MB
    pub fn with_working_set(mut self, min_mb: u64, max_mb: u64) -> Self {
        self.min_working_set_mb = min_mb;
        self.max_working_set_mb = max_mb;
        self
    }
    
    /// Check if any limits are set
    pub fn has_limits(&self) -> bool {
        self.max_memory_mb > 0 
            || self.cpu_affinity > 0 
            || self.priority != ProcessPriority::Normal
            || self.max_working_set_mb > 0
    }
    
    /// Get recommended limits for winws process
    pub fn for_winws() -> Self {
        Self {
            max_memory_mb: 128,      // winws is lightweight
            cpu_affinity: 0,         // All CPUs
            priority: ProcessPriority::AboveNormal, // Needs responsiveness
            max_working_set_mb: 64,
            min_working_set_mb: 8,
        }
    }
    
    /// Get recommended limits for sing-box process
    pub fn for_singbox() -> Self {
        Self {
            max_memory_mb: 256,      // sing-box needs more memory
            cpu_affinity: 0,         // All CPUs
            priority: ProcessPriority::Normal,
            max_working_set_mb: 128,
            min_working_set_mb: 16,
        }
    }
}

/// Current resource usage of a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessUsage {
    /// Process ID
    pub pid: u32,
    
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    
    /// Working set size in bytes
    pub working_set_bytes: u64,
    
    /// Peak working set size in bytes
    pub peak_working_set_bytes: u64,
    
    /// CPU time used (user mode) in milliseconds
    pub cpu_time_user_ms: u64,
    
    /// CPU time used (kernel mode) in milliseconds
    pub cpu_time_kernel_ms: u64,
    
    /// Whether the process is still running
    pub is_running: bool,
}

impl Default for ProcessUsage {
    fn default() -> Self {
        Self {
            pid: 0,
            memory_bytes: 0,
            peak_memory_bytes: 0,
            working_set_bytes: 0,
            peak_working_set_bytes: 0,
            cpu_time_user_ms: 0,
            cpu_time_kernel_ms: 0,
            is_running: false,
        }
    }
}

/// Process limiter using Windows Job Objects
#[cfg(windows)]
pub struct ProcessLimiter {
    /// Handle to the job object (if created)
    job_handle: Option<windows_sys::Win32::Foundation::HANDLE>,
}

#[cfg(windows)]
impl ProcessLimiter {
    /// Create a new process limiter
    pub fn new() -> Result<Self> {
        Ok(Self { job_handle: None })
    }
    
    /// Apply resource limits to a process
    pub fn apply_limits(&mut self, pid: u32, limits: &ResourceLimits) -> Result<()> {
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::Threading::*;
        
        if !limits.has_limits() {
            debug!(pid, "No limits to apply");
            return Ok(());
        }
        
        info!(pid, ?limits, "Applying resource limits to process");
        
        unsafe {
            // Open the process
            let process_handle = OpenProcess(
                PROCESS_SET_QUOTA | PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
                FALSE,
                pid,
            );
            
            if process_handle == 0 {
                let err = GetLastError();
                error!(pid, error = err, "Failed to open process");
                return Err(IsolateError::Process(format!(
                    "Failed to open process {}: error {}",
                    pid, err
                )));
            }
            
            // Apply priority
            if limits.priority != ProcessPriority::Normal {
                let priority_class = limits.priority.to_windows_priority();
                if SetPriorityClass(process_handle, priority_class) == 0 {
                    let err = GetLastError();
                    warn!(pid, error = err, "Failed to set priority class");
                } else {
                    debug!(pid, priority = ?limits.priority, "Set process priority");
                }
            }
            
            // Apply CPU affinity
            if limits.cpu_affinity > 0 {
                if SetProcessAffinityMask(process_handle, limits.cpu_affinity as usize) == 0 {
                    let err = GetLastError();
                    warn!(pid, error = err, "Failed to set CPU affinity");
                } else {
                    debug!(pid, affinity = limits.cpu_affinity, "Set CPU affinity");
                }
            }
            
            // Apply working set limits
            if limits.max_working_set_mb > 0 || limits.min_working_set_mb > 0 {
                let min_ws = if limits.min_working_set_mb > 0 {
                    limits.min_working_set_mb * 1024 * 1024
                } else {
                    1024 * 1024 // 1 MB minimum
                };
                let max_ws = if limits.max_working_set_mb > 0 {
                    limits.max_working_set_mb * 1024 * 1024
                } else {
                    min_ws * 10 // Default to 10x minimum
                };
                
                if SetProcessWorkingSetSize(process_handle, min_ws as usize, max_ws as usize) == 0 {
                    let err = GetLastError();
                    warn!(pid, error = err, "Failed to set working set size");
                } else {
                    debug!(pid, min_mb = limits.min_working_set_mb, max_mb = limits.max_working_set_mb, "Set working set limits");
                }
            }
            
            // Apply memory limit via Job Object
            if limits.max_memory_mb > 0 {
                self.apply_memory_limit_via_job(process_handle, pid, limits.max_memory_mb)?;
            }
            
            CloseHandle(process_handle);
        }
        
        info!(pid, "Resource limits applied successfully");
        Ok(())
    }
    
    /// Apply memory limit using Job Object
    unsafe fn apply_memory_limit_via_job(
        &mut self,
        process_handle: windows_sys::Win32::Foundation::HANDLE,
        pid: u32,
        max_memory_mb: u64,
    ) -> Result<()> {
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::JobObjects::*;
        
        // Create a job object
        let job_handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job_handle == 0 {
            let err = GetLastError();
            error!(pid, error = err, "Failed to create job object");
            return Err(IsolateError::Process(format!(
                "Failed to create job object: error {}",
                err
            )));
        }
        
        // Set memory limit
        let mut extended_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        extended_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        extended_info.ProcessMemoryLimit = (max_memory_mb * 1024 * 1024) as usize;
        
        let result = SetInformationJobObject(
            job_handle,
            JobObjectExtendedLimitInformation,
            &extended_info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        
        if result == 0 {
            let err = GetLastError();
            CloseHandle(job_handle);
            error!(pid, error = err, "Failed to set job object limits");
            return Err(IsolateError::Process(format!(
                "Failed to set job object limits: error {}",
                err
            )));
        }
        
        // Assign process to job
        let result = AssignProcessToJobObject(job_handle, process_handle);
        if result == 0 {
            let err = GetLastError();
            // Error 5 (ACCESS_DENIED) often means process is already in a job
            if err == ERROR_ACCESS_DENIED {
                warn!(pid, "Process already assigned to a job object, memory limit not applied");
                CloseHandle(job_handle);
                return Ok(());
            }
            CloseHandle(job_handle);
            error!(pid, error = err, "Failed to assign process to job");
            return Err(IsolateError::Process(format!(
                "Failed to assign process to job: error {}",
                err
            )));
        }
        
        debug!(pid, max_memory_mb, "Applied memory limit via job object");
        
        // Store job handle to keep it alive
        if let Some(old_handle) = self.job_handle.take() {
            CloseHandle(old_handle);
        }
        self.job_handle = Some(job_handle);
        
        Ok(())
    }
    
    /// Get current resource usage of a process
    pub fn get_process_usage(pid: u32) -> Result<ProcessUsage> {
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::Threading::*;
        use windows_sys::Win32::System::ProcessStatus::*;
        
        // STILL_ACTIVE constant (259)
        const STILL_ACTIVE_CODE: u32 = 259;
        
        unsafe {
            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                FALSE,
                pid,
            );
            
            if process_handle == 0 {
                let err = GetLastError();
                // Process might have exited
                if err == ERROR_INVALID_PARAMETER {
                    return Ok(ProcessUsage {
                        pid,
                        is_running: false,
                        ..Default::default()
                    });
                }
                return Err(IsolateError::Process(format!(
                    "Failed to open process {}: error {}",
                    pid, err
                )));
            }
            
            // Check if process is still running
            let mut exit_code: u32 = 0;
            let is_running = if GetExitCodeProcess(process_handle, &mut exit_code) != 0 {
                exit_code == STILL_ACTIVE_CODE
            } else {
                false
            };
            
            // Get memory info
            let mut mem_counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            mem_counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            
            let (memory_bytes, peak_memory_bytes, working_set_bytes, peak_working_set_bytes) = 
                if GetProcessMemoryInfo(
                    process_handle,
                    &mut mem_counters,
                    std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                ) != 0 {
                    (
                        mem_counters.PagefileUsage as u64,
                        mem_counters.PeakPagefileUsage as u64,
                        mem_counters.WorkingSetSize as u64,
                        mem_counters.PeakWorkingSetSize as u64,
                    )
                } else {
                    (0, 0, 0, 0)
                };
            
            // Get CPU times
            let mut creation_time: FILETIME = std::mem::zeroed();
            let mut exit_time: FILETIME = std::mem::zeroed();
            let mut kernel_time: FILETIME = std::mem::zeroed();
            let mut user_time: FILETIME = std::mem::zeroed();
            
            let (cpu_time_user_ms, cpu_time_kernel_ms) = if GetProcessTimes(
                process_handle,
                &mut creation_time,
                &mut exit_time,
                &mut kernel_time,
                &mut user_time,
            ) != 0 {
                let user_100ns = ((user_time.dwHighDateTime as u64) << 32) | (user_time.dwLowDateTime as u64);
                let kernel_100ns = ((kernel_time.dwHighDateTime as u64) << 32) | (kernel_time.dwLowDateTime as u64);
                // Convert from 100-nanosecond intervals to milliseconds
                (user_100ns / 10_000, kernel_100ns / 10_000)
            } else {
                (0, 0)
            };
            
            CloseHandle(process_handle);
            
            Ok(ProcessUsage {
                pid,
                memory_bytes,
                peak_memory_bytes,
                working_set_bytes,
                peak_working_set_bytes,
                cpu_time_user_ms,
                cpu_time_kernel_ms,
                is_running,
            })
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessLimiter {
    fn drop(&mut self) {
        if let Some(handle) = self.job_handle.take() {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(handle);
            }
        }
    }
}

// Non-Windows stub implementation
#[cfg(not(windows))]
pub struct ProcessLimiter;

#[cfg(not(windows))]
impl ProcessLimiter {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub fn apply_limits(&mut self, pid: u32, limits: &ResourceLimits) -> Result<()> {
        warn!(pid, "Resource limits are only supported on Windows");
        Ok(())
    }
    
    pub fn get_process_usage(pid: u32) -> Result<ProcessUsage> {
        Ok(ProcessUsage {
            pid,
            is_running: false,
            ..Default::default()
        })
    }
}

/// Get default resource limits for the application
pub fn get_default_limits() -> ResourceLimits {
    ResourceLimits::default()
}

/// Get recommended limits for a specific process type
pub fn get_recommended_limits(process_type: &str) -> ResourceLimits {
    match process_type.to_lowercase().as_str() {
        "winws" => ResourceLimits::for_winws(),
        "singbox" | "sing-box" => ResourceLimits::for_singbox(),
        _ => ResourceLimits::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, 0);
        assert_eq!(limits.cpu_affinity, 0);
        assert_eq!(limits.priority, ProcessPriority::Normal);
        assert!(!limits.has_limits());
    }

    #[test]
    fn test_resource_limits_builder() {
        let limits = ResourceLimits::new()
            .with_max_memory_mb(256)
            .with_cpu_affinity(0b1111) // First 4 CPUs
            .with_priority(ProcessPriority::BelowNormal)
            .with_working_set(16, 128);
        
        assert_eq!(limits.max_memory_mb, 256);
        assert_eq!(limits.cpu_affinity, 0b1111);
        assert_eq!(limits.priority, ProcessPriority::BelowNormal);
        assert_eq!(limits.min_working_set_mb, 16);
        assert_eq!(limits.max_working_set_mb, 128);
        assert!(limits.has_limits());
    }

    #[test]
    fn test_resource_limits_has_limits() {
        let default = ResourceLimits::default();
        assert!(!default.has_limits());
        
        let with_memory = ResourceLimits::new().with_max_memory_mb(100);
        assert!(with_memory.has_limits());
        
        let with_affinity = ResourceLimits::new().with_cpu_affinity(1);
        assert!(with_affinity.has_limits());
        
        let with_priority = ResourceLimits::new().with_priority(ProcessPriority::High);
        assert!(with_priority.has_limits());
    }

    #[test]
    fn test_preset_limits() {
        let winws = ResourceLimits::for_winws();
        assert_eq!(winws.max_memory_mb, 128);
        assert_eq!(winws.priority, ProcessPriority::AboveNormal);
        
        let singbox = ResourceLimits::for_singbox();
        assert_eq!(singbox.max_memory_mb, 256);
        assert_eq!(singbox.priority, ProcessPriority::Normal);
    }

    #[test]
    fn test_get_recommended_limits() {
        let winws = get_recommended_limits("winws");
        assert_eq!(winws.max_memory_mb, 128);
        
        let singbox = get_recommended_limits("singbox");
        assert_eq!(singbox.max_memory_mb, 256);
        
        let unknown = get_recommended_limits("unknown");
        assert_eq!(unknown.max_memory_mb, 0);
    }

    #[test]
    fn test_process_usage_default() {
        let usage = ProcessUsage::default();
        assert_eq!(usage.pid, 0);
        assert_eq!(usage.memory_bytes, 0);
        assert!(!usage.is_running);
    }

    #[test]
    fn test_resource_limits_serialization() {
        let limits = ResourceLimits::new()
            .with_max_memory_mb(256)
            .with_priority(ProcessPriority::High);
        
        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("256"));
        assert!(json.contains("high"));
        
        let deserialized: ResourceLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_memory_mb, 256);
        assert_eq!(deserialized.priority, ProcessPriority::High);
    }

    #[test]
    fn test_process_priority_serialization() {
        let priorities = vec![
            ProcessPriority::Idle,
            ProcessPriority::BelowNormal,
            ProcessPriority::Normal,
            ProcessPriority::AboveNormal,
            ProcessPriority::High,
            ProcessPriority::Realtime,
        ];
        
        for priority in priorities {
            let json = serde_json::to_string(&priority).unwrap();
            let deserialized: ProcessPriority = serde_json::from_str(&json).unwrap();
            assert_eq!(priority, deserialized);
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_process_limiter_creation() {
        let limiter = ProcessLimiter::new();
        assert!(limiter.is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn test_get_current_process_usage() {
        let pid = std::process::id();
        let usage = ProcessLimiter::get_process_usage(pid);
        assert!(usage.is_ok());
        
        let usage = usage.unwrap();
        assert_eq!(usage.pid, pid);
        assert!(usage.is_running);
        assert!(usage.memory_bytes > 0);
        assert!(usage.working_set_bytes > 0);
    }

    #[cfg(windows)]
    #[test]
    fn test_get_nonexistent_process_usage() {
        // Use an unlikely PID
        let usage = ProcessLimiter::get_process_usage(999999);
        assert!(usage.is_ok());
        
        let usage = usage.unwrap();
        assert!(!usage.is_running);
    }
}
