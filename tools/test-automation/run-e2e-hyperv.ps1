#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Запускает E2E тесты Isolate с DPI симулятором в Hyper-V VM.

.DESCRIPTION
    Автоматизирует полный цикл E2E тестирования:
    1. Запускает VM с DPI симулятором
    2. Настраивает маршрутизацию через VM
    3. Собирает и запускает Isolate
    4. Запускает Playwright тесты
    5. Собирает результаты
    6. Очищает окружение

.PARAMETER TestSuite
    Набор тестов для запуска: all, smoke, strategies

.PARAMETER DpiMode
    Режим DPI симулятора: drop (сброс пакетов), rst (TCP RST), mitm (MITM атака)

.PARAMETER KeepVM
    Не останавливать VM после тестов (для отладки)

.PARAMETER Verbose
    Подробный вывод

.EXAMPLE
    .\run-e2e-hyperv.ps1 -TestSuite smoke
    .\run-e2e-hyperv.ps1 -TestSuite strategies -DpiMode rst -KeepVM
    .\run-e2e-hyperv.ps1 -TestSuite all -Verbose

.NOTES
    Требует:
    - Windows 11 Pro с Hyper-V
    - Права администратора
    - Настроенную VM (через setup-hyperv-vm.ps1)
    - Node.js, pnpm, Rust toolchain
#>

[CmdletBinding()]
param(
    [Parameter()]
    [ValidateSet("all", "smoke", "strategies")]
    [string]$TestSuite = "smoke",

    [Parameter()]
    [ValidateSet("drop", "rst", "mitm")]
    [string]$DpiMode = "drop",

    [Parameter()]
    [switch]$KeepVM,

    [Parameter()]
    [switch]$VerboseOutput
)

# ============================================================================
# Configuration
# ============================================================================

$ErrorActionPreference = "Stop"
$Script:ExitCode = 0

# VM Configuration
$VMName = "DPI-Simulator"
$SwitchName = "DPI-Internal"
$VMIP = "192.168.100.10"
$HostIP = "192.168.100.1"
$VMUser = "dpi"
$SSHKeyPath = "$env:USERPROFILE\.ssh\id_rsa"

# Paths
$ProjectRoot = (Get-Item $PSScriptRoot).Parent.Parent.FullName
$DpiSimulatorPath = Join-Path $ProjectRoot "tools\dpi-simulator"
$ResultsPath = Join-Path $ProjectRoot "test-results\e2e-hyperv"
$LogPath = Join-Path $ResultsPath "logs"

# Timeouts (in seconds)
$VMStartTimeout = 120
$VMReadyTimeout = 180
$SSHTimeout = 30
$BuildTimeout = 300
$TestTimeout = 600

# ============================================================================
# Logging Functions
# ============================================================================

function Write-Status {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] [*] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] [+] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] [!] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] [-] $Message" -ForegroundColor Red
}

function Write-Debug {
    param([string]$Message)
    if ($VerboseOutput) {
        $timestamp = Get-Date -Format "HH:mm:ss"
        Write-Host "[$timestamp] [D] $Message" -ForegroundColor DarkGray
    }
}

function Write-Section {
    param([string]$Title)
    Write-Host ""
    Write-Host ("=" * 60) -ForegroundColor Magenta
    Write-Host " $Title" -ForegroundColor Magenta
    Write-Host ("=" * 60) -ForegroundColor Magenta
}

# ============================================================================
# Helper Functions
# ============================================================================

