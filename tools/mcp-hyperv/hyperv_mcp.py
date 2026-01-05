#!/usr/bin/env python3
"""
Hyper-V MCP Server для Kiro.

Архитектура:
- MCP сервер работает через stdio (запускается Kiro)
- Админские команды выполняются через named pipe к демону (hyperv_daemon.py)
- Демон запускается отдельно от админа

Использование:
1. Запустить демон от админа: python hyperv_daemon.py
2. MCP сервер подключается автоматически через Kiro
"""
import sys
import json
import subprocess
import time
import os

PIPE_NAME = r"\\.\pipe\hyperv_mcp"
PIPE_TIMEOUT = 30000  # 30 секунд

def send_to_daemon(command: str, args: dict = None) -> dict:
    """Отправляет команду демону через named pipe."""
    try:
        import win32file
        import win32pipe
        import pywintypes
        
        # Подключаемся к pipe
        handle = win32file.CreateFile(
            PIPE_NAME,
            win32file.GENERIC_READ | win32file.GENERIC_WRITE,
            0, None,
            win32file.OPEN_EXISTING,
            0, None
        )
        
        # Отправляем запрос
        request = json.dumps({"command": command, "args": args or {}})
        win32file.WriteFile(handle, request.encode())
        
        # Читаем ответ
        result, data = win32file.ReadFile(handle, 65536)
        win32file.CloseHandle(handle)
        
        return json.loads(data.decode())
    except ImportError:
        # Fallback без pywin32 - используем PowerShell
        return run_powershell_fallback(command, args)
    except Exception as e:
        return {"error": f"Daemon connection failed: {e}. Run hyperv_daemon.py as admin."}

