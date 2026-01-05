# Copy SSH key to DPI-Simulator VM
# Run this script, it will open SSH and you just need to enter password: dpi

$VM_IP = "192.168.100.10"
$VM_USER = "dpi"
$SSH_KEY = Get-Content "$env:USERPROFILE\.ssh\id_rsa.pub"

Write-Host "Copying SSH key to $VM_USER@$VM_IP" -ForegroundColor Cyan
Write-Host "Password is: dpi" -ForegroundColor Yellow
Write-Host ""

# Use ssh-copy-id if available, otherwise manual
$sshCopyId = Get-Command ssh-copy-id -ErrorAction SilentlyContinue
if ($sshCopyId) {
    ssh-copy-id -o StrictHostKeyChecking=no "$VM_USER@$VM_IP"
} else {
    # Manual method - pipe commands through ssh
    Write-Host "Enter password 'dpi' when prompted:" -ForegroundColor Yellow
    $commands = @"
mkdir -p ~/.ssh
chmod 700 ~/.ssh
echo '$SSH_KEY' >> ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
echo 'SSH key added successfully!'
"@
    echo $commands | ssh -o StrictHostKeyChecking=no "$VM_USER@$VM_IP" "cat | bash"
}

Write-Host ""
Write-Host "Testing passwordless SSH..." -ForegroundColor Cyan
ssh -o BatchMode=yes -o ConnectTimeout=5 "$VM_USER@$VM_IP" "echo 'SUCCESS: Passwordless SSH works!'"
