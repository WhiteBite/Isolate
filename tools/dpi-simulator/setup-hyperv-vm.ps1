#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Creates and manages Hyper-V VM for DPI Simulator testing
.DESCRIPTION
    Автоматически создаёт и настраивает Linux VM для тестирования DPI bypass в Isolate.
    Требует: Windows 11 Pro с Hyper-V, права администратора.
    
    Workflow:
    1. Create - создать VM и скачать Ubuntu ISO
    2. Start - запустить VM и установить Ubuntu вручную
    3. Configure - настроить сеть внутри VM (после установки Ubuntu)
    4. Deploy - скопировать DPI симулятор в VM
    5. StartSim - запустить DPI симулятор
    6. Test - проверить готовность к тестированию
    
.EXAMPLE
    .\setup-hyperv-vm.ps1 -Action Create
    .\setup-hyperv-vm.ps1 -Action Start
    .\setup-hyperv-vm.ps1 -Action Configure -VMUser dpi -VMIP 192.168.100.10
    .\setup-hyperv-vm.ps1 -Action Deploy
    .\setup-hyperv-vm.ps1 -Action StartSim
    .\setup-hyperv-vm.ps1 -Action Test
    .\setup-hyperv-vm.ps1 -Action StopSim
    .\setup-hyperv-vm.ps1 -Action Stop
    .\setup-hyperv-vm.ps1 -Action Remove
.PARAMETER Action
    Create    - Создать VM с Ubuntu ISO
    Start     - Запустить VM
    Stop      - Остановить VM
    Remove    - Удалить VM
    Status    - Показать статус VM
    GetIP     - Получить IP адреса VM
    Configure - Настроить сеть внутри VM (требует SSH)
    Deploy    - Скопировать DPI симулятор в VM
    StartSim  - Запустить DPI симулятор
    StopSim   - Остановить DPI симулятор
    Test      - Проверить готовность к тестированию
.PARAMETER VMUser
    Имя пользователя для SSH (default: dpi)
.PARAMETER VMIP
    IP адрес VM (default: 192.168.100.10)
.PARAMETER SSHKeyPath
    Путь к SSH ключу (default: ~/.ssh/id_rsa)
#>

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("Create", "Start", "Stop", "Remove", "Status", "GetIP", "Configure", "Deploy", "StartSim", "StopSim", "Test")]
    [string]$Action,
    
    [string]$VMName = "DPI-Simulator",
    [string]$SwitchName = "DPI-Internal",
    [int64]$MemoryMB = 2048,
    [int64]$DiskGB = 20,
    
    # SSH параметры для Configure/Deploy/StartSim/Test
    [string]$VMUser = "dpi",
    [string]$VMIP = "192.168.100.10",
    [string]$SSHKeyPath = "$env:USERPROFILE\.ssh\id_rsa"
)

$ErrorActionPreference = "Stop"

# Paths
$VMPath = "$env:USERPROFILE\Hyper-V\$VMName"
$VHDPath = "$VMPath\$VMName.vhdx"
$ISOPath = "$VMPath\ubuntu-server.iso"
$UbuntuURL = "https://releases.ubuntu.com/22.04/ubuntu-22.04.5-live-server-amd64.iso"
$SimulatorPath = $PSScriptRoot  # tools/dpi-simulator/
$RemoteSimPath = "/opt/dpi-simulator"

function Write-Status($msg) { Write-Host "[*] $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "[+] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "[!] $msg" -ForegroundColor Yellow }
function Write-Error($msg) { Write-Host "[-] $msg" -ForegroundColor Red }

# SSH helper function
function Invoke-SSHCommand {
    param(
        [string]$Command,
        [switch]$Silent
    )
    
    $sshArgs = @(
        "-o", "StrictHostKeyChecking=no",
        "-o", "UserKnownHostsFile=/dev/null",
        "-o", "ConnectTimeout=5"
    )
    
    if (Test-Path $SSHKeyPath) {
        $sshArgs += @("-i", $SSHKeyPath)
    }
    
    $sshArgs += @("$VMUser@$VMIP", $Command)
    
    if ($Silent) {
        $result = & ssh @sshArgs 2>$null
        return $result
    } else {
        & ssh @sshArgs
        return $LASTEXITCODE -eq 0
    }
}

