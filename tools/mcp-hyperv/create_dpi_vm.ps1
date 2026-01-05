# Create DPI-Simulator VM with auto-configuration
# Run as Administrator

$VMName = "DPI-Simulator"
$VMPath = "$env:USERPROFILE\Hyper-V\$VMName"
$VHDPath = "$VMPath\$VMName.vhdx"
$CloudInitISO = "$VMPath\cloud-init.iso"
$SwitchName = "DPI-Internal"
$HostIP = "192.168.100.1"
$VMIP = "192.168.100.10"

# Create directory
New-Item -ItemType Directory -Path $VMPath -Force | Out-Null

# Create cloud-init ISO with network config and SSH key
$MetaData = @"
instance-id: dpi-simulator
local-hostname: dpi
"@

$UserData = @"
#cloud-config
hostname: dpi
users:
  - name: root
    lock_passwd: false
    plain_text_passwd: dpi
    ssh_authorized_keys:
      - $(Get-Content "$env:USERPROFILE\.ssh\id_rsa.pub")
ssh_pwauth: true
disable_root: false
chpasswd:
  expire: false
runcmd:
  - echo "PermitRootLogin yes" >> /etc/ssh/sshd_config
  - systemctl restart sshd
"@

$NetworkConfig = @"
version: 2
ethernets:
  eth0:
    addresses: [$VMIP/24]
    gateway4: $HostIP
    nameservers:
      addresses: [8.8.8.8]
"@


# Save cloud-init files
$ciDir = "$VMPath\cloud-init"
New-Item -ItemType Directory -Path $ciDir -Force | Out-Null
$MetaData | Out-File -FilePath "$ciDir\meta-data" -Encoding ascii -NoNewline
$UserData | Out-File -FilePath "$ciDir\user-data" -Encoding ascii -NoNewline
$NetworkConfig | Out-File -FilePath "$ciDir\network-config" -Encoding ascii -NoNewline

# Download Ubuntu cloud image if not exists
$CloudImage = "$VMPath\ubuntu-cloud.img"
if (-not (Test-Path $CloudImage)) {
    Write-Host "Downloading Ubuntu cloud image..."
    $url = "https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.img"
    Invoke-WebRequest -Uri $url -OutFile $CloudImage
}

# Convert to VHDX
Write-Host "Converting to VHDX..."
if (Test-Path $VHDPath) { Remove-Item $VHDPath -Force }

# Use qemu-img if available, otherwise use raw copy
$qemu = Get-Command qemu-img -ErrorAction SilentlyContinue
if ($qemu) {
    & qemu-img convert -f qcow2 -O vhdx $CloudImage $VHDPath
} else {
    # Download and use qemu-img
    Write-Host "Installing qemu-img..."
    winget install --id=SoftwareFreedomConservancy.QEMU -e --accept-source-agreements --accept-package-agreements 2>$null
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine")
    & "C:\Program Files\qemu\qemu-img.exe" convert -f qcow2 -O vhdx $CloudImage $VHDPath
}

# Resize VHDX to 20GB
Resize-VHD -Path $VHDPath -SizeBytes 20GB

# Create cloud-init ISO using oscdimg or genisoimage
Write-Host "Creating cloud-init ISO..."
$oscdimg = "C:\Program Files (x86)\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
if (Test-Path $oscdimg) {
    & $oscdimg -j2 -lcidata $ciDir $CloudInitISO
} else {
    Write-Host "WARNING: oscdimg not found. Install Windows ADK or create ISO manually."
}

# Create VM
Write-Host "Creating VM..."
New-VM -Name $VMName -MemoryStartupBytes 2GB -Generation 2 -VHDPath $VHDPath -SwitchName $SwitchName
Set-VM -Name $VMName -ProcessorCount 2 -DynamicMemory -MemoryMinimumBytes 512MB -MemoryMaximumBytes 4GB
Set-VMFirmware -VMName $VMName -EnableSecureBoot Off

# Add cloud-init ISO
if (Test-Path $CloudInitISO) {
    Add-VMDvdDrive -VMName $VMName -Path $CloudInitISO
}

# Add second NIC for internet
Add-VMNetworkAdapter -VMName $VMName -SwitchName "Default Switch" -Name "Internet"

Write-Host "Starting VM..."
Start-VM -Name $VMName

Write-Host "Done! VM will auto-configure. Wait 2-3 minutes then test SSH:"
Write-Host "ssh root@$VMIP"
