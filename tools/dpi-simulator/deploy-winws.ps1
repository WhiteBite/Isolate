# Deploy winws to WindhawkTest VM
# Usage: .\deploy-winws.ps1

$ErrorActionPreference = "Stop"

$WINWS_SOURCE = "D:\Sources\StartUp\Isolate\thirdparty\zapret\bin\win64"
$VM_HOST = "VM-test@192.168.100.20"
$VM_PATH = "C:\Tools\winws"

Write-Host "Deploying winws to $VM_HOST..."

# Создаём папку на VM
ssh $VM_HOST "if not exist $VM_PATH mkdir $VM_PATH"

# Копируем файлы
$files = @(
    "winws.exe",
    "WinDivert.dll", 
    "WinDivert64.sys"
)

foreach ($file in $files) {
    $src = Join-Path $WINWS_SOURCE $file
    if (Test-Path $src) {
        Write-Host "Copying $file..."
        scp -o BatchMode=yes $src "${VM_HOST}:${VM_PATH}\"
    } else {
        Write-Warning "File not found: $src"
    }
}

# Проверяем
Write-Host "`nVerifying deployment..."
ssh $VM_HOST "dir $VM_PATH"

Write-Host "`nDone! winws deployed to $VM_PATH"
