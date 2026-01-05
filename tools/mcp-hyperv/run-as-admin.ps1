# Запускает MCP сервер от имени администратора
# Использование: .\run-as-admin.ps1

$scriptPath = $PSScriptRoot
$pythonScript = Join-Path $scriptPath "hyperv_mcp_server.py"

# Проверяем права админа
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "Requesting administrator privileges..." -ForegroundColor Yellow
    Start-Process powershell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -Command `"cd '$scriptPath'; python '$pythonScript'`""
    exit
}

# Уже админ - запускаем напрямую
Write-Host "Starting Hyper-V MCP Server as Administrator..." -ForegroundColor Green
python $pythonScript
