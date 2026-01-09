#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT}"

USER_HOME="/home/$USER"
SSH_DIR="${USER_HOME}/.ssh"
TUNNEL_KEY="${SSH_DIR}/tunnel_key"
BORG_KEY="${SSH_DIR}/borg_${CLIENT}_key"

install -d -m 0700 -o $USER -g $USER "$SSH_DIR"

if [ ! -f "$TUNNEL_KEY" ]; then
  sudo -u $USER ssh-keygen -t ed25519 -a 64 -f "$TUNNEL_KEY" -N "" -C "tunnel_${CLIENT}"
fi

if [ ! -f "$BORG_KEY" ]; then
  sudo -u $USER ssh-keygen -t ed25519 -a 64 -f "$BORG_KEY" -N "" -C "borg_${CLIENT}"
fi

echo "OK"
echo "Tunnel pubkey: ${TUNNEL_KEY}.pub"
echo "Borg pubkey:   ${BORG_KEY}.pub"