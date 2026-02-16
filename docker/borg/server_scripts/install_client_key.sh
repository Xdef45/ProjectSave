#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT /path/to/pubkey}"

PUBKEY_PATH="${2:-/srv/repos/${CLIENT}/bootstrap/borg_${CLIENT}.pub}"

BORG_USER="$CLIENT"
REPO_DIR="/srv/repos/${CLIENT}/repo"

# --- checks ---
id "$BORG_USER" >/dev/null 2>&1 || { echo "missing user $BORG_USER (run create_user.sh)"; exit 1; }
[ -s "$PUBKEY_PATH" ] || { echo "missing pubkey: $PUBKEY_PATH"; exit 1; }
[ -f "${REPO_DIR}/config" ] || { echo "missing repo: ${REPO_DIR}"; exit 1; }

# Valide que c'est bien une clé SSH publique
ssh-keygen -l -f "$PUBKEY_PATH" >/dev/null 2>&1 || { echo "invalid public key: $PUBKEY_PATH"; exit 1; }

HOME_DIR="$(getent passwd "$BORG_USER" | cut -d: -f6)"
[ -n "$HOME_DIR" ] || { echo "cannot resolve home for $BORG_USER"; exit 1; }

SSH_DIR="${HOME_DIR}/.ssh"
AUTH_KEYS="${SSH_DIR}/authorized_keys"

# --- ensure ssh dir + file (strict perms) ---
install -d -m 0700 -o "$BORG_USER" -g "$BORG_USER" "$SSH_DIR"
touch "$AUTH_KEYS"
chown "$BORG_USER:$BORG_USER" "$AUTH_KEYS"
chmod 0600 "$AUTH_KEYS"

# --- build restricted line ---
PUBKEY_CONTENT="$(cat "$PUBKEY_PATH")"
LINE="command=\"borg serve --restrict-to-path ${REPO_DIR}\",no-pty,no-agent-forwarding,no-port-forwarding,no-X11-forwarding ${PUBKEY_CONTENT}"

# Avoid duplicates (simple match on the key blob)
KEY_BLOB="$(echo "$PUBKEY_CONTENT" | awk '{print $2}')"
if grep -q "$KEY_BLOB" "$AUTH_KEYS"; then
  echo "OK: key already installed for $BORG_USER"
  exit 0
fi

echo "$LINE" >> "$AUTH_KEYS"

# Re-assert perms
chown "$BORG_USER:$BORG_USER" "$AUTH_KEYS"
chmod 0600 "$AUTH_KEYS"

echo "OK: installed borg key for $BORG_USER into $AUTH_KEYS"