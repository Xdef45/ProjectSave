#!/bin/bash
set -euo pipefail
set -x

CLIENT="${1:?Usage: $0 CLIENT ARCHIVE}" #nom client

REPOSITORY_PATH="/srv/repos/${CLIENT}/repo/"

ARCHIVE="${2-}"

if [ -z $ARCHIVE ]; then
    sudo -u "${CLIENT}" borg list "${REPOSITORY_PATH}" 
else
    sudo -u "${CLIENT}" borg list "${REPOSITORY_PATH}"::"${ARCHIVE}" --json-lines
fi