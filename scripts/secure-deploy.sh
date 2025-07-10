#!/bin/bash

# DataMesh Secure Deployment Script
# This script helps set up a secure production deployment of DataMesh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DATAMESH_USER="datamesh"
DATAMESH_HOME="/opt/datamesh"
DATAMESH_DATA="/var/lib/datamesh"
DATAMESH_CONFIG="/etc/datamesh"
DATAMESH_LOGS="/var/log/datamesh"

echo -e "${BLUE}DataMesh Secure Deployment Script${NC}"
echo "======================================"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}" 
   exit 1
fi

# Function to generate secure random strings
generate_secret() {
    local length=${1:-32}
    openssl rand -base64 $length | tr -d "=+/" | cut -c1-$length
}

# Function to create directories with proper permissions
create_secure_directory() {
    local dir=$1
    local owner=$2
    local perms=$3
    
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir"
        chown "$owner" "$dir"
        chmod "$perms" "$dir"
        echo -e "${GREEN}Created directory: $dir${NC}"
    fi
}

echo -e "${YELLOW}Step 1: Creating DataMesh user and directories...${NC}"

# Create datamesh user
if ! id "$DATAMESH_USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$DATAMESH_HOME" "$DATAMESH_USER"
    echo -e "${GREEN}Created user: $DATAMESH_USER${NC}"
fi

# Create directories
create_secure_directory "$DATAMESH_HOME" "$DATAMESH_USER:$DATAMESH_USER" "750"
create_secure_directory "$DATAMESH_DATA" "$DATAMESH_USER:$DATAMESH_USER" "700"
create_secure_directory "$DATAMESH_CONFIG" "$DATAMESH_USER:$DATAMESH_USER" "750"
create_secure_directory "$DATAMESH_LOGS" "$DATAMESH_USER:$DATAMESH_USER" "750"
create_secure_directory "$DATAMESH_DATA/keys" "$DATAMESH_USER:$DATAMESH_USER" "700"

echo -e "${YELLOW}Step 2: Generating secure secrets...${NC}"

# Generate JWT secret
JWT_SECRET=$(generate_secret 32)
DB_ENCRYPTION_KEY=$(openssl rand -hex 32)

# Create environment file
cat > "$DATAMESH_CONFIG/.env" << EOF
# DataMesh Production Environment Configuration
# Generated on $(date)

# Security Secrets
DATAMESH_JWT_SECRET=$JWT_SECRET
DATAMESH_DB_ENCRYPTION_KEY=$DB_ENCRYPTION_KEY

# Network Configuration
DATAMESH_API_HOST=0.0.0.0
DATAMESH_API_PORT=8443
DATAMESH_ENABLE_HTTPS=true

# TLS Configuration
DATAMESH_TLS_CERT_PATH=$DATAMESH_CONFIG/tls/cert.pem
DATAMESH_TLS_KEY_PATH=$DATAMESH_CONFIG/tls/key.pem

# Data Paths
DATAMESH_DATA_DIR=$DATAMESH_DATA
DATAMESH_KEYS_DIR=$DATAMESH_DATA/keys

# Logging
RUST_LOG=info
DATAMESH_LOG_FORMAT=json
DATAMESH_ENABLE_AUDIT_LOG=true

# Security Features
DATAMESH_ENABLE_RATE_LIMITING=true
DATAMESH_ENABLE_SECURITY_HEADERS=true
DATAMESH_AUTO_KEY_ROTATION=true
DATAMESH_KEY_ROTATION_INTERVAL_HOURS=24
EOF

# Secure the environment file
chown "$DATAMESH_USER:$DATAMESH_USER" "$DATAMESH_CONFIG/.env"
chmod 600 "$DATAMESH_CONFIG/.env"

echo -e "${GREEN}Generated secure environment configuration${NC}"

echo -e "${YELLOW}Step 3: Setting up TLS certificates...${NC}"

# Create TLS directory
create_secure_directory "$DATAMESH_CONFIG/tls" "$DATAMESH_USER:$DATAMESH_USER" "700"

# Check if certificates exist
if [[ ! -f "$DATAMESH_CONFIG/tls/cert.pem" ]]; then
    echo -e "${YELLOW}Generating self-signed certificate for development...${NC}"
    echo -e "${RED}WARNING: Use proper certificates from a CA in production!${NC}"
    
    # Generate self-signed certificate
    openssl req -x509 -newkey rsa:4096 -keyout "$DATAMESH_CONFIG/tls/key.pem" \
        -out "$DATAMESH_CONFIG/tls/cert.pem" -days 365 -nodes \
        -subj "/C=US/ST=State/L=City/O=DataMesh/CN=localhost"
    
    chown "$DATAMESH_USER:$DATAMESH_USER" "$DATAMESH_CONFIG/tls/"*
    chmod 600 "$DATAMESH_CONFIG/tls/"*
    
    echo -e "${GREEN}Generated self-signed TLS certificate${NC}"
