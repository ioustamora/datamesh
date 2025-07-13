# DataMesh Bootstrap Node VPS Requirements

This document outlines the minimal and recommended VPS requirements for running a DataMesh bootstrap node in production.

## 🖥️ Minimal VPS Requirements

### **Basic Bootstrap Node**
- **CPU**: 1 vCPU (2.4GHz+)
- **RAM**: 1GB RAM
- **Storage**: 10GB SSD
- **Network**: 100 Mbps bandwidth
- **OS**: Ubuntu 20.04+ / Debian 11+ / CentOS 8+

### **Recommended Production Setup**
- **CPU**: 2 vCPU (2.4GHz+)
- **RAM**: 2GB RAM
- **Storage**: 20GB SSD
- **Network**: 1 Gbps bandwidth
- **OS**: Ubuntu 22.04 LTS

## 📋 Software Prerequisites

```bash
# Rust toolchain (1.68.0+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget
```

## 🚀 Bootstrap Node Deployment

### **1. Build DataMesh**
```bash
git clone https://github.com/ioustamora/datamesh.git
cd datamesh
cargo build --release
```

### **2. Start Bootstrap Node**
```bash
# Basic bootstrap node on port 40000
./target/release/datamesh bootstrap --port 40000

# With custom configuration
./target/release/datamesh bootstrap --port 40000 --config-path /etc/datamesh/config.toml
```

### **3. Network Configuration**
```bash
# Open required ports
sudo ufw allow 40000/tcp
sudo ufw allow 8080/tcp  # For API server (optional)
```

## 📊 Resource Usage Expectations

### **Bootstrap Node Footprint**
- **Idle CPU**: ~5-10%
- **Active CPU**: ~20-30% (under load)
- **Memory**: ~100-200MB baseline
- **Storage Growth**: ~1-5MB per day (DHT data)
- **Network**: ~10-50 MB/day (peer discovery)

### **Network Port Requirements**
- **40000**: P2P networking (configurable)
- **8080**: REST API (optional)
- **443**: HTTPS API (optional)

## 🔧 Performance Scaling

### **Light Load** (1-10 peers)
- 1 vCPU, 1GB RAM sufficient

### **Medium Load** (10-50 peers)
- 2 vCPU, 2GB RAM recommended

### **Heavy Load** (50+ peers)
- 4+ vCPU, 4GB+ RAM
- Consider load balancing multiple bootstrap nodes

## 🏗️ Advanced Features Impact

If enabling advanced features:

### **With Monitoring System**
- **+200MB RAM** for time-series data
- **+5GB storage** for metrics history

### **With API Server**
- **+100MB RAM** for web server
- **+1 vCPU** for concurrent API requests

### **With Billing System**
- **+300MB RAM** for billing database
- **+2GB storage** for transaction history

## 💰 Cost Estimates

### **Budget VPS Providers**
- **DigitalOcean**: $6/month (1GB RAM, 1 vCPU)
- **Vultr**: $5/month (1GB RAM, 1 vCPU)
- **Linode**: $5/month (1GB RAM, 1 vCPU)
- **Hetzner**: €3.29/month (1GB RAM, 1 vCPU)

### **Production VPS**
- **DigitalOcean**: $12/month (2GB RAM, 1 vCPU)
- **AWS t3.small**: ~$15/month (2GB RAM, 2 vCPU)
- **Google Cloud e2-small**: ~$13/month (2GB RAM, 2 vCPU)

## 🛡️ Security Considerations

### **Basic Security**
```bash
# Firewall setup
sudo ufw enable
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 40000/tcp

# Optional: Fail2ban for SSH protection
sudo apt install fail2ban
```

### **SSL/TLS** (if using API server)
```bash
# Let's Encrypt for HTTPS
sudo apt install certbot
sudo certbot certonly --standalone -d your-domain.com
```

## 📈 Monitoring Bootstrap Node

```bash
# Check node status
./target/release/datamesh advanced --status

# Monitor system resources
htop
iostat -x 1
netstat -tlnp | grep 40000
```

## 🔧 Configuration Options

### **Basic Configuration** (`datamesh.toml`)
```toml
[network]
port = 40000
max_peers = 50
dht_storage_path = "/var/lib/datamesh/dht"

[performance]
enable_optimization = true
monitoring_interval = 60
auto_scaling = false

[api]
host = "0.0.0.0"
port = 8080
enable_https = false
enable_swagger = true
```

### **Production Configuration**
```toml
[network]
port = 40000
max_peers = 100
dht_storage_path = "/var/lib/datamesh/dht"

[performance]
enable_optimization = true
monitoring_interval = 30
auto_scaling = true

[api]
host = "0.0.0.0"
port = 8080
enable_https = true
cert_path = "/etc/letsencrypt/live/your-domain.com/fullchain.pem"
key_path = "/etc/letsencrypt/live/your-domain.com/privkey.pem"

[load_balancer]
strategy = "AdaptiveIntelligent"
auto_scaling = true
min_nodes = 3
max_nodes = 20

[failover]
strategy = "CircuitBreaker"
failure_threshold = 5
recovery_timeout = 60
```

## 🚦 Systemd Service Setup

Create a systemd service for automatic startup:

```bash
sudo tee /etc/systemd/system/datamesh-bootstrap.service > /dev/null <<EOF
[Unit]
Description=DataMesh Bootstrap Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=datamesh
Group=datamesh
WorkingDirectory=/opt/datamesh
ExecStart=/opt/datamesh/target/release/datamesh bootstrap --port 40000
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=datamesh

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable datamesh-bootstrap
sudo systemctl start datamesh-bootstrap
```

## 📝 Deployment Checklist

- [ ] VPS meets minimum requirements
- [ ] Rust toolchain installed
- [ ] DataMesh built successfully
- [ ] Firewall configured
- [ ] SSL certificates installed (if using HTTPS)
- [ ] Configuration file created
- [ ] Systemd service configured
- [ ] Monitoring setup (optional)
- [ ] Backup strategy implemented

## 🆘 Troubleshooting

### **Common Issues**

**Port already in use:**
```bash
sudo netstat -tlnp | grep 40000
sudo kill -9 <PID>
```

**Memory issues:**
```bash
# Check memory usage
free -h
# Add swap if needed
sudo fallocate -l 1G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

**Network connectivity:**
```bash
# Test P2P connectivity
telnet your-domain.com 40000
# Check if port is open
sudo ufw status
```

## 📞 Support

For additional support:
- **Documentation**: See main README.md and ADMIN.md
- **Issues**: GitHub Issues repository
- **Logs**: Check `/var/log/syslog` or `journalctl -u datamesh-bootstrap`

---

**Recommendation**: Start with the minimal setup (1GB RAM, 1 vCPU) and scale up based on actual network usage and peer connections. Monitor resource usage during the first week to determine if upgrades are needed.