#!/bin/bash
set -euo pipefail
set -x

# config
BACKUP_USER="backup"
BACKUP_HOME="/srv/repos"

TUNNEL_USER="tunnel"
TUNNEL_HOME="/home/tunnel"

SECRET_DIR="/etc/backup_secrets"
SECRET_GROUP="backupsecrets"
SECRET_FILE="${SECRET_DIR}/key.pass"

SERVER_KEYS_DIR="/etc/backup_server_keys"   # plus clair que /etc/.ssh
SERVER_TO_CLIENT_KEY="${SERVER_KEYS_DIR}/server_to_client_ed25519"

TUNNEL_STATE_DIR="/var/lib/tunnel"          # utilisé par alloc_reverse_port.sh

SUDOERS_BACKUP="/etc/sudoers.d/backup-maint"

CREATE_USER_SCRIPT="/usr/local/sbin/create_user.sh"
INSTALL_CLIENT_KEY_SCRIPT="/usr/local/sbin/install_client_key.sh"
SERVER_CLEANUP_SCRIPT="/usr/local/sbin/server_cleanup_key.sh"



SUDOERS_TUNNEL="/etc/sudoers.d/tunnel-backup"

# scripts que tunnel a le droit d'exécuter en sudo
ALLOC_SCRIPT="/usr/local/sbin/alloc_reverse_port.sh"
PREPAREDECRYPT_SCRIPT="/usr/local/sbin/preparedecrypt.sh"
CLEANUP_SCRIPT="/usr/local/sbin/server_cleanup_key.sh"

#on set l'installatiion des packages
need_root() { [ "$(id -u)" -eq 0 ] || { echo "Run as root." >&2; exit 1; }; }
pkg_install() {
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -y
  apt-get install -y --no-install-recommends \
    openssh-server openssh-client \
    borgbackup \
    gpg \
    acl \
    rsync \
    ca-certificates \
    util-linux \
    sudo
}

need_root

echo "[prepareserv] Installing packages"
pkg_install

# === Users ===
echo "[prepareserv] Ensure users"
if ! id "${BACKUP_USER}" >/dev/null 2>&1; then
  useradd -d "${BACKUP_HOME}" -m -s /usr/sbin/nologin "${BACKUP_USER}"
fi

if ! id "${TUNNEL_USER}" >/dev/null 2>&1; then
  useradd -d "${TUNNEL_HOME}" -m -s /usr/sbin/nologin "${TUNNEL_USER}"
fi

# repos 
echo "[prepareserv] Prepare ${BACKUP_HOME} and permissions"
install -d -o "${BACKUP_USER}" -g "${BACKUP_USER}" -m 0750 "${BACKUP_HOME}"

# IMPORTANT: permettre aux users borg_* de traverser /srv/repos (sinon authorized_keys ignoré)
# -> on donne juste "x" aux autres, pas "r"
chmod o+x "${BACKUP_HOME}"

# secrets (oulala)
echo "[prepareserv] Prepare secrets dir ${SECRET_DIR}"
groupadd -f "${SECRET_GROUP}"

# backup et tunnel peuvent lire les secrets (si besoin)
usermod -aG "${SECRET_GROUP}" "${BACKUP_USER}"
usermod -aG "${SECRET_GROUP}" "${TUNNEL_USER}"

install -d -m 0750 -o root -g "${SECRET_GROUP}" "${SECRET_DIR}"

#perms pr la clé gpg
if [ -f "${SECRET_FILE}" ]; then
  chown root:"${SECRET_GROUP}" "${SECRET_FILE}"
  chmod 0640 "${SECRET_FILE}"
fi

# state dir pour les ports (very important :D)
echo "[prepareserv] Prepare tunnel state dir ${TUNNEL_STATE_DIR}"
install -d -m 0700 -o root -g root "${TUNNEL_STATE_DIR}"
install -d -m 0700 -o root -g root "${TUNNEL_STATE_DIR}/locks"
install -d -m 0700 -o root -g root "${TUNNEL_STATE_DIR}/clients"

