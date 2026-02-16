#!/bin/bash

# sudo ./install_all_client.sh

set -euo pipefail

CLIENT_USER="${1:?Usage: $0 USER CLIENT_ID}"            # l’utilisateur qui lance borg (subject to change lol)
CLIENT_ID="${2:?Usage: $USER CLIENT_ID}"

[ "$(id -u)" -eq 0 ] || { echo "Run as root"; exit 1; }

SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
DST_DIR="/usr/local/sbin"

install -d -m 0755 -o root -g root "$DST_DIR"

for s in \
  prepclient.sh \
  client_backup.sh \
  borghelper_dispatch.sh \
  install_borghelper_key.sh
do
  install -m 0755 -o root -g root "$SRC_DIR/$s" "$DST_DIR/$s"
done

/usr/local/sbin/prepclient.sh $CLIENT_USER $CLIENT_ID
echo "[install_all_client] Done."