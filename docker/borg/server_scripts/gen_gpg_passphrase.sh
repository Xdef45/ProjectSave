#!/bin/bash

#sudo ./gen_gpg_passphrase.sh

set -euo pipefail
set -x

PASSFILE="/etc/backup_secrets/key.pass"
GROUP="backupsecrets"

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root." >&2
  exit 1
fi

if [ ! -f "$PASSFILE" ]; then
  echo "Passphrase file does not exist i am creating it :D"
  umask 077
  head -c 64 /dev/urandom | base64 > "$PASSFILE"
fi



chown root:"$GROUP" "$PASSFILE"
chmod 0640 "$PASSFILE"

echo "OK: GPG symmetric passphrase generated at $PASSFILE 😁"