function Test-SSHConnection {
    Write-Status "Testing SSH connection to $VMUser@$VMIP..."
    $result = Invoke-SSHCommand -Command "echo 'SSH OK'" -Silent
    if ($result -eq "SSH OK") {
        Write-Success "SSH connection successful"
        return $true
    } else {
        Write-Error "SSH connection failed"
        Write-Warn "Make sure:"
        Write-Warn "  1. VM is running and has IP $VMIP"
        Write-Warn "  2. SSH server is installed: sudo apt install openssh-server"
        Write-Warn "  3. SSH key is configured or use password auth"
        return $false
    }
}

function Create-VM {
    Write-Status "Creating DPI Simulator VM..."
    
    # Create VM directory
    if (-not (Test-Path $VMPath)) {
        New-Item -ItemType Directory -Path $VMPath -Force | Out-Null
        Write-Success "Created VM directory: $VMPath"
    }
    
    # Check if VM already exists
    if (Get-VM -Name $VMName -ErrorAction SilentlyContinue) {
        Write-Error "VM '$VMName' already exists. Use -Action Remove first."
        return
    }
    
    # Create Internal Switch for isolated network
    if (-not (Get-VMSwitch -Name $SwitchName -ErrorAction SilentlyContinue)) {
        Write-Status "Creating Internal Switch: $SwitchName"
        New-VMSwitch -Name $SwitchName -SwitchType Internal | Out-Null
        
        # Configure host IP on the switch
        $adapter = Get-NetAdapter | Where-Object { $_.Name -like "*$SwitchName*" }
        if ($adapter) {
            New-NetIPAddress -InterfaceIndex $adapter.ifIndex -IPAddress 192.168.100.1 -PrefixLength 24 -ErrorAction SilentlyContinue
            Write-Success "Configured host IP: 192.168.100.1/24"
        }
    } else {
        Write-Status "Switch '$SwitchName' already exists"
    }
    
    # Download Ubuntu ISO if not exists
    if (-not (Test-Path $ISOPath)) {
        Write-Status "Downloading Ubuntu Server 22.04 ISO (~2GB)..."
        Write-Status "URL: $UbuntuURL"
        
        $ProgressPreference = 'SilentlyContinue'
        try {
            Invoke-WebRequest -Uri $UbuntuURL -OutFile $ISOPath -UseBasicParsing
            Write-Success "Downloaded Ubuntu ISO"
        } catch {
            Write-Error "Failed to download ISO: $_"
            Write-Status "Please download manually from: $UbuntuURL"
            Write-Status "Save to: $ISOPath"
            return
        }
    } else {
        Write-Success "Ubuntu ISO already exists"
    }
    
    # Create VHDX
    Write-Status "Creating virtual disk: $DiskGB GB"
    New-VHD -Path $VHDPath -SizeBytes ($DiskGB * 1GB) -Dynamic | Out-Null
    
    # Create VM
    Write-Status "Creating VM: $VMName"
    New-VM -Name $VMName `
        -MemoryStartupBytes ($MemoryMB * 1MB) `
        -Generation 2 `
        -VHDPath $VHDPath `
        -SwitchName $SwitchName | Out-Null
    
    # Configure VM
    Set-VM -Name $VMName `
        -ProcessorCount 2 `
        -DynamicMemory `
        -MemoryMinimumBytes (512MB) `
        -MemoryMaximumBytes ($MemoryMB * 1MB) `
        -AutomaticStartAction Nothing `
        -AutomaticStopAction ShutDown
    
    # Add DVD drive with Ubuntu ISO
    Add-VMDvdDrive -VMName $VMName -Path $ISOPath
    
    # Set boot order (DVD first for installation)
    $dvd = Get-VMDvdDrive -VMName $VMName
    Set-VMFirmware -VMName $VMName -FirstBootDevice $dvd
    
    # Disable Secure Boot for Ubuntu
    Set-VMFirmware -VMName $VMName -EnableSecureBoot Off
    
    # Add second network adapter (NAT for internet)
    $defaultSwitch = Get-VMSwitch | Where-Object { $_.SwitchType -eq "Internal" -and $_.Name -eq "Default Switch" }
    if ($defaultSwitch) {
        Add-VMNetworkAdapter -VMName $VMName -SwitchName "Default Switch" -Name "Internet"
        Write-Success "Added NAT adapter for internet access"
    } else {
        Write-Status "No Default Switch found - VM will only have internal network"
        Write-Status "You may need to add NAT manually for internet access"
    }
    
    Write-Success "VM '$VMName' created successfully!"
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "1. Run: .\setup-hyperv-vm.ps1 -Action Start"
    Write-Host "2. Connect to VM via Hyper-V Manager"
    Write-Host "3. Install Ubuntu Server (minimal install)"
    Write-Host "4. Configure static IP: 192.168.100.10/24, gateway: 192.168.100.1"
    Write-Host "5. Copy tools/dpi-simulator/ to VM"
    Write-Host "6. Run: sudo ./setup.sh setup"
}

