FROM alpine:latest

RUN apk add --no-cache \
    borgbackup \
    openssh \
    bash \
    shadow

RUN ssh-keygen -A \
    && mkdir /root/.ssh \
    && chmod 700 /root/.ssh \
    && sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config \
    && sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

RUN useradd -m -s /bin/bash borg \
    && mkdir /home/borg/.ssh \
    && chmod 700 /home/borg/.ssh \
    && chown -R borg:borg /home/borg/.ssh

RUN mkdir -p /var/backups/borg && chown borg:borg /var/backups/borg
VOLUME /var/backups/borg

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D", "-e"]