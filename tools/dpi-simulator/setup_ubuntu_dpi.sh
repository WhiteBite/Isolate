#!/bin/bash
# Setup script for DPI Simulator on Ubuntu Server
# Run this after Ubuntu installation via SSH

set -e

echo "=== DPI Simulator Setup ==="

# 1. Update system
echo "[1/6] Updating system..."
apt-get update
apt-get upgrade -y

# 2. Install dependencies
echo "[2/6] Installing dependencies..."
apt-get install -y \
    python3 \
    python3-pip \
    python3-venv \
    iptables \
    libnetfilter-queue-dev \
    python3-dev \
    build-essential \
    net-tools \
    tcpdump \
    curl

# 3. Create DPI simulator directory
echo "[3/6] Setting up DPI simulator..."
mkdir -p /opt/dpi-simulator
cd /opt/dpi-simulator

# 4. Create Python virtual environment
python3 -m venv venv
source venv/bin/activate

# 5. Install Python packages
pip install --upgrade pip
pip install netfilterqueue scapy

# 6. Configure network (static IP on eth0)
echo "[4/6] Configuring network..."
cat > /etc/netplan/01-dpi-network.yaml << 'EOF'
network:
  version: 2
  ethernets:
    eth0:
      addresses:
        - 192.168.100.10/24
      routes:
        - to: 192.168.100.0/24
          via: 192.168.100.1
      nameservers:
        addresses: [8.8.8.8, 8.8.4.4]
    eth1:
      dhcp4: true
EOF

netplan apply || true

# 7. Setup iptables rules for NFQUEUE
echo "[5/6] Setting up iptables..."
cat > /opt/dpi-simulator/setup_iptables.sh << 'EOF'
#!/bin/bash
# Forward traffic through NFQUEUE for DPI inspection

# Clear existing rules
iptables -F FORWARD
iptables -t nat -F

# Enable IP forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# Send traffic to NFQUEUE for inspection
# Only traffic from host (192.168.100.1) going to internet
iptables -A FORWARD -i eth0 -o eth1 -j NFQUEUE --queue-num 0
iptables -A FORWARD -i eth1 -o eth0 -m state --state ESTABLISHED,RELATED -j ACCEPT

# NAT for outgoing traffic
iptables -t nat -A POSTROUTING -o eth1 -j MASQUERADE

echo "iptables configured for DPI simulation"
EOF
chmod +x /opt/dpi-simulator/setup_iptables.sh

# 8. Create systemd service
echo "[6/6] Creating systemd service..."
cat > /etc/systemd/system/dpi-simulator.service << 'EOF'
[Unit]
Description=DPI Simulator
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/dpi-simulator
ExecStartPre=/opt/dpi-simulator/setup_iptables.sh
ExecStart=/opt/dpi-simulator/venv/bin/python3 dpi_simulator.py -d blocked_domains.txt --api-port 8888 --mode drop
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Next steps:"
echo "1. Copy dpi_simulator.py to /opt/dpi-simulator/"
echo "2. Copy blocked_domains.txt to /opt/dpi-simulator/"
echo "3. Start service: systemctl start dpi-simulator"
echo "4. Enable on boot: systemctl enable dpi-simulator"
echo ""
echo "API will be available at http://192.168.100.10:8888"
