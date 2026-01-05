#!/usr/bin/env python3
"""
Hyper-V Daemon - выполняет команды от имени администратора.

Слушает named pipe и выполняет PowerShell команды.
Запускать от администратора!

Использование:
    python hyperv_daemon.py
"""
import json
import subprocess
import sys
import ctypes
import os

PIPE_NAME = r"\\.\pipe\hyperv_mcp"
WINWS_PATH = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "bin", "winws")

# Фикс кодировки для subprocess
SUBPROCESS_KWARGS = {"encoding": "utf-8", "errors": "replace"}

def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def run_powershell(command: str, timeout: int = 30) -> dict:
    """Выполняет PowerShell команду."""
    try:
        result = subprocess.run(
            ["powershell", "-NoProfile", "-Command", command],
            capture_output=True, timeout=timeout, **SUBPROCESS_KWARGS
        )
        return {
            "success": result.returncode == 0,
            "stdout": result.stdout.strip() if result.stdout else "",
            "stderr": result.stderr.strip() if result.stderr else ""
        }
    except subprocess.TimeoutExpired:
        return {"success": False, "error": "Timeout"}
    except Exception as e:
        return {"success": False, "error": str(e)}

def run_cmd(args, timeout=30, shell=False) -> dict:
    """Выполняет команду с правильной кодировкой."""
    try:
        result = subprocess.run(args, capture_output=True, timeout=timeout, shell=shell, **SUBPROCESS_KWARGS)
        return {
            "success": result.returncode == 0,
            "stdout": result.stdout.strip() if result.stdout else "",
            "stderr": result.stderr.strip() if result.stderr else ""
        }
    except subprocess.TimeoutExpired:
        return {"success": False, "error": "Timeout"}
    except Exception as e:
        return {"success": False, "error": str(e)}

COMMANDS = {
    "status": "Get-Service vmms | Select Status; Get-VM | Select Name,State | ConvertTo-Json -Compress",
    "vm_list": """Get-VM | ForEach-Object { 
        $vm=$_
        $na=Get-VMNetworkAdapter -VMName $vm.Name | Select Name,SwitchName,IPAddresses
        [PSCustomObject]@{
            Name=$vm.Name
            State=$vm.State.ToString()
            CPU=$vm.ProcessorCount
            MemMB=[int]($vm.MemoryAssigned/1MB)
            Net=$na
        }
    } | ConvertTo-Json -Depth 3 -Compress""",
    "network_status": """@{
        Switches=Get-VMSwitch | Select Name,SwitchType
        IPs=Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.InterfaceAlias -like '*vEthernet*'} | Select InterfaceAlias,IPAddress
    } | ConvertTo-Json -Depth 2 -Compress""",
}