# clé server to client
echo "[prepareserv] Prepare server->client SSH key in ${SERVER_KEYS_DIR}"
install -d -m 0700 -o root -g root "${SERVER_KEYS_DIR}"

if [ ! -f "${SERVER_TO_CLIENT_KEY}" ]; then
  ssh-keygen -t ed25519 -a 64 -f "${SERVER_TO_CLIENT_KEY}" -N "" -C "server_to_client"
  chmod 0600 "${SERVER_TO_CLIENT_KEY}"
  chmod 0644 "${SERVER_TO_CLIENT_KEY}.pub"
fi

# installation clé tunnel
echo "[prepareserv] Install server->client key into ${TUNNEL_HOME}/.ssh"
install -d -m 0700 -o "${TUNNEL_USER}" -g "${TUNNEL_USER}" "${TUNNEL_HOME}/.ssh"
install -m 0600 -o "${TUNNEL_USER}" -g "${TUNNEL_USER}" "${SERVER_TO_CLIENT_KEY}" "${TUNNEL_HOME}/.ssh/server_to_client_ed25519"
install -m 0644 -o "${TUNNEL_USER}" -g "${TUNNEL_USER}" "${SERVER_TO_CLIENT_KEY}.pub" "${TUNNEL_HOME}/.ssh/server_to_client_ed25519.pub"

# sudoers pr tunnel
echo "[prepareserv] Configure sudoers for ${TUNNEL_USER} (restricted)"
# vérif présence scripts
for s in "$ALLOC_SCRIPT" "$PREPAREDECRYPT_SCRIPT" "$CLEANUP_SCRIPT"; do
  if [ ! -x "$s" ]; then
    echo "[prepareserv] WARNING: $s missing or not executable (install_all.sh should have copied it)" >&2
  fi
done

cat > "${SUDOERS_TUNNEL}" <<EOF
# Allow tunnel user to run only specific maintenance scripts without password
${TUNNEL_USER} ALL=(root) NOPASSWD: ${ALLOC_SCRIPT}, ${PREPAREDECRYPT_SCRIPT}, ${CLEANUP_SCRIPT}
EOF
chmod 0440 "${SUDOERS_TUNNEL}"

# sudoers pr backup
echo "[prepareserv] Configure sudoers for ${BACKUP_USER} (restricted)"

# Vérif que les scripts existent (sinon warning)
for s in "$CREATE_USER_SCRIPT" "$INSTALL_CLIENT_KEY_SCRIPT"; do
  if [ ! -x "$s" ]; then
    echo "[prepareserv] WARNING: $s missing or not executable" >&2
  fi
done

install -d -m 0755 -o root -g root /etc/sudoers.d

cat > "${SUDOERS_BACKUP}" <<EOF
# Allow backup user to run only specific maintenance scripts without password
${BACKUP_USER} ALL=(root) NOPASSWD: ${CREATE_USER_SCRIPT}, ${INSTALL_CLIENT_KEY_SCRIPT}
EOF
chmod 0440 "${SUDOERS_BACKUP}"

if command -v visudo >/dev/null 2>&1; then
  visudo -cf "${SUDOERS_BACKUP}" || { echo "[prepareserv] ERROR: invalid sudoers file ${SUDOERS_BACKUP}" >&2; exit 1; }
fi

echo "OK: backup_user=${BACKUP_USER}, repos=${BACKUP_HOME}, secrets=${SECRET_DIR}, server_keys=${SERVER_KEYS_DIR}"

echo "[install_all] Running gpggen"
/usr/local/sbin/gen_gpg_passphrase.sh

echo "Server->client pubkey (à mettre côté client dans authorized_keys borghelper):"
cat "${SERVER_TO_CLIENT_KEY}.pub"
