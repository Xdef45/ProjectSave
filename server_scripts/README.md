- sudo ./install_all.sh

sudo ./gen_gpg_passphrase.sh

send pubkey through api

sudo nano /home/borghelper/.ssh/authorized_keys  

command="/usr/local/sbin/borghelper_dispatch.sh",no-pty,no-agent-forwarding,no-X11-forwarding,no-port-forwarding <server_to_client_pubkey> server_to_client

sudo chown borghelper:borghelper /home/borghelper/.ssh/authorized_keys
sudo chmod 600 /home/borghelper/.ssh/authorized_keys


sudo create_user.sh <client_name>


sudo install_client_key.sh <client_name> <path_to_pubkey>
