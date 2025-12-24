#!/bin/bash

echo "xxx.xxx.xxx.xxx saveserver.test" | sudo tee -a /etc/hosts

echo "xxx.xxx.xxx.xxx saveserver" | sudo tee -a /etc/hosts

sudo apt install jq

ssh-keygen -t ed25519 -f ~/.ssh/tunnel_key -N""


# flemme de test ça pour l'instant
# Host saveserver-tunnel
#   HostName saveserver
#   User tunnel
#   IdentityFile ~/.ssh/tunnel_key
#   ExitOnForwardFailure yes
#   ServerAliveInterval 30
#   ServerAliveCountMax 3
#   RemoteForward 22221 localhost:22
