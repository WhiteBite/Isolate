#!/usr/bin/env python3
"""
DPI Simulator - симулятор DPI-блокировок для тестирования Isolate.

Перехватывает пакеты через NetfilterQueue и блокирует:
- TLS ClientHello по SNI
- HTTP запросы по Host header
- QUIC трафик (UDP 443)

Режимы блокировки:
- drop: дропать пакеты (вызывает timeout)
- rst: отправлять TCP RST (мгновенный сброс)
- mitm: редирект на локальный сервер
- throttle: добавлять задержку (slow connection)

Требует root прав и настроенных iptables правил.
"""

import os
import sys
import re
import struct
import logging
import argparse
import json
import time
import socket
import threading
from datetime import datetime
from pathlib import Path
from typing import Set, Optional, Dict, Any
from dataclasses import dataclass, field, asdict
from enum import Enum
from http.server import HTTPServer, BaseHTTPRequestHandler
from collections import defaultdict

try:
    from netfilterqueue import NetfilterQueue
    from scapy.all import IP, TCP, UDP, Raw, send
except ImportError as e:
    print(f"Error: Missing dependency - {e}")
    print("Install with: pip install -r requirements.txt")
    print("Also install: apt install libnetfilter-queue-dev python3-dev")
    sys.exit(1)

# Logging setup
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger('dpi_simulator')


class BlockMode(Enum):
    """Режимы блокировки DPI."""
    DROP = "drop"        # Просто дропать пакеты (timeout)
    RST = "rst"          # Отправлять TCP RST (мгновенный сброс)
    MITM = "mitm"        # Редирект на локальный сервер
    THROTTLE = "throttle"  # Добавлять задержку


@dataclass
class DomainMetrics:
    """Метрики для домена."""
    blocked: int = 0
    passed: int = 0
    last_blocked: Optional[str] = None
    block_types: Dict[str, int] = field(default_factory=lambda: defaultdict(int))


@dataclass
class SimulatorStats:
    """Статистика симулятора."""
    total_packets: int = 0
    blocked_sni: int = 0
    blocked_http: int = 0
    blocked_quic: int = 0
    passed: int = 0
    start_time: str = field(default_factory=lambda: datetime.now().isoformat())
    domain_metrics: Dict[str, DomainMetrics] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """Конвертирует в словарь для JSON."""
        result = {
            'total_packets': self.total_packets,
            'blocked_sni': self.blocked_sni,
            'blocked_http': self.blocked_http,
            'blocked_quic': self.blocked_quic,
            'passed': self.passed,
            'start_time': self.start_time,
            'uptime_seconds': (datetime.now() - datetime.fromisoformat(self.start_time)).total_seconds(),
            'domain_metrics': {}
        }
        for domain, metrics in self.domain_metrics.items():
            result['domain_metrics'][domain] = {
                'blocked': metrics.blocked,
                'passed': metrics.passed,
                'last_blocked': metrics.last_blocked,
                'block_types': dict(metrics.block_types)
            }
        return result


class JSONLogHandler(logging.Handler):
    """Handler для логирования в JSON формате."""
    
    def __init__(self, filepath: str):
        super().__init__()
        self.filepath = filepath
        self.file = open(filepath, 'a')
    
    def emit(self, record: logging.LogRecord):
        try:
            log_entry = {
                'timestamp': datetime.now().isoformat(),
                'level': record.levelname,
                'message': record.getMessage(),
                'logger': record.name
            }
            # Добавляем extra поля если есть
            if hasattr(record, 'extra_data'):
                log_entry['data'] = record.extra_data
            
            self.file.write(json.dumps(log_entry) + '\n')
            self.file.flush()
        except Exception:
            self.handleError(record)
    
    def close(self):
        self.file.close()
        super().close()


