#!/usr/bin/env python3
"""
Hyper-V MCP Server - управление Hyper-V VM через Model Context Protocol.

Запускается от имени администратора и предоставляет HTTP SSE endpoint для MCP.

Использование:
    # Запуск от имени администратора (откроет UAC prompt)
    python hyperv_mcp_server.py
    
    # Или напрямую если уже админ
    python hyperv_mcp_server.py --port 3100
"""

import asyncio
import json
import subprocess
import sys
import os
import ctypes
from typing import Any, Optional
from dataclasses import dataclass
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import urllib.parse

# ============================================================================
# Configuration
# ============================================================================

DEFAULT_PORT = 3100

@dataclass
class VMConfig:
    name: str = "DPI-Simulator"
    switch_name: str = "DPI-Internal"
    memory_mb: int = 2048
    disk_gb: int = 20
    cpu_count: int = 2
    vm_ip: str = "192.168.100.10"
    host_ip: str = "192.168.100.1"
    vm_user: str = "dpi"
    ssh_key_path: str = ""
    
    def __post_init__(self):
        if not self.ssh_key_path:
            self.ssh_key_path = str(Path.home() / ".ssh" / "id_rsa")

CONFIG = VMConfig()
SCRIPT_DIR = Path(__file__).parent.parent / "dpi-simulator"

# ============================================================================
# Admin Check & Elevation
# ============================================================================

def is_admin():
    """Проверяет права администратора."""
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def run_as_admin():
    """Перезапускает скрипт с правами администратора."""
    if is_admin():
        return True
    
    script = sys.argv[0]
    params = ' '.join(sys.argv[1:])
    
    print("Requesting administrator privileges...")
    ctypes.windll.shell32.ShellExecuteW(
        None, "runas", sys.executable, f'"{script}" {params}', None, 1
    )
    sys.exit(0)

# ============================================================================
# PowerShell & SSH Helpers
# ============================================================================

def run_powershell_sync(command: str, timeout: int = 30) -> dict:
    """Выполняет PowerShell команду синхронно."""
    try:
        result = subprocess.run(
            ["powershell", "-NoProfile", "-Command", command],
            capture_output=True,
            text=True,
            timeout=timeout
        )
        return {
            "success": result.returncode == 0,
            "exit_code": result.returncode,
            "stdout": result.stdout.strip(),
            "stderr": result.stderr.strip()
        }
    except subprocess.TimeoutExpired:
        return {"success": False, "exit_code": -1, "stdout": "", "stderr": "Timeout"}
    except Exception as e:
        return {"success": False, "exit_code": -1, "stdout": "", "stderr": str(e)}

def run_ssh_sync(command: str, vm_ip: str = None, vm_user: str = None) -> dict:
    """Выполняет SSH команду синхронно."""
    ip = vm_ip or CONFIG.vm_ip
    user = vm_user or CONFIG.vm_user
    
    ssh_args = [
        "ssh", "-o", "StrictHostKeyChecking=no",
        "-o", "UserKnownHostsFile=/dev/null",
        "-o", "ConnectTimeout=10", "-o", "BatchMode=yes"
    ]
    
    if Path(CONFIG.ssh_key_path).exists():
        ssh_args.extend(["-i", CONFIG.ssh_key_path])
    
    ssh_args.extend([f"{user}@{ip}", command])
    
    try:
        result = subprocess.run(ssh_args, capture_output=True, text=True, timeout=60)
        return {
            "success": result.returncode == 0,
            "exit_code": result.returncode,
            "stdout": result.stdout.strip(),
            "stderr": result.stderr.strip()
        }
    except Exception as e:
        return {"success": False, "exit_code": -1, "stdout": "", "stderr": str(e)}

# ============================================================================
# Tool Handlers
# ============================================================================