function Initialize-Environment {
    Write-Section "Initializing Environment"
    
    # Create results directory
    if (-not (Test-Path $ResultsPath)) {
        New-Item -ItemType Directory -Path $ResultsPath -Force | Out-Null
        Write-Debug "Created results directory: $ResultsPath"
    }
    
    if (-not (Test-Path $LogPath)) {
        New-Item -ItemType Directory -Path $LogPath -Force | Out-Null
        Write-Debug "Created logs directory: $LogPath"
    }
    
    # Check prerequisites
    Write-Status "Checking prerequisites..."
    
    # Check Hyper-V
    $hyperv = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-All -ErrorAction SilentlyContinue
    if ($hyperv.State -ne "Enabled") {
        throw "Hyper-V is not enabled. Enable it first: Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-All"
    }
    Write-Debug "Hyper-V: OK"
    
    # Check VM exists
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm) {
        throw "VM '$VMName' not found. Create it first: .\setup-hyperv-vm.ps1 -Action Create"
    }
    Write-Debug "VM '$VMName': OK"
    
    # Check SSH key
    if (-not (Test-Path $SSHKeyPath)) {
        Write-Warning "SSH key not found at $SSHKeyPath"
        Write-Warning "You may need to configure SSH access to VM manually"
    } else {
        Write-Debug "SSH key: OK"
    }
    
    # Check Node.js
    try {
        $nodeVersion = & node --version 2>$null
        Write-Debug "Node.js: $nodeVersion"
    } catch {
        throw "Node.js not found. Install Node.js first."
    }
    
    # Check pnpm
    try {
        $pnpmVersion = & pnpm --version 2>$null
        Write-Debug "pnpm: $pnpmVersion"
    } catch {
        throw "pnpm not found. Install with: npm install -g pnpm"
    }
    
    # Check Rust
    try {
        $rustVersion = & rustc --version 2>$null
        Write-Debug "Rust: $rustVersion"
    } catch {
        throw "Rust not found. Install from https://rustup.rs"
    }
    
    Write-Success "All prerequisites OK"
}

function Start-VMAndWait {
    Write-Section "Starting VM"
    
    $vm = Get-VM -Name $VMName
    
    if ($vm.State -eq "Running") {
        Write-Status "VM is already running"
    } else {
        Write-Status "Starting VM '$VMName'..."
        Start-VM -Name $VMName
        
        # Wait for VM to start
        $startTime = Get-Date
        while ($vm.State -ne "Running") {
            if (((Get-Date) - $startTime).TotalSeconds -gt $VMStartTimeout) {
                throw "VM failed to start within $VMStartTimeout seconds"
            }
            Start-Sleep -Seconds 2
            $vm = Get-VM -Name $VMName
        }
        Write-Success "VM started"
    }
    
    # Wait for VM to be ready (SSH accessible)
    Write-Status "Waiting for VM to be ready (SSH)..."
    $startTime = Get-Date
    $ready = $false
    
    while (-not $ready) {
        if (((Get-Date) - $startTime).TotalSeconds -gt $VMReadyTimeout) {
            throw "VM not ready within $VMReadyTimeout seconds. Check VM network configuration."
        }
        
        # Try to ping VM
        $ping = Test-Connection -ComputerName $VMIP -Count 1 -Quiet -ErrorAction SilentlyContinue
        if ($ping) {
            # Try SSH connection
            $sshTest = & ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no -o BatchMode=yes -i $SSHKeyPath "$VMUser@$VMIP" "echo ready" 2>$null
            if ($sshTest -eq "ready") {
                $ready = $true
            }
        }
        
        if (-not $ready) {
            Write-Debug "VM not ready yet, waiting..."
            Start-Sleep -Seconds 5
        }
    }
    
    Write-Success "VM is ready and SSH accessible"
}

function Set-NetworkRouting {
    Write-Section "Configuring Network Routing"
    
    # Get the internal switch adapter
    $adapter = Get-NetAdapter | Where-Object { $_.Name -like "*$SwitchName*" -and $_.Status -eq "Up" }
    
    if (-not $adapter) {
        Write-Warning "Internal switch adapter not found. Checking switch configuration..."
        
        # Ensure switch exists
        $switch = Get-VMSwitch -Name $SwitchName -ErrorAction SilentlyContinue
        if (-not $switch) {
            throw "Virtual switch '$SwitchName' not found. Run setup-hyperv-vm.ps1 -Action Create first."
        }
        
        # Wait for adapter to appear
        Start-Sleep -Seconds 5
        $adapter = Get-NetAdapter | Where-Object { $_.Name -like "*$SwitchName*" }
        
        if (-not $adapter) {
            throw "Could not find network adapter for switch '$SwitchName'"
        }
    }
    
    Write-Debug "Found adapter: $($adapter.Name)"
    
    # Configure host IP if not set
    $existingIP = Get-NetIPAddress -InterfaceIndex $adapter.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue | 
                  Where-Object { $_.IPAddress -eq $HostIP }
    
    if (-not $existingIP) {
        Write-Status "Configuring host IP: $HostIP"
        New-NetIPAddress -InterfaceIndex $adapter.ifIndex -IPAddress $HostIP -PrefixLength 24 -ErrorAction SilentlyContinue | Out-Null
    } else {
        Write-Debug "Host IP already configured: $HostIP"
    }
    
    # Add route to VM network
    $route = Get-NetRoute -DestinationPrefix "192.168.100.0/24" -ErrorAction SilentlyContinue
    if (-not $route) {
        Write-Status "Adding route to VM network"
        New-NetRoute -DestinationPrefix "192.168.100.0/24" -InterfaceIndex $adapter.ifIndex -NextHop "0.0.0.0" -ErrorAction SilentlyContinue | Out-Null
    }
    
    # Verify connectivity
    Write-Status "Verifying connectivity to VM..."
    $ping = Test-Connection -ComputerName $VMIP -Count 3 -Quiet
    if (-not $ping) {
        throw "Cannot reach VM at $VMIP. Check VM network configuration."
    }
    
    Write-Success "Network routing configured"
}

