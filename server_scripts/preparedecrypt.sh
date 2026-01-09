  #!/bin/bash
  set -euo pipefail

  CLIENT="${1:?Usage: $0 CLIENT REVERSE_PORT}"
  REVERSE_PORT="${2:?Usage: $0 CLIENT REVERSE_PORT}"

  SERVER_TO_CLIENT_KEY="/home/tunnel/.ssh/server_to_client_ed25519"
  SERVER_GPG_PASSFILE="/etc/backup_secrets/key.pass"

  BOOTSTRAP_DIR="/srv/repos/${CLIENT}/bootstrap"
  ENC_LOCAL="${BOOTSTRAP_DIR}/${CLIENT}.gpg"

  # Temporaire en RAM (mieux)
  TMPBASE="/run/borgkey"
  PLAIN_LOCAL="${TMPBASE}/${CLIENT}.key"

  # Pré-requis
  [ -f "$SERVER_TO_CLIENT_KEY" ] || { echo "missing $SERVER_TO_CLIENT_KEY"; exit 1; }
  [ -f "$SERVER_GPG_PASSFILE" ] || { echo "missing $SERVER_GPG_PASSFILE"; exit 1; }

  # Dossiers
  install -d -m 700 "$BOOTSTRAP_DIR"
  install -d -m 700 "$TMPBASE"

  SSH_TUN_OPTS=(
    -i "$SERVER_TO_CLIENT_KEY"
    -p "$REVERSE_PORT"
    -o IdentitiesOnly=yes
    -o BatchMode=yes
    -o StrictHostKeyChecking=no
    -o UserKnownHostsFile=/dev/null
  )

  # 1) Pull le fichier chiffré depuis le client -> stockage persistent bootstrap
  ssh "${SSH_TUN_OPTS[@]}" borghelper@localhost "pullkey $CLIENT" > "$ENC_LOCAL"
  test -s "$ENC_LOCAL" || { echo "pullkey returned empty data" >&2; exit 2; }
  chmod 600 "$ENC_LOCAL"

  # 2) Déchiffre côté serveur vers RAM
  gpg --batch --yes --pinentry-mode loopback \
    --passphrase-file "$SERVER_GPG_PASSFILE" \
    -o "$PLAIN_LOCAL" -d "$ENC_LOCAL"
  test -s "$PLAIN_LOCAL" || { echo "gpg produced empty keyfile" >&2; exit 3; }
  chmod 600 "$PLAIN_LOCAL"

  # 3) Push le keyfile clair vers le client
  cat "$PLAIN_LOCAL" | ssh "${SSH_TUN_OPTS[@]}" borghelper@localhost "pushkey $CLIENT"

  # 4) Nettoyage du clair côté serveur
  rm -f "$PLAIN_LOCAL"

  echo "OK decrypt prepared"
