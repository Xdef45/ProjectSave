FROM alpine:3.14
RUN apk update
RUN apk add openssl openssh
COPY entry-point.sh /root
RUN chmod u+x /root/entry-point.sh
RUN mkdir credentials
WORKDIR /root/credentials
ENTRYPOINT [ "/root/entry-point.sh" ]
