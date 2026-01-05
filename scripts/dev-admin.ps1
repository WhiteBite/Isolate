# Isolate Dev Server with Admin Rights
# Run: Right-click -> Run as Administrator, or from VS Code F5

$env:CARGO_HOME = "D:\SDKs\Rust\cargo"
$env:RUSTUP_HOME = "D:\SDKs\Rust\rustup"
$env:PATH = "D:\SDKs\Rust\cargo\bin;$env:PATH"

Set-Location $PSScriptRoot\..
Write-Host "Starting Tauri dev server with admin rights..." -ForegroundColor Cyan
Write-Host "Working directory: $(Get-Location)" -ForegroundColor Gray

pnpm tauri dev

Write-Host "`nPress any key to exit..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
