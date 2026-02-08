#!/bin/bash
set -euo pipefail

need_root() {
  [ "$(id -u)" -eq 0 ] || { echo "Run as root"; exit 1; }
}

need_root

SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
DST_DIR="/usr/local/sbin"

echo "[install_all] Installing server scripts to ${DST_DIR}"

install -d -m 0755 -o root -g root "${DST_DIR}"

for s in \
  alloc_reverse_port.sh \
  preparedecrypt.sh \
  server_cleanup_key.sh \
  create_user.sh \
  install_client_key.sh \
  install_client_tunnel_key.sh \
  prepserv.sh \
  gen_gpg_passphrase.sh
do
  install -m 0755 -o root -g root "${SRC_DIR}/${s}" "${DST_DIR}/${s}"
done

cp -a "${SRC_DIR}/client_folder" "${DST_DIR}/client_folder"

echo "[install_all] Done."

echo "[install_all] Running prepserv.sh"
/usr/local/sbin/prepserv.sh