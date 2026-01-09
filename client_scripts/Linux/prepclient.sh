#!/bin/bash
set -euo pipefail

[ "$(id -u)" -eq 0 ] || { echo "Run as root"; exit 1; }

CLIENT_USER="ts"                 # l’utilisateur qui lance borg (toi)
BORGHELPER_USER="borghelper"

SERVER_HOST="saveserver"

# Dossiers client (doivent exister avant les backups)
BORG_KEYS_DIR="/home/${CLIENT_USER}/.config/borg/keys"
LOCAL_SSH_DIR="/home/${CLIENT_USER}/.ssh"

# Dispatcher forcé (clé server->client sur borghelper)
DISPATCH_PATH="/usr/local/sbin/borghelper_dispatch.sh"
SUDOERS_FILE="/etc/sudoers.d/borghelper"

echo "[prepclient] Installing required packages"
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
  borgbackup gpg openssh-client ca-certificates

echo "[prepclient] Ensure borg keys dir exists"
install -d -m 0700 -o "${CLIENT_USER}" -g "${CLIENT_USER}" "${BORG_KEYS_DIR}"

echo "[prepclient] Ensure ${CLIENT_USER} ssh dir exists"
install -d -m 0700 -o "${CLIENT_USER}" -g "${CLIENT_USER}" "${LOCAL_SSH_DIR}"
touch "${LOCAL_SSH_DIR}/known_hosts"
chown "${CLIENT_USER}:${CLIENT_USER}" "${LOCAL_SSH_DIR}/known_hosts"
chmod 0600 "${LOCAL_SSH_DIR}/known_hosts"

echo "[prepclient] Pin server host key (avoid interactive prompts)"
# accepte la clé du serveur (1 fois) sans interaction
ssh-keyscan -H "${SERVER_HOST}" >> "${LOCAL_SSH_DIR}/known_hosts" 2>/dev/null || true

echo "[prepclient] Ensure borghelper user exists"
if ! id "${BORGHELPER_USER}" >/dev/null 2>&1; then
  useradd -m -s /bin/bash "${BORGHELPER_USER}"
fi

echo "[prepclient] Install dispatcher"
install -m 0755 -o root -g root "${DISPATCH_PATH}" "${DISPATCH_PATH}"

echo "[prepclient] Configure sudoers for borghelper (strict)"
cat > "${SUDOERS_FILE}" <<'EOF'
borghelper ALL=(root) NOPASSWD: /bin/cat, /usr/bin/tee, /usr/bin/install, /bin/chown, /bin/chmod, /bin/rm
EOF
chmod 0440 "${SUDOERS_FILE}"

echo "[prepclient] Ensure borghelper authorized_keys directory exists"
install -d -m 0700 -o borghelper -g borghelper /home/borghelper/.ssh
# authorized_keys sera rempli par toi (avec la pubkey server_to_client), pas ici
touch /home/borghelper/.ssh/authorized_keys
chown borghelper:borghelper /home/borghelper/.ssh/authorized_keys
chmod 0600 /home/borghelper/.ssh/authorized_keys

echo "[prepclient] Done."
echo "Next steps:"
echo "  1) Put server_to_client pubkey into /home/borghelper/.ssh/authorized_keys (forced command)."
echo "  2) Run client_genkey.sh to create tunnel_key + borg_repo_key."
