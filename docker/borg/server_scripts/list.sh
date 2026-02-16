#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT ARCHIVE}" #nom client

REPOSITORY_PATH="/srv/repos/${CLIENT}/repo/"

HOME_DIR="/srv/repos/${CLIENT}"
KEY_CLEAR="${HOME_DIR}/.config/borg/keys/srv_repos_${CLIENT}_repo"
API_USER="api"

chmod 644 "${KEY_CLEAR}"
chown "${CLIENT}":"${API_USER}" "${KEY_CLEAR}"

ARCHIVE="${2-}"
if [ -z $ARCHIVE ]; then
    sudo -u "${CLIENT}" borg list "${REPOSITORY_PATH}" 
else
    sudo -u "${CLIENT}" borg list "${REPOSITORY_PATH}"::"${ARCHIVE}" --json-lines
fi

shred "${KEY_CLEAR}"
rm "${KEY_CLEAR}"