fi

echo -e "${YELLOW}Step 4: Setting up firewall rules...${NC}"

# Check if ufw is available
if command -v ufw >/dev/null 2>&1; then
    # Allow SSH (assuming it's on port 22)
    ufw allow 22/tcp comment "SSH"
    
    # Allow DataMesh API (HTTPS)
    ufw allow 8443/tcp comment "DataMesh API HTTPS"
    
    # Allow DataMesh P2P
    ufw allow 4001/tcp comment "DataMesh P2P"
    
    # Enable firewall if not already enabled
    echo "y" | ufw enable
    
    echo -e "${GREEN}Configured firewall rules${NC}"
else
    echo -e "${YELLOW}UFW not found, please configure firewall manually${NC}"
fi

echo -e "${YELLOW}Step 5: Creating systemd service...${NC}"

# Create systemd service file
cat > /etc/systemd/system/datamesh.service << EOF
[Unit]
Description=DataMesh Distributed Storage System
After=network.target
Wants=network.target

[Service]
Type=simple
User=$DATAMESH_USER
Group=$DATAMESH_USER
WorkingDirectory=$DATAMESH_HOME
EnvironmentFile=$DATAMESH_CONFIG/.env
ExecStart=$DATAMESH_HOME/datamesh service
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATAMESH_DATA $DATAMESH_LOGS
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=datamesh

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo -e "${GREEN}Created systemd service${NC}"

echo -e "${YELLOW}Step 6: Setting up log rotation...${NC}"

# Create logrotate configuration
cat > /etc/logrotate.d/datamesh << EOF
$DATAMESH_LOGS/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 640 $DATAMESH_USER $DATAMESH_USER
    postrotate
        systemctl reload datamesh >/dev/null 2>&1 || true
    endscript
}
EOF

echo -e "${GREEN}Configured log rotation${NC}"

echo -e "${YELLOW}Step 7: Setting up monitoring...${NC}"

# Create monitoring directory
create_secure_directory "$DATAMESH_DATA/monitoring" "$DATAMESH_USER:$DATAMESH_USER" "750"

# Create basic monitoring script
cat > "$DATAMESH_HOME/health-check.sh" << 'EOF'
#!/bin/bash
# Basic health check script for DataMesh

DATAMESH_API_PORT=${DATAMESH_API_PORT:-8443}
DATAMESH_API_HOST=${DATAMESH_API_HOST:-localhost}

# Check if DataMesh API is responding
if curl -k -f "https://$DATAMESH_API_HOST:$DATAMESH_API_PORT/api/v1/health" >/dev/null 2>&1; then
    echo "DataMesh API: HEALTHY"
    exit 0
else
    echo "DataMesh API: UNHEALTHY"
    exit 1
fi
EOF

chmod +x "$DATAMESH_HOME/health-check.sh"
chown "$DATAMESH_USER:$DATAMESH_USER" "$DATAMESH_HOME/health-check.sh"

echo -e "${GREEN}Created health check script${NC}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}DataMesh Secure Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo "1. Copy the DataMesh binary to $DATAMESH_HOME/datamesh"
echo "2. Replace self-signed certificates with proper CA-issued certificates"
echo "3. Review and adjust configuration in $DATAMESH_CONFIG/.env"
echo "4. Start the service: systemctl start datamesh"
echo "5. Enable auto-start: systemctl enable datamesh"
echo "6. Check status: systemctl status datamesh"
echo "7. View logs: journalctl -u datamesh -f"
echo ""
echo -e "${YELLOW}Important Security Notes:${NC}"
echo "- JWT Secret: Stored in $DATAMESH_CONFIG/.env (keep secure!)"
echo "- Database Encryption Key: Also in .env file"
echo "- TLS Certificates: Located in $DATAMESH_CONFIG/tls/"
echo "- All data: Stored in $DATAMESH_DATA (backup regularly)"
echo "- Health check: $DATAMESH_HOME/health-check.sh"
echo ""
echo -e "${RED}⚠️  SECURITY REMINDERS:${NC}"
echo "- Replace self-signed certificates with proper CA certificates"
echo "- Regularly rotate secrets and certificates"
echo "- Monitor logs for security events"
echo "- Keep the system updated"
echo "- Use a proper monitoring solution"
echo ""
echo -e "${GREEN}Deployment completed successfully!${NC}"