function Start-DPISimulator {
    Write-Section "Starting DPI Simulator"
    
    Write-Status "Deploying DPI simulator to VM..."
    
    # Copy simulator files to VM
    $simulatorFiles = @(
        "dpi_simulator.py",
        "blocked_domains.txt",
        "requirements.txt",
        "setup.sh"
    )
    
    foreach ($file in $simulatorFiles) {
        $sourcePath = Join-Path $DpiSimulatorPath $file
        if (Test-Path $sourcePath) {
            Write-Debug "Copying $file to VM..."
            & scp -o StrictHostKeyChecking=no -i $SSHKeyPath $sourcePath "${VMUser}@${VMIP}:~/" 2>$null
        }
    }
    
    # Install dependencies and start simulator
    Write-Status "Installing dependencies on VM..."
    $installCmd = "cd ~ && chmod +x setup.sh && sudo ./setup.sh install"
    & ssh -o StrictHostKeyChecking=no -i $SSHKeyPath "$VMUser@$VMIP" $installCmd 2>$null
    
    Write-Status "Starting DPI simulator (mode: $DpiMode)..."
    
    # Map DPI mode to simulator options
    $dpiOptions = switch ($DpiMode) {
        "drop" { "" }  # Default mode - drop packets
        "rst"  { "--rst" }  # Send TCP RST
        "mitm" { "--mitm" }  # MITM mode
        default { "" }
    }
    
    # Start simulator in background
    $startCmd = "cd ~ && sudo nohup python3 dpi_simulator.py $dpiOptions -l /var/log/dpi_simulator.log > /dev/null 2>&1 &"
    & ssh -o StrictHostKeyChecking=no -i $SSHKeyPath "$VMUser@$VMIP" $startCmd 2>$null
    
    # Wait for simulator to start
    Start-Sleep -Seconds 3
    
    # Verify simulator is running
    $checkCmd = "pgrep -f dpi_simulator.py"
    $pid = & ssh -o StrictHostKeyChecking=no -i $SSHKeyPath "$VMUser@$VMIP" $checkCmd 2>$null
    
    if (-not $pid) {
        throw "DPI simulator failed to start. Check VM logs."
    }
    
    Write-Success "DPI simulator started (PID: $pid)"
}

function Build-Isolate {
    Write-Section "Building Isolate"
    
    Push-Location $ProjectRoot
    
    try {
        # Install frontend dependencies
        Write-Status "Installing frontend dependencies..."
        & pnpm install --frozen-lockfile 2>&1 | ForEach-Object { Write-Debug $_ }
        
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install frontend dependencies"
        }
        
        # Build Tauri app
        Write-Status "Building Tauri application..."
        $env:TAURI_SKIP_DEVSERVER_CHECK = "true"
        
        & pnpm tauri build 2>&1 | ForEach-Object { Write-Debug $_ }
        
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to build Tauri application"
        }
        
        # Find built executable
        $exePath = Get-ChildItem -Path "$ProjectRoot\src-tauri\target\release\*.exe" -ErrorAction SilentlyContinue | 
                   Where-Object { $_.Name -notlike "*uninstall*" } | 
                   Select-Object -First 1
        
        if (-not $exePath) {
            throw "Built executable not found"
        }
        
        Write-Success "Build complete: $($exePath.Name)"
        return $exePath.FullName
        
    } finally {
        Pop-Location
    }
}