function Start-DPISimVM {
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm) {
        Write-Error "VM '$VMName' not found"
        return
    }
    
    if ($vm.State -eq "Running") {
        Write-Status "VM is already running"
    } else {
        Write-Status "Starting VM..."
        Start-VM -Name $VMName
        Write-Success "VM started"
    }
    
    # Open Hyper-V Manager connection
    Write-Status "Opening VM connection..."
    vmconnect.exe localhost $VMName
}

function Stop-DPISimVM {
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm) {
        Write-Error "VM '$VMName' not found"
        return
    }
    
    if ($vm.State -eq "Off") {
        Write-Status "VM is already stopped"
    } else {
        Write-Status "Stopping VM..."
        Stop-VM -Name $VMName -Force
        Write-Success "VM stopped"
    }
}

function Remove-DPISimVM {
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm) {
        Write-Status "VM '$VMName' not found"
        return
    }
    
    # Stop if running
    if ($vm.State -ne "Off") {
        Write-Status "Stopping VM..."
        Stop-VM -Name $VMName -Force -TurnOff
    }
    
    Write-Status "Removing VM..."
    Remove-VM -Name $VMName -Force
    
    # Remove VHD
    if (Test-Path $VHDPath) {
        Remove-Item $VHDPath -Force
        Write-Success "Removed virtual disk"
    }
    
    # Remove switch (optional)
    $switch = Get-VMSwitch -Name $SwitchName -ErrorAction SilentlyContinue
    if ($switch) {
        $otherVMs = Get-VM | Get-VMNetworkAdapter | Where-Object { $_.SwitchName -eq $SwitchName }
        if (-not $otherVMs) {
            Remove-VMSwitch -Name $SwitchName -Force
            Write-Success "Removed virtual switch"
        }
    }
    
    Write-Success "VM '$VMName' removed"
}

function Get-VMStatus {
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm) {
        Write-Error "VM '$VMName' not found"
        return
    }
    
    Write-Host ""
    Write-Host "=== VM Status ===" -ForegroundColor Cyan
    Write-Host "Name:      $($vm.Name)"
    Write-Host "State:     $($vm.State)"
    Write-Host "CPU:       $($vm.ProcessorCount) cores"
    Write-Host "Memory:    $([math]::Round($vm.MemoryAssigned / 1MB)) MB"
    Write-Host "Uptime:    $($vm.Uptime)"
    
    Write-Host ""
    Write-Host "=== Network Adapters ===" -ForegroundColor Cyan
    Get-VMNetworkAdapter -VMName $VMName | ForEach-Object {
        Write-Host "  $($_.Name): $($_.SwitchName) - $($_.IPAddresses -join ', ')"
    }
}

function Get-VMIP {
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm) {
        Write-Error "VM '$VMName' not found"
        return
    }
    
    $adapters = Get-VMNetworkAdapter -VMName $VMName
    foreach ($adapter in $adapters) {
        if ($adapter.IPAddresses) {
            Write-Host "$($adapter.Name): $($adapter.IPAddresses -join ', ')"
        }
    }
}

