#!/usr/bin/env bash
set -euo pipefail

CLIENT_ID="${1:?Usage: $0 CLIENT_ID}"

KEY_DIR="$HOME/.ssh"
KEY_PATH="$KEY_DIR/id_tunnel_${CLIENT_ID}"
PUB_PATH="${KEY_PATH}.pub"

mkdir -p "$KEY_DIR"
chmod 700 "$KEY_DIR"

if [ -f "$KEY_PATH" ]; then
  echo "déja creé $KEY_PATH"
else
  ssh-keygen -t ed25519 -f "$KEY_PATH" -N "" -C "tunnel_${CLIENT_ID}"
  chmod 600 "$KEY_PATH"
  chmod 644 "$PUB_PATH"
fi

echo
echo "=== PUBLIC KEY ==="
cat "$PUB_PATH"
echo "=================="
echo

# Envoi vers Electron via Node.js
node "$HOME/scripts/send_key.js" "$CLIENT_ID" "$PUB_PATH"
