#!/bin/bash

cat /root/.ssh/id_ed25519.pub > /srv/repos/api/.ssh/authorized_keys 

cat > /etc/ssh/sshd_config <<'EOF'
Include /etc/ssh/sshd_config.d/*.conf

KbdInteractiveAuthentication no
UsePAM no

X11Forwarding yes
PrintMotd no

# Allow client to pass locale and color environment variables
AcceptEnv LANG LC_* COLORTERM NO_COLOR

# override default of no subsystems
Subsystem       sftp    /usr/lib/openssh/sftp-server
EOF

exec /usr/sbin/sshd -D