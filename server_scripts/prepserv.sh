#!/bin/bash

#/usr/local/sbin/prepareserv.sh

BACKUP_USER="backup"
BACKUP_HOME="/srv/repos"
GNUPG_DIR="${BACKUP_HOME}/.gnupg"

SECRET_DIR="/etc/backup_secrets"
SECRET_FILE="${SECRET_DIR}/key.pass"
SECRET_GROUP="backupsecrets"

sudo mkdir -p /srv/clients
sudo chown -R www-data:www-data /srv/clients
sudo chmod 700 /srv/clients


#on crée le user backup
if ! id "${BACKUP_USER}" >/dev/null 2>&1; then
  useradd -d "${BACKUP_HOME}" -m "${BACKUP_USER}"
fi


mkdir -p "${BACKUP_HOME}"
chown "${BACKUP_USER}:${BACKUP_USER}" "${BACKUP_HOME}"
chmod 750 "${BACKUP_HOME}"


sudo -u "${BACKUP_USER}" mkdir -p "${GNUPG_DIR}"
sudo -u "${BACKUP_USER}" chmod 700 "${GNUPG_DIR}"


groupadd -f "${SECRET_GROUP}"
usermod -aG "${SECRET_GROUP}" "${BACKUP_USER}"

install -d -m 750 -o root -g "${SECRET_GROUP}" "${SECRET_DIR}"

if [ ! -f "${SECRET_FILE}" ]; then
  umask 077
  openssl rand -base64 48 > "${SECRET_FILE}"
fi

chown root:"${SECRET_GROUP}" "${SECRET_FILE}"
chmod 640 "${SECRET_FILE}"

echo "OK: user=${BACKUP_USER}, gnupg=${GNUPG_DIR}, secret=${SECRET_FILE}"

#user tunnel
sudo useradd -m -s /bin/bash tunnel
sudo mkdir -p /home/tunnel/.ssh
sudo chmod 700 /home/tunnel/.ssh
sudo touch /home/tunnel/.ssh/authorized_keys
sudo chmod 600 /home/tunnel/.ssh/authorized_keys
sudo chown -R tunnel:tunnel /home/tunnel/.ssh

sudo apt install -y acl
sudo setfacl -m u:www-data:x /home/tunnel
sudo setfacl -m u:www-data:rx /home/tunnel/.ssh
sudo setfacl -m u:www-data:rw /home/tunnel/.ssh/authorized_keys



# sudo -u backup mkdir -p /srv/repos/.gnupg
# sudo -u backup chmod 700 /srv/repos/.gnupg

# sudo install -d -m 700 /etc/backup_secrets
# sudo sh -c 'openssl rand -base64 48 > /etc/backup_secrets/client_test.pass'
# sudo chmod 600 /etc/backup_secrets/client_test.pass
# sudo chown root:root /etc/backup_secrets/client_test.pass


# sudo groupadd -f backupsecrets
# sudo usermod -aG backupsecrets backup

# sudo chown root:backupsecrets /etc/backup_secrets/client_test.pass
# sudo chmod 640 /etc/backup_secrets/client_test.pass
# sudo chmod 750 /etc/backup_secrets