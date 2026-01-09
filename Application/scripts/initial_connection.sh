#!/bin/bash
set -euo pipefail

CLIENT_NAME="${1:?usage: $0 CLIENT_NAME}"

SSH_PUB_KEY="$(cat ~/.ssh/tunnel_key.pub)"

# 1) save de la clé tunnel
curl -sS -f -X POST "http://api.test/v1/clients/register-tunnel-key" \
  -H "Content-Type: application/json" \
  -d "$(jq -n \
        --arg client_id "$CLIENT_NAME" \
        --arg ssh_public_key "$SSH_PUB_KEY" \
        '{client_id:$client_id, ssh_public_key:$ssh_public_key}')" \
  >/dev/null

# 2) 1st connect
curl -sS -f -X POST "http://api.test/v1/clients/first-connect" \
  -H "Content-Type: application/json" \
  -d "$(jq -n \
        --arg client_id "$CLIENT_NAME" \
        '{client_id:$client_id}')" \
  > resp.json

# 3) récup des items
jq -er '.borg_keyfile_gpg_b64' resp.json | base64 -d > borg_keyfile.gpg
chmod 600 borg_keyfile.gpg

jq -er '.server_access_ssh_public_key' resp.json > server_access_ssh_public_key.pub
chmod 644 server_access_ssh_public_key.pub

cat server_access_ssh_public_key.pub | sudo tee -a .ssh/authorized_keys