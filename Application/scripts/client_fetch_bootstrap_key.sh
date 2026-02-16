#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT}"
SERVER_HOST="${2:-saveserver}"

SRC_PATH="/srv/repos/${CLIENT}/bootstrap/${CLIENT}.gpg"
DST_PATH="/home/ts/.config/borg/keys/${CLIENT}.gpg"

# Prérequis : tunnel_key existe, dirs créés par prepclient.sh
sudo -u ts scp -i /home/ts/.ssh/tunnel_key \
  -o IdentitiesOnly=yes \
  -o BatchMode=yes \
  "tunnel@${SERVER_HOST}:${SRC_PATH}" \
  "${DST_PATH}"

sudo chown ts:ts "${DST_PATH}"
sudo chmod 600 "${DST_PATH}"

echo "OK: bootstrap key installed at ${DST_PATH}"