def run_powershell_fallback(command: str, args: dict = None) -> dict:
    """Fallback: выполняет PowerShell напрямую (работает если уже админ)."""
    ps_commands = {
        "status": "Get-Service vmms | Select Status; Get-VM | Select Name,State | ConvertTo-Json -Compress",
        "vm_list": """Get-VM | ForEach-Object { $vm=$_; $na=Get-VMNetworkAdapter -VMName $vm.Name | Select Name,SwitchName,IPAddresses; [PSCustomObject]@{Name=$vm.Name;State=$vm.State.ToString();CPU=$vm.ProcessorCount;MemMB=[int]($vm.MemoryAssigned/1MB);Net=$na} } | ConvertTo-Json -Depth 3 -Compress""",
        "vm_start": f"Start-VM -Name '{args.get('name','')}' -EA Stop; 'OK'",
        "vm_stop": f"Stop-VM -Name '{args.get('name','')}' -Force -EA Stop; 'OK'",
        "network_status": "Get-VMSwitch | Select Name,SwitchType | ConvertTo-Json -Compress",
    }
    
    ps_cmd = ps_commands.get(command)
    if not ps_cmd:
        return {"error": f"Unknown command: {command}"}
    
    try:
        result = subprocess.run(
            ["powershell", "-NoProfile", "-Command", ps_cmd],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0:
            return {"output": result.stdout.strip()}
        return {"error": result.stderr.strip()}
    except subprocess.TimeoutExpired:
        return {"error": "Timeout"}
    except Exception as e:
        return {"error": str(e)}


# MCP Protocol Implementation
TOOLS = [
    {"name": "hyperv_status", "description": "Статус Hyper-V сервиса и VM", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "hyperv_vm_list", "description": "Список всех VM", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "hyperv_vm_start", "description": "Запустить VM", "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}}, "required": ["name"]}},
    {"name": "hyperv_vm_stop", "description": "Остановить VM", "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}}, "required": ["name"]}},
    {"name": "hyperv_network_status", "description": "Статус сети", "inputSchema": {"type": "object", "properties": {}}},
    {"name": "vm_ssh_exec", "description": "Выполнить команду на VM через SSH", 
     "inputSchema": {"type": "object", "properties": {
       "command": {"type": "string", "description": "Команда для выполнения"},
       "host": {"type": "string", "description": "SSH хост (default: VM-test@192.168.100.20)"}
     }, "required": ["command"]}},
    {"name": "dpi_status", "description": "Статус DPI-симулятора", 
     "inputSchema": {"type": "object", "properties": {}}},
    {"name": "dpi_stats", "description": "Статистика блокировок DPI", 
     "inputSchema": {"type": "object", "properties": {}}},
    {"name": "dpi_reset_stats", "description": "Сбросить статистику DPI", 
     "inputSchema": {"type": "object", "properties": {}}},
    {"name": "dpi_test_block", "description": "Проверить блокировку домена через DPI", 
     "inputSchema": {"type": "object", "properties": {
       "domain": {"type": "string", "description": "Домен для проверки (default: youtube.com)"}
     }, "required": []}},
    {"name": "powershell_exec", "description": "Выполнить PowerShell команду", "inputSchema": {"type": "object", "properties": {"command": {"type": "string"}}, "required": ["command"]}},
    {"name": "winws_deploy", "description": "Развернуть winws на тестовой VM", 
     "inputSchema": {"type": "object", "properties": {
       "local_path": {"type": "string", "description": "Путь к winws.exe на хосте"}
     }}},
    {"name": "winws_start", "description": "Запустить winws на VM с параметрами стратегии", 
     "inputSchema": {"type": "object", "properties": {
       "args": {"type": "string", "description": "Аргументы winws (например: --wf-tcp=80,443)"}
     }, "required": ["args"]}},
    {"name": "winws_stop", "description": "Остановить winws на VM", 
     "inputSchema": {"type": "object", "properties": {}}},
    {"name": "dpi_test_domain", "description": "Проверить доступность домена через DPI", 
     "inputSchema": {"type": "object", "properties": {
       "domain": {"type": "string", "description": "Домен для проверки (default: youtube.com)"}
     }}},
    {"name": "dpi_full_test", "description": "Полный тест стратегии: блокировка → запуск winws → проверка обхода → остановка", 
     "inputSchema": {"type": "object", "properties": {
       "args": {"type": "string", "description": "Аргументы winws"},
       "domain": {"type": "string", "description": "Домен для теста"}
     }, "required": ["args"]}},
]

def handle_tool_call(name: str, args: dict) -> dict:
    """Обрабатывает вызов инструмента."""
    if name == "hyperv_status":
        return send_to_daemon("status")
    elif name == "hyperv_vm_list":
        return send_to_daemon("vm_list")
    elif name == "hyperv_vm_start":
        return send_to_daemon("vm_start", args)
    elif name == "hyperv_vm_stop":
        return send_to_daemon("vm_stop", args)
    elif name == "hyperv_network_status":
        return send_to_daemon("network_status")
    elif name == "vm_ssh_exec":
        return send_to_daemon("ssh", args)
    elif name == "dpi_status":
        return send_to_daemon("dpi_api", {"endpoint": "status"})
    elif name == "dpi_stats":
        return send_to_daemon("dpi_api", {"endpoint": "stats"})
    elif name == "dpi_reset_stats":
        return send_to_daemon("dpi_api", {"endpoint": "reset-stats", "method": "POST"})
    elif name == "dpi_test_block":
        domain = args.get("domain", "youtube.com")
        return send_to_daemon("ssh", {"command": f"curl.exe -s --connect-timeout 5 -o NUL -w \"%{{http_code}}\" https://{domain}"})
    elif name == "powershell_exec":
        return send_to_daemon("powershell", args)
    elif name == "winws_deploy":
        return send_to_daemon("winws_deploy", args)
    elif name == "winws_start":
        return send_to_daemon("winws_start", args)
    elif name == "winws_stop":
        return send_to_daemon("winws_stop", args)
    elif name == "dpi_test_domain":
        return send_to_daemon("dpi_test_domain", args)
    elif name == "dpi_full_test":
        # Полный цикл тестирования
        domain = args.get("domain", "youtube.com")
        winws_args = args.get("args", "")
        
        # 1. Проверяем блокировку
        before = send_to_daemon("dpi_test_domain", {"domain": domain})
        if not before.get("blocked"):
            return {"error": f"Domain {domain} is not blocked, cannot test bypass"}
        
        # 2. Запускаем winws
        start = send_to_daemon("winws_start", {"args": winws_args})
        if not start.get("success"):
            return {"error": f"Failed to start winws: {start}"}
        
        time.sleep(2)  # Ждём применения
        
        # 3. Проверяем обход
        after = send_to_daemon("dpi_test_domain", {"domain": domain})
        
        # 4. Останавливаем winws
        send_to_daemon("winws_stop", {})
        
        return {
            "success": not after.get("blocked"),
            "domain": domain,
            "blocked_before": before.get("blocked"),
            "blocked_after": after.get("blocked"),
            "http_code_before": before.get("http_code"),
            "http_code_after": after.get("http_code")
        }
    return {"error": f"Unknown tool: {name}"}

def handle_request(request: dict) -> dict | None:
    """Обрабатывает MCP JSON-RPC запрос."""
    method = request.get("method")
    req_id = request.get("id")
    params = request.get("params", {})
    
    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "hyperv-mcp", "version": "1.0"}
            }
        }
    
    elif method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {"tools": TOOLS}
        }
    
    elif method == "tools/call":
        tool_name = params.get("name", "")
        tool_args = params.get("arguments", {})
        result = handle_tool_call(tool_name, tool_args)
        text = json.dumps(result, ensure_ascii=False, indent=2)
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {"content": [{"type": "text", "text": text}]}
        }
    
    elif method == "notifications/initialized":
        return None  # Notification - no response
    
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {"code": -32601, "message": f"Unknown method: {method}"}
    }

def main():
    """Основной цикл MCP сервера."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            response = handle_request(request)
            if response:
                sys.stdout.write(json.dumps(response) + "\n")
                sys.stdout.flush()
        except json.JSONDecodeError:
            pass
        except Exception as e:
            sys.stderr.write(f"Error: {e}\n")

if __name__ == "__main__":
    main()
