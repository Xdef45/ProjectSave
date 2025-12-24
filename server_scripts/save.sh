#!/bin/bash
set -euo pipefail

CLIENT_NAME="${1:?usage: $0 CLIENT_NAME FOLDER PORT}"
FOLDER="${2:?usage: $0 CLIENT_NAME FOLDER PORT}"
PORT="${3:?usage: $0 CLIENT_NAME FOLDER PORT}"

CLIENT_HOME="/srv/repos/$CLIENT_NAME"
REPO="$CLIENT_HOME/saves/$CLIENT_NAME"

KEYS_DIR="$CLIENT_HOME/.config/borg/keys"
KEY_GPG="$KEYS_DIR/$CLIENT_NAME.gpg"
KEY_CLR="$KEYS_DIR/srv_repos_${CLIENT_NAME}_saves_${CLIENT_NAME}"

MNT="/mnt/$CLIENT_NAME"

# --- cleanup auto ---
cleanup() {
  fusermount -u "$MNT" 2>/dev/null || umount -l "$MNT" 2>/dev/null || true
  [ -f "$KEY_CLR" ] && shred -u "$KEY_CLR" || true
}
trap cleanup EXIT

# --- make sure mountpoint exists ---
mkdir -p "$MNT"
chown root:backup "$MNT"
chmod 750 "$MNT"

# --- GPG decrypt (run as root with root-owned GNUPGHOME) ---
export HOME="/srv/repos"
export GNUPGHOME="$HOME/.gnupg"
umask 077

# fix gpg home perms once (you can remove after initial setup)
chown -R root:root "$GNUPGHOME"
chmod 700 "$GNUPGHOME"
find "$GNUPGHOME" -type f -exec chmod 600 {} \;

GPG_PASSPHRASE="$(< /etc/backup_secrets/key.pass)"

gpg --batch --yes --pinentry-mode loopback \
  --passphrase "$GPG_PASSPHRASE" \
  --output "$KEY_CLR" \
  --decrypt "$KEY_GPG"

# ensure backup user can read the clear keyfile for borg
chown backup:backup "$KEY_CLR"
chmod 600 "$KEY_CLR"

# --- SSH sanity check (optional but recommended) ---
ssh -p $PORT \
  -i "/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key" \
  -o IdentitiesOnly=yes -o ConnectTimeout=5 \
  ts@localhost 'echo OK' >/dev/null

# --- mount remote folder read-only ---
sshfs -p $PORT \
  -o IdentityFile="/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key" \
  -o IdentitiesOnly=yes \
  -o StrictHostKeyChecking=no \
  -o ro,reconnect,ServerAliveInterval=15,ServerAliveCountMax=3 \
  -o allow_other,default_permissions \
  "ts@localhost:$FOLDER" \
  "$MNT"

# --- run borg as backup, with correct HOME ---
sudo -u backup env HOME="$CLIENT_HOME" \
  borg create --compression zlib --stats --progress \
  "$REPO::$(date +%F_%H-%M-%S)" \
  "$MNT"

echo "backup ok"
