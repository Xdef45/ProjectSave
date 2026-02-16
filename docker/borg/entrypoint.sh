#!/bin/bash

cat /root/.ssh/id_ed25519.pub > /srv/repos/api/.ssh/authorized_keys 

exec /usr/sbin/sshd -D