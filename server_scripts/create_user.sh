#!/bin/bash

#/usr/local/sbin/create_user.sh

#usage: Usage: $0 CLIENT_NAME

# on check que la commande a bien été écrite
if [ -z "$1" ]; then
    echo "Usage: $0 CLIENT_NAME"
    exit 1
fi

# must be root
if [ "$EUID" -ne 0 ]; then
    echo "Ce script doit être exécuté en root ( ou root SSH)."
    exit 1
fi



BORG_USER="backup"
CLIENT_NAME=$1 # logique lol
BASE_DIR="/srv/repos/$CLIENT_NAME" #director assigné au client (il pourra pas use autre chose que ça)
SAVES_DIR="$BASE_DIR/saves"
LOGS_DIR="$BASE_DIR/logs" # les logs de la sauvegarde (taille, heure de save et tt)
BORG_BASE_DIR="$BASE_DIR"
export HOME="/srv/repos"
export GNUPGHOME="$HOME/.gnupg"
umask 077

KEY_GPG="/srv/repos/$1/.config/borg/keys/$1.gpg"
KEY_CLR="/srv/repos/$1/.config/borg/keys/srv_repos_"$1"_saves_$1"

# 2. on crée le répertoire pr le client
echo "Création des dossiers pour le client $CLIENT_NAME..."
#  mkdir -p   "$SAVES_DIR"\
#             "$LOGS_DIR"\
#             # "$CLIENT_SSH_DIR"
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0750 "$SAVES_DIR"
install -d -o "$BORG_USER" -g "$BORG_USER" -m 0700 "$BORG_BASE_DIR"
install -d -o root -g root -m 0755 "/mnt/$CLIENT_NAME"




# 7 on crée le repo borg :D

export BORG_PASSCOMMAND="printf ''" # on mets une passphrase vide 
export GPG_PASSPHRASE="$(< /etc/backup_secrets/key.pass)"

sudo -u "$BORG_USER" env \
    HOME="$BASE_DIR" \
    BORG_BASE_DIR="$BORG_BASE_DIR" \
    BORG_PASSCOMMAND="printf ''"\
    borg init -e keyfile $SAVES_DIR/$CLIENT_NAME 

# ln -s $BORG_BASE_DIR/.config/borg/keys/srv_repos_"$CLIENT_NAME"_saves_"$CLIENT_NAME" $BORG_BASE_DIR/.config/borg/keys/$CLIENT_NAME

gpg --batch --yes --pinentry-mode loopback \
    --passphrase "$GPG_PASSPHRASE" \
    --output "$KEY_GPG" \
    --symmetric "$KEY_CLR" || exit

sudo chown backup:backup /srv/repos/$CLIENT_NAME/.config/borg/keys/$CLIENT_NAME.gpg
sudo chmod 600 /srv/repos/$CLIENT_NAME/.config/borg/keys/$CLIENT_NAME.gpg

chown -R backup:backup "/srv/repos/$CLIENT_NAME"
chmod 750 "/srv/repos/$CLIENT_NAME"
chmod 700 "/srv/repos/$CLIENT_NAME/.config"
chmod 700 "/srv/repos/$CLIENT_NAME/.config/borg"
chmod 700 "/srv/repos/$CLIENT_NAME/.config/borg/keys"
chmod 600 "/srv/repos/$CLIENT_NAME/.config/borg/keys/"*


sudo mkdir -p "/srv/clients/$CLIENT_NAME"

sudo touch /srv/clients/$CLIENT_NAME/tunnel_port.txt
sudo chown www-data:www-data /srv/clients/$CLIENT_NAME/tunnel_port.txt
sudo chmod 600 /srv/clients/$CLIENT_NAME/tunnel_port.txt

ssh-keygen -t ed25519 -f "/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key" -N "" -C "${CLIENT_NAME}_access"

sudo cp "/srv/repos/$CLIENT_NAME/.config/borg/keys/$CLIENT_NAME.gpg" "/srv/clients/$CLIENT_NAME/$CLIENT_NAME.gpg"

# dossier
sudo chown root:www-data "/srv/clients/$CLIENT_NAME"
sudo chmod 750 "/srv/clients/$CLIENT_NAME"

# clé privée (root only)
sudo chown root:root "/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key"
sudo chmod 600 "/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key"

# clé publique + gpg (lisibles par l'API)
sudo chown root:www-data "/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key.pub"
sudo chmod 640 "/srv/clients/$CLIENT_NAME/${CLIENT_NAME}_access_key.pub"

sudo chown root:www-data "/srv/clients/$CLIENT_NAME/$CLIENT_NAME.gpg"
sudo chmod 640 "/srv/clients/$CLIENT_NAME/$CLIENT_NAME.gpg"



