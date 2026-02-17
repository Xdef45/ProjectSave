#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT ARCHIVE}" #nom client
ARCHIVE="${2:?Usage: $0 CLIENT ARCHIVE}"

TARGET="${3-}"

API_USER="api"

REPOSITORY_PATH="/srv/repos/${CLIENT}/repo/"
RESTORE_PATH="/srv/repos/${CLIENT}/restore"

HOME_DIR="/srv/repos/${CLIENT}"
KEY_CLEAR="${HOME_DIR}/.config/borg/keys/srv_repos_${CLIENT}_repo"
API_USER="api"

if [ ! -f $KEY_CLEAR ]; then
    echo "Repository key: $KEY_CLEAR not present in .config/borg/keys of $CLIENT."
    exit 1
fi

chmod 644 "${KEY_CLEAR}"
chown ${CLIENT}:"${API_USER}" "${KEY_CLEAR}"

if [ -z $TARGET ]; then
    sudo -u "${CLIENT}" borg export-tar "${REPOSITORY_PATH}"::"${ARCHIVE}" "${RESTORE_PATH}"/"${ARCHIVE}".tar.gz
else
    cd $RESTORE_PATH
    sudo -u "${CLIENT}" borg extract "${REPOSITORY_PATH}"::"${ARCHIVE}" "${TARGET}"
    cp "${TARGET}" "${RESTORE_PATH}"
    rm -r "${TARGET}"
fi

chown "${API_USER}":"${API_USER}" "${RESTORE_PATH}"/*
chmod 700 "${RESTORE_PATH}"/*


# option pour restore un fichier seul