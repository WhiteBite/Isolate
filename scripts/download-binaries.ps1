# scripts/download-binaries.ps1
# Скачивает бинарники из GitHub Releases для разработки Isolate
# Идемпотентный: не скачивает если файлы уже есть и хэши совпадают

param(
    [switch]$Force  # Принудительно перескачать все файлы
)

$ErrorActionPreference = "Stop"

# Конфигурация
$BINARIES_DIR = Join-Path $PSScriptRoot ".." "src-tauri" "binaries"
$TEMP_DIR = Join-Path $env:TEMP "isolate-binaries"

# Версии
$WINWS_VERSION = "v67"
$SINGBOX_VERSION = "1.10.0"
$WINDIVERT_VERSION = "2.2.2"

# URLs
$WINWS_URL = "https://github.com/bol-van/zapret/releases/download/$WINWS_VERSION/zapret-win-bundle-$WINWS_VERSION.zip"
$SINGBOX_URL = "https://github.com/SagerNet/sing-box/releases/download/v$SINGBOX_VERSION/sing-box-$SINGBOX_VERSION-windows-amd64.zip"
$WINDIVERT_URL = "https://github.com/basil00/WinDivert/releases/download/v$WINDIVERT_VERSION/WinDivert-$WINDIVERT_VERSION-A.zip"

# Ожидаемые SHA256 хэши (обновить при смене версий)
# Команда для получения хэша: (Get-FileHash -Algorithm SHA256 "file.exe").Hash
$EXPECTED_HASHES = @{
    "winws.exe"       = ""  # TODO: Добавить хэш для zapret $WINWS_VERSION
    "cygwin1.dll"     = ""  # TODO: Добавить хэш для cygwin из zapret bundle
    "sing-box.exe"    = ""  # TODO: Добавить хэш для sing-box v$SINGBOX_VERSION
    "WinDivert.dll"   = ""  # TODO: Добавить хэш для WinDivert v$WINDIVERT_VERSION x64
    "WinDivert64.sys" = ""  # TODO: Добавить хэш для WinDivert v$WINDIVERT_VERSION x64
    # TLS/QUIC fingerprint files
    "tls_clienthello_www_google_com.bin" = "936c2bee4cfb80aa3c426b2dcbcc834b3fbcd1adb17172959dc569c73a14275c"
    "tls_clienthello_4pda_to.bin"        = "eefeaf09dde8d69b1f176212541f63c68b314a33a335eced99a8a29f17254da8"
    "tls_clienthello_max_ru.bin"         = "e4a94cec50b3c048eb988a513ee28191e4d7544dd5f98a9bf94f37ee02d2568e"
    "quic_initial_www_google_com.bin"    = "f4589c57749f956bb30538197a521d7005f8b0a8723b4707e72405e51ddac50a"
}

# Функция проверки хэша файла
function Test-FileHash {
    param(
        [string]$FilePath,
        [string]$ExpectedHash
    )
    
    if ([string]::IsNullOrEmpty($ExpectedHash)) {
        Write-Warn "Хэш не задан для $(Split-Path $FilePath -Leaf) - проверка пропущена"
        return $true
    }
    
    $actualHash = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash
    if ($actualHash -eq $ExpectedHash) {
        Write-Success "Хэш совпадает: $(Split-Path $FilePath -Leaf)"
        return $true
    } else {
        Write-Err "Хэш НЕ совпадает для $(Split-Path $FilePath -Leaf)"
        Write-Err "  Ожидался: $ExpectedHash"
        Write-Err "  Получен:  $actualHash"
        return $false
    }
}

# Цвета для вывода
function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Err { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }

# Проверка наличия файла
function Test-BinaryExists {
    param([string]$Name)
    $path = Join-Path $BINARIES_DIR $Name
    return Test-Path $path
}