TOOLS = {
    "hyperv_status": {
        "description": "Проверяет статус Hyper-V и VM",
        "params": {}
    },
    "hyperv_vm_list": {
        "description": "Список всех VM с информацией",
        "params": {}
    },
    "hyperv_vm_create": {
        "description": "Создаёт VM для DPI симулятора",
        "params": {"name": "str", "memory_mb": "int", "disk_gb": "int"}
    },
    "hyperv_vm_start": {
        "description": "Запускает VM",
        "params": {"name": "str"}
    },
    "hyperv_vm_stop": {
        "description": "Останавливает VM",
        "params": {"name": "str", "force": "bool"}
    },
    "hyperv_vm_delete": {
        "description": "Удаляет VM",
        "params": {"name": "str"}
    },
    "hyperv_network_setup": {
        "description": "Настраивает Internal Switch",
        "params": {"switch_name": "str", "host_ip": "str"}
    },
    "hyperv_network_status": {
        "description": "Статус сети",
        "params": {}
    },
    "vm_ssh_test": {
        "description": "Тест SSH подключения",
        "params": {"vm_ip": "str", "vm_user": "str"}
    },
    "vm_ssh_exec": {
        "description": "Выполнить команду через SSH",
        "params": {"command": "str", "vm_ip": "str", "vm_user": "str"}
    },
    "dpi_simulator_deploy": {
        "description": "Развернуть DPI симулятор на VM",
        "params": {"vm_ip": "str"}
    },
    "dpi_simulator_start": {
        "description": "Запустить DPI симулятор",
        "params": {"mode": "str", "vm_ip": "str"}
    },
    "dpi_simulator_stop": {
        "description": "Остановить DPI симулятор",
        "params": {"vm_ip": "str"}
    },
    "dpi_simulator_status": {
        "description": "Статус DPI симулятора",
        "params": {"vm_ip": "str"}
    },
    "dpi_simulator_logs": {
        "description": "Логи DPI симулятора",
        "params": {"lines": "int", "vm_ip": "str"}
    },
    "powershell_exec": {
        "description": "Выполнить PowerShell команду",
        "params": {"command": "str"}
    }
}

