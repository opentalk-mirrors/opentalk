global:
  otDomain: $OT_DOMAIN
  controllerDB:
    databasePassword: ${CONTROLLER_DB_PASSWORD}
  keycloakDB:
    databasePassword: ${KEYCLOAK_DB_PASSWORD}
  keycloak:
    keycloakAdminPassword: ${KEYCLOAK_ADMIN_PASSWORD}
    keycloakClientSecret: ${KEYCLOAK_CLIENT_SECRET_FRONTEND}
    keycloakClientSecret: ${KEYCLOAK_CLIENT_SECRET_BACKEND}
    keycloakClientSecret: ${KEYCLOAK_CLIENT_SECRET_RECORDER}
    keycloakClientSecret: ${KEYCLOAK_CLIENT_SECRET_OBELISK}
  spacedeckApiKey: ${WHITEBOARD_API_KEY}
  spacedeckApiKey: ${WHITEBOARD_API_TOKEN_INVITE_CODE}
  etherpadApiKey: ${ETHERPAD_API_KEY}
  rabbitmqPassword: ${RABBITMQ_PASSWORD}
  rabbitmqErlangCookie: ${RABBITMQ_ERLANG_COOKIE}
  minioAccKey: ${MINIO_ACC_KEY}
  minioSecretKey: ${MINIO_SECRET_KEY}

controller:
  http:
    corsAllowedOrigin: ["https://${OT_DOMAIN}"]
  keycloak:
    baseUrl: "https://${OT_DOMAIN}/auth"
  livekit:
    url: "https://${OT_DOMAIN}/livekit"
  etherpad:
    url: "https://${OT_DOMAIN}/etherpad/"
  spacedeck:
    url: "https://${OT_DOMAIN}/whiteboard/"

