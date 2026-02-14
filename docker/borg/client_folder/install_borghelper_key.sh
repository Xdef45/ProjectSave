#!/bin/bash
set -euo pipefail

PUBKEY_FILE="${1:?Usage: /path/to/pubkey}"

BORGHELPER_USER="borghelper"
BORGHELPER_HOME="/home/${BORGHELPER_USER}"
BORGHELPER_SSH_DIR="${BORGHELPER_HOME}/.ssh"
DISPATCH_SCRIPT="/usr/local/sbin/borghelper_dispatch.sh"


echo "[install_borghelper_key] installing key"


PUB="$(cat "${PUBKEY_FILE}")"
LINE="command=\"${DISPATCH_SCRIPT}\" ${PUB}"

grep -qxF "${LINE}" $BORGHELPER_SSH_DIR/authorized_keys 2>/dev/null || \
echo "${LINE}" > $BORGHELPER_SSH_DIR/authorized_keys

chown $BORGHELPER_USER:$BORGHELPER_USER $BORGHELPER_SSH_DIR/authorized_keys
chmod 0600 $BORGHELPER_SSH_DIR/authorized_keys

echo "[install_borghelper_key] key installed :D"