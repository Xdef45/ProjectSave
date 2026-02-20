#!/bin/bash
#set -x
set -euo pipefail

log() {
  echo "[client_backup] $(date '+%H:%M:%S') $*"
}


CLIENT="${1:?Usage: $0 CLIENT /path/to/save}"
PATTERN_FILE="${2:?Usage: $0 CLIENT /path/to/save PATTERN_FILE}"
LOCAL_USER="$(id -un)"


SAVE_NAME=$(date +%F_%H-%M-%S)

LOG_DIRECTORY="$HOME/.config/borg/logs"
LOG_FILE="${LOG_DIRECTORY}/${SAVE_NAME}_${CLIENT}.log"

if [ ! -d $LOG_DIRECTORY ]; then
mkdir $LOG_DIRECTORY
fi

SERVER_HOST="strongholder.fr"
SERVER_SSH_PORT=22

SERVER_USER="tunnel"

#Clé client -> serveur (reverse tunnel + commande remote)
TUNNEL_SSH_KEY="$HOME/.ssh/borg_${CLIENT}_tunnel_key"
CLIENT_SSH_KEY="$HOME/.ssh/borg_${CLIENT}_key"

TUNNEL_PID=""

cleanup() {
  if [[ -n "${TUNNEL_PID}" ]] && kill -0 "$TUNNEL_PID" 2>/dev/null; then
    kill "$TUNNEL_PID" 2>/dev/null || true
    wait "$TUNNEL_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# reverse port unique par client
log "Requesting reverse port from server"
REVERSE_PORT="$(
  ssh -i $TUNNEL_SSH_KEY \
    -p $SERVER_SSH_PORT \
    -o IdentitiesOnly=yes \
    -o BatchMode=yes \
    -o StrictHostKeyChecking=no \
    tunnel@strongholder.fr \
    "sudo /usr/local/sbin/alloc_reverse_port.sh '${CLIENT}'"
)"
log "Received port: $REVERSE_PORT"

if [[ ! "$REVERSE_PORT" =~ ^[0-9]+$ ]]; then
  echo "Invalid port received: $REVERSE_PORT" >&2
  exit 1
fi


REPO="ssh://${CLIENT}@${SERVER_HOST}:${SERVER_SSH_PORT}/srv/repos/${CLIENT}/repo"

# On ouvre le tunnel en background, puis on orchestre via SSH sur le serveur.
SSH_OPTS=(
  -i "$TUNNEL_SSH_KEY"
  -o IdentitiesOnly=yes
  -o ExitOnForwardFailure=yes
  -o ServerAliveInterval=15
  -o ServerAliveCountMax=3
  -p "$SERVER_SSH_PORT"
  -o LogLevel=ERROR
  -o StrictHostKeyChecking=no
  -o UserKnownHostsFile=/dev/null
  -o GlobalKnownHostsFile=/dev/null
)

# 1) Ouvrir le reverse tunnel (background)
log "Opening reverse tunnel on port $REVERSE_PORT"
ssh "${SSH_OPTS[@]}" -N -R "127.0.0.1:${REVERSE_PORT}:localhost:22" "${SERVER_USER}@${SERVER_HOST}" &

TUNNEL_PID=$!

sleep 0.2
kill -0 "$TUNNEL_PID" 2>/dev/null || { echo "Tunnel died immediately" >&2; exit 1; }

# 2) Demander au serveur de déclencher le decrypt via tunnel
log "Calling preparedecrypt on server"
ssh "${SSH_OPTS[@]}" "${SERVER_USER}@${SERVER_HOST}" \
  "sudo /usr/local/sbin/preparedecrypt.sh ${CLIENT} ${REVERSE_PORT} ${LOCAL_USER}"

# 3) Faire le backup Borg (côté client)
log "Starting borg backup"

export BORG_RSH="ssh -p $SERVER_SSH_PORT -i $CLIENT_SSH_KEY -o IdentitiesOnly=yes -o BatchMode=yes"
{
borg create --compression zstd,6 --stats --list --json \
  "${REPO}::${SAVE_NAME}" \
  "--patterns-from" \
  "$PATTERN_FILE"
} >> "$LOG_FILE" 2>&1
borg create --compression zstd,6 \
  "${REPO}::${SAVE_NAME}_logs" \
  $LOG_FILE
shred -u $LOG_DIRECTORY/*
# 4) Cleanup de la clé claire côté client (déclenché par le serveur via tunnel)
log "Calling cleanup on server"
ssh "${SSH_OPTS[@]}" "${SERVER_USER}@${SERVER_HOST}" \
  "sudo /usr/local/sbin/server_cleanup_key.sh ${CLIENT} ${REVERSE_PORT} ${LOCAL_USER}"

echo "Backup OK"
log "Backup finished successfully"