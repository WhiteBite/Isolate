@echo off
:: Запуск Hyper-V MCP Server от имени администратора
:: Сервер будет доступен на http://127.0.0.1:3100

cd /d "%~dp0"

:: Проверяем права админа
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo Requesting administrator privileges...
    powershell -Command "Start-Process cmd -Verb RunAs -ArgumentList '/c cd /d \"%~dp0\" && python hyperv_mcp_server.py --port 3100'"
    exit /b
)

:: Уже админ - запускаем
python hyperv_mcp_server.py --port 3100
pause
