#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT}"

BASE=22000
MAX=22999

ROOT="/var/lib/tunnel"
LOCKDIR="$ROOT/locks"
CLIENTDIR="$ROOT/clients/$CLIENT"
PORTFILE="$CLIENTDIR/tunnel_port.txt"
LOCKFILE="$LOCKDIR/alloc.lock"

# Plus de mkdir ici : préparé par prepareserv.sh
[ -d "$LOCKDIR" ] || { echo "missing $LOCKDIR (run prepareserv.sh)"; exit 1; }
[ -d "$ROOT/clients" ] || { echo "missing $ROOT/clients (run prepareserv.sh)"; exit 1; }

# Le dossier client est le seul truc "dynamique". Si tu veux ZERO mkdir ici aussi,
# alors il faut que create_user.sh pré-crée /var/lib/tunnel/clients/$CLIENT.
[ -d "$CLIENTDIR" ] || { echo "missing $CLIENTDIR (create_user.sh must create it)"; exit 1; }

exec 9>"$LOCKFILE"
flock -x 9

if [[ -f "$PORTFILE" ]]; then
  PORT="$(<"$PORTFILE")"
  [[ "$PORT" =~ ^[0-9]+$ ]] && { echo "$PORT"; exit 0; }
fi

hash="$(printf '%s' "$CLIENT" | cksum | awk '{print $1}')"
range=$((MAX - BASE + 1))
start=$((BASE + (hash % range)))

is_listening() {
  local p="$1"
  ss -lnt 2>/dev/null | awk '{print $4}' | grep -qE "127\.0\.0\.1:${p}$|:${p}$"
}

port="$start"
for _ in $(seq 1 "$range"); do
  if ! is_listening "$port"; then
    echo "$port" > "$PORTFILE"
    chmod 600 "$PORTFILE" || true
    echo "$port"
    exit 0
  fi
  port=$((port + 1))
  (( port > MAX )) && port="$BASE"
done

echo "No free port available in ${BASE}-${MAX}" >&2
exit 1