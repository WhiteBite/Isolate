# Enable routing through DPI-Simulator VM
# Run as Administrator

$vmGateway = "192.168.100.10"
$interfaceIndex = (Get-NetAdapter | Where-Object { $_.Name -like "*DPI*" -or $_.InterfaceDescription -like "*Hyper-V*Internal*" } | Select-Object -First 1).ifIndex

if (-not $interfaceIndex) {
    # Find by IP
    $interfaceIndex = (Get-NetIPAddress -IPAddress "192.168.100.1" -ErrorAction SilentlyContinue).InterfaceIndex
}

if (-not $interfaceIndex) {
    Write-Error "DPI-Internal interface not found"
    exit 1
}

Write-Host "Using interface index: $interfaceIndex"

# Add routes for blocked domains (resolve to IPs)
$domains = @(
    "youtube.com",
    "www.youtube.com", 
    "googlevideo.com",
    "discord.com",
    "gateway.discord.gg"
)

foreach ($domain in $domains) {
    try {
        $ips = [System.Net.Dns]::GetHostAddresses($domain) | Where-Object { $_.AddressFamily -eq 'InterNetwork' }
        foreach ($ip in $ips) {
            Write-Host "Adding route: $ip ($domain) -> $vmGateway"
            route add $ip.IPAddressToString mask 255.255.255.255 $vmGateway metric 1 2>$null
        }
    } catch {
        Write-Warning "Could not resolve $domain"
    }
}

Write-Host "`nDPI routing enabled. Test with:"
Write-Host "  curl -v --connect-timeout 5 https://youtube.com"
Write-Host "`nTo disable: .\disable-dpi-route.ps1"