def handle_tool(name: str, args: dict) -> dict:
    """Обрабатывает вызов инструмента."""
    
    if name == "hyperv_status":
        svc = run_powershell_sync("Get-Service vmms | Select-Object Status, DisplayName | ConvertTo-Json")
        vms = run_powershell_sync(
            "Get-VM | Select-Object Name, State, ProcessorCount, "
            "@{N='MemoryMB';E={[math]::Round($_.MemoryAssigned/1MB)}}, Uptime | ConvertTo-Json"
        )
        return {
            "service": json.loads(svc["stdout"]) if svc["success"] and svc["stdout"] else None,
            "vms": json.loads(vms["stdout"]) if vms["success"] and vms["stdout"] else [],
            "is_admin": is_admin()
        }
    
    elif name == "hyperv_vm_list":
        result = run_powershell_sync("""
            Get-VM | ForEach-Object {
                $vm = $_
                $adapters = Get-VMNetworkAdapter -VMName $vm.Name | Select-Object Name, SwitchName, IPAddresses
                [PSCustomObject]@{
                    Name = $vm.Name
                    State = $vm.State.ToString()
                    CPUCount = $vm.ProcessorCount
                    MemoryMB = [math]::Round($vm.MemoryAssigned/1MB)
                    NetworkAdapters = $adapters
                }
            } | ConvertTo-Json -Depth 3
        """)
        if result["success"] and result["stdout"]:
            data = json.loads(result["stdout"])
            return {"vms": data if isinstance(data, list) else [data]}
        return {"vms": [], "error": result["stderr"]}
    
    elif name == "hyperv_vm_create":
        vm_name = args.get("name", CONFIG.name)
        memory_mb = args.get("memory_mb", CONFIG.memory_mb)
        disk_gb = args.get("disk_gb", CONFIG.disk_gb)
        
        script = f"""
            $ErrorActionPreference = 'Stop'
            $VMName = '{vm_name}'
            $SwitchName = '{CONFIG.switch_name}'
            $VMPath = "$env:USERPROFILE\\Hyper-V\\$VMName"
            $VHDPath = "$VMPath\\$VMName.vhdx"
            $ISOPath = "$VMPath\\ubuntu-server.iso"
            
            if (Get-VM -Name $VMName -ErrorAction SilentlyContinue) {{ throw "VM exists" }}
            
            New-Item -ItemType Directory -Path $VMPath -Force | Out-Null
            
            if (-not (Get-VMSwitch -Name $SwitchName -ErrorAction SilentlyContinue)) {{
                New-VMSwitch -Name $SwitchName -SwitchType Internal | Out-Null
                $adapter = Get-NetAdapter | Where-Object {{ $_.Name -like "*$SwitchName*" }}
                if ($adapter) {{ New-NetIPAddress -InterfaceIndex $adapter.ifIndex -IPAddress '{CONFIG.host_ip}' -PrefixLength 24 -ErrorAction SilentlyContinue }}
            }}
            
            if (-not (Test-Path $ISOPath)) {{
                Write-Output "Downloading Ubuntu ISO..."
                Invoke-WebRequest -Uri "https://releases.ubuntu.com/22.04/ubuntu-22.04.5-live-server-amd64.iso" -OutFile $ISOPath
            }}
            
            New-VHD -Path $VHDPath -SizeBytes ({disk_gb}GB) -Dynamic | Out-Null
            New-VM -Name $VMName -MemoryStartupBytes ({memory_mb}MB) -Generation 2 -VHDPath $VHDPath -SwitchName $SwitchName | Out-Null
            Set-VM -Name $VMName -ProcessorCount 2 -DynamicMemory
            Add-VMDvdDrive -VMName $VMName -Path $ISOPath
            Set-VMFirmware -VMName $VMName -FirstBootDevice (Get-VMDvdDrive -VMName $VMName) -EnableSecureBoot Off
            
            $ds = Get-VMSwitch | Where-Object {{ $_.Name -eq "Default Switch" }}
            if ($ds) {{ Add-VMNetworkAdapter -VMName $VMName -SwitchName "Default Switch" -Name "Internet" }}
            
            Write-Output "VM created"
        """
        result = run_powershell_sync(script)
        return {"success": result["success"], "output": result["stdout"], "error": result["stderr"] if not result["success"] else None}

    elif name == "hyperv_vm_start":
        vm_name = args.get("name", CONFIG.name)
        result = run_powershell_sync(f"Start-VM -Name '{vm_name}' -ErrorAction Stop; Get-VM -Name '{vm_name}' | Select-Object Name, State | ConvertTo-Json")
        return {"success": result["success"], "output": result["stdout"], "error": result["stderr"] if not result["success"] else None}
    
    elif name == "hyperv_vm_stop":
        vm_name = args.get("name", CONFIG.name)
        force = "-Force -TurnOff" if args.get("force") else "-Force"
        result = run_powershell_sync(f"Stop-VM -Name '{vm_name}' {force}; Get-VM -Name '{vm_name}' | Select-Object Name, State | ConvertTo-Json")
        return {"success": result["success"], "output": result["stdout"], "error": result["stderr"] if not result["success"] else None}
    
    elif name == "hyperv_vm_delete":
        vm_name = args.get("name", CONFIG.name)
        result = run_powershell_sync(f"""
            $vm = Get-VM -Name '{vm_name}' -ErrorAction SilentlyContinue
            if ($vm) {{
                if ($vm.State -ne 'Off') {{ Stop-VM -Name '{vm_name}' -Force -TurnOff }}
                $vhd = (Get-VMHardDiskDrive -VMName '{vm_name}').Path
                Remove-VM -Name '{vm_name}' -Force
                if ($vhd -and (Test-Path $vhd)) {{ Remove-Item $vhd -Force }}
                Write-Output "Deleted"
            }} else {{ Write-Output "Not found" }}
        """)
        return {"success": result["success"], "output": result["stdout"]}
    
    elif name == "hyperv_network_setup":
        switch_name = args.get("switch_name", CONFIG.switch_name)
        host_ip = args.get("host_ip", CONFIG.host_ip)
        result = run_powershell_sync(f"""
            if (-not (Get-VMSwitch -Name '{switch_name}' -ErrorAction SilentlyContinue)) {{
                New-VMSwitch -Name '{switch_name}' -SwitchType Internal | Out-Null
            }}
            $adapter = Get-NetAdapter | Where-Object {{ $_.Name -like "*{switch_name}*" }}
            if ($adapter) {{
                $ip = Get-NetIPAddress -InterfaceIndex $adapter.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue | Where-Object {{ $_.IPAddress -eq '{host_ip}' }}
                if (-not $ip) {{ New-NetIPAddress -InterfaceIndex $adapter.ifIndex -IPAddress '{host_ip}' -PrefixLength 24 | Out-Null }}
            }}
            Get-VMSwitch -Name '{switch_name}' | Select-Object Name, SwitchType | ConvertTo-Json
        """)
        return {"success": result["success"], "output": result["stdout"]}
    
    elif name == "hyperv_network_status":
        result = run_powershell_sync("""
            @{
                Switches = Get-VMSwitch | Select-Object Name, SwitchType
                IPs = Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.InterfaceAlias -like "*vEthernet*" } | Select-Object InterfaceAlias, IPAddress
            } | ConvertTo-Json -Depth 2
        """)
        if result["success"] and result["stdout"]:
            return json.loads(result["stdout"])
        return {"error": result["stderr"]}
    
    elif name == "vm_ssh_test":
        vm_ip = args.get("vm_ip", CONFIG.vm_ip)
        vm_user = args.get("vm_user", CONFIG.vm_user)
        
        ping = run_powershell_sync(f"Test-Connection -ComputerName {vm_ip} -Count 1 -Quiet")
        ping_ok = ping["success"] and ping["stdout"].strip().lower() == "true"
        
        ssh = run_ssh_sync("echo SSH_OK", vm_ip, vm_user)
        ssh_ok = ssh["success"] and "SSH_OK" in ssh["stdout"]
        
        return {"ping": ping_ok, "ssh": ssh_ok, "vm_ip": vm_ip, "details": ssh["stderr"] if not ssh_ok else None}
    
    elif name == "vm_ssh_exec":
        command = args.get("command", "")
        if not command:
            return {"error": "command required"}
        result = run_ssh_sync(command, args.get("vm_ip"), args.get("vm_user"))
        return result
    
    elif name == "dpi_simulator_deploy":
        vm_ip = args.get("vm_ip", CONFIG.vm_ip)
        results = []
        
        run_ssh_sync(f"sudo mkdir -p /opt/dpi-simulator && sudo chown $USER:$USER /opt/dpi-simulator", vm_ip)
        
        for f in ["dpi_simulator.py", "requirements.txt", "blocked_domains.txt", "setup.sh"]:
            local = SCRIPT_DIR / f
            if local.exists():
                r = run_powershell_sync(f'scp -o StrictHostKeyChecking=no "{local}" {CONFIG.vm_user}@{vm_ip}:/opt/dpi-simulator/')
                results.append({"file": f, "ok": r["success"]})
        
        install = run_ssh_sync("cd /opt/dpi-simulator && sudo apt-get update && sudo apt-get install -y python3-pip libnetfilter-queue-dev && sudo pip3 install netfilterqueue scapy", vm_ip)
        results.append({"action": "install", "ok": install["success"]})
        
        return {"results": results}

    elif name == "dpi_simulator_start":
        vm_ip = args.get("vm_ip", CONFIG.vm_ip)
        mode = args.get("mode", "drop")
        
        run_ssh_sync("sudo iptables -t mangle -F && sudo iptables -t mangle -A PREROUTING -p tcp --dport 80 -j NFQUEUE --queue-num 0 && sudo iptables -t mangle -A PREROUTING -p tcp --dport 443 -j NFQUEUE --queue-num 0", vm_ip)
        run_ssh_sync(f"cd /opt/dpi-simulator && sudo nohup python3 dpi_simulator.py -m {mode} --api-port 8888 > /var/log/dpi-simulator.log 2>&1 &", vm_ip)
        
        import time; time.sleep(2)
        check = run_ssh_sync("pgrep -f dpi_simulator.py", vm_ip)
        
        return {"running": check["success"], "pid": check["stdout"].strip() if check["success"] else None, "mode": mode}
    
    elif name == "dpi_simulator_stop":
        vm_ip = args.get("vm_ip", CONFIG.vm_ip)
        run_ssh_sync("sudo pkill -f dpi_simulator.py; sudo iptables -t mangle -F", vm_ip)
        return {"stopped": True}
    
    elif name == "dpi_simulator_status":
        vm_ip = args.get("vm_ip", CONFIG.vm_ip)
        proc = run_ssh_sync("pgrep -f dpi_simulator.py", vm_ip)
        running = proc["success"] and proc["stdout"].strip()
        
        stats = None
        if running:
            s = run_ssh_sync("curl -s http://127.0.0.1:8888/stats 2>/dev/null", vm_ip)
            if s["success"] and s["stdout"]:
                try: stats = json.loads(s["stdout"])
                except: pass
        
        return {"running": bool(running), "pid": proc["stdout"].strip() if running else None, "stats": stats}
    
    elif name == "dpi_simulator_logs":
        vm_ip = args.get("vm_ip", CONFIG.vm_ip)
        lines = args.get("lines", 50)
        result = run_ssh_sync(f"sudo tail -n {lines} /var/log/dpi-simulator.log 2>/dev/null", vm_ip)
        return {"logs": result["stdout"]}
    
    elif name == "powershell_exec":
        command = args.get("command", "")
        if not command:
            return {"error": "command required"}
        return run_powershell_sync(command)
    
    return {"error": f"Unknown tool: {name}"}

