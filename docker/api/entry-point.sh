#!/bin/sh
cat <<EOF > /root/.ssh/config
Host borg
    IdentityFile ~/.ssh/id_ed25519
EOF
chmod 600 /root/.ssh/id_ed25519
chown root:root /root/.ssh/id_ed25519
/usr/local/sbin/strongholer