def handle_command(cmd: str, args: dict) -> dict:
    """Обрабатывает команду."""
    if cmd == "vm_start":
        name = args.get("name", "")
        return run_powershell(f"Start-VM -Name '{name}' -EA Stop; Get-VM -Name '{name}' | Select Name,State | ConvertTo-Json -Compress")
    
    elif cmd == "vm_stop":
        name = args.get("name", "")
        return run_powershell(f"Stop-VM -Name '{name}' -Force -EA Stop; Get-VM -Name '{name}' | Select Name,State | ConvertTo-Json -Compress")
    
    elif cmd == "ssh":
        host = args.get("host", "VM-test@192.168.100.20")
        user = args.get("user", "")
        if user and "@" not in host:
            host = f"{user}@{host}"
        command = args.get("command", "hostname")
        
        ssh_args = ["ssh", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", 
                   "-o", "UserKnownHostsFile=NUL", "-o", "ConnectTimeout=10", host, command]
        return run_cmd(ssh_args, timeout=30)
    
    elif cmd == "winws_deploy":
        local_path = args.get("local_path", os.path.join(WINWS_PATH, "winws.exe"))
        remote_path = args.get("remote_path", "C:/Tools/winws/winws.exe")
        
        # Создаём папку на VM
        run_cmd('ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "mkdir C:\\Tools\\winws 2>NUL"', timeout=15, shell=True)
        
        # Копируем winws.exe
        scp_cmd = f'scp -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL "{local_path}" VM-test@192.168.100.20:"{remote_path}"'
        result = run_cmd(scp_cmd, timeout=60, shell=True)
        
        # Копируем WinDivert файлы
        local_dir = os.path.dirname(local_path)
        remote_dir = os.path.dirname(remote_path).replace("/", "\\")
        
        for divert_file in ["WinDivert.dll", "WinDivert64.sys"]:
            divert_path = os.path.join(local_dir, divert_file)
            if os.path.exists(divert_path):
                scp_divert = f'scp -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL "{divert_path}" VM-test@192.168.100.20:"{remote_dir}\\{divert_file}"'
                run_cmd(scp_divert, timeout=60, shell=True)
        
        result["source"] = local_dir
        return result
    
    elif cmd == "winws_start":
        strategy_args = args.get("args", "--wf-tcp=80,443 --wf-udp=443")
        
        # Используем schtasks для запуска с правами SYSTEM
        # Сначала удаляем старую задачу если есть
        run_cmd('ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "schtasks /delete /tn WinWS /f 2>NUL"', timeout=10, shell=True)
        
        # Создаём задачу
        task_cmd = f'C:\\Tools\\winws\\winws.exe {strategy_args}'
        ssh_cmd = f'ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "schtasks /create /tn WinWS /tr \\"{task_cmd}\\" /sc once /st 00:00 /ru SYSTEM /f"'
        create_result = run_cmd(ssh_cmd, timeout=15, shell=True)
        
        if not create_result.get("success"):
            return create_result
        
        # Запускаем задачу
        run_result = run_cmd('ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "schtasks /run /tn WinWS"', timeout=10, shell=True)
        return run_result
    
    elif cmd == "winws_stop":
        # Останавливаем через taskkill
        result = run_cmd('ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "taskkill /F /IM winws.exe 2>NUL"', timeout=10, shell=True)
        # Удаляем задачу
        run_cmd('ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "schtasks /delete /tn WinWS /f 2>NUL"', timeout=10, shell=True)
        result["success"] = True  # taskkill может вернуть ошибку если процесс не найден
        return result
    
    elif cmd == "dpi_test_domain":
        domain = args.get("domain", "youtube.com")
        timeout_sec = args.get("timeout", 5)
        
        ssh_cmd = f'ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=NUL VM-test@192.168.100.20 "curl.exe -s --connect-timeout {timeout_sec} -o NUL -w %{{http_code}} https://{domain}"'
        result = run_cmd(ssh_cmd, timeout=timeout_sec + 10, shell=True)
        
        http_code = result.get("stdout", "").replace("'", "").replace('"', '')
        blocked = http_code == "000" or http_code == "" or not http_code.isdigit()
        
        return {
            "success": True, 
            "domain": domain, 
            "http_code": http_code, 
            "blocked": blocked,
            "accessible": not blocked and http_code.startswith(("2", "3"))
        }
    
    elif cmd == "powershell":
        ps_cmd = args.get("command", "")
        return run_powershell(ps_cmd, timeout=60)
    
    elif cmd in COMMANDS:
        return run_powershell(COMMANDS[cmd])
    
    return {"error": f"Unknown command: {cmd}"}


def handle_client(pipe):
    """Обрабатывает клиента."""
    import win32file
    try:
        result, data = win32file.ReadFile(pipe, 65536)
        request = json.loads(data.decode())
        
        cmd = request.get("command", "")
        args = request.get("args", {})
        
        response = handle_command(cmd, args)
        win32file.WriteFile(pipe, json.dumps(response).encode())
    except Exception as e:
        try:
            win32file.WriteFile(pipe, json.dumps({"error": str(e)}).encode())
        except:
            pass