function Configure-VMNetwork {
    <#
    .SYNOPSIS
        Настраивает сеть внутри VM после установки Ubuntu
    #>
    
    if (-not (Test-SSHConnection)) { return }
    
    Write-Status "Configuring network inside VM..."
    
    # Определяем сетевые интерфейсы
    $interfaces = Invoke-SSHCommand -Command "ip -o link show | grep -v 'lo:' | awk -F': ' '{print \`$2}'" -Silent
    Write-Status "Found interfaces: $interfaces"
    
    # Создаём netplan конфигурацию
    $netplanConfig = @"
network:
  version: 2
  ethernets:
    eth0:
      addresses:
        - 192.168.100.10/24
      routes:
        - to: default
          via: 192.168.100.1
      nameservers:
        addresses:
          - 8.8.8.8
          - 8.8.4.4
    eth1:
      dhcp4: true
"@
    
    Write-Status "Creating netplan configuration..."
    
    # Записываем конфиг через SSH
    $escapedConfig = $netplanConfig -replace '"', '\"' -replace '\$', '\$'
    Invoke-SSHCommand -Command "echo '$escapedConfig' | sudo tee /etc/netplan/01-dpi-network.yaml > /dev/null"
    
    # Применяем конфигурацию
    Write-Status "Applying network configuration..."
    Invoke-SSHCommand -Command "sudo netplan apply"
    
    # Проверяем результат
    Start-Sleep -Seconds 2
    $ip = Invoke-SSHCommand -Command "ip addr show | grep '192.168.100.10'" -Silent
    if ($ip) {
        Write-Success "Network configured successfully!"
        Write-Host "  Internal IP: 192.168.100.10"
    } else {
        Write-Warn "Network configuration applied, but IP not detected yet"
        Write-Warn "Try: ip addr show"
    }
    
    # Включаем IP forwarding для роутинга трафика
    Write-Status "Enabling IP forwarding..."
    Invoke-SSHCommand -Command "echo 'net.ipv4.ip_forward=1' | sudo tee -a /etc/sysctl.conf > /dev/null"
    Invoke-SSHCommand -Command "sudo sysctl -p"
    
    Write-Success "VM network configuration complete!"
}

function Deploy-Simulator {
    <#
    .SYNOPSIS
        Копирует DPI симулятор в VM через SCP
    #>
    
    if (-not (Test-SSHConnection)) { return }
    
    Write-Status "Deploying DPI Simulator to VM..."
    
    # Создаём директорию на VM
    Write-Status "Creating directory $RemoteSimPath..."
    Invoke-SSHCommand -Command "sudo mkdir -p $RemoteSimPath && sudo chown $VMUser`:$VMUser $RemoteSimPath"
    
    # Файлы для копирования
    $files = @(
        "dpi_simulator.py",
        "setup.sh",
        "requirements.txt",
        "blocked_domains.txt"
    )
    
    # SCP параметры
    $scpArgs = @(
        "-o", "StrictHostKeyChecking=no",
        "-o", "UserKnownHostsFile=/dev/null"
    )
    
    if (Test-Path $SSHKeyPath) {
        $scpArgs += @("-i", $SSHKeyPath)
    }
    
    # Копируем каждый файл
    foreach ($file in $files) {
        $localPath = Join-Path $SimulatorPath $file
        if (Test-Path $localPath) {
            Write-Status "Copying $file..."
            $scpTarget = "$VMUser@$VMIP`:$RemoteSimPath/$file"
            & scp @scpArgs $localPath $scpTarget
            if ($LASTEXITCODE -eq 0) {
                Write-Success "  $file copied"
            } else {
                Write-Error "  Failed to copy $file"
            }
        } else {
            Write-Warn "  $file not found locally, skipping"
        }
    }
    
    # Делаем setup.sh исполняемым
    Write-Status "Setting permissions..."
    Invoke-SSHCommand -Command "chmod +x $RemoteSimPath/setup.sh"
    
    # Устанавливаем зависимости
    Write-Status "Installing dependencies..."
    Invoke-SSHCommand -Command "cd $RemoteSimPath && sudo ./setup.sh setup"
    
    Write-Success "DPI Simulator deployed to $RemoteSimPath"
    Write-Host ""
    Write-Host "To start manually:" -ForegroundColor Yellow
    Write-Host "  ssh $VMUser@$VMIP"
    Write-Host "  cd $RemoteSimPath"
    Write-Host "  sudo ./setup.sh start"
}

