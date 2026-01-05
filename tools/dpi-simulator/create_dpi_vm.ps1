# Create DPI-Simulator VM with Ubuntu Server
# Автоматическая установка через cloud-init

param(
    [string]$VMName = "DPI-Simulator",
    [string]$ISOPath = "C:\Users\Mind\Downloads\ubuntu-22.04.5-live-server-amd64.iso",
    [string]$VMPath = "C:\Hyper-V\VMs",
    [string]$VHDPath = "C:\Hyper-V\VHDs",
    [int]$MemoryMB = 2048,
    [int]$CPUs = 2,
    [int]$DiskGB = 20,
    [string]$InternalSwitch = "DPI-Internal",
    [string]$ExternalSwitch = "Default Switch",
    [string]$StaticIP = "192.168.100.10",
    [string]$Gateway = "192.168.100.1",
    [string]$SSHPubKeyPath = "C:\Users\Mind\.ssh\id_rsa.pub"
)

$ErrorActionPreference = "Stop"

Write-Host "=== DPI-Simulator VM Creator ===" -ForegroundColor Cyan

# 1. Проверяем ISO
if (-not (Test-Path $ISOPath)) {
    Write-Host "ERROR: ISO not found at $ISOPath" -ForegroundColor Red
    Write-Host "Checking BITS download status..." -ForegroundColor Yellow
    Get-BitsTransfer | Where-Object { $_.DisplayName -like "*Ubuntu*" } | Format-Table DisplayName, JobState, @{N='Progress';E={[math]::Round($_.BytesTransferred/$_.BytesTotal*100,1)}}
    exit 1
}

# 2. Проверяем/создаём Internal Switch
$switch = Get-VMSwitch -Name $InternalSwitch -ErrorAction SilentlyContinue
if (-not $switch) {
    Write-Host "Creating Internal Switch: $InternalSwitch" -ForegroundColor Yellow
    New-VMSwitch -Name $InternalSwitch -SwitchType Internal
    
    # Настраиваем IP на хосте
    $adapter = Get-NetAdapter | Where-Object { $_.Name -like "*$InternalSwitch*" }
    if ($adapter) {
        New-NetIPAddress -InterfaceIndex $adapter.ifIndex -IPAddress $Gateway -PrefixLength 24 -ErrorAction SilentlyContinue
    }
}

# 3. Удаляем старую VM если есть
$existingVM = Get-VM -Name $VMName -ErrorAction SilentlyContinue
if ($existingVM) {
    Write-Host "Removing existing VM: $VMName" -ForegroundColor Yellow
    Stop-VM -Name $VMName -Force -ErrorAction SilentlyContinue
    Remove-VM -Name $VMName -Force
}

# 4. Создаём директории
New-Item -ItemType Directory -Path $VMPath -Force | Out-Null
New-Item -ItemType Directory -Path $VHDPath -Force | Out-Null

# 5. Создаём VHD
$vhdFile = Join-Path $VHDPath "$VMName.vhdx"
if (Test-Path $vhdFile) {
    Remove-Item $vhdFile -Force
}
Write-Host "Creating VHD: $vhdFile ($DiskGB GB)" -ForegroundColor Yellow
New-VHD -Path $vhdFile -SizeBytes ($DiskGB * 1GB) -Dynamic | Out-Null

# 6. Создаём VM
Write-Host "Creating VM: $VMName" -ForegroundColor Yellow
New-VM -Name $VMName -Path $VMPath -MemoryStartupBytes ($MemoryMB * 1MB) -Generation 2 -VHDPath $vhdFile | Out-Null

# 7. Настраиваем VM
Set-VM -Name $VMName -ProcessorCount $CPUs -AutomaticCheckpointsEnabled $false
Set-VMFirmware -VMName $VMName -EnableSecureBoot Off

# 8. Добавляем сетевые адаптеры
# Удаляем дефолтный адаптер
Get-VMNetworkAdapter -VMName $VMName | Remove-VMNetworkAdapter

# Добавляем Internal (для DPI тестов)
Add-VMNetworkAdapter -VMName $VMName -Name "Internal" -SwitchName $InternalSwitch
# Добавляем External (для интернета/установки)
Add-VMNetworkAdapter -VMName $VMName -Name "External" -SwitchName $ExternalSwitch

# 9. Подключаем ISO
Write-Host "Attaching ISO: $ISOPath" -ForegroundColor Yellow
Add-VMDvdDrive -VMName $VMName -Path $ISOPath

# 10. Настраиваем порядок загрузки (DVD первым)
$dvd = Get-VMDvdDrive -VMName $VMName
$hdd = Get-VMHardDiskDrive -VMName $VMName
Set-VMFirmware -VMName $VMName -BootOrder $dvd, $hdd

# 11. Выводим информацию
Write-Host ""
Write-Host "=== VM Created Successfully ===" -ForegroundColor Green
Write-Host "Name:     $VMName"
Write-Host "Memory:   $MemoryMB MB"
Write-Host "CPUs:     $CPUs"
Write-Host "Disk:     $DiskGB GB"
Write-Host "Networks: $InternalSwitch, $ExternalSwitch"
Write-Host ""
Write-Host "=== Next Steps ===" -ForegroundColor Cyan
Write-Host "1. Start VM:  Start-VM -Name '$VMName'"
Write-Host "2. Connect:   vmconnect localhost '$VMName'"
Write-Host "3. Install Ubuntu Server with these settings:"
Write-Host "   - Network: Configure eth0 (Internal) with static IP $StaticIP/24, gateway $Gateway"
Write-Host "   - Network: eth1 (External) - DHCP for internet"
Write-Host "   - Install OpenSSH server"
Write-Host "   - Username: dpi, Password: dpi"
Write-Host ""
Write-Host "4. After install, copy SSH key:"
Write-Host "   ssh-copy-id dpi@$StaticIP"
Write-Host ""

# Опционально: запускаем VM
$start = Read-Host "Start VM now? (y/n)"
if ($start -eq 'y') {
    Start-VM -Name $VMName
    Write-Host "VM started. Opening console..." -ForegroundColor Green
    vmconnect localhost $VMName
}
