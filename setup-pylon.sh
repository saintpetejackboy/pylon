#!/bin/bash
# setup-pylon.sh
#
# This script sets up the environment for the Pylon service on Ubuntu.
# It will:
# 1. Create /opt/pylon with the proper permissions.
# 2. Create a wrapper script that waits for the pylon binary to be executable.
# 3. Write a systemd service file for the Pylon service.
# 4. Create a sudoers drop-in so that the www-data user can control the service.
# 5. Reload systemd and enable/start the service.
#
# Usage: Run as root or with sudo: sudo ./setup-pylon.sh

# Check for root privileges.
if [ "$(id -u)" -ne 0 ]; then
    echo "This script must be run as root. Please use sudo."
    exit 1
fi

set -e

echo "Creating /opt/pylon directory..."
mkdir -p /opt/pylon
chown www-data:www-data /opt/pylon
chmod 775 /opt/pylon

echo "Writing the wrapper script /opt/pylon/start-pylon.sh..."
cat > /opt/pylon/start-pylon.sh << 'EOF'
#!/bin/bash
# Wrapper script that waits until the pylon binary is executable.
while [ ! -x /opt/pylon/pylon ]; do
  echo "Waiting for /opt/pylon/pylon to be executable..."
  sleep 1
done
exec /opt/pylon/pylon
EOF
chmod +x /opt/pylon/start-pylon.sh

echo "Writing the systemd service file /etc/systemd/system/pylon.service..."
cat > /etc/systemd/system/pylon.service << 'EOF'
[Unit]
Description=Pylon Server
After=network.target

[Service]
ExecStart=/opt/pylon/start-pylon.sh
Restart=always
User=www-data
Group=www-data
WorkingDirectory=/opt/pylon
StandardOutput=journal
StandardError=journal
SyslogIdentifier=pylon
RestartSec=2

[Install]
WantedBy=multi-user.target
EOF

echo "Creating sudoers drop-in for www-data to control the pylon service..."
cat > /etc/sudoers.d/pylon << 'EOF'
www-data ALL=(root) NOPASSWD: /usr/bin/systemctl restart pylon, /usr/bin/systemctl start pylon, /usr/bin/systemctl stop pylon
EOF
chmod 0440 /etc/sudoers.d/pylon

echo "Reloading systemd daemon..."
systemctl daemon-reload

echo "Enabling and starting the Pylon service..."
systemctl enable --now pylon.service

echo "Setup complete. The Pylon service is active and waiting for /opt/pylon/pylon to become executable."
