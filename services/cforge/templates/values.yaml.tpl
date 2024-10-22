global:
  domain: $OT_DOMAIN
  defaultWebServiceProtocol: https
  controllerDB:
    password: ${CONTROLLER_DB_PASSWORD}
  keycloakDB:
    password: ${KEYCLOAK_DB_PASSWORD}
  keycloak:
    domain: "${OT_DOMAIN}"  # You must provide your own domain here - we can not set a default
    path: ''
    basePath: /auth
    # you may provide the full baseUrl instead if that is easier for you - all previous keycloak parameters would not apply then
    # baseUrl:
    realm: opentalk
    oidcClientId: Controller
    webappClientId: Webapp
    recorderClientId: Recorder
    obeliskClientId: Obelisk
    oidcClientSecret: ${KEYCLOAK_CONTROLLER_CLIENT_SECRET}
    webappClientSecret: ${KEYCLOAK_WEBAPP_CLIENT_SECRET}
    recorderClientSecret: ${KEYCLOAK_RECORDER_CLIENT_SECRET}
    obeliskClientSecret: ${KEYCLOAK_OBELISK_CLIENT_SECRET}
    testUserEnable: true
    loginTheme: opentalk
    accountTheme: opentalk
  livekit:
    #url: "https://example.com/livekit"
    domain: ${OT_DOMAIN}
    path: /livekit
    apiKey: controller
    apiSecret: ${LIVEKIT_API_SECRET}
  rabbitmq:
    user: guest
    password: guest  # You must provide your own password here - we will not set a default
    host: rabbitmq
    port: 5672
    path: '/%2F'
    # you may provide the full url instead if that is easier for you - all previous rabbitmq parameters would not apply then
    # url: amqp://guest:guest@localhost:5672
    mailTaskQueue: opentalk_mailer
    recordingTaskQueue: opentalk_recorder
  minio:
    domain: minio
    port: 9000
    path: ''
    protocol: http
    # you may provide the full uri instead if that is easier for you - all previous minio parameters would not apply then
    # uri:
    #uri: "http://minio:9000"
    accessKey: minioadmin   # You must provide your own key here - we will not set a default
    secretKey: ${MINIO_ROOT_PASSWORD}   # You must provide your own key here - we will not set a default
    bucket: opentalk
  etherpad:
    domain: ${OT_DOMAIN}
    path: '/etherpad/'
    apiKey: ${ETHERPAD_API_KEY}
  spacedeck:
    domain: ${OT_DOMAIN}
    path: /whiteboard/
    apiKey: ${SPACEDECK_API_KEY}
    inviteCode: ${SPACEDECK_INVITE_CODE}
  rabbitmqPassword: ${RABBITMQ_PASSWORD}
  rabbitmqErlangCookie: ${RABBITMQ_ERLANG_COOKIE}

controller:
  http:
    corsAllowedOrigin: ["https://${OT_DOMAIN}"]
  etherpad:
    url: "https://${OT_DOMAIN}/etherpad/"
  spacedeck:
    url: "https://${OT_DOMAIN}/whiteboard/"
keycloak:
  hostname: $OT_DOMAIN
obelisk:
  auth:
    issuer: "https://${OT_DOMAIN}/auth/realms/opentalk"
    clientId: Obelisk
    clientSecret: ${KEYCLOAK_OBELISK_CLIENT_SECRET}
  controller:
    domain: "https://${OT_DOMAIN}/controller"
recorder:
  auth:
    issuer: "https://${OT_DOMAIN}/auth/realms/opentalk"
    clientId: "Recorder"
    clientSecret: ${KEYCLOAK_RECORDER_CLIENT_SECRET}