class APIHandler(BaseHTTPRequestHandler):
    """HTTP API для управления симулятором."""
    
    simulator = None  # Будет установлен при запуске
    
    def log_message(self, format, *args):
        """Переопределяем для использования нашего логгера."""
        logger.debug(f"API: {args[0]}")
    
    def _send_json(self, data: Dict, status: int = 200):
        """Отправляет JSON ответ."""
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data, indent=2).encode())
    
    def do_GET(self):
        """Обрабатывает GET запросы."""
        if self.path == '/status':
            self._send_json({
                'running': self.simulator.running if self.simulator else False,
                'mode': self.simulator.mode.value if self.simulator else None,
                'stats': self.simulator.stats.to_dict() if self.simulator else None,
                'blocked_domains_count': len(self.simulator.blocked_domains) if self.simulator else 0
            })
        elif self.path == '/stats':
            if self.simulator:
                self._send_json(self.simulator.stats.to_dict())
            else:
                self._send_json({'error': 'Simulator not running'}, 503)
        elif self.path == '/domains':
            if self.simulator:
                self._send_json({
                    'domains': list(self.simulator.blocked_domains),
                    'count': len(self.simulator.blocked_domains)
                })
            else:
                self._send_json({'error': 'Simulator not running'}, 503)
        else:
            self._send_json({'error': 'Not found'}, 404)
    
    def do_POST(self):
        """Обрабатывает POST запросы."""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length).decode() if content_length > 0 else '{}'
        
        try:
            data = json.loads(body) if body else {}
        except json.JSONDecodeError:
            self._send_json({'error': 'Invalid JSON'}, 400)
            return
        
        if self.path == '/mode':
            if not self.simulator:
                self._send_json({'error': 'Simulator not running'}, 503)
                return
            
            new_mode = data.get('mode')
            if not new_mode:
                self._send_json({'error': 'Mode not specified'}, 400)
                return
            
            try:
                self.simulator.mode = BlockMode(new_mode)
                logger.info(f"Mode changed to: {new_mode}")
                self._send_json({
                    'success': True,
                    'mode': self.simulator.mode.value
                })
            except ValueError:
                self._send_json({
                    'error': f'Invalid mode: {new_mode}',
                    'valid_modes': [m.value for m in BlockMode]
                }, 400)
        
        elif self.path == '/stop':
            if self.simulator:
                self.simulator.running = False
                self._send_json({'success': True, 'message': 'Stopping simulator'})
            else:
                self._send_json({'error': 'Simulator not running'}, 503)
        
        elif self.path == '/reset-stats':
            if self.simulator:
                self.simulator.stats = SimulatorStats()
                self._send_json({'success': True, 'message': 'Stats reset'})
            else:
                self._send_json({'error': 'Simulator not running'}, 503)
        
        else:
            self._send_json({'error': 'Not found'}, 404)


