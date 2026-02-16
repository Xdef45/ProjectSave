#!/bin/bash
set -euo pipefail
set -x

CLIENT="${1:?Usage: $0 CLIENT ARCHIVE}" #nom client
ARCHIVE="${2:?Usage: $0 CLIENT ARCHIVE}"

TARGET="${3-}"

REPOSITORY_PATH="/srv/repos/${CLIENT}/repo/"
RESTORE_PATH="/srv/repos/${CLIENT}/restore"

KEY_CLEAR="${HOME_DIR}/.config/borg/keys/srv_repos_${CLIENT}_repo"
API_USER="api"

chmod 644 "${KEY_CLEAR}"
chown ${CLIENT}:"${API_USER}" "${KEY_CLEAR}"

if [ -z $TARGET ]; then
    sudo -u "${CLIENT}" borg export-tar "${REPOSITORY_PATH}"::"${ARCHIVE}" "${RESTORE_PATH}"/"${ARCHIVE}".tar.gz
else
    cd $RESTORE_PATH
    sudo -u "${CLIENT}" borg extract "${REPOSITORY_PATH}"::"${ARCHIVE}" "${TARGET}"
fi

shred "${KEY_CLEAR}"
rm "${KEY_CLEAR}"

# option pour restore un fichier seul