def create_security_attributes():
    """Создаёт security attributes для доступа всем пользователям."""
    import win32security
    import ntsecuritycon
    
    dacl = win32security.ACL()
    everyone_sid = win32security.CreateWellKnownSid(win32security.WinWorldSid, None)
    dacl.AddAccessAllowedAce(
        win32security.ACL_REVISION,
        ntsecuritycon.GENERIC_READ | ntsecuritycon.GENERIC_WRITE,
        everyone_sid
    )
    
    sd = win32security.SECURITY_DESCRIPTOR()
    sd.SetSecurityDescriptorDacl(1, dacl, 0)
    
    sa = win32security.SECURITY_ATTRIBUTES()
    sa.SECURITY_DESCRIPTOR = sd
    sa.bInheritHandle = False
    
    return sa

def main():
    if not is_admin():
        print("ERROR: Run as Administrator!")
        print("Right-click -> Run as administrator")
        sys.exit(1)
    
    try:
        import win32pipe
        import win32file
        import win32event
        import win32security
        import ntsecuritycon
        import pywintypes
        import winerror
    except ImportError:
        print("ERROR: pywin32 not installed")
        print("Run: pip install pywin32")
        sys.exit(1)
    
    sa = create_security_attributes()
    
    print(f"""
╔════════════════════════════════════════════╗
║       Hyper-V MCP Daemon v1.4              ║
╠════════════════════════════════════════════╣
║  Pipe: {PIPE_NAME:<33}║
║  Admin: {str(is_admin()):<32}║
║  WinWS: {WINWS_PATH:<33}║
╚════════════════════════════════════════════╝

Waiting for connections... (Ctrl+C to stop)
""")
    
    while True:
        pipe = None
        overlapped = None
        try:
            pipe = win32pipe.CreateNamedPipe(
                PIPE_NAME,
                win32pipe.PIPE_ACCESS_DUPLEX | win32file.FILE_FLAG_OVERLAPPED,
                win32pipe.PIPE_TYPE_MESSAGE | win32pipe.PIPE_READMODE_MESSAGE | win32pipe.PIPE_WAIT,
                win32pipe.PIPE_UNLIMITED_INSTANCES,
                65536, 65536, 0, sa
            )
            
            overlapped = pywintypes.OVERLAPPED()
            overlapped.hEvent = win32event.CreateEvent(None, True, False, None)
            
            try:
                win32pipe.ConnectNamedPipe(pipe, overlapped)
            except pywintypes.error as e:
                if e.winerror != winerror.ERROR_IO_PENDING:
                    raise
            
            while True:
                wait_result = win32event.WaitForSingleObject(overlapped.hEvent, 1000)
                if wait_result == win32event.WAIT_OBJECT_0:
                    break
                elif wait_result == win32event.WAIT_TIMEOUT:
                    continue
            
            handle_client(pipe)
            
            try:
                win32pipe.DisconnectNamedPipe(pipe)
            except:
                pass
            win32file.CloseHandle(pipe)
            win32file.CloseHandle(overlapped.hEvent)
            
        except KeyboardInterrupt:
            print("\n\nShutting down gracefully...")
            if pipe:
                try:
                    win32file.CloseHandle(pipe)
                except:
                    pass
            if overlapped and overlapped.hEvent:
                try:
                    win32file.CloseHandle(overlapped.hEvent)
                except:
                    pass
            break
        except pywintypes.error as e:
            if e.winerror == 232:
                if pipe:
                    try:
                        win32file.CloseHandle(pipe)
                    except:
                        pass
                continue
            print(f"Pipe error: {e}")
            if pipe:
                try:
                    win32file.CloseHandle(pipe)
                except:
                    pass
        except Exception as e:
            print(f"Error: {e}")
            if pipe:
                try:
                    win32file.CloseHandle(pipe)
                except:
                    pass
    
    print("Daemon stopped.")

if __name__ == "__main__":
    main()
