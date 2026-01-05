#!/bin/bash
# DPI Simulator setup script for Ubuntu VM

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/var/log/dpi-simulator.log"

case "$1" in
    setup|install)
        echo "Installing DPI Simulator dependencies..."
        apt-get update
        apt-get install -y python3-pip libnetfilter-queue-dev python3-dev iptables
        pip3 install netfilterqueue scapy
        
        # Create log file
        touch $LOG_FILE
        chmod 666 $LOG_FILE
        
        echo "Setup complete!"
        ;;
    
    start)
        echo "Starting DPI Simulator..."
        cd $SCRIPT_DIR
        
        # Setup iptables rules
        iptables -t mangle -F
        iptables -t mangle -A PREROUTING -p tcp --dport 80 -j NFQUEUE --queue-num 0
        iptables -t mangle -A PREROUTING -p tcp --dport 443 -j NFQUEUE --queue-num 0
        iptables -t mangle -A PREROUTING -p udp --dport 443 -j NFQUEUE --queue-num 0
        
        # Start simulator
        nohup python3 dpi_simulator.py -m drop --api-port 8888 > $LOG_FILE 2>&1 &
        echo "Started with PID: $!"
        ;;
    
    stop)
        echo "Stopping DPI Simulator..."
        pkill -f dpi_simulator.py || true
        iptables -t mangle -F
        echo "Stopped"
        ;;
    
    status)
        if pgrep -f dpi_simulator.py > /dev/null; then
            echo "Running (PID: $(pgrep -f dpi_simulator.py))"
            curl -s http://127.0.0.1:8888/status 2>/dev/null || echo "API not responding"
        else
            echo "Not running"
        fi
        ;;
    
    logs)
        tail -f $LOG_FILE
        ;;
    
    cleanup)
        iptables -t mangle -F
        echo "iptables rules cleared"
        ;;
    
    *)
        echo "Usage: $0 {setup|start|stop|status|logs|cleanup}"
        exit 1
        ;;
esac
