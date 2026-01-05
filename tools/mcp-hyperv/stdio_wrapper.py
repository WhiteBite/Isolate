#!/usr/bin/env python3
"""
MCP stdio сервер для Hyper-V.
Проксирует запросы к HTTP серверу (запущенному от админа).
"""
import sys
import json
import requests

HTTP_URL = "http://127.0.0.1:3100"

def call_http(tool: str, args: dict = None) -> dict:
    """Вызывает HTTP сервер"""
    try:
        r = requests.post(f"{HTTP_URL}/call", json={"tool": tool, "args": args or {}}, timeout=35, proxies={"http": None, "https": None})
        return r.json().get("result", {})
    except Exception as e:
        return {"error": str(e)}

TOOLS = [
    {"name": "hyperv_status", "description": "Статус Hyper-V сервиса и VM", "params": {}},
    {"name": "hyperv_vm_list", "description": "Список всех VM", "params": {}},
    {"name": "hyperv_vm_start", "description": "Запустить VM", "params": {"name": "string"}},
    {"name": "hyperv_vm_stop", "description": "Остановить VM", "params": {"name": "string"}},
    {"name": "hyperv_network_status", "description": "Статус сети", "params": {}},
    {"name": "powershell_exec", "description": "Выполнить PowerShell команду", "params": {"command": "string"}},
]

def call_tool(name: str, args: dict) -> dict:
    return call_http(name, args)
def handle_request(request: dict) -> dict | None:
    """Обрабатывает MCP запрос"""
    method = request.get("method")
    req_id = request.get("id")
    params = request.get("params", {})
    
    if method == "initialize":
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "hyperv-mcp", "version": "1.0"}
            }
        }
    elif method == "tools/list":
        tools = [{"name": t["name"], "description": t["description"], 
                  "inputSchema": {"type": "object", "properties": {k: {"type": v} for k, v in t["params"].items()}}}
                 for t in TOOLS]
        return {"jsonrpc": "2.0", "id": req_id, "result": {"tools": tools}}
    
    elif method == "tools/call":
        result = call_tool(params.get("name", ""), params.get("arguments", {}))
        text = json.dumps(result, ensure_ascii=False, indent=2)
        return {"jsonrpc": "2.0", "id": req_id, "result": {"content": [{"type": "text", "text": text}]}}
    
    elif method == "notifications/initialized":
        return None
    
    return {"jsonrpc": "2.0", "id": req_id, "error": {"code": -32601, "message": f"Unknown: {method}"}}

def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
            resp = handle_request(req)
            if resp:
                sys.stdout.write(json.dumps(resp) + "\n")
                sys.stdout.flush()
        except Exception as e:
            sys.stderr.write(f"Error: {e}\n")

if __name__ == "__main__":
    main()
