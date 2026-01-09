#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT REVERSE_PORT}"
REVERSE_PORT="${2:?Usage: $0 CLIENT REVERSE_PORT}"

SERVER_TO_CLIENT_KEY="/home/tunnel/.ssh/server_to_client_ed25519"
SERVER_GPG_PASSFILE="/etc/backup_secrets/key.pass"

# Préparé par prepareserv.sh
TMPBASE="/run/borgkey"
[ -d "$TMPBASE" ] || { echo "missing $TMPBASE (run prepareserv.sh)"; exit 1; }

ENC_LOCAL="${TMPBASE}/${CLIENT}.gpg"
PLAIN_LOCAL="${TMPBASE}/${CLIENT}.key"

[ -f "$SERVER_TO_CLIENT_KEY" ] || { echo "missing $SERVER_TO_CLIENT_KEY"; exit 1; }
[ -f "$SERVER_GPG_PASSFILE" ] || { echo "missing $SERVER_GPG_PASSFILE"; exit 1; }

SSH_TUN_OPTS=(
  -i "$SERVER_TO_CLIENT_KEY"
  -p "$REVERSE_PORT"
  -o IdentitiesOnly=yes
  -o BatchMode=yes
  -o StrictHostKeyChecking=no
  -o UserKnownHostsFile=/dev/null
)

# 1) Pull le fichier chiffré depuis le client (wawa.gpg = ${CLIENT}.gpg)
ssh "${SSH_TUN_OPTS[@]}" borghelper@localhost "pullkey $CLIENT" > "$ENC_LOCAL"
test -s "$ENC_LOCAL" || { echo "pullkey returned empty data" >&2; exit 2; }

# 2) Déchiffre côté serveur
gpg --batch --yes --pinentry-mode loopback \
  --passphrase-file "$SERVER_GPG_PASSFILE" \
  -o "$PLAIN_LOCAL" -d "$ENC_LOCAL"
test -s "$PLAIN_LOCAL" || { echo "gpg produced empty keyfile" >&2; exit 3; }

# 3) Push le keyfile clair vers le client
cat "$PLAIN_LOCAL" | ssh "${SSH_TUN_OPTS[@]}" borghelper@localhost "pushkey $CLIENT"

echo "OK decrypt prepared"