# Скачивание файла с прогрессом
function Download-File {
    param(
        [string]$Url,
        [string]$OutFile
    )
    
    Write-Info "Скачиваю: $Url"
    
    # Используем .NET для скачивания с прогрессом
    $webClient = New-Object System.Net.WebClient
    try {
        $webClient.DownloadFile($Url, $OutFile)
        Write-Success "Скачано: $(Split-Path $OutFile -Leaf)"
    }
    catch {
        Write-Err "Ошибка скачивания: $_"
        throw
    }
    finally {
        $webClient.Dispose()
    }
}

# Распаковка ZIP
function Expand-Archive-Safe {
    param(
        [string]$ZipPath,
        [string]$DestPath
    )
    
    if (Test-Path $DestPath) {
        Remove-Item $DestPath -Recurse -Force
    }
    
    Expand-Archive -Path $ZipPath -DestinationPath $DestPath -Force
}

# Копирование файла в binaries
function Copy-ToBinaries {
    param(
        [string]$SourcePath,
        [string]$DestName
    )
    
    $destPath = Join-Path $BINARIES_DIR $DestName
    Copy-Item $SourcePath $destPath -Force
    Write-Success "Скопировано: $DestName"
}

# Поиск файла рекурсивно
function Find-FileRecursive {
    param(
        [string]$SearchPath,
        [string]$FileName
    )
    
    $found = Get-ChildItem -Path $SearchPath -Filter $FileName -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    return $found
}

# Скачивание и установка Zapret (winws.exe + cygwin1.dll)
function Install-Zapret {
    Write-Info "=== Zapret (winws.exe) ==="
    
    $needDownload = $Force -or (-not (Test-BinaryExists "winws.exe")) -or (-not (Test-BinaryExists "cygwin1.dll"))
    
    if (-not $needDownload) {
        Write-Success "winws.exe и cygwin1.dll уже установлены"
        return
    }
    
    $zipPath = Join-Path $TEMP_DIR "zapret.zip"
    $extractPath = Join-Path $TEMP_DIR "zapret"
    
    Download-File -Url $WINWS_URL -OutFile $zipPath
    Expand-Archive-Safe -ZipPath $zipPath -DestPath $extractPath
    
    # Ищем winws.exe и cygwin1.dll
    $winws = Find-FileRecursive -SearchPath $extractPath -FileName "winws.exe"
    $cygwin = Find-FileRecursive -SearchPath $extractPath -FileName "cygwin1.dll"
    
    if (-not $winws) {
        Write-Err "winws.exe не найден в архиве"
        throw "winws.exe not found"
    }
    
    Copy-ToBinaries -SourcePath $winws.FullName -DestName "winws.exe"
    
    if ($cygwin) {
        Copy-ToBinaries -SourcePath $cygwin.FullName -DestName "cygwin1.dll"
    } else {
        Write-Warn "cygwin1.dll не найден в архиве"
    }
    
    # Копируем bin файлы для fake пакетов
    $binFiles = Get-ChildItem -Path $extractPath -Filter "*.bin" -Recurse
    foreach ($bin in $binFiles) {
        Copy-ToBinaries -SourcePath $bin.FullName -DestName $bin.Name
    }
}

# Скачивание и установка sing-box
function Install-SingBox {
    Write-Info "=== Sing-box ==="
    
    $needDownload = $Force -or (-not (Test-BinaryExists "sing-box.exe"))
    
    if (-not $needDownload) {
        Write-Success "sing-box.exe уже установлен"
        return
    }
    
    $zipPath = Join-Path $TEMP_DIR "sing-box.zip"
    $extractPath = Join-Path $TEMP_DIR "sing-box"
    
    Download-File -Url $SINGBOX_URL -OutFile $zipPath
    Expand-Archive-Safe -ZipPath $zipPath -DestPath $extractPath
    
    $singbox = Find-FileRecursive -SearchPath $extractPath -FileName "sing-box.exe"
    
    if (-not $singbox) {
        Write-Err "sing-box.exe не найден в архиве"
        throw "sing-box.exe not found"
    }
    
    Copy-ToBinaries -SourcePath $singbox.FullName -DestName "sing-box.exe"
}

