#!/bin/bash
set -euo pipefail

[ "$(id -u)" -eq 0 ] || { echo "Run as root"; exit 1; }

CLIENT_USER="$USER"                 # l’utilisateur qui lance borg (subject to change lol)
BORGHELPER_USER="borghelper"

SERVER_HOST="saveserver"

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

# echo "[prepclient] Pin server host key (avoid interactive prompts)"
echo "xxx.xxx.xxx.xxx saveserver" >> sudo tee -aa /etc/hosts > /dev/null
# accepte la clé du serveur (1 fois) sans interaction
ssh-keyscan -H "${SERVER_HOST}" >> "${LOCAL_SSH_DIR}/known_hosts" 2>/dev/null || true

echo "[prepclient] Ensure borghelper user exists"
if ! id "${BORGHELPER_USER}" >/dev/null 2>&1; then
  useradd -m -s /bin/bash "${BORGHELPER_USER}"
fi

echo "[prepclient] Configure sudoers for borghelper (strict)"
cat > "${SUDOERS_FILE}" <<'EOF'
borghelper ALL=(root) NOPASSWD: /bin/cat, /usr/bin/tee, /usr/bin/install, /bin/chown, /bin/chmod, /bin/rm
EOF
chmod 0440 "${SUDOERS_FILE}"

echo "[prepclient] Ensure borghelper authorized_keys directory exists"
install -d -m 0700 -o borghelper -g borghelper /home/borghelper/.ssh
touch /home/borghelper/.ssh/authorized_keys
chown borghelper:borghelper /home/borghelper/.ssh/authorized_keys
chmod 0600 /home/borghelper/.ssh/authorized_keys

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


echo "[prepclient] Install keys from bootstrap folder (if present)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOOT_KEYS_DIR="${SCRIPT_DIR}/keys"

# 1) known_hosts (pin host key du serveur)
if [ -f "${BOOT_KEYS_DIR}/known_hosts" ]; then
  echo "[prepclient] Installing provided known_hosts"
  install -m 0600 -o "${CLIENT_USER}" -g "${CLIENT_USER}" \
    "${BOOT_KEYS_DIR}/known_hosts" "${LOCAL_SSH_DIR}/known_hosts"
fi

# 2) server_to_client.pub -> authorized_keys de borghelper
if [ -f "${BOOT_KEYS_DIR}/server_to_client.pub" ]; then
  echo "[prepclient] Installing server_to_client.pub into borghelper authorized_keys"

  # Option: clé brute (simple)
  PUB="$(cat "${BOOT_KEYS_DIR}/server_to_client.pub")"

  # Option recommandée: on force une commande
  LINE="command=\"${DISPATCH_PATH}\" ${PUB}"

  grep -qxF "${LINE}" /home/borghelper/.ssh/authorized_keys 2>/dev/null || \
    echo "${LINE}" >> /home/borghelper/.ssh/authorized_keys

  chown borghelper:borghelper /home/borghelper/.ssh/authorized_keys
  chmod 0600 /home/borghelper/.ssh/authorized_keys
fi



echo "[prepclient] Done."

echo "Next steps:"
echo "  1) Put server_to_client pubkey into /home/borghelper/.ssh/authorized_keys (forced command)."

