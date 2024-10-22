{{- with .Values }}
# Basic settings for running in production. Change accordingly before deploying the server.

# Database

# The database vendor.
db={{ .keycloak.db }}

# The username of the database user.
db-username={{ .keycloak.dbUsername }}

# The password of the database user.
db-password={{ $.Values.global.keycloakDB.password }}

db-schema=public

db-url-database={{ .keycloak.dbUrlDatabase }}
db-url-host={{ .keycloak.dbUrlHost }}
db-url-port={{ .keycloak.dbUrlPort }}

# Observability

# If the server should expose healthcheck endpoints.
health-enabled={{ .keycloak.healthEnabled }}

# If the server should expose metrics endpoints.
metrics-enabled={{ .keycloak.metricsEnabled }}

# HTTP

http-relative-path={{ .keycloak.httpRelativePath }}

# The file path to a server certificate or certificate chain in PEM format.
# https-certificate-file={{ .keycloak.httpsCertificateFile }}

# The file path to a private key in PEM format.
# https-certificate-key-file={{ .keycloak.httpsCertificateKeyFile }}

# The proxy address forwarding mode if the server is behind a reverse proxy.
proxy={{ .keycloak.proxy }}

proxy-headers={{ .keycloak.proxyHeaders }}

# Do not attach route to cookies and rely on the session affinity capabilities from reverse proxy
spi-sticky-session-encoder-infinispan-should-attach-route={{ .keycloak.spiStickySessionEncoderInfinispanShouldAttachRoute }}

# Hostname for the Keycloak server.
hostname={{ .keycloak.hostname }}
{{- end }}