function Start-Simulator {
    <#
    .SYNOPSIS
        Запускает DPI симулятор на VM
    #>
    
    if (-not (Test-SSHConnection)) { return }
    
    Write-Status "Starting DPI Simulator..."
    
    # Проверяем, установлен ли симулятор
    $exists = Invoke-SSHCommand -Command "test -f $RemoteSimPath/dpi_simulator.py && echo 'yes'" -Silent
    if ($exists -ne "yes") {
        Write-Error "DPI Simulator not found at $RemoteSimPath"
        Write-Warn "Run: .\setup-hyperv-vm.ps1 -Action Deploy"
        return
    }
    
    # Проверяем, не запущен ли уже
    $running = Invoke-SSHCommand -Command "pgrep -f 'dpi_simulator.py' > /dev/null && echo 'running'" -Silent
    if ($running -eq "running") {
        Write-Warn "DPI Simulator is already running"
        Write-Status "Use -Action StopSim to stop it first"
        return
    }
    
    # Запускаем симулятор в фоне через nohup
    Write-Status "Starting simulator in background..."
    Invoke-SSHCommand -Command "cd $RemoteSimPath && sudo nohup python3 dpi_simulator.py > /var/log/dpi-simulator.log 2>&1 &"
    
    # Ждём запуска
    Start-Sleep -Seconds 2
    
    # Проверяем статус
    $running = Invoke-SSHCommand -Command "pgrep -f 'dpi_simulator.py' > /dev/null && echo 'running'" -Silent
    if ($running -eq "running") {
        Write-Success "DPI Simulator started!"
        Write-Host ""
        Write-Host "Simulator is intercepting traffic on 192.168.100.10" -ForegroundColor Green
        Write-Host "Logs: ssh $VMUser@$VMIP 'tail -f /var/log/dpi-simulator.log'"
    } else {
        Write-Error "Failed to start DPI Simulator"
        Write-Warn "Check logs: ssh $VMUser@$VMIP 'cat /var/log/dpi-simulator.log'"
    }
}

function Stop-Simulator {
    <#
    .SYNOPSIS
        Останавливает DPI симулятор на VM
    #>
    
    if (-not (Test-SSHConnection)) { return }
    
    Write-Status "Stopping DPI Simulator..."
    
    $running = Invoke-SSHCommand -Command "pgrep -f 'dpi_simulator.py' > /dev/null && echo 'running'" -Silent
    if ($running -ne "running") {
        Write-Status "DPI Simulator is not running"
        return
    }
    
    Invoke-SSHCommand -Command "sudo pkill -f 'dpi_simulator.py'"
    
    Start-Sleep -Seconds 1
    
    $stillRunning = Invoke-SSHCommand -Command "pgrep -f 'dpi_simulator.py' > /dev/null && echo 'running'" -Silent
    if ($stillRunning -ne "running") {
        Write-Success "DPI Simulator stopped"
    } else {
        Write-Warn "Simulator still running, force killing..."
        Invoke-SSHCommand -Command "sudo pkill -9 -f 'dpi_simulator.py'"
    }
}