function Start-IsolateApp {
    param([string]$ExePath)
    
    Write-Section "Starting Isolate"
    
    if (-not (Test-Path $ExePath)) {
        throw "Executable not found: $ExePath"
    }
    
    Write-Status "Starting Isolate application..."
    
    # Start app in background
    $process = Start-Process -FilePath $ExePath -PassThru -WindowStyle Normal
    
    # Wait for app to initialize
    Start-Sleep -Seconds 5
    
    if ($process.HasExited) {
        throw "Isolate application exited unexpectedly"
    }
    
    Write-Success "Isolate started (PID: $($process.Id))"
    return $process
}

function Invoke-E2ETests {
    param([string]$TestSuite)
    
    Write-Section "Running E2E Tests"
    
    Push-Location $ProjectRoot
    
    try {
        # Determine test pattern
        $testPattern = switch ($TestSuite) {
            "smoke"      { "tests/e2e/smoke/**/*.spec.ts" }
            "strategies" { "tests/e2e/strategies/**/*.spec.ts" }
            "all"        { "tests/e2e/**/*.spec.ts" }
            default      { "tests/e2e/**/*.spec.ts" }
        }
        
        Write-Status "Running test suite: $TestSuite"
        Write-Debug "Test pattern: $testPattern"
        
        # Set environment variables for tests
        $env:DPI_SIMULATOR_IP = $VMIP
        $env:DPI_MODE = $DpiMode
        $env:TEST_RESULTS_PATH = $ResultsPath
        
        # Run Playwright tests
        $playwrightArgs = @(
            "playwright",
            "test",
            "--config=playwright.config.ts",
            "--reporter=html,json",
            "--output=$ResultsPath"
        )
        
        if ($VerboseOutput) {
            $playwrightArgs += "--debug"
        }
        
        Write-Status "Executing Playwright tests..."
        $testOutput = & pnpm @playwrightArgs 2>&1
        $testExitCode = $LASTEXITCODE
        
        # Log output
        $testOutput | ForEach-Object { 
            if ($VerboseOutput) {
                Write-Host $_
            }
        }
        
        # Save test output to log
        $testOutput | Out-File -FilePath (Join-Path $LogPath "playwright-output.log") -Encoding UTF8
        
        if ($testExitCode -eq 0) {
            Write-Success "All tests passed!"
        } else {
            Write-Error "Some tests failed (exit code: $testExitCode)"
            $Script:ExitCode = $testExitCode
        }
        
        return $testExitCode
        
    } finally {
        Pop-Location
    }
}

function Get-TestResults {
    Write-Section "Collecting Results"
    
    # Collect DPI simulator logs from VM
    Write-Status "Collecting DPI simulator logs..."
    $dpiLogPath = Join-Path $LogPath "dpi_simulator.log"
    & scp -o StrictHostKeyChecking=no -i $SSHKeyPath "${VMUser}@${VMIP}:/var/log/dpi_simulator.log" $dpiLogPath 2>$null
    
    # Get DPI statistics
    $statsCmd = "cd ~ && sudo python3 -c \"import json; print(json.dumps({'blocked': 0}))\" 2>/dev/null || echo '{}'"
    $dpiStats = & ssh -o StrictHostKeyChecking=no -i $SSHKeyPath "$VMUser@$VMIP" $statsCmd 2>$null
    
    # Generate summary
    $summary = @{
        Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        TestSuite = $TestSuite
        DpiMode = $DpiMode
        VMName = $VMName
        VMIP = $VMIP
        ResultsPath = $ResultsPath
        ExitCode = $Script:ExitCode
    }
    
    $summaryPath = Join-Path $ResultsPath "test-summary.json"
    $summary | ConvertTo-Json -Depth 3 | Out-File -FilePath $summaryPath -Encoding UTF8
    
    Write-Success "Results saved to: $ResultsPath"
    
    # Print summary
    Write-Host ""
    Write-Host "Test Summary:" -ForegroundColor Cyan
    Write-Host "  Suite:    $TestSuite"
    Write-Host "  DPI Mode: $DpiMode"
    Write-Host "  Results:  $ResultsPath"
    Write-Host "  Status:   $(if ($Script:ExitCode -eq 0) { 'PASSED' } else { 'FAILED' })"
}

