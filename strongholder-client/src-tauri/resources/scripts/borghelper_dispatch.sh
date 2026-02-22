#!/bin/bash
set -euo pipefail
set -x
cmd="${SSH_ORIGINAL_COMMAND:-}"

die(){ echo "ERR: $*" >&2; exit 2; }

# autoriser seulement des noms safe
valid_id(){ [[ "$1" =~ ^[a-zA-Z0-9._-]+$ ]]; }

parse2() {
  # attend: <action> <client> <user>
  local rest="$1"
  CLIENT="${rest%% *}"; rest="${rest#* }"
  USERNAME="${rest}"
  valid_id "$CLIENT" || die "bad client"
  valid_id "$USERNAME" || die "bad user"
  USER_HOME="$(getent passwd "$USERNAME" | cut -d: -f6)"
  [ -n "$USER_HOME" ] || die "unknown user"
  KEYS_DIR="${USER_HOME}/.config/borg/keys"
}

case "$cmd" in
  pullkey\ *)
    rest="${cmd#pullkey }"
    parse2 "$rest"
    sudo -n /bin/cat "${KEYS_DIR}/${CLIENT}.gpg"
    ;;

  pushkey\ *)
    rest="${cmd#pushkey }"
    parse2 "$rest"
    sudo -n /usr/bin/install -d -m 700 "${KEYS_DIR}"
    sudo -n /usr/bin/tee "${KEYS_DIR}/${CLIENT}.key" >/dev/null
    sudo -n /bin/chown "${USERNAME}:${USERNAME}" "${KEYS_DIR}/${CLIENT}.key"
    sudo -n /bin/chmod 600 "${KEYS_DIR}/${CLIENT}.key"
    ;;

  cleanup\ *)
    rest="${cmd#cleanup }"
    parse2 "$rest"
    sudo -n /bin/rm -f "${KEYS_DIR}/${CLIENT}.key"
    ;;

  *)
    die "forbidden"
    ;;
esac