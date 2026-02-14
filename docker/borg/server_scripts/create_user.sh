#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT}"

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root." >&2
  exit 1
fi

# --- Per-client server layout ---
BORG_USER="$CLIENT"
HOME_DIR="/srv/repos/${CLIENT}"
REPO_DIR="${HOME_DIR}/repo"
BOOTSTRAP_DIR="${HOME_DIR}/bootstrap"
KEY_GPG="${BOOTSTRAP_DIR}/${CLIENT}.gpg"
RESTORE_PATH="/srv/repos/${CLIENT}/restore"
API_USER="api"

# --- Global paths / prerequisites (prepared by prepserv.sh) ---
SECRET_FILE="/etc/backup_secrets/key.pass"
TMPBASE="/tmp/borgkey"
TUNNEL_STATE_BASE="/var/lib/tunnel/clients"

SERVER_KEYS_DIR="/etc/backup_server_keys"   # plus clair que /etc/.ssh
SERVER_TO_CLIENT_KEY="${SERVER_KEYS_DIR}/server_to_client_ed25519"

[ -s "$SECRET_FILE" ] || { echo "missing/empty $SECRET_FILE (create it first)"; exit 1; }
if [ ! -f "${TMPBASE}" ]; then
  install -d -m 2770 -o $API_USER -g borgkey /tmp/borgkey "${TMPBASE}"
fi
# [ -d "$TUNNEL_STATE_BASE" ] || { echo "missing $TUNNEL_STATE_BASE (prepserv.sh must create it)"; exit 1; }


# temp key export (server-side, short-lived)
KEY_TMP_CLEAR="${TMPBASE}/${CLIENT}.key"

# --- Create borg user (no interactive login) ---
if ! id -u "$BORG_USER" >/dev/null 2>&1; then
  # -M: do not auto-create home (we create with correct perms ourselves)
  useradd -M -d "$HOME_DIR" -s /bin/sh "$BORG_USER"
  passwd -d $BORG_USER
fi

# Ensure home + repo dirs with strict perms
install -d -o "$BORG_USER" -g "$API_USER" -m 0750 "$HOME_DIR"
install -d -o "$BORG_USER" -g "$API_USER" -m 0750 "$REPO_DIR"
install -d -o "$BORG_USER" -g "$API_USER" -m 0750 "$BOOTSTRAP_DIR"
install -d -o "$BORG_USER" -g "$API_USER" -m 0750 "$RESTORE_PATH"

# Make sure alloc_reverse_port.sh never needs mkdir
install -d -o root -g root -m 0700 "${TUNNEL_STATE_BASE}/${CLIENT}"

# --- Init borg repo if needed ---
# We use keyfile encryption with EMPTY passphrase (so borg doesn't prompt).
# The protection is your .gpg wrapping + short-lived plaintext on client during backup.
if [ ! -f "${REPO_DIR}/config" ]; then
  sudo -u "$BORG_USER" env HOME="$HOME_DIR" BORG_PASSCOMMAND="printf ''" \
    borg init -e keyfile "$REPO_DIR"
fi

KEY_CLEAR="${HOME_DIR}/.config/borg/keys/srv_repos_${CLIENT}_repo"
KEY_GPG="${BOOTSTRAP_DIR}/${CLIENT}.gpg"

# export keyfile clair (écrit par le user, donc OK)
sudo -u "$BORG_USER" env HOME="$HOME_DIR" \
  borg key export "$REPO_DIR" "$KEY_CLEAR"

test -s "$KEY_CLEAR" || { echo "ERROR: borg key export produced empty file: $KEY_CLEAR" >&2; exit 20; }

# chiffrer avec la passphrase serveur
gpg --batch --yes --pinentry-mode loopback \
  --passphrase-file "$SECRET_FILE" \
  --output "$KEY_GPG" \
  --symmetric "$KEY_CLEAR"

chown root:backupsecrets "$KEY_GPG"
chmod 0640 "$KEY_GPG"

# Lock down bootstrap file readability (tunnel user is in backupsecrets group via prepserv)
chown root:backupsecrets "$KEY_GPG"
chmod 0640 "$KEY_GPG"

# --- Prepare SSH skeleton for install_client_key.sh (no mkdir elsewhere) ---
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0700 "$HOME_DIR/.ssh"
install -o "$BORG_USER" -g "$BORG_USER" -m 0600 /dev/null "$HOME_DIR/.ssh/authorized_keys"
chmod -R 750 $HOME_DIR
chown -R $BORG_USER:$API_USER $HOME_DIR


echo "OK created/updated: user=$BORG_USER home=$HOME_DIR repo=$REPO_DIR"
echo "Bootstrap key (encrypted): $KEY_GPG"
echo "Next: run install_client_key.sh to allow borg serve access for this client."