# ============================================================================
# HTTP Server
# ============================================================================

class MCPHandler(BaseHTTPRequestHandler):
    """HTTP handler для MCP запросов."""
    
    def log_message(self, format, *args):
        print(f"[HTTP] {args[0]}")
    
    def _send_json(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False, indent=2).encode())
    
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()
    
    def do_GET(self):
        if self.path == "/":
            self._send_json({"name": "hyperv-mcp", "version": "1.0", "is_admin": is_admin()})
        elif self.path == "/tools":
            self._send_json({"tools": list(TOOLS.keys())})
        elif self.path == "/tools/list":
            tools_list = [{"name": k, "description": v["description"], "params": v["params"]} for k, v in TOOLS.items()]
            self._send_json({"tools": tools_list})
        else:
            self._send_json({"error": "Not found"}, 404)
    
    def do_POST(self):
        if self.path == "/call" or self.path.startswith("/tool/"):
            content_len = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_len).decode() if content_len else "{}"
            
            try:
                data = json.loads(body)
            except:
                self._send_json({"error": "Invalid JSON"}, 400)
                return
            
            tool_name = data.get("tool") or self.path.replace("/tool/", "")
            args = data.get("args", {})
            
            if tool_name not in TOOLS:
                self._send_json({"error": f"Unknown tool: {tool_name}"}, 404)
                return
            
            result = handle_tool(tool_name, args)
            self._send_json({"tool": tool_name, "result": result})
        else:
            self._send_json({"error": "Not found"}, 404)


# ============================================================================
# Main
# ============================================================================

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Hyper-V MCP Server")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="HTTP port")
    parser.add_argument("--no-admin", action="store_true", help="Skip admin check")
    args = parser.parse_args()
    
    if not args.no_admin and not is_admin():
        print("Not running as administrator. Requesting elevation...")
        run_as_admin()
        return
    
    print(f"""
╔══════════════════════════════════════════════════════════╗
║           Hyper-V MCP Server v1.0                        ║
╠══════════════════════════════════════════════════════════╣
║  Port: {args.port:<51}║
║  Admin: {str(is_admin()):<50}║
║  VM Config: {CONFIG.name} @ {CONFIG.vm_ip:<30}║
╚══════════════════════════════════════════════════════════╝

Endpoints:
  GET  /           - Server info
  GET  /tools      - List tool names
  GET  /tools/list - List tools with descriptions
  POST /call       - Call tool {{"tool": "name", "args": {{}}}}
  POST /tool/<name> - Call specific tool

Press Ctrl+C to stop.
""")
    
    server = HTTPServer(("127.0.0.1", args.port), MCPHandler)
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()


if __name__ == "__main__":
    main()
