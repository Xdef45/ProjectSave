#!/bin/bash
set -euo pipefail

cmd="${SSH_ORIGINAL_COMMAND:-}"

CLIENT_USER="ts"
KEYS_DIR="/home/${CLIENT_USER}/.config/borg/keys"

case "$cmd" in
  pullkey\ *)
    CLIENT="${cmd#pullkey }"
    sudo -n /bin/cat "${KEYS_DIR}/${CLIENT}.gpg"
    ;;

  pushkey\ *)
    CLIENT="${cmd#pushkey }"
    sudo -n /usr/bin/install -d -m 700 "${KEYS_DIR}"
    sudo -n /usr/bin/tee "${KEYS_DIR}/${CLIENT}.key" >/dev/null
    sudo -n /bin/chown "${CLIENT_USER}:${CLIENT_USER}" "${KEYS_DIR}/${CLIENT}.key"
    sudo -n /bin/chmod 600 "${KEYS_DIR}/${CLIENT}.key"
    ;;

  cleanup\ *)
    CLIENT="${cmd#cleanup }"
    sudo -n /bin/rm -f "${KEYS_DIR}/${CLIENT}.key"
    ;;

  *)
    echo "forbidden" >&2
    exit 2
    ;;
esac