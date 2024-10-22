FROM alpine
RUN apk add --no-cache bash curl pwgen gettext openssl ansible helm yq-go unzip tar
ADD sh/* /usr/local/bin/
ADD templates/ /templates
USER root
WORKDIR /ansible
CMD ["/usr/local/bin/init.sh"]