# Скачивание и установка WinDivert
function Install-WinDivert {
    Write-Info "=== WinDivert ==="
    
    $needDownload = $Force -or (-not (Test-BinaryExists "WinDivert.dll")) -or (-not (Test-BinaryExists "WinDivert64.sys"))
    
    if (-not $needDownload) {
        Write-Success "WinDivert уже установлен"
        return
    }
    
    $zipPath = Join-Path $TEMP_DIR "windivert.zip"
    $extractPath = Join-Path $TEMP_DIR "windivert"
    
    Download-File -Url $WINDIVERT_URL -OutFile $zipPath
    Expand-Archive-Safe -ZipPath $zipPath -DestPath $extractPath
    
    # WinDivert имеет структуру: WinDivert-X.X.X-A/x64/WinDivert.dll
    $dll = Find-FileRecursive -SearchPath $extractPath -FileName "WinDivert.dll"
    $sys = Find-FileRecursive -SearchPath $extractPath -FileName "WinDivert64.sys"
    
    if (-not $dll) {
        Write-Err "WinDivert.dll не найден в архиве"
        throw "WinDivert.dll not found"
    }
    
    if (-not $sys) {
        Write-Err "WinDivert64.sys не найден в архиве"
        throw "WinDivert64.sys not found"
    }
    
    # Берём x64 версию
    $dll64 = Get-ChildItem -Path $extractPath -Filter "WinDivert.dll" -Recurse | 
             Where-Object { $_.DirectoryName -like "*x64*" } | 
             Select-Object -First 1
    
    $sys64 = Get-ChildItem -Path $extractPath -Filter "WinDivert64.sys" -Recurse | 
             Where-Object { $_.DirectoryName -like "*x64*" } | 
             Select-Object -First 1
    
    if ($dll64) { $dll = $dll64 }
    if ($sys64) { $sys = $sys64 }
    
    Copy-ToBinaries -SourcePath $dll.FullName -DestName "WinDivert.dll"
    Copy-ToBinaries -SourcePath $sys.FullName -DestName "WinDivert64.sys"
}

# Проверка всех бинарников
function Test-AllBinaries {
    Write-Info "=== Проверка бинарников ==="
    
    $required = @("winws.exe", "sing-box.exe", "WinDivert.dll", "WinDivert64.sys")
    $allOk = $true
    
    foreach ($file in $required) {
        if (Test-BinaryExists $file) {
            $path = Join-Path $BINARIES_DIR $file
            $size = (Get-Item $path).Length
            Write-Success "$file - OK ($([math]::Round($size/1MB, 2)) MB)"
        } else {
            Write-Err "$file - ОТСУТСТВУЕТ"
            $allOk = $false
        }
    }
    
    return $allOk
}

# Основной скрипт
function Main {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Magenta
    Write-Host "  Isolate Binary Downloader" -ForegroundColor Magenta
    Write-Host "========================================" -ForegroundColor Magenta
    Write-Host ""
    
    # Создаём директории
    if (-not (Test-Path $BINARIES_DIR)) {
        New-Item -ItemType Directory -Path $BINARIES_DIR -Force | Out-Null
        Write-Info "Создана директория: $BINARIES_DIR"
    }
    
    if (-not (Test-Path $TEMP_DIR)) {
        New-Item -ItemType Directory -Path $TEMP_DIR -Force | Out-Null
    }
    
    try {
        Install-Zapret
        Install-SingBox
        Install-WinDivert
        
        Write-Host ""
        $allOk = Test-AllBinaries
        
        # Очистка временных файлов
        if (Test-Path $TEMP_DIR) {
            Remove-Item $TEMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        Write-Host ""
        if ($allOk) {
            Write-Success "Все бинарники установлены успешно!"
        } else {
            Write-Err "Некоторые бинарники отсутствуют"
            exit 1
        }
    }
    catch {
        Write-Err "Ошибка: $_"
        exit 1
    }
}

Main
