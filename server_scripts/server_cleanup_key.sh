#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT REVERSE_PORT}"
REVERSE_PORT="${2:?Usage: $0 CLIENT REVERSE_PORT}"

SERVER_TO_CLIENT_KEY="/home/tunnel/.ssh/server_to_client_ed25519"
[ -f "$SERVER_TO_CLIENT_KEY" ] || { echo "missing $SERVER_TO_CLIENT_KEY"; exit 1; }

ssh -i "$SERVER_TO_CLIENT_KEY" -p "$REVERSE_PORT" \
  -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  borghelper@localhost \
  "cleanup $CLIENT"

echo "OK cleaned"
