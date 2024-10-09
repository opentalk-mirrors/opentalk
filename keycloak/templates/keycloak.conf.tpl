# Basic settings for running in production. Change accordingly before deploying the server.

# Database

# The database vendor.
db={{ .Values.keycloak.db }}

# The username of the database user.
db-username={{ .Values.keycloak.dbUsername }}

# The password of the database user.
db-password={{ .Values.keycloak.dbPassword }}

db-schema=public

db-url-database={{ .Values.keycloak.dbUrlDatabase }}
db-url-host={{ .Values.keycloak.dbUrlHost }}
db-url-port={{ .Values.keycloak.dbUrlPort }}

# Observability

# If the server should expose healthcheck endpoints.
health-enabled={{ .Values.keycloak.healthEnabled }}

# If the server should expose metrics endpoints.
metrics-enabled={{ .Values.keycloak.metricsEnabled }}

# HTTP

http-relative-path={{ .Values.keycloak.httpRelativePath }}

# The file path to a server certificate or certificate chain in PEM format.
# https-certificate-file={{ .Values.keycloak.httpsCertificateFile }}

# The file path to a private key in PEM format.
# https-certificate-key-file={{ .Values.keycloak.httpsCertificateKeyFile }}

# The proxy address forwarding mode if the server is behind a reverse proxy.
proxy={{ .Values.keycloak.proxy }}

proxy-headers={{ .Values.keycloak.proxyHeaders }}

# Do not attach route to cookies and rely on the session affinity capabilities from reverse proxy
spi-sticky-session-encoder-infinispan-should-attach-route={{ .Values.keycloak.spiStickySessionEncoderInfinispanShouldAttachRoute }}

# Hostname for the Keycloak server.
hostname={{ .Values.keycloak.hostname }}