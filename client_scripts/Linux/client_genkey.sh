#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT}"

TS_HOME="/home/ts"
SSH_DIR="${TS_HOME}/.ssh"
TUNNEL_KEY="${SSH_DIR}/tunnel_key"
BORG_KEY="${SSH_DIR}/borg_${CLIENT}_key"

install -d -m 0700 -o ts -g ts "$SSH_DIR"

if [ ! -f "$TUNNEL_KEY" ]; then
  sudo -u ts ssh-keygen -t ed25519 -a 64 -f "$TUNNEL_KEY" -N "" -C "tunnel_${CLIENT}"
fi

if [ ! -f "$BORG_KEY" ]; then
  sudo -u ts ssh-keygen -t ed25519 -a 64 -f "$BORG_KEY" -N "" -C "borg_${CLIENT}"
fi

echo "OK"
echo "Tunnel pubkey: ${TUNNEL_KEY}.pub"
echo "Borg pubkey:   ${BORG_KEY}.pub"
