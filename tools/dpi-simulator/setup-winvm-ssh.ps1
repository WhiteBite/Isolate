# Setup SSH key authentication on WindhawkTest VM
# Run this script INSIDE the VM via Hyper-V console

$pubKey = @"
ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCZcNOSAxqoccvvgTgRwjCXrHqgn8qmjyU02ZM/ZZQzyToFtN5v+an1UlYGyMFO2TvnbhDDkW4z7Wha3JiEt9eZlDPqzwF7zoq2dXdWWajSUnkhWCtZ3EfvVxf+W5KVeWyxZo6O84I2kv0vmSO0FRHgGz9q6XZDy3kcgvq2d1rjIwpBMDFMAMGIO7k9Lu2ak6DzBtxSFp+kTEcoUNyHic11yPWhxlf/wd+EzDTM6UFk67dc9rQhC7e2pAdG/Vi1PdtwCnC9JJgYayo7qw7fwuZgOcGj8dbDWTdy0DM9geUZ3yEAXDdcfS0VxMuI2sQGqdnlH6COtod0G2NUIviJmUjlKspNcc4qvR20sD2Dss0fbCHn4bdVyXejD2fG0iWtI0+FwQjr+RwZiHW6TTbY1B7wufTS7OLheLUvUbbSi5KE3UST+KFEuhKiHBA97hvpBKEa+/lSNSbnZL2Syzqv6/cVA26Rzn7EpE0tItmzIqQeBo
"@

# Create .ssh directory
$sshDir = "$env:USERPROFILE\.ssh"
if (-not (Test-Path $sshDir)) {
    New-Item -ItemType Directory -Path $sshDir -Force | Out-Null
    Write-Host "Created $sshDir"
}

# Write authorized_keys
$authKeysPath = "$sshDir\authorized_keys"
$pubKey | Out-File -FilePath $authKeysPath -Encoding ASCII -Force
Write-Host "Written SSH key to $authKeysPath"

# For Windows OpenSSH, admin keys go to ProgramData
$adminAuthKeys = "C:\ProgramData\ssh\administrators_authorized_keys"
$pubKey | Out-File -FilePath $adminAuthKeys -Encoding ASCII -Force

# Fix permissions on admin keys file
icacls $adminAuthKeys /inheritance:r /grant "Administrators:F" /grant "SYSTEM:F"
Write-Host "Written admin SSH key to $adminAuthKeys"

# Restart sshd
Restart-Service sshd
Write-Host "SSH service restarted"

Write-Host "`nDone! Test from host: ssh VM-test@192.168.100.20 hostname"
