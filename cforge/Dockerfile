FROM golang:alpine
# https://github.com/bluebrown/go-template-cli
RUN apk add --no-cache bash gcompat curl pwgen gettext openssl \
    && wget https://github.com/bluebrown/go-template-cli/releases/download/v0.3.1/tpl-linux-amd64 -O /usr/local/bin/tpl \
	&& chmod +x /usr/local/bin/tpl \
	&& wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/local/bin/yq \
	&& chmod +x /usr/local/bin/yq
ADD sh/* /usr/local/bin/
USER root
WORKDIR /tpl
CMD ["/usr/local/bin/init.sh"]