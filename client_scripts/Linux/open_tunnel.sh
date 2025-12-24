#!/bin/bash

set -euo pipefail

PORT=$(jq -r .tunnel.remote_port resp.json)

ssh -i ~/.ssh/tunnel_key -N -R $PORT:localhost:22 tunnel@saveserver