function Test-Readiness {
    <#
    .SYNOPSIS
        Проверяет готовность VM к тестированию DPI bypass
    #>
    
    Write-Host ""
    Write-Host "=== DPI Simulator Readiness Check ===" -ForegroundColor Cyan
    Write-Host ""
    
    $allPassed = $true
    
    # 1. Проверка VM
    Write-Status "Checking VM status..."
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if ($vm -and $vm.State -eq "Running") {
        Write-Success "[PASS] VM is running"
    } else {
        Write-Error "[FAIL] VM is not running"
        Write-Warn "  Run: .\setup-hyperv-vm.ps1 -Action Start"
        $allPassed = $false
    }
    
    # 2. Проверка сети хоста
    Write-Status "Checking host network..."
    $hostAdapter = Get-NetAdapter | Where-Object { $_.Name -like "*$SwitchName*" }
    if ($hostAdapter) {
        $hostIP = Get-NetIPAddress -InterfaceIndex $hostAdapter.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
        if ($hostIP -and $hostIP.IPAddress -eq "192.168.100.1") {
            Write-Success "[PASS] Host IP configured: 192.168.100.1"
        } else {
            Write-Error "[FAIL] Host IP not configured"
            $allPassed = $false
        }
    } else {
        Write-Error "[FAIL] Virtual switch adapter not found"
        $allPassed = $false
    }
    
    # 3. Проверка ping до VM
    Write-Status "Checking connectivity to VM..."
    $ping = Test-Connection -ComputerName $VMIP -Count 1 -Quiet -ErrorAction SilentlyContinue
    if ($ping) {
        Write-Success "[PASS] VM is reachable at $VMIP"
    } else {
        Write-Error "[FAIL] Cannot ping VM at $VMIP"
        Write-Warn "  Run: .\setup-hyperv-vm.ps1 -Action Configure"
        $allPassed = $false
    }
    
    # 4. Проверка SSH
    Write-Status "Checking SSH access..."
    if (Test-SSHConnection) {
        Write-Success "[PASS] SSH connection works"
    } else {
        $allPassed = $false
    }
    
    # 5. Проверка симулятора
    if ($allPassed) {
        Write-Status "Checking DPI Simulator..."
        $simExists = Invoke-SSHCommand -Command "test -f $RemoteSimPath/dpi_simulator.py && echo 'yes'" -Silent
        if ($simExists -eq "yes") {
            Write-Success "[PASS] DPI Simulator is deployed"
            
            # Проверяем запущен ли
            $running = Invoke-SSHCommand -Command "pgrep -f 'dpi_simulator.py' > /dev/null && echo 'running'" -Silent
            if ($running -eq "running") {
                Write-Success "[PASS] DPI Simulator is running"
            } else {
                Write-Warn "[WARN] DPI Simulator is not running"
                Write-Warn "  Run: .\setup-hyperv-vm.ps1 -Action StartSim"
            }
        } else {
            Write-Warn "[WARN] DPI Simulator not deployed"
            Write-Warn "  Run: .\setup-hyperv-vm.ps1 -Action Deploy"
        }
        
        # 6. Проверка iptables/nftables
        Write-Status "Checking traffic interception..."
        $iptables = Invoke-SSHCommand -Command "sudo iptables -t nat -L PREROUTING -n 2>/dev/null | grep -c REDIRECT" -Silent
        if ([int]$iptables -gt 0) {
            Write-Success "[PASS] Traffic redirection rules active"
        } else {
            Write-Warn "[WARN] No traffic redirection rules found"
            Write-Warn "  Simulator may need to set up iptables rules"
        }
    }
    
    # Итог
    Write-Host ""
    Write-Host "=== Summary ===" -ForegroundColor Cyan
    if ($allPassed) {
        Write-Success "All checks passed! Ready for DPI testing."
        Write-Host ""
        Write-Host "To test DPI bypass:" -ForegroundColor Yellow
        Write-Host "  1. Configure Windows to route traffic through 192.168.100.10"
        Write-Host "  2. Or set VM as gateway in Isolate settings"
        Write-Host "  3. Run Isolate strategy tests"
    } else {
        Write-Error "Some checks failed. Fix issues above before testing."
    }
}

# Main
switch ($Action) {
    "Create"   { Create-VM }
    "Start"    { Start-DPISimVM }
    "Stop"     { Stop-DPISimVM }
    "Remove"   { Remove-DPISimVM }
    "Status"   { Get-VMStatus }
    "GetIP"    { Get-VMIP }
    "Configure" { Configure-VMNetwork }
    "Deploy"   { Deploy-Simulator }
    "StartSim" { Start-Simulator }
    "StopSim"  { Stop-Simulator }
    "Test"     { Test-Readiness }
}
