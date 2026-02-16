#!/bin/sh
credentials_dir=/root/credentials
ssh-keygen -t ed25519 -f $credentials_dir/id_ed25519 -P ""

MARIADB_ROOT_PASSWORD=$(openssl rand -base64 48)
MARIADB_DATABASE=strongholder
MARIADB_USER=api
MARIADB_PASSWORD=$(openssl rand -base64 48)
JWT_SECRET=$(openssl rand -base64 48)
DB_PORT=3306
DB_HOST=db

#api
if ! [ -d "$credentials_dir/api" ]; then
  mkdir $credentials_dir/api 
  cat <<EOF > $credentials_dir/.env_api
  DB_PASSWORD="$MARIADB_PASSWORD"
  DB="$MARIADB_DATABASE"
  DB_USER="$MARIADB_USER"
  DB_HOST="$DB_HOST"
  DB_PORT=$DB_PORT
  JWT_SECRET="$JWT_SECRET"
EOF
  mv $credentials_dir/.env_api $credentials_dir/api/.env
fi

#borg
if ! [ -d "$credentials_dir/borg" ]; then
  mkdir $credentials_dir/borg
  mv $credentials_dir/id_ed25519.pub $credentials_dir/borg/id_ed25519.pub
  mv $credentials_dir/id_ed25519 $credentials_dir/api/id_ed25519
fi

#db
if ! [ -d "$credentials_dir/db" ]; then
mkdir $credentials_dir/db
cat <<EOF > $credentials_dir/.env_db
MARIADB_ROOT_PASSWORD="$MARIADB_ROOT_PASSWORD"
MARIADB_PASSWORD="$MARIADB_PASSWORD"
MARIADB_DATABASE="$MARIADB_DATABASE"
MARIADB_USER="$MARIADB_USER"
EOF
mv $credentials_dir/.env_db $credentials_dir/db/.env
fi
#nginx
if ! [ -d "$credentials_dir/nginx" ]; then
  mkdir $credentials_dir/nginx
  openssl genrsa -out $credentials_dir/privatekey.pem 4096
  openssl req -x509 -new \
    -key $credentials_dir/privatekey.pem \
    -sha256 \
    -days 365 \
    -out $credentials_dir/fullchain.pem \
    -subj "/C=FR/O=Lab/OU=Infra/CN=TLS-ONLY"
  mv $credentials_dir/privatekey.pem $credentials_dir/nginx/privatekey.pem
  mv $credentials_dir/fullchain.pem $credentials_dir/nginx/fullchain.pem
fi