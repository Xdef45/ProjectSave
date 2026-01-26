#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT}"

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root." >&2
  exit 1
fi

# --- Global paths / prerequisites (prepared by prepserv.sh) ---
SECRET_FILE="/etc/backup_secrets/key.pass"
TMPBASE="/tmp/borgkey"
TUNNEL_STATE_BASE="/var/lib/tunnel/clients"

SERVER_KEYS_DIR="/etc/backup_server_keys"   # plus clair que /etc/.ssh
SERVER_TO_CLIENT_KEY="${SERVER_KEYS_DIR}/server_to_client_ed25519"

[ -s "$SECRET_FILE" ] || { echo "missing/empty $SECRET_FILE (create it first)"; exit 1; }
[ -d "$TMPBASE" ] || { echo "missing $TMPBASE (prepserv.sh must create it)"; exit 1; }
[ -d "$TUNNEL_STATE_BASE" ] || { echo "missing $TUNNEL_STATE_BASE (prepserv.sh must create it)"; exit 1; }

# --- Per-client server layout ---
BORG_USER="borg_${CLIENT}"
HOME_DIR="/srv/repos/${CLIENT}"
REPO_DIR="${HOME_DIR}/repo"
BOOTSTRAP_DIR="${HOME_DIR}/bootstrap"
KEY_GPG="${BOOTSTRAP_DIR}/${CLIENT}.gpg"

# temp key export (server-side, short-lived)
KEY_TMP_CLEAR="${TMPBASE}/${CLIENT}.key"

# --- Create borg user (no interactive login) ---
if ! id -u "$BORG_USER" >/dev/null 2>&1; then
  # -M: do not auto-create home (we create with correct perms ourselves)
  useradd -M -d "$HOME_DIR" -s /usr/sbin/nologin "$BORG_USER"
fi

# Ensure home + repo dirs with strict perms
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0700 "$HOME_DIR"
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0700 "$REPO_DIR"
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0700 "$BOOTSTRAP_DIR"

# Make sure alloc_reverse_port.sh never needs mkdir
install -d -o root -g root -m 0700 "${TUNNEL_STATE_BASE}/${CLIENT}"

# --- Init borg repo if needed ---
# We use keyfile encryption with EMPTY passphrase (so borg doesn't prompt).
# The protection is your .gpg wrapping + short-lived plaintext on client during backup.
if [ ! -f "${REPO_DIR}/config" ]; then
  sudo -u "$BORG_USER" env HOME="$HOME_DIR" BORG_PASSCOMMAND="printf ''" \
    borg init -e keyfile "$REPO_DIR"
fi


KEY_CLEAR="${BOOTSTRAP_DIR}/${CLIENT}.key"
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

# supprimer le clair
shred -u "$KEY_CLEAR" 2>/dev/null || rm -f "$KEY_CLEAR"


# Lock down bootstrap file readability (tunnel user is in backupsecrets group via prepserv)
chown root:backupsecrets "$KEY_GPG"
chmod 0640 "$KEY_GPG"

# --- Prepare SSH skeleton for install_client_key.sh (no mkdir elsewhere) ---
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0700 "$HOME_DIR/.ssh"
install -o "$BORG_USER" -g "$BORG_USER" -m 0600 /dev/null "$HOME_DIR/.ssh/authorized_keys"

# --- Build per-client bootstrap folder (template + keys) ---
CLIENT_FOLDER_TEMPLATE="/usr/local/sbin/client_folder"
CLIENT_FOLDER_DST="${HOME_DIR}/${CLIENT}_folder"

# Copie du template (préserve perms, fichiers cachés, etc.)
rm -rf "$CLIENT_FOLDER_DST"
cp -a "$CLIENT_FOLDER_TEMPLATE" "$CLIENT_FOLDER_DST"

# Assure l'existence du sous-dossier keys/
install -d -m 0700 -o root -g root "${CLIENT_FOLDER_DST}/keys"

# Copie la clé publique server->client dans le kit
install -m 0644 -o root -g root "${SERVER_TO_CLIENT_KEY}.pub" "${CLIENT_FOLDER_DST}/keys/server_to_client.pub"
install -m 0644 -o root -g root "${KEY_GPG}" "${CLIENT_FOLDER_DST}/keys/${CLIENT}.gpg"


# (optionnel) Mets aussi la host key du serveur pour éviter prompts SSH côté client
# ssh-keyscan -H "${SERVER_HOST}" > "${CLIENT_FOLDER_DST}/keys/known_hosts" 2>/dev/null || true
#chmod 0644 "${CLIENT_FOLDER_DST}/keys/known_hosts" 2>/dev/null || true


echo "OK created/updated: user=$BORG_USER home=$HOME_DIR repo=$REPO_DIR"
echo "Bootstrap key (encrypted): $KEY_GPG"
echo "Next: run install_client_key.sh to allow borg serve access for this client."
