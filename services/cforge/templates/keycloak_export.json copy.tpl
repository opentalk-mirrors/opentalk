{{- with .Values }}
{{- with .global }}
{{- with .keycloak }}
{
  "id" : "{{ .realm }}",
  "realm" : "{{ .realm }}",
  "displayName" : "{{ .realm }}",
  "displayNameHtml" : "<div class=\"kc-logo-text\"><span>opentalk</span></div>",
  "enabled" : true,
  "roles" : {
    "realm" : [ {
      "name" : "opentalk-recorder",
      "description" : "${role_opentalk-recorder}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk"
    }, {
      "name" : "uma_authorization",
      "description" : "${role_uma_authorization}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk"
    }, {
      "name" : "offline_access",
      "description" : "${role_offline-access}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk"
    }, {
      "name" : "default-roles-opentalk",
      "description" : "${role_default-roles}",
      "composite" : true,
      "composites" : {
        "realm" : [ "offline_access", "uma_authorization" ],
        "client" : {
          "account" : [ "view-profile", "manage-account" ]
        }
      },
      "clientRole" : false,
      "containerId" : "opentalk"
    }, {
      "name" : "opentalk-call-in",
      "description" : "${role_opentalk-call-in}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk"
    } ],
    "client" : {
      "{{ .oidcClientId }}" : [ ],
      "realm-management" : [ {
        "name" : "realm-admin",
        "description" : "${role_realm-admin}",
        "composite" : true,
        "composites" : {
          "client" : {
            "realm-management" : [ "manage-users", "manage-realm", "view-users", "manage-clients", "query-groups", "view-realm", "manage-events", "query-users", "create-client", "query-clients", "view-identity-providers", "manage-identity-providers", "view-clients", "manage-authorization", "query-realms", "impersonation", "view-events", "view-authorization" ]
          }
        },
        "clientRole" : true
      }, {
        "name" : "manage-users",
        "description" : "${role_manage-users}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "manage-realm",
        "description" : "${role_manage-realm}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-users",
        "description" : "${role_view-users}",
        "composite" : true,
        "composites" : {
          "client" : {
            "realm-management" : [ "query-users", "query-groups" ]
          }
        },
        "clientRole" : true
      }, {
        "name" : "manage-clients",
        "description" : "${role_manage-clients}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "query-groups",
        "description" : "${role_query-groups}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-realm",
        "description" : "${role_view-realm}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "manage-events",
        "description" : "${role_manage-events}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "query-users",
        "description" : "${role_query-users}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "create-client",
        "description" : "${role_create-client}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "query-clients",
        "description" : "${role_query-clients}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-identity-providers",
        "description" : "${role_view-identity-providers}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "manage-identity-providers",
        "description" : "${role_manage-identity-providers}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-clients",
        "description" : "${role_view-clients}",
        "composite" : true,
        "composites" : {
          "client" : {
            "realm-management" : [ "query-clients" ]
          }
        },
        "clientRole" : true
      }, {
        "name" : "manage-authorization",
        "description" : "${role_manage-authorization}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "query-realms",
        "description" : "${role_query-realms}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "impersonation",
        "description" : "${role_impersonation}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-events",
        "description" : "${role_view-events}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-authorization",
        "description" : "${role_view-authorization}",
        "composite" : false,
        "clientRole" : true
      } ],
      "{{ .webappClientId }}" : [ ],
      "security-admin-console" : [ ],
      "{{ .recorderClientId }}" : [ ],
      "admin-cli" : [ ],
      "account-console" : [ ],
      "broker" : [ {
        "name" : "read-token",
        "description" : "${role_read-token}",
        "composite" : false,
        "clientRole" : true
      } ],
      "{{ .obeliskClientId }}" : [ ],
      "account" : [ {
        "name" : "view-groups",
        "description" : "${role_view-groups}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "delete-account",
        "description" : "${role_delete-account}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-applications",
        "description" : "${role_view-applications}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-profile",
        "description" : "${role_view-profile}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "manage-consent",
        "description" : "${role_manage-consent}",
        "composite" : true,
        "composites" : {
          "client" : {
            "account" : [ "view-consent" ]
          }
        },
        "clientRole" : true
      }, {
        "name" : "manage-account-links",
        "description" : "${role_manage-account-links}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "view-consent",
        "description" : "${role_view-consent}",
        "composite" : false,
        "clientRole" : true
      }, {
        "name" : "manage-account",
        "description" : "${role_manage-account}",
        "composite" : true,
        "composites" : {
          "client" : {
            "account" : [ "manage-account-links" ]
          }
        },
        "clientRole" : true
      } ]
    }
  },
  "groups" : [ ],
  "defaultRole" : {
    "name" : "default-roles-opentalk",
    "description" : "${role_default-roles}",
    "composite" : true,
    "clientRole" : false,
    "containerId" : "opentalk"
  },

  "users" : [ {
    "username" : "service-account-obelisk",
    "emailVerified" : false,
    "createdTimestamp" : 1668685370945,
    "enabled" : true,
    "totp" : false,
    "serviceAccountClientId" : "{{ .obeliskClientId }}",
    "credentials" : [ ],
    "disableableCredentialTypes" : [ ],
    "requiredActions" : [ ],
    "realmRoles" : [ "default-roles-opentalk", "opentalk-call-in" ],
    "notBefore" : 0,
    "groups" : [ ]
  }, {
    "username" : "service-account-otbackend",
    "emailVerified" : false,
    "createdTimestamp" : 1666710782774,
    "enabled" : true,
    "totp" : false,
    "serviceAccountClientId" : "{{ .oidcClientId }}",
    "credentials" : [ ],
    "disableableCredentialTypes" : [ ],
    "requiredActions" : [ ],
    "realmRoles" : [ "default-roles-opentalk" ],
    "clientRoles" : {
      "realm-management" : [ "query-users", "view-users" ]
    },
    "notBefore" : 0,
    "groups" : [ ]
  }, {
    "username" : "service-account-recorder",
    "emailVerified" : false,
    "createdTimestamp" : 1669034343905,
    "enabled" : true,
    "totp" : false,
    "serviceAccountClientId" : "{{ .recorderClientId }}",
    "credentials" : [ ],
    "disableableCredentialTypes" : [ ],
    "requiredActions" : [ ],
    "realmRoles" : [ "opentalk-recorder", "default-roles-opentalk" ],
    "notBefore" : 0,
    "groups" : [ ]
  }, {
    "username" : "testuser",
    "firstName" : "test",
    "lastName" : "user",
    "email" : "testuser@foo.bar",
    "emailVerified" : true,
    "createdTimestamp" : 1667901317927,
    "enabled" : {{ .testUserEnable }},
    "totp" : false,
    "credentials" : [ {
      "type" : "password",
      "userLabel" : "My password",
      "createdDate" : 1667901336402,
      "secretData" : "{\"value\":\"HgLZpV0NiOfCgFk2TLC2d764VI1HysvylXVBOiG88j2K0RVV4cnY0E6upm+efaL1fRulI0b7dXD5RQvfcR3P/A==\",\"salt\":\"aa07LVMU7JgDEjcsHTryLg==\",\"additionalParameters\":{}}",
      "credentialData" : "{\"hashIterations\":27500,\"algorithm\":\"pbkdf2-sha256\",\"additionalParameters\":{}}"
    } ],
    "disableableCredentialTypes" : [ ],
    "requiredActions" : [ ],
    "realmRoles" : [ "default-roles-opentalk" ],
    "notBefore" : 0,
    "groups" : [ ]
  } ],
  "scopeMappings" : [ {
    "clientScope" : "offline_access",
    "roles" : [ "offline_access" ]
  } ],
  "clientScopeMappings" : {
    "account" : [ {
      "client" : "account-console",
      "roles" : [ "manage-account", "view-groups" ]
    } ]
  },
  "clients" : [ {
    "clientId" : "{{ .obeliskClientId }}",
    "name" : "SIP Call-In ({{ .obeliskClientId }})",
    "description" : "",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "secret" : "{{ .obeliskClientSecret }}",
    "redirectUris" : [ ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : false,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : true,
    "publicClient" : false,
    "frontchannelLogout" : true,
    "protocol" : "openid-connect",
    "attributes" : {
      "oidc.ciba.grant.enabled" : "false",
      "client.secret.creation.time" : "1668685370",
      "backchannel.logout.session.required" : "true",
      "post.logout.redirect.uris" : "+",
      "oauth2.device.authorization.grant.enabled" : "false",
      "backchannel.logout.revoke.offline.tokens" : "false"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : true,
    "nodeReRegistrationTimeout" : -1,
    "protocolMappers" : [ {
      "name" : "Client Host",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientHost",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientHost",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "Client IP Address",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientAddress",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientAddress",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "Client ID",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientId",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientId",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ],
    "defaultClientScopes" : [ "web-origins", "acr", "profile", "roles", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "{{ .oidcClientId }}",
    "name" : "OpenTalk {{ .oidcClientId }}",
    "description" : "",
    "adminUrl" : "",
    "baseUrl" : "",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "secret" : "{{ .oidcClientSecret }}",
    "redirectUris" : [ "https://{{ .domain }}" ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : true,
    "publicClient" : false,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "backchannel.logout.session.required" : "true",
      "client_credentials.use_refresh_token" : "false",
      "post.logout.redirect.uris" : "+",
      "display.on.consent.screen" : "false",
      "oauth2.device.authorization.grant.enabled" : "false",
      "backchannel.logout.revoke.offline.tokens" : "false",
      "use.refresh.tokens" : "true",
      "exclude.session.state.from.auth.response" : "false"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : true,
    "nodeReRegistrationTimeout" : -1,
    "protocolMappers" : [ {
      "name" : "Client IP Address",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientAddress",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientAddress",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "Client ID",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientId",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientId",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "Client Host",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientHost",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientHost",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ],
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "{{ .webappClientId }}",
    "name" : "OpenTalk WebApp",
    "description" : "",
    "adminUrl" : "",
    "baseUrl" : "",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ "https://{{ .domain }}/auth/popup_callback", "https://{{ .domain }}/auth/callback", "https://{{ .domain }}/dashboard", "https://{{ .domain }}/" ],
    "webOrigins" : [ "https://{{ .domain }}" ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : false,
    "publicClient" : true,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "backchannel.logout.session.required" : "true",
      "client_credentials.use_refresh_token" : "false",
      "post.logout.redirect.uris" : "+",
      "display.on.consent.screen" : "false",
      "oauth2.device.authorization.grant.enabled" : "false",
      "backchannel.logout.revoke.offline.tokens" : "false",
      "use.refresh.tokens" : "true",
      "exclude.session.state.from.auth.response" : "false"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : true,
    "nodeReRegistrationTimeout" : -1,
    "protocolMappers" : [ {
      "name" : "group membership",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-group-membership-mapper",
      "consentRequired" : false,
      "config" : {
        "full.path" : "true",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "x_grp",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "phone number",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "aggregate.attrs" : "false",
        "multivalued" : "false",
        "userinfo.token.claim" : "true",
        "user.attribute" : "phone",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "phone_number",
        "jsonType.label" : "String"
      }
    } ],
    "defaultClientScopes" : [ "web-origins", "phone", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "{{ .recorderClientId }}",
    "name" : "Opentalk Session {{ .recorderClientId }}",
    "description" : "",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "secret" : "{{ .recorderClientSecret }}",
    "redirectUris" : [ ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : false,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : true,
    "publicClient" : false,
    "frontchannelLogout" : true,
    "protocol" : "openid-connect",
    "attributes" : {
      "oidc.ciba.grant.enabled" : "false",
      "client.secret.creation.time" : "1669034343",
      "backchannel.logout.session.required" : "true",
      "post.logout.redirect.uris" : "+",
      "oauth2.device.authorization.grant.enabled" : "false",
      "backchannel.logout.revoke.offline.tokens" : "false"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : true,
    "nodeReRegistrationTimeout" : -1,
    "protocolMappers" : [ {
      "name" : "Client IP Address",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientAddress",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientAddress",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "Client ID",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientId",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientId",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "name" : "Client Host",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "clientHost",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "clientHost",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ],
    "defaultClientScopes" : [ "web-origins", "acr", "profile", "roles", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "account",
    "name" : "${client_account}",
    "rootUrl" : "${authBaseUrl}",
    "baseUrl" : "/realms/opentalk/account/",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ "/realms/opentalk/account/*" ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : false,
    "publicClient" : true,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "post.logout.redirect.uris" : "+"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : false,
    "nodeReRegistrationTimeout" : 0,
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "account-console",
    "name" : "${client_account-console}",
    "rootUrl" : "${authBaseUrl}",
    "baseUrl" : "/realms/opentalk/account/",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ "/realms/opentalk/account/*" ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : false,
    "publicClient" : true,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "post.logout.redirect.uris" : "+",
      "pkce.code.challenge.method" : "S256"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : false,
    "nodeReRegistrationTimeout" : 0,
    "protocolMappers" : [ {
      "name" : "audience resolve",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-audience-resolve-mapper",
      "consentRequired" : false,
      "config" : { }
    } ],
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "admin-cli",
    "name" : "${client_admin-cli}",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : false,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : true,
    "serviceAccountsEnabled" : false,
    "publicClient" : true,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "post.logout.redirect.uris" : "+"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : false,
    "nodeReRegistrationTimeout" : 0,
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "broker",
    "name" : "${client_broker}",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : true,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : false,
    "publicClient" : false,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "post.logout.redirect.uris" : "+"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : false,
    "nodeReRegistrationTimeout" : 0,
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "realm-management",
    "name" : "${client_realm-management}",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ ],
    "webOrigins" : [ ],
    "notBefore" : 0,
    "bearerOnly" : true,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : false,
    "publicClient" : false,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "post.logout.redirect.uris" : "+"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : false,
    "nodeReRegistrationTimeout" : 0,
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "clientId" : "security-admin-console",
    "name" : "${client_security-admin-console}",
    "rootUrl" : "${authAdminUrl}",
    "baseUrl" : "/admin/opentalk/console/",
    "surrogateAuthRequired" : false,
    "enabled" : true,
    "alwaysDisplayInConsole" : false,
    "clientAuthenticatorType" : "client-secret",
    "redirectUris" : [ "/admin/opentalk/console/*" ],
    "webOrigins" : [ "+" ],
    "notBefore" : 0,
    "bearerOnly" : false,
    "consentRequired" : false,
    "standardFlowEnabled" : true,
    "implicitFlowEnabled" : false,
    "directAccessGrantsEnabled" : false,
    "serviceAccountsEnabled" : false,
    "publicClient" : true,
    "frontchannelLogout" : false,
    "protocol" : "openid-connect",
    "attributes" : {
      "post.logout.redirect.uris" : "+",
      "pkce.code.challenge.method" : "S256"
    },
    "authenticationFlowBindingOverrides" : { },
    "fullScopeAllowed" : false,
    "nodeReRegistrationTimeout" : 0,
    "protocolMappers" : [ {
      "name" : "locale",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "locale",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "locale",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ],
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  } ],
  "loginTheme" : "opentalk",
  "accountTheme" : "opentalk",
  "keycloakVersion" : "25.0.6"
}
{{- end }}
{{- end }}
{{- end }}