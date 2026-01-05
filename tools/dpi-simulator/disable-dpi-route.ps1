# Disable routing through DPI-Simulator VM
# Run as Administrator

$vmGateway = "192.168.100.10"

# Remove all routes through VM gateway
$routes = route print | Select-String $vmGateway
foreach ($route in $routes) {
    $parts = $route.Line.Trim() -split '\s+'
    if ($parts.Count -ge 1) {
        $dest = $parts[0]
        if ($dest -match '^\d+\.\d+\.\d+\.\d+$') {
            Write-Host "Removing route: $dest"
            route delete $dest 2>$null
        }
    }
}

Write-Host "DPI routing disabled"
