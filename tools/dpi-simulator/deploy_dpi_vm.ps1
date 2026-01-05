# Deploy DPI-Simulator VM - полная автоматизация
# Ждёт ISO, создаёт VM, настраивает после установки Ubuntu

param(
    [string]$VMName = "DPI-Simulator",
    [string]$ISOPath = "C:\Users\Mind\Downloads\ubuntu-22.04.5-live-server-amd64.iso",
    [string]$StaticIP = "192.168.100.10",
    [string]$Gateway = "192.168.100.1",
    [string]$User = "dpi",
    [string]$Password = "dpi",
    [switch]$SkipISOWait,
    [switch]$SkipVMCreate,
    [switch]$SetupOnly
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

function Write-Step($msg) { Write-Host "`n[$([DateTime]::Now.ToString('HH:mm:ss'))] $msg" -ForegroundColor Cyan }
function Write-OK($msg) { Write-Host "  OK: $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "  WARN: $msg" -ForegroundColor Yellow }
function Write-Err($msg) { Write-Host "  ERROR: $msg" -ForegroundColor Red }

# === PHASE 1: Wait for ISO ===
if (-not $SkipISOWait -and -not $SetupOnly) {
    Write-Step "Waiting for Ubuntu ISO download..."
    
    while ($true) {
        $bits = Get-BitsTransfer | Where-Object { $_.DisplayName -like "*Ubuntu*" }
        
        if ($bits -and $bits.JobState -eq "Transferring") {
            $progress = [math]::Round($bits.BytesTransferred / $bits.BytesTotal * 100, 1)
            Write-Host "`r  Downloading: $progress% " -NoNewline
            Start-Sleep -Seconds 5
        }
        elseif (Test-Path $ISOPath) {
            Write-OK "ISO ready: $ISOPath"
            break
        }
        else {
            Write-Err "ISO not found and no active download"
            exit 1
        }
    }
}

# === PHASE 2: Create VM ===
if (-not $SkipVMCreate -and -not $SetupOnly) {
    Write-Step "Creating VM: $VMName"
    
    # Удаляем старую VM
    $existing = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if ($existing) {
        Write-Warn "Removing existing VM"
        Stop-VM -Name $VMName -Force -TurnOff -ErrorAction SilentlyContinue
        Remove-VM -Name $VMName -Force
    }
    
    # Создаём Internal Switch если нет
    $switch = Get-VMSwitch -Name "DPI-Internal" -ErrorAction SilentlyContinue
    if (-not $switch) {
        Write-Host "  Creating DPI-Internal switch..."
        New-VMSwitch -Name "DPI-Internal" -SwitchType Internal
        $adapter = Get-NetAdapter | Where-Object { $_.Name -like "*DPI-Internal*" }
        New-NetIPAddress -InterfaceIndex $adapter.ifIndex -IPAddress $Gateway -PrefixLength 24 -ErrorAction SilentlyContinue
    }
    
    # Создаём VHD
    $vhdPath = "C:\Hyper-V\VHDs\$VMName.vhdx"
    New-Item -ItemType Directory -Path "C:\Hyper-V\VHDs" -Force | Out-Null
    if (Test-Path $vhdPath) { Remove-Item $vhdPath -Force }
    New-VHD -Path $vhdPath -SizeBytes 20GB -Dynamic | Out-Null
    Write-OK "VHD created"
    
    # Создаём VM
    New-VM -Name $VMName -Path "C:\Hyper-V\VMs" -MemoryStartupBytes 2GB -Generation 2 -VHDPath $vhdPath | Out-Null
    Set-VM -Name $VMName -ProcessorCount 2 -AutomaticCheckpointsEnabled $false
    Set-VMFirmware -VMName $VMName -EnableSecureBoot Off
    
    # Сетевые адаптеры
    Get-VMNetworkAdapter -VMName $VMName | Remove-VMNetworkAdapter
    Add-VMNetworkAdapter -VMName $VMName -Name "Internal" -SwitchName "DPI-Internal"
    Add-VMNetworkAdapter -VMName $VMName -Name "External" -SwitchName "Default Switch"
    
    # ISO
    Add-VMDvdDrive -VMName $VMName -Path $ISOPath
    $dvd = Get-VMDvdDrive -VMName $VMName
    $hdd = Get-VMHardDiskDrive -VMName $VMName
    Set-VMFirmware -VMName $VMName -BootOrder $dvd, $hdd
    
    Write-OK "VM created"
    
    # Запускаем VM
    Start-VM -Name $VMName
    Write-OK "VM started"
    
    Write-Host ""
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Yellow
    Write-Host "║  MANUAL STEP REQUIRED: Install Ubuntu Server               ║" -ForegroundColor Yellow
    Write-Host "╠════════════════════════════════════════════════════════════╣" -ForegroundColor Yellow
    Write-Host "║  1. Run: vmconnect localhost '$VMName'              ║" -ForegroundColor Yellow
    Write-Host "║  2. Install Ubuntu with:                                   ║" -ForegroundColor Yellow
    Write-Host "║     - Username: $User                                        ║" -ForegroundColor Yellow
    Write-Host "║     - Password: $Password                                        ║" -ForegroundColor Yellow
    Write-Host "║     - Install OpenSSH server: YES                          ║" -ForegroundColor Yellow
    Write-Host "║     - Network eth0: $StaticIP/24, gw $Gateway       ║" -ForegroundColor Yellow
    Write-Host "║     - Network eth1: DHCP (for internet)                    ║" -ForegroundColor Yellow
    Write-Host "║  3. After reboot, run this script with -SetupOnly          ║" -ForegroundColor Yellow
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Yellow
    
    # Открываем консоль
    Start-Process vmconnect -ArgumentList "localhost", $VMName
    exit 0
}

# === PHASE 3: Setup via SSH ===
Write-Step "Setting up DPI Simulator via SSH..."

# Ждём SSH
Write-Host "  Waiting for SSH on $StaticIP..."
$maxWait = 60
$waited = 0
while ($waited -lt $maxWait) {
    $tcp = Test-NetConnection -ComputerName $StaticIP -Port 22 -WarningAction SilentlyContinue
    if ($tcp.TcpTestSucceeded) {
        Write-OK "SSH available"
        break
    }
    Start-Sleep -Seconds 2
    $waited += 2
    Write-Host "`r  Waiting... ${waited}s " -NoNewline
}

if ($waited -ge $maxWait) {
    Write-Err "SSH not available after ${maxWait}s"
    Write-Host "  Make sure Ubuntu is installed and network is configured"
    exit 1
}

# Копируем файлы через SCP
Write-Step "Copying files to VM..."

$setupScript = Join-Path $PSScriptRoot "setup_ubuntu_dpi.sh"
$dpiSimulator = Join-Path $PSScriptRoot "dpi_simulator.py"
$blockedDomains = Join-Path $PSScriptRoot "blocked_domains.txt"
$requirements = Join-Path $PSScriptRoot "requirements.txt"

# Используем sshpass или plink для автоматизации
$sshCmd = "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL"
$scpCmd = "scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL"

# Копируем файлы
& cmd /c "echo y | $scpCmd `"$setupScript`" ${User}@${StaticIP}:/tmp/setup.sh" 2>$null
& cmd /c "echo y | $scpCmd `"$dpiSimulator`" ${User}@${StaticIP}:/tmp/dpi_simulator.py" 2>$null
& cmd /c "echo y | $scpCmd `"$blockedDomains`" ${User}@${StaticIP}:/tmp/blocked_domains.txt" 2>$null

Write-OK "Files copied"

# Запускаем setup
Write-Step "Running setup script on VM..."
Write-Host "  This will take a few minutes..."

# Выполняем setup через SSH
$setupCommands = @"
echo '$Password' | sudo -S bash /tmp/setup.sh
sudo mv /tmp/dpi_simulator.py /opt/dpi-simulator/
sudo mv /tmp/blocked_domains.txt /opt/dpi-simulator/
sudo chown -R root:root /opt/dpi-simulator/
sudo systemctl enable dpi-simulator
sudo systemctl start dpi-simulator
"@

# Сохраняем команды во временный файл и выполняем
$tempScript = [System.IO.Path]::GetTempFileName()
$setupCommands | Out-File -FilePath $tempScript -Encoding ASCII
& cmd /c "$sshCmd ${User}@${StaticIP} 'bash -s' < `"$tempScript`"" 2>$null
Remove-Item $tempScript -Force

Write-Step "Verifying DPI Simulator..."
Start-Sleep -Seconds 3

# Проверяем API
try {
    $response = Invoke-RestMethod -Uri "http://${StaticIP}:8888/status" -TimeoutSec 5
    Write-OK "DPI Simulator is running!"
    Write-Host "  Mode: $($response.mode)"
    Write-Host "  Blocked domains: $($response.blocked_domains_count)"
} catch {
    Write-Warn "API not responding yet, service may still be starting"
}

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║  DPI Simulator Deployed Successfully!                      ║" -ForegroundColor Green
Write-Host "╠════════════════════════════════════════════════════════════╣" -ForegroundColor Green
Write-Host "║  VM IP:     $StaticIP                               ║" -ForegroundColor Green
Write-Host "║  API:       http://${StaticIP}:8888                  ║" -ForegroundColor Green
Write-Host "║  SSH:       ssh ${User}@${StaticIP}                       ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Green
