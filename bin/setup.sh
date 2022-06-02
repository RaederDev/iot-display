#!/usr/bin/env bash

if [[ $(id -u) -ne 0 ]]; then
  echo "Setup requires root"
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "Git is required. Please install before continuing"
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "NodeJS is required. Please install before continuing"
  exit 1
fi

if ! id -u "display" >/dev/null 2>&1; then
  echo useradd -m display
fi

mkdir /opt/iot-display
chown display:display /opt/iot-display
sudo -u display git clone https://github.com/RaederDev/iot-display.git /opt/iot-display
ln -s /opt/iot-display/bin/system/iot-display.service /lib/systemd/system/iot-display.service

systemctl daemon-reload
systemctl enable iot-display
systemctl start iot-display