function Stop-DPISimulator {
    Write-Status "Stopping DPI simulator..."
    
    $stopCmd = "sudo pkill -f dpi_simulator.py; sudo ./setup.sh cleanup"
    & ssh -o StrictHostKeyChecking=no -i $SSHKeyPath "$VMUser@$VMIP" $stopCmd 2>$null
    
    Write-Debug "DPI simulator stopped"
}

function Stop-IsolateApp {
    param($Process)
    
    if ($Process -and -not $Process.HasExited) {
        Write-Status "Stopping Isolate application..."
        $Process.Kill()
        $Process.WaitForExit(5000)
        Write-Debug "Isolate stopped"
    }
}

function Stop-VMIfNeeded {
    if (-not $KeepVM) {
        Write-Status "Stopping VM..."
        Stop-VM -Name $VMName -Force -ErrorAction SilentlyContinue
        Write-Debug "VM stopped"
    } else {
        Write-Warning "VM kept running (-KeepVM specified)"
    }
}

function Reset-NetworkRouting {
    Write-Status "Resetting network routing..."
    
    # Remove custom routes
    $route = Get-NetRoute -DestinationPrefix "192.168.100.0/24" -ErrorAction SilentlyContinue
    if ($route) {
        Remove-NetRoute -DestinationPrefix "192.168.100.0/24" -Confirm:$false -ErrorAction SilentlyContinue
    }
    
    Write-Debug "Network routing reset"
}

# ============================================================================
# Main Execution
# ============================================================================

$isolateProcess = $null
$startTime = Get-Date

try {
    Write-Host ""
    Write-Host "╔══════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║         Isolate E2E Tests with Hyper-V DPI Simulator     ║" -ForegroundColor Cyan
    Write-Host "╚══════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Configuration:" -ForegroundColor Yellow
    Write-Host "  Test Suite: $TestSuite"
    Write-Host "  DPI Mode:   $DpiMode"
    Write-Host "  Keep VM:    $KeepVM"
    Write-Host ""
    
    # Step 1: Initialize
    Initialize-Environment
    
    # Step 2: Start VM
    Start-VMAndWait
    
    # Step 3: Configure network
    Set-NetworkRouting
    
    # Step 4: Start DPI simulator
    Start-DPISimulator
    
    # Step 5: Build Isolate
    $exePath = Build-Isolate
    
    # Step 6: Start Isolate
    $isolateProcess = Start-IsolateApp -ExePath $exePath
    
    # Step 7: Run tests
    $testResult = Invoke-E2ETests -TestSuite $TestSuite
    
    # Step 8: Collect results
    Get-TestResults
    
} catch {
    Write-Error "Test execution failed: $_"
    Write-Error $_.ScriptStackTrace
    $Script:ExitCode = 1
    
} finally {
    Write-Section "Cleanup"
    
    # Always cleanup, even on error
    try {
        Stop-IsolateApp -Process $isolateProcess
    } catch {
        Write-Debug "Error stopping Isolate: $_"
    }
    
    try {
        Stop-DPISimulator
    } catch {
        Write-Debug "Error stopping DPI simulator: $_"
    }
    
    try {
        Reset-NetworkRouting
    } catch {
        Write-Debug "Error resetting network: $_"
    }
    
    try {
        Stop-VMIfNeeded
    } catch {
        Write-Debug "Error stopping VM: $_"
    }
    
    $duration = (Get-Date) - $startTime
    Write-Host ""
    Write-Host "Total duration: $($duration.ToString('hh\:mm\:ss'))" -ForegroundColor Cyan
    Write-Host ""
    
    if ($Script:ExitCode -eq 0) {
        Write-Success "E2E tests completed successfully!"
    } else {
        Write-Error "E2E tests failed with exit code: $Script:ExitCode"
    }
}

exit $Script:ExitCode
