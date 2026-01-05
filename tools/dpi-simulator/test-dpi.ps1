# Test DPI blocking from inside VM
param(
    [string]$Domain = "youtube.com",
    [int]$Timeout = 5
)

$vmHost = "dpi@192.168.100.10"

Write-Host "=== DPI Simulator Test ===" -ForegroundColor Cyan
Write-Host ""

# Reset stats
Write-Host "Resetting stats..." -ForegroundColor Yellow
Invoke-RestMethod -Uri "http://localhost:8888/reset-stats" -Method POST | Out-Null

# Get current mode
$status = Invoke-RestMethod -Uri "http://localhost:8888/status"
Write-Host "Mode: $($status.mode)" -ForegroundColor Yellow
Write-Host "Blocked domains: $($status.blocked_domains_count)" -ForegroundColor Yellow
Write-Host ""

# Test blocked domain
Write-Host "Testing BLOCKED domain: $Domain" -ForegroundColor Red
$result = ssh $vmHost "curl -s --connect-timeout $Timeout -o /dev/null -w '%{http_code}' https://$Domain 2>&1 || echo 'TIMEOUT'"
Write-Host "Result: $result"
Write-Host ""

# Test allowed domain
Write-Host "Testing ALLOWED domain: google.com" -ForegroundColor Green  
$result = ssh $vmHost "curl -s --connect-timeout $Timeout -o /dev/null -w '%{http_code}' https://google.com 2>&1"
Write-Host "Result: $result"
Write-Host ""

# Get stats
$stats = Invoke-RestMethod -Uri "http://localhost:8888/stats"
Write-Host "=== Statistics ===" -ForegroundColor Cyan
Write-Host "Total packets: $($stats.total_packets)"
Write-Host "Blocked SNI:   $($stats.blocked_sni)" -ForegroundColor Red
Write-Host "Blocked HTTP:  $($stats.blocked_http)" -ForegroundColor Red
Write-Host "Blocked QUIC:  $($stats.blocked_quic)" -ForegroundColor Red
Write-Host "Passed:        $($stats.passed)" -ForegroundColor Green

if ($stats.domain_metrics) {
    Write-Host ""
    Write-Host "Per-domain:" -ForegroundColor Yellow
    $stats.domain_metrics.PSObject.Properties | ForEach-Object {
        Write-Host "  $($_.Name): blocked=$($_.Value.blocked), passed=$($_.Value.passed)"
    }
}
