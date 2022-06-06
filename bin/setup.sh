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
  useradd -m display
  usermod -a -G i2c display
fi

mkdir /opt/iot-display
chown display:display /opt/iot-display
sudo -u display git clone https://github.com/RaederDev/iot-display.git /opt/iot-display
sudo -u display cp /opt/iot-display/config/config.tpl.json /opt/iot-display/config/config.json
sudo -u display sh -c "cd /opt/iot-display && /opt/iot-display/bin/update-server.sh"
ln -s /opt/iot-display/bin/system/iot-display.service /lib/systemd/system/iot-display.service
ln -s /opt/iot-display/bin/system/iot-human-monitoring.service /lib/systemd/system/iot-human-monitoring.service

systemctl daemon-reload

echo Please add valid configuration file to /opt/iot-display/config, afterwards start the server with
echo "systemctl enable iot-display && systemctl start iot-display"

echo If you have an EP-0106 Sensor Hub Development board you can automatically turn on your screen when a human is detected
echo "systemctl enable iot-human-monitoring && systemctl start iot-human-monitoring"