class DPISimulator:
    """Симулятор DPI-блокировок."""
    
    def __init__(
        self,
        domains_file: str,
        mode: BlockMode = BlockMode.DROP,
        log_file: Optional[str] = None,
        json_log: Optional[str] = None,
        mitm_redirect: str = "127.0.0.1:8080",
        throttle_delay: float = 2.0
    ):
        self.blocked_domains: Set[str] = set()
        self.stats = SimulatorStats()
        self.mode = mode
        self.running = False
        self.mitm_redirect = mitm_redirect
        self.throttle_delay = throttle_delay
        
        self._load_domains(domains_file)
        
        # Обычное логирование в файл
        if log_file:
            file_handler = logging.FileHandler(log_file)
            file_handler.setFormatter(logging.Formatter(
                '%(asctime)s [%(levelname)s] %(message)s'
            ))
            logger.addHandler(file_handler)
        
        # JSON логирование
        self.json_handler = None
        if json_log:
            self.json_handler = JSONLogHandler(json_log)
            logger.addHandler(self.json_handler)
    
    def _load_domains(self, filepath: str) -> None:
        """Загружает список доменов для блокировки."""
        path = Path(filepath)
        if not path.exists():
            logger.error(f"Domains file not found: {filepath}")
            sys.exit(1)
        
        with open(path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    self.blocked_domains.add(line.lower())
        
        logger.info(f"Loaded {len(self.blocked_domains)} blocked domains")
    
    def _is_domain_blocked(self, domain: str) -> bool:
        """Проверяет, заблокирован ли домен."""
        domain = domain.lower()
        
        if domain in self.blocked_domains:
            return True
        
        for blocked in self.blocked_domains:
            if domain.endswith('.' + blocked):
                return True
        
        return False
    
    def _update_domain_metrics(self, domain: str, blocked: bool, block_type: str = None):
        """Обновляет метрики для домена."""
        if domain not in self.stats.domain_metrics:
            self.stats.domain_metrics[domain] = DomainMetrics()
        
        metrics = self.stats.domain_metrics[domain]
        if blocked:
            metrics.blocked += 1
            metrics.last_blocked = datetime.now().isoformat()
            if block_type:
                metrics.block_types[block_type] += 1
        else:
            metrics.passed += 1
    
    def _log_block_event(self, block_type: str, src: str, dst: str, domain: str = None):
        """Логирует событие блокировки с extra данными для JSON."""
        extra_data = {
            'event': 'block',
            'block_type': block_type,
            'mode': self.mode.value,
            'src': src,
            'dst': dst,
            'domain': domain
        }
        record = logging.LogRecord(
            name='dpi_simulator',
            level=logging.WARNING,
            pathname='',
            lineno=0,
            msg=f"BLOCKED [{block_type}] {src} -> {dst} | Domain: {domain}",
            args=(),
            exc_info=None
        )
        record.extra_data = extra_data
        logger.handle(record)

    def _extract_sni(self, payload: bytes) -> Optional[str]:
        """
        Извлекает SNI из TLS ClientHello.
        """
        try:
            if len(payload) < 43:
                return None
            
            if payload[0] != 0x16:  # Content Type: Handshake
                return None
            
            if payload[5] != 0x01:  # Handshake Type: ClientHello
                return None
            
            offset = 43
            
            if offset >= len(payload):
                return None
            
            session_id_len = payload[offset]
            offset += 1 + session_id_len
            
            if offset + 2 > len(payload):
                return None
            
            cipher_suites_len = struct.unpack('!H', payload[offset:offset+2])[0]
            offset += 2 + cipher_suites_len
            
            if offset + 1 > len(payload):
                return None
            
            compression_len = payload[offset]
            offset += 1 + compression_len
            
            if offset + 2 > len(payload):
                return None
            
            extensions_len = struct.unpack('!H', payload[offset:offset+2])[0]
            offset += 2
            
            extensions_end = offset + extensions_len
            while offset + 4 <= extensions_end and offset + 4 <= len(payload):
                ext_type = struct.unpack('!H', payload[offset:offset+2])[0]
                ext_len = struct.unpack('!H', payload[offset+2:offset+4])[0]
                offset += 4
                
                if ext_type == 0x0000 and offset + ext_len <= len(payload):
                    if offset + 2 > len(payload):
                        break
                    
                    sni_list_len = struct.unpack('!H', payload[offset:offset+2])[0]
                    offset += 2
                    
                    if offset + 1 > len(payload):
                        break
                    
                    sni_type = payload[offset]
                    offset += 1
                    
                    if sni_type == 0x00:
                        if offset + 2 > len(payload):
                            break
                        
                        sni_len = struct.unpack('!H', payload[offset:offset+2])[0]
                        offset += 2
                        
                        if offset + sni_len <= len(payload):
                            sni = payload[offset:offset+sni_len].decode('ascii', errors='ignore')
                            return sni
                    break
                
                offset += ext_len
            
        except Exception as e:
            logger.debug(f"SNI extraction error: {e}")
        
        return None
    
    def _extract_http_host(self, payload: bytes) -> Optional[str]:
        """Извлекает Host header из HTTP запроса."""
        try:
            data = payload.decode('ascii', errors='ignore')
            
            if not any(data.startswith(m) for m in ['GET ', 'POST ', 'HEAD ', 'PUT ', 'DELETE ', 'CONNECT ']):
                return None
            
            match = re.search(r'Host:\s*([^\r\n]+)', data, re.IGNORECASE)
            if match:
                host = match.group(1).strip()
                if ':' in host:
                    host = host.split(':')[0]
                return host
        
        except Exception as e:
            logger.debug(f"HTTP host extraction error: {e}")
        
        return None

    def _is_quic_initial(self, payload: bytes) -> bool:
        """Проверяет, является ли пакет QUIC Initial."""
        try:
            if len(payload) < 6:
                return False
            
            if not (payload[0] & 0x80):
                return False
            
            if not (payload[0] & 0x40):
                return False
            
            packet_type = (payload[0] & 0x30) >> 4
            if packet_type != 0x00:
                return False
            
            version = struct.unpack('!I', payload[1:5])[0]
            if version == 0:
                return False
            
            return True
        
        except Exception:
            return False

    def _send_rst(self, ip_packet) -> None:
        """Отправляет TCP RST пакет."""
        try:
            if ip_packet.haslayer(TCP):
                tcp = ip_packet[TCP]
                rst_packet = IP(
                    src=ip_packet.dst,
                    dst=ip_packet.src
                ) / TCP(
                    sport=tcp.dport,
                    dport=tcp.sport,
                    seq=tcp.ack if tcp.ack else 0,
                    ack=tcp.seq + 1,
                    flags='RA'
                )
                send(rst_packet, verbose=False)
                logger.debug(f"Sent RST to {ip_packet.src}:{tcp.sport}")
        except Exception as e:
            logger.error(f"Failed to send RST: {e}")
    
    def _apply_throttle(self) -> None:
        """Применяет задержку для throttle режима."""
        time.sleep(self.throttle_delay)
    
    def _block_packet(self, packet, ip_packet, block_type: str, domain: str = None) -> None:
        """Блокирует пакет в соответствии с текущим режимом."""
        src = f"{ip_packet.src}:{ip_packet[TCP].sport if ip_packet.haslayer(TCP) else ip_packet[UDP].sport}"
        dst = f"{ip_packet.dst}:{ip_packet[TCP].dport if ip_packet.haslayer(TCP) else ip_packet[UDP].dport}"
        
        self._log_block_event(block_type, src, dst, domain)
        self._update_domain_metrics(domain or 'unknown', blocked=True, block_type=block_type)
        
        if self.mode == BlockMode.DROP:
            # Просто дропаем пакет - вызовет timeout
            packet.drop()
        
        elif self.mode == BlockMode.RST:
            # Отправляем RST и дропаем
            self._send_rst(ip_packet)
            packet.drop()
        
        elif self.mode == BlockMode.MITM:
            # TODO: Реализовать редирект на локальный сервер
            # Пока просто дропаем
            logger.debug(f"MITM redirect to {self.mitm_redirect} (not implemented, dropping)")
            packet.drop()
        
        elif self.mode == BlockMode.THROTTLE:
            # Добавляем задержку и пропускаем
            self._apply_throttle()
            packet.accept()
        
        else:
            packet.drop()
    
    def process_packet(self, packet) -> None:
        """Обрабатывает перехваченный пакет."""
        self.stats.total_packets += 1
        
        try:
            ip_packet = IP(packet.get_payload())
            
            # TCP пакеты (HTTP/HTTPS)
            if ip_packet.haslayer(TCP):
                tcp = ip_packet[TCP]
                
                if ip_packet.haslayer(Raw):
                    payload = bytes(ip_packet[Raw].load)
                    
                    # TLS ClientHello (порт 443)
                    if tcp.dport == 443:
                        sni = self._extract_sni(payload)
                        if sni and self._is_domain_blocked(sni):
                            self.stats.blocked_sni += 1
                            self._block_packet(packet, ip_packet, 'SNI', sni)
                            return
                    
                    # HTTP (порт 80)
                    if tcp.dport == 80:
                        host = self._extract_http_host(payload)
                        if host and self._is_domain_blocked(host):
                            self.stats.blocked_http += 1
                            self._block_packet(packet, ip_packet, 'HTTP', host)
                            return
            
            # UDP пакеты (QUIC)
            elif ip_packet.haslayer(UDP):
                udp = ip_packet[UDP]
                
                if udp.dport == 443 and ip_packet.haslayer(Raw):
                    payload = bytes(ip_packet[Raw].load)
                    
                    if self._is_quic_initial(payload):
                        self.stats.blocked_quic += 1
                        self._block_packet(packet, ip_packet, 'QUIC')
                        return
            
            # Пропускаем пакет
            self.stats.passed += 1
            packet.accept()
        
        except Exception as e:
            logger.error(f"Packet processing error: {e}")
            packet.accept()

    def print_stats(self) -> None:
        """Выводит статистику."""
        logger.info("=" * 50)
        logger.info("DPI Simulator Statistics:")
        logger.info(f"  Mode:           {self.mode.value}")
        logger.info(f"  Total packets:  {self.stats.total_packets}")
        logger.info(f"  Blocked (SNI):  {self.stats.blocked_sni}")
        logger.info(f"  Blocked (HTTP): {self.stats.blocked_http}")
        logger.info(f"  Blocked (QUIC): {self.stats.blocked_quic}")
        logger.info(f"  Passed:         {self.stats.passed}")
        
        if self.stats.domain_metrics:
            logger.info("-" * 50)
            logger.info("Per-domain metrics:")
            for domain, metrics in sorted(
                self.stats.domain_metrics.items(),
                key=lambda x: x[1].blocked,
                reverse=True
            )[:10]:  # Top 10
                logger.info(f"  {domain}: blocked={metrics.blocked}, passed={metrics.passed}")
        
        logger.info("=" * 50)
    
    def _start_api_server(self, port: int) -> None:
        """Запускает API сервер в отдельном потоке."""
        APIHandler.simulator = self
        server = HTTPServer(('0.0.0.0', port), APIHandler)
        server.timeout = 1
        
        logger.info(f"API server started on http://127.0.0.1:{port}")
        logger.info("  GET  /status      - Get simulator status")
        logger.info("  GET  /stats       - Get detailed statistics")
        logger.info("  GET  /domains     - List blocked domains")
        logger.info("  POST /mode        - Change blocking mode")
        logger.info("  POST /stop        - Stop simulator")
        logger.info("  POST /reset-stats - Reset statistics")
        
        while self.running:
            server.handle_request()
        
        server.server_close()
        logger.info("API server stopped")
    
    def run(self, queue_num: int = 0, api_port: int = None) -> None:
        """Запускает симулятор."""
        self.running = True
        
        logger.info(f"Starting DPI Simulator on NFQUEUE {queue_num}")
        logger.info(f"Mode: {self.mode.value}")
        logger.info(f"Blocking {len(self.blocked_domains)} domains")
        logger.info("Press Ctrl+C to stop")
        
        # Запускаем API сервер если указан порт
        api_thread = None
        if api_port:
            api_thread = threading.Thread(target=self._start_api_server, args=(api_port,))
            api_thread.daemon = True
            api_thread.start()
        
        nfqueue = NetfilterQueue()
        nfqueue.bind(queue_num, self.process_packet)
        
        try:
            while self.running:
                nfqueue.run(block=False)
                time.sleep(0.01)
        except KeyboardInterrupt:
            logger.info("Stopping DPI Simulator...")
        finally:
            self.running = False
            nfqueue.unbind()
            self.print_stats()
            
            if self.json_handler:
                self.json_handler.close()
            
            if api_thread:
                api_thread.join(timeout=2)


def main():
    parser = argparse.ArgumentParser(
        description='DPI Simulator - симулятор DPI-блокировок для тестирования',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Режимы блокировки:
  drop      Дропать пакеты (вызывает timeout у клиента)
  rst       Отправлять TCP RST (мгновенный сброс соединения)
  mitm      Редирект на локальный сервер (для проверки сертификатов)
  throttle  Добавлять задержку (медленное соединение)

Примеры:
  # Запуск с режимом RST
  sudo python3 dpi_simulator.py -d blocked.txt --mode rst

  # Запуск с API сервером
  sudo python3 dpi_simulator.py -d blocked.txt --api-port 8888

  # Запуск с JSON логированием
  sudo python3 dpi_simulator.py -d blocked.txt --json-log dpi.jsonl

API endpoints:
  GET  /status      - Статус симулятора
  GET  /stats       - Детальная статистика
  GET  /domains     - Список блокируемых доменов
  POST /mode        - Изменить режим {"mode": "rst"}
  POST /stop        - Остановить симулятор
  POST /reset-stats - Сбросить статистику
        """
    )
    parser.add_argument(
        '-d', '--domains',
        default='blocked_domains.txt',
        help='Файл со списком блокируемых доменов (default: blocked_domains.txt)'
    )
    parser.add_argument(
        '-q', '--queue',
        type=int,
        default=0,
        help='Номер NFQUEUE (default: 0)'
    )
    parser.add_argument(
        '-m', '--mode',
        choices=['drop', 'rst', 'mitm', 'throttle'],
        default='drop',
        help='Режим блокировки (default: drop)'
    )
    parser.add_argument(
        '-l', '--log',
        help='Файл для текстового логирования'
    )
    parser.add_argument(
        '--json-log',
        help='Файл для JSON логирования (формат JSONL)'
    )
    parser.add_argument(
        '--api-port',
        type=int,
        help='Порт для HTTP API управления'
    )
    parser.add_argument(
        '--mitm-redirect',
        default='127.0.0.1:8080',
        help='Адрес для MITM редиректа (default: 127.0.0.1:8080)'
    )
    parser.add_argument(
        '--throttle-delay',
        type=float,
        default=2.0,
        help='Задержка в секундах для throttle режима (default: 2.0)'
    )
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Подробный вывод'
    )
    
    args = parser.parse_args()
    
    if args.verbose:
        logger.setLevel(logging.DEBUG)
    
    # Проверяем root права
    if os.geteuid() != 0:
        logger.error("This script requires root privileges!")
        logger.error("Run with: sudo python3 dpi_simulator.py")
        sys.exit(1)
    
    # Определяем режим
    mode = BlockMode(args.mode)
    
    # Запускаем симулятор
    simulator = DPISimulator(
        domains_file=args.domains,
        mode=mode,
        log_file=args.log,
        json_log=args.json_log,
        mitm_redirect=args.mitm_redirect,
        throttle_delay=args.throttle_delay
    )
    simulator.run(queue_num=args.queue, api_port=args.api_port)


if __name__ == '__main__':
    main()
