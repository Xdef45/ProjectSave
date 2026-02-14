#!/bin/bash
set -euo pipefail

[ "$(id -u)" -eq 0 ] || { echo "Run as root"; exit 1; }

CLIENT_USER="$USER"                 # l’utilisateur qui lance borg (subject to change lol)
BORGHELPER_USER="borghelper"
BORGHELPER_HOME="/home/${BORGHELPER_USER}"
BORGHELPER_SSH_DIR="${BORGHELPER_HOME}/.ssh"

SERVER_HOST="saveserver"
SERVER_IP=""

USER_HOME="$(getent passwd "$CLIENT_USER" | cut -d: -f6)"
[ -n "$USER_HOME" ] || { echo "Can't resolve home for $CLIENT_USER"; exit 1; }

USER_HOME="/home/$USER"
SSH_DIR="${USER_HOME}/.ssh"
TUNNEL_KEY="${SSH_DIR}/tunnel_key"
BORG_KEY="${SSH_DIR}/borg_${CLIENT_USER}_key"

echo "[prepclient] Install all local .sh scripts to /usr/local/sbin"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for f in "${SCRIPT_DIR}"/*.sh; do
  [ -e "$f" ] || continue
  install -m 0755 -o root -g root "$f" "/usr/local/sbin/$(basename "$f")"
done

# Dossiers client (doivent exister avant les backups)
BORG_KEYS_DIR="${USER_HOME}/.config/borg/keys"
LOCAL_SSH_DIR="${USER_HOME}/.ssh"

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

echo "ajout de l'ip du server"
echo "${SERVER_IP} ${SERVER_HOST}" | sudo tee -a /etc/hosts > /dev/null
# accepte la clé du serveur (1 fois) sans interaction
ssh-keyscan -H "${SERVER_HOST}" >> "${LOCAL_SSH_DIR}/known_hosts" 2>/dev/null || true

echo "[prepclient] Ensure borghelper user exists"
if ! id "${BORGHELPER_USER}" >/dev/null 2>&1; then
  useradd -m -s /bin/bash "${BORGHELPER_USER}"
fi

echo "[prepclient] Configure sudoers for borghelper"
cat > "${SUDOERS_FILE}" <<'EOF'
borghelper ALL=(root) NOPASSWD: /bin/cat, /usr/bin/tee, /usr/bin/install, /bin/chown, /bin/chmod, /bin/rm
EOF
chmod 0440 "${SUDOERS_FILE}"

echo "[prepclient] Ensure borghelper authorized_keys directory exists"
if [ ! -f $BORGHELPER_SSH_DIR/authorized_keys ]; then
install -d -m 0700 -o borghelper -g borghelper /home/borghelper/.ssh
  touch $BORGHELPER_SSH_DIR/authorized_keys
  chown borghelper:borghelper $BORGHELPER_SSH_DIR/authorized_keys
  chmod 0600 $BORGHELPER_SSH_DIR/authorized_keys
fi


echo "[prepclient] Creating tunnel keys"

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

echo "[prepclient] Done."

