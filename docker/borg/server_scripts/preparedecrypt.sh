#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT REVERSE_PORT}" #nom client
REVERSE_PORT="${2:?Usage: $0 CLIENT REVERSE_PORT}" # port g  n  r pour reverse
LOCAL_USER="${3:?Usage: $0 CLIENT REVERSE_PORT}"

SERVER_TO_CLIENT_KEY="/home/tunnel/.ssh/server_to_client_ed25519" #cl      feed pour pouvoir reverse
SERVER_GPG_PASSFILE="/etc/backup_secrets/key.pass" # cl   gpg

# dossiers temporaires
TMPBASE="/tmp/borgkey"
CLIENT_TMP="${TMPBASE}/${CLIENT}"
PLAIN_LOCAL="${TMPBASE}/${CLIENT}.key"
ENC_LOCAL="${CLIENT_TMP}/${CLIENT}.gpg"

# Pr  -requis
[ -f "$SERVER_TO_CLIENT_KEY" ] || { echo "missing $SERVER_TO_CLIENT_KEY"; exit 1; }
[ -f "$SERVER_GPG_PASSFILE" ] || { echo "missing $SERVER_GPG_PASSFILE"; exit 1; }

# Dossiers
install -d -m 0700 -o tunnel -g tunnel "$CLIENT_TMP"
install -d -m 700 "$TMPBASE"

SSH_TUN_OPTS=(
-i "$SERVER_TO_CLIENT_KEY"
-p "$REVERSE_PORT"
-o IdentitiesOnly=yes
-o BatchMode=yes
-o StrictHostKeyChecking=no
-o UserKnownHostsFile=/dev/null
)

# 1) pull fichier chiffr   client -> stockage temporaire
ssh "${SSH_TUN_OPTS[@]}" borghelper@localhost "pullkey $CLIENT" > "$ENC_LOCAL"
test -s "$ENC_LOCAL" || { echo "pullkey returned empty data" >&2; exit 2; }
chmod 600 "$ENC_LOCAL"

# 2) d  chiffre vers dosssier temp
gpg --batch --yes --pinentry-mode loopback \
--passphrase-file "$SERVER_GPG_PASSFILE" \
-o "$PLAIN_LOCAL" -d "$ENC_LOCAL"
test -s "$PLAIN_LOCAL" || { echo "gpg produced empty keyfile" >&2; exit 3; }
chmod 600 "$PLAIN_LOCAL"

# 3) Push le keyfile clair vers le client
cat "$PLAIN_LOCAL" | ssh "${SSH_TUN_OPTS[@]}" borghelper@localhost "pushkey $CLIENT $LOCAL_USER"

# 4) Nettoyage des cl  s c  t   server
rm -f "$PLAIN_LOCAL"
rm -f "$ENC_LOCAL"

echo "OK decrypt done"