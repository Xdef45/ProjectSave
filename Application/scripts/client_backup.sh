#!/bin/bash
set -euo pipefail
set -x
log(){ echo "[client_backup] $(date '+%H:%M:%S') $*"; }

CLIENT="${1:?Usage: $0 CLIENT /path/to/save}"
SRC="${2:?Usage: $0 CLIENT /path/to/save}"

SERVER_HOST="saveserver"
SERVER_SSH_PORT=22
SERVER_USER="tunnel"

CLIENT_SSH_KEY="/home/ts/.ssh/tunnel_key"
BORG_SSH_KEY="/home/ts/.ssh/borg_${CLIENT}_key"

TUNNEL_PID=""

cleanup() {
  if [[ -n "${TUNNEL_PID}" ]] && kill -0 "$TUNNEL_PID" 2>/dev/null; then
    kill "$TUNNEL_PID" 2>/dev/null || true
    wait "$TUNNEL_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# 1) demander port au serveur
log "Requesting reverse port"
REVERSE_PORT="$(
  ssh -i "$CLIENT_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=/home/ts/.ssh/known_hosts \
    "${SERVER_USER}@${SERVER_HOST}" \
    "sudo /usr/local/sbin/alloc_reverse_port.sh '${CLIENT}'"
)"
[[ "$REVERSE_PORT" =~ ^[0-9]+$ ]] || { echo "Invalid port: $REVERSE_PORT" >&2; exit 1; }
log "Got port: $REVERSE_PORT"

# 2) ouvrir tunnel
log "Opening reverse tunnel"
ssh -i "$CLIENT_SSH_KEY" \
  -o IdentitiesOnly=yes \
  -o ExitOnForwardFailure=yes \
  -o ServerAliveInterval=15 \
  -o ServerAliveCountMax=3 \
  -p "$SERVER_SSH_PORT" \
  -N -R "127.0.0.1:${REVERSE_PORT}:localhost:22" \
  "${SERVER_USER}@${SERVER_HOST}" &
TUNNEL_PID=$!

sleep 0.2
kill -0 "$TUNNEL_PID" 2>/dev/null || { echo "Tunnel died immediately" >&2; exit 1; }

# 3) decrypt (serveur -> client via tunnel)
log "Calling preparedecrypt on server"
ssh -i "$CLIENT_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes \
  -p "$SERVER_SSH_PORT" "${SERVER_USER}@${SERVER_HOST}" \
  "sudo /usr/local/sbin/preparedecrypt.sh '${CLIENT}' '${REVERSE_PORT}'"

# 4) borg create
log "Starting borg backup"
export BORG_RSH="ssh -i ${BORG_SSH_KEY} -o IdentitiesOnly=yes -o BatchMode=yes"
REPO="ssh://borg_${CLIENT}@${SERVER_HOST}:${SERVER_SSH_PORT}/srv/repos/${CLIENT}/repo"

borg create --compression zstd,6 --stats \
  "${REPO}::$(date +%F_%H-%M-%S)" \
  "$SRC"

# 5) cleanup key clair côté client (déclenché par serveur via tunnel)
log "Calling cleanup on server"
ssh -i "$CLIENT_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes \
  -p "$SERVER_SSH_PORT" "${SERVER_USER}@${SERVER_HOST}" \
  "sudo /usr/local/sbin/server_cleanup_key.sh '${CLIENT}' '${REVERSE_PORT}'"

log "Backup OK"