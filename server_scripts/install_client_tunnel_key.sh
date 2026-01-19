#!/bin/bash
set -euo pipefail

CLIENT="$1"
PUBKEY_FILE="$2"

TUNNEL_USER="tunnel"
SSH_DIR="/home/${TUNNEL_USER}/.ssh"
AUTH_KEYS="${SSH_DIR}/authorized_keys"

if [[ -z "${CLIENT}" || -z "${PUBKEY_FILE}" ]]; then
    echo "Usage: $0 CLIENT_NAME /path/to/client_tunnel_key.pub"
    exit 1
fi

if [[ ! -f "${PUBKEY_FILE}" ]]; then
    echo "Public key not found: ${PUBKEY_FILE}"
    exit 1
fi

echo "[install_tunnel_key] Installing tunnel key for client: ${CLIENT}"

sudo mkdir -p "${SSH_DIR}"
sudo chmod 700 "${SSH_DIR}"
sudo chown ${TUNNEL_USER}:${TUNNEL_USER} "${SSH_DIR}"

# Optionnel mais recommandé : restrictions SSH
KEY_LINE=$(cat "${PUBKEY_FILE}")
ENTRY="no-pty,no-agent-forwarding,no-X11-forwarding ${KEY_LINE} ${CLIENT}"

# Évite les doublons
if sudo grep -q "${KEY_LINE}" "${AUTH_KEYS}" 2>/dev/null; then
    echo "[install_tunnel_key] Key already present, skipping"
else
    echo "${ENTRY}" | sudo tee -a "${AUTH_KEYS}" >/dev/null
    echo "[install_tunnel_key] Key added"
fi

sudo chmod 600 "${AUTH_KEYS}"
sudo chown ${TUNNEL_USER}:${TUNNEL_USER} "${AUTH_KEYS}"

echo "[install_tunnel_key] Done"
