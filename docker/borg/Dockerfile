FROM debian:latest

RUN apt-get update && apt-get install -y --no-install-recommends \
    borgbackup \
    openssh-server \
    gpg-agent \
    gpg \
    acl \
    util-linux \
    sudo

EXPOSE 22

COPY borg/server_scripts/ /usr/local/sbin/scripts/

RUN chmod +x /usr/local/sbin/scripts/*.sh

RUN /usr/local/sbin/scripts/install_all.sh

COPY borg/entrypoint.sh .

RUN chmod +x entrypoint.sh

ENTRYPOINT [ "./entrypoint.sh" ]