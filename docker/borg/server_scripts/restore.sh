#!/bin/bash
set -euo pipefail

CLIENT="${1:?Usage: $0 CLIENT ARCHIVE}" #nom client
ARCHIVE="${2:?Usage: $0 CLIENT ARCHIVE}"

REPOSITORY_PATH="/srv/repos/${CLIENT}/repo/"

