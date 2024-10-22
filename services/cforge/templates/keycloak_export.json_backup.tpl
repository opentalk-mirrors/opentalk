{{- with .Values }}
{{- with .global }}
{{- with .keycloak }}
{
  "id" : "{{ .realm }}",
  "realm" : "{{ .realm }}",
  "displayName" : "{{ .realm }}",
  "displayNameHtml" : "<div class=\"kc-logo-text\"><span>opentalk</span></div>",
  "notBefore" : 0,
  "defaultSignatureAlgorithm" : "",
  "revokeRefreshToken" : false,
  "refreshTokenMaxReuse" : 0,
  "accessTokenLifespan" : 300,
  "accessTokenLifespanForImplicitFlow" : 900,
  "ssoSessionIdleTimeout" : 1800,
  "ssoSessionMaxLifespan" : 36000,
  "ssoSessionIdleTimeoutRememberMe" : 0,
  "ssoSessionMaxLifespanRememberMe" : 0,
  "offlineSessionIdleTimeout" : 2592000,
  "offlineSessionMaxLifespanEnabled" : false,
  "offlineSessionMaxLifespan" : 5184000,
  "clientSessionIdleTimeout" : 0,
  "clientSessionMaxLifespan" : 0,
  "clientOfflineSessionIdleTimeout" : 0,
  "clientOfflineSessionMaxLifespan" : 0,
  "accessCodeLifespan" : 60,
  "accessCodeLifespanUserAction" : 300,
  "accessCodeLifespanLogin" : 1800,
  "actionTokenGeneratedByAdminLifespan" : 43200,
  "actionTokenGeneratedByUserLifespan" : 300,
  "oauth2DeviceCodeLifespan" : 600,
  "oauth2DevicePollingInterval" : 5,
  "enabled" : true,
  "sslRequired" : "external",
  "registrationAllowed" : false,
  "registrationEmailAsUsername" : false,
  "rememberMe" : true,
  "verifyEmail" : false,
  "loginWithEmailAllowed" : true,
  "duplicateEmailsAllowed" : false,
  "resetPasswordAllowed" : true,
  "editUsernameAllowed" : false,
  "bruteForceProtected" : true,
  "permanentLockout" : false,
  "maxTemporaryLockouts" : 0,
  "maxFailureWaitSeconds" : 900,
  "minimumQuickLoginWaitSeconds" : 60,
  "waitIncrementSeconds" : 60,
  "quickLoginCheckMilliSeconds" : 1000,
  "maxDeltaTimeSeconds" : 43200,
  "failureFactor" : 30,
  "roles" : {
    "realm" : [ {
      "id" : "f099a60e-24f9-4281-99f6-e48feaf20447",
      "name" : "opentalk-recorder",
      "description" : "${role_opentalk-recorder}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk",
      "attributes" : { }
    }, {
      "id" : "51f06436-092a-40e9-83bd-066707326c10",
      "name" : "uma_authorization",
      "description" : "${role_uma_authorization}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk",
      "attributes" : { }
    }, {
      "id" : "2926bfcb-adaf-4609-886b-c6c00f4147a1",
      "name" : "offline_access",
      "description" : "${role_offline-access}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk",
      "attributes" : { }
    }, {
      "id" : "b6a673fe-e708-4bac-9dd8-223fe65df76d",
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
      "containerId" : "opentalk",
      "attributes" : { }
    }, {
      "id" : "69150a12-e39b-4ec5-a44f-afa2f699a055",
      "name" : "opentalk-call-in",
      "description" : "${role_opentalk-call-in}",
      "composite" : false,
      "clientRole" : false,
      "containerId" : "opentalk",
      "attributes" : { }
    } ],
    "client" : {
      "{{ .oidcClientId }}" : [ ],
      "realm-management" : [ {
        "id" : "97ed4a84-c089-462c-abee-1909e163c1b4",
        "name" : "realm-admin",
        "description" : "${role_realm-admin}",
        "composite" : true,
        "composites" : {
          "client" : {
            "realm-management" : [ "manage-users", "manage-realm", "view-users", "manage-clients", "query-groups", "view-realm", "manage-events", "query-users", "create-client", "query-clients", "view-identity-providers", "manage-identity-providers", "view-clients", "manage-authorization", "query-realms", "impersonation", "view-events", "view-authorization" ]
          }
        },
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "443fca1a-34a7-4b71-8643-1ff7ff14e8c9",
        "name" : "manage-users",
        "description" : "${role_manage-users}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "dec80935-9e78-4f1a-9171-01e5199d0b60",
        "name" : "manage-realm",
        "description" : "${role_manage-realm}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "e3dcb867-8f93-4b34-8dce-1142ca233892",
        "name" : "view-users",
        "description" : "${role_view-users}",
        "composite" : true,
        "composites" : {
          "client" : {
            "realm-management" : [ "query-users", "query-groups" ]
          }
        },
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "783382df-cd68-44d3-b757-ac51a1ba5b6e",
        "name" : "manage-clients",
        "description" : "${role_manage-clients}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "87ccda42-bb0c-4e24-80df-a5a788c9502a",
        "name" : "query-groups",
        "description" : "${role_query-groups}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "3ed78fde-af20-4f92-bfe3-ee69fcc68fe7",
        "name" : "view-realm",
        "description" : "${role_view-realm}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "bebf7998-7256-4b8a-b3b8-3d3bac5e8f33",
        "name" : "manage-events",
        "description" : "${role_manage-events}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "b284dd8c-c1d7-4549-a111-4dd0be254161",
        "name" : "query-users",
        "description" : "${role_query-users}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "49b312de-1801-4193-948c-8a96a54842fa",
        "name" : "create-client",
        "description" : "${role_create-client}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "a0dae6c4-8a97-4155-b69c-d1c707cff6b6",
        "name" : "query-clients",
        "description" : "${role_query-clients}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "cc00516b-ae68-49c4-8936-6233cbb569c9",
        "name" : "view-identity-providers",
        "description" : "${role_view-identity-providers}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "83950cf2-381c-4e7c-bf77-2e3d5e61585b",
        "name" : "manage-identity-providers",
        "description" : "${role_manage-identity-providers}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "593d09a0-54f7-437b-8812-575c101cdeba",
        "name" : "view-clients",
        "description" : "${role_view-clients}",
        "composite" : true,
        "composites" : {
          "client" : {
            "realm-management" : [ "query-clients" ]
          }
        },
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "8876ffcd-5469-4d2e-87c5-9f7298d8ff91",
        "name" : "manage-authorization",
        "description" : "${role_manage-authorization}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "63e75a80-b5b5-4515-8ae0-73837d0c1651",
        "name" : "query-realms",
        "description" : "${role_query-realms}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "2e37ac1a-3ae8-47bd-8444-e91bd29c70f8",
        "name" : "impersonation",
        "description" : "${role_impersonation}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "bc601b8a-997a-47e5-afd1-2b30edd2ab58",
        "name" : "view-events",
        "description" : "${role_view-events}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      }, {
        "id" : "2df0f735-8185-4ac8-baba-f4f7e504a312",
        "name" : "view-authorization",
        "description" : "${role_view-authorization}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
        "attributes" : { }
      } ],
      "{{ .webappClientId }}" : [ ],
      "security-admin-console" : [ ],
      "{{ .recorderClientId }}" : [ ],
      "admin-cli" : [ ],
      "account-console" : [ ],
      "broker" : [ {
        "id" : "5052d28b-8b49-44f2-93fb-4f8beb8ff636",
        "name" : "read-token",
        "description" : "${role_read-token}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "43b53d8e-7af3-4876-b058-b8a29d60eb85",
        "attributes" : { }
      } ],
      "{{ .obeliskClientId }}" : [ ],
      "account" : [ {
        "id" : "dc6b0a05-20d6-4a7f-aa90-a75de2cb6a8a",
        "name" : "view-groups",
        "description" : "${role_view-groups}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "d18b628a-8746-4996-bc36-942b1c8a9438",
        "name" : "delete-account",
        "description" : "${role_delete-account}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "a7bc3639-e8be-4d20-89a2-c201bd0ca243",
        "name" : "view-applications",
        "description" : "${role_view-applications}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "a11c0fe9-effb-4745-b207-880691fea62e",
        "name" : "view-profile",
        "description" : "${role_view-profile}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "0d5bf788-0e68-4a57-8aba-dd0000b1579e",
        "name" : "manage-consent",
        "description" : "${role_manage-consent}",
        "composite" : true,
        "composites" : {
          "client" : {
            "account" : [ "view-consent" ]
          }
        },
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "bed7d716-567b-412f-9670-7bd6c02dc225",
        "name" : "manage-account-links",
        "description" : "${role_manage-account-links}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "f9c3a23c-bfca-4643-8870-800f803bb9eb",
        "name" : "view-consent",
        "description" : "${role_view-consent}",
        "composite" : false,
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      }, {
        "id" : "b0bc1ae4-8383-47fc-afe6-4a38644b6c0a",
        "name" : "manage-account",
        "description" : "${role_manage-account}",
        "composite" : true,
        "composites" : {
          "client" : {
            "account" : [ "manage-account-links" ]
          }
        },
        "clientRole" : true,
        "containerId" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
        "attributes" : { }
      } ]
    }
  },
  "groups" : [ ],
  "defaultRole" : {
    "id" : "b6a673fe-e708-4bac-9dd8-223fe65df76d",
    "name" : "default-roles-opentalk",
    "description" : "${role_default-roles}",
    "composite" : true,
    "clientRole" : false,
    "containerId" : "opentalk"
  },
  "requiredCredentials" : [ "password" ],
  "passwordPolicy" : "length(8) and notUsername",
  "otpPolicyType" : "totp",
  "otpPolicyAlgorithm" : "HmacSHA1",
  "otpPolicyInitialCounter" : 0,
  "otpPolicyDigits" : 6,
  "otpPolicyLookAheadWindow" : 1,
  "otpPolicyPeriod" : 30,
  "otpPolicyCodeReusable" : false,
  "otpSupportedApplications" : [ "totpAppFreeOTPName", "totpAppGoogleName", "totpAppMicrosoftAuthenticatorName" ],
  "localizationTexts" : { },
  "webAuthnPolicyRpEntityName" : "{{ .domain }} SSO",
  "webAuthnPolicySignatureAlgorithms" : [ "RS256", "ES256", "ES512", "RS512" ],
  "webAuthnPolicyRpId" : "{{ .domain }}",
  "webAuthnPolicyAttestationConveyancePreference" : "not specified",
  "webAuthnPolicyAuthenticatorAttachment" : "not specified",
  "webAuthnPolicyRequireResidentKey" : "not specified",
  "webAuthnPolicyUserVerificationRequirement" : "not specified",
  "webAuthnPolicyCreateTimeout" : 0,
  "webAuthnPolicyAvoidSameAuthenticatorRegister" : false,
  "webAuthnPolicyAcceptableAaguids" : [ ],
  "webAuthnPolicyExtraOrigins" : [ ],
  "webAuthnPolicyPasswordlessRpEntityName" : "keycloak",
  "webAuthnPolicyPasswordlessSignatureAlgorithms" : [ "ES256" ],
  "webAuthnPolicyPasswordlessRpId" : "",
  "webAuthnPolicyPasswordlessAttestationConveyancePreference" : "not specified",
  "webAuthnPolicyPasswordlessAuthenticatorAttachment" : "not specified",
  "webAuthnPolicyPasswordlessRequireResidentKey" : "not specified",
  "webAuthnPolicyPasswordlessUserVerificationRequirement" : "not specified",
  "webAuthnPolicyPasswordlessCreateTimeout" : 0,
  "webAuthnPolicyPasswordlessAvoidSameAuthenticatorRegister" : false,
  "webAuthnPolicyPasswordlessAcceptableAaguids" : [ ],
  "webAuthnPolicyPasswordlessExtraOrigins" : [ ],
  "users" : [ {
    "id" : "0568ab57-c6e7-45e9-85c5-b9081fcae119",
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
    "id" : "b389e1d8-35ca-42e3-aa1c-d89831ef8a8d",
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
    "id" : "e8f043cf-76a2-41e9-862c-95d32e6bcb3a",
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
    "id" : "40b652bb-fe83-4363-a025-7fa4ac0d4ef8",
    "username" : "testuser",
    "firstName" : "test",
    "lastName" : "user",
    "email" : "testuser@foo.bar",
    "emailVerified" : true,
    "createdTimestamp" : 1667901317927,
    "enabled" : {{ .testUserEnable }},
    "totp" : false,
    "credentials" : [ {
      "id" : "0ffbcb30-1178-4c36-839a-87014f2db288",
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
    "id" : "ba65d269-cb6b-4b73-9b0a-d98ed054ca7c",
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
      "id" : "405cba01-259f-43d1-9ed2-1938e9dc0a19",
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
      "id" : "15e90aaa-74e8-4b51-aef0-5131a2242c3d",
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
      "id" : "91ac5b4b-22d8-4bad-9518-23a2991aee76",
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
    "id" : "1013466f-8bee-4a4d-889d-52cbdf27c5f0",
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
      "id" : "e17ba82a-ea54-4877-87ea-979d41ef107e",
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
      "id" : "0130d112-2f22-431e-bc29-61298eaec62c",
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
      "id" : "0564a5b8-1259-429c-bab9-8feaf8f06166",
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
    "id" : "011492ed-3859-44f2-820c-76eebf1233d5",
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
      "id" : "7dbf2a6a-ac9f-4b83-a53d-c9f9e29d63f1",
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
      "id" : "fc846426-5c63-45db-9517-591944820c3e",
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
    "id" : "dccafa50-3fa1-4575-b2db-9096e63a9c13",
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
      "id" : "854a8a9e-dfb4-40af-8dd7-2f6fb9469fd6",
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
      "id" : "edda8204-c05a-4250-ab4c-7992af3cddfe",
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
      "id" : "3f049503-b66d-4917-b137-d69d804be2dd",
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
    "id" : "55f21e46-0b32-4132-b54f-04c17f9c918f",
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
    "id" : "0cc26ca6-bd23-458e-9672-69d76026e506",
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
      "id" : "ab7f0d1d-b2bf-463e-9d20-6f73e78c278d",
      "name" : "audience resolve",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-audience-resolve-mapper",
      "consentRequired" : false,
      "config" : { }
    } ],
    "defaultClientScopes" : [ "web-origins", "acr", "roles", "profile", "basic", "email" ],
    "optionalClientScopes" : [ "address", "phone", "offline_access", "microprofile-jwt" ]
  }, {
    "id" : "4b49af14-b1c2-4441-95c3-73ed18ec4239",
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
    "id" : "43b53d8e-7af3-4876-b058-b8a29d60eb85",
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
    "id" : "cad91a15-3c4d-4893-9897-ebc6dde8aef9",
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
    "id" : "5370e090-0184-4a3d-97e7-f5f0dbc5b0c2",
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
      "id" : "b3cae095-caaa-44f8-ac92-4150b6c1345e",
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
  "clientScopes" : [ {
    "id" : "aded001f-245c-415e-9c28-1478cbd57af6",
    "name" : "phone",
    "description" : "OpenID Connect built-in scope: phone",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "true",
      "consent.screen.text" : "${phoneScopeConsentText}",
      "display.on.consent.screen" : "true"
    },
    "protocolMappers" : [ {
      "id" : "a7f6b452-fc35-48cd-b04e-49c88c8eaaa6",
      "name" : "phone number verified",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "phoneNumberVerified",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "phone_number_verified",
        "jsonType.label" : "boolean",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "f4362c5f-1436-4caf-959a-6a016c99e248",
      "name" : "phone number",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "phoneNumber",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "phone_number",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ]
  }, {
    "id" : "60bc5153-cb88-47a0-9943-b72a52675411",
    "name" : "role_list",
    "description" : "SAML role list",
    "protocol" : "saml",
    "attributes" : {
      "consent.screen.text" : "${samlRoleListScopeConsentText}",
      "display.on.consent.screen" : "true"
    },
    "protocolMappers" : [ {
      "id" : "5bdea3dc-32db-4c80-8fbc-c442e869bf89",
      "name" : "role list",
      "protocol" : "saml",
      "protocolMapper" : "saml-role-list-mapper",
      "consentRequired" : false,
      "config" : {
        "single" : "false",
        "attribute.nameformat" : "Basic",
        "attribute.name" : "Role"
      }
    } ]
  }, {
    "id" : "a31c53a7-947d-4dca-9495-31b7eed515ba",
    "name" : "offline_access",
    "description" : "OpenID Connect built-in scope: offline_access",
    "protocol" : "openid-connect",
    "attributes" : {
      "consent.screen.text" : "${offlineAccessScopeConsentText}",
      "display.on.consent.screen" : "true"
    }
  }, {
    "id" : "e54da585-0daa-40ec-8776-a67374a22e15",
    "name" : "roles",
    "description" : "OpenID Connect scope for add user roles to the access token",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "false",
      "consent.screen.text" : "${rolesScopeConsentText}",
      "display.on.consent.screen" : "true"
    },
    "protocolMappers" : [ {
      "id" : "31f62439-c5c0-4511-8b4d-577c86aea3d7",
      "name" : "realm roles",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-realm-role-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "foo",
        "access.token.claim" : "true",
        "claim.name" : "realm_access.roles",
        "jsonType.label" : "String",
        "multivalued" : "true"
      }
    }, {
      "id" : "eaa3d6c9-28c5-43fd-9963-306d302c6157",
      "name" : "audience resolve",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-audience-resolve-mapper",
      "consentRequired" : false,
      "config" : { }
    }, {
      "id" : "13246ed7-64a7-4797-91cc-5390df6cc72f",
      "name" : "client roles",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-client-role-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "foo",
        "access.token.claim" : "true",
        "claim.name" : "resource_access.${client_id}.roles",
        "jsonType.label" : "String",
        "multivalued" : "true"
      }
    } ]
  }, {
    "id" : "f38b1c66-1841-4043-a5ef-36fce3154e90",
    "name" : "profile",
    "description" : "OpenID Connect built-in scope: profile",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "true",
      "consent.screen.text" : "${profileScopeConsentText}",
      "display.on.consent.screen" : "true"
    },
    "protocolMappers" : [ {
      "id" : "9eb054ab-936e-42bc-a2d9-91302040cd62",
      "name" : "family name",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-property-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "lastName",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "family_name",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "418d88ec-1dca-4327-96b5-df0b8690ed8d",
      "name" : "website",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "website",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "website",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "eb914257-f774-4b7b-a930-5100e5dbc540",
      "name" : "updated at",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "updatedAt",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "updated_at",
        "jsonType.label" : "long",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "ec998163-b0c1-4369-b68f-60a7e82dd6f5",
      "name" : "middle name",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "middleName",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "middle_name",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "a7ebfcce-9c1b-4077-b28e-c465784dda20",
      "name" : "username",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-property-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "username",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "preferred_username",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "54dbb006-6b58-40a7-b3a3-6740666cee68",
      "name" : "full name",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-full-name-mapper",
      "consentRequired" : false,
      "config" : {
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "fce80472-7ac5-465d-a095-f3995266b2f2",
      "name" : "zoneinfo",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "zoneinfo",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "zoneinfo",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "41824076-a285-4d72-a329-dc85ed2a4e27",
      "name" : "picture",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "picture",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "picture",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "620b5851-db1e-4327-9af8-3ee9eeb24b03",
      "name" : "profile",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "profile",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "profile",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "bd3f0f06-76bf-43b7-887c-dc054f86c019",
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
    }, {
      "id" : "97764e94-76c2-4ceb-bafe-c4a3e275aa81",
      "name" : "gender",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "gender",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "gender",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "46f11e73-adfb-4b6d-99d0-9f9bf263c33a",
      "name" : "given name",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-property-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "firstName",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "given_name",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "fa7ca5f7-3063-44fe-a3c0-e5bb4e9a70d0",
      "name" : "birthdate",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "birthdate",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "birthdate",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "cc225dde-18f2-452c-acc3-24b6ca92fdda",
      "name" : "nickname",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-attribute-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "nickname",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "nickname",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ]
  }, {
    "id" : "44e0050c-c159-4450-8d73-5c3efef25bf2",
    "name" : "address",
    "description" : "OpenID Connect built-in scope: address",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "true",
      "consent.screen.text" : "${addressScopeConsentText}",
      "display.on.consent.screen" : "true"
    },
    "protocolMappers" : [ {
      "id" : "f667b250-d9a5-4f5e-8b56-e85b865a472e",
      "name" : "address",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-address-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute.formatted" : "formatted",
        "user.attribute.country" : "country",
        "user.attribute.postal_code" : "postal_code",
        "userinfo.token.claim" : "true",
        "user.attribute.street" : "street",
        "id.token.claim" : "true",
        "user.attribute.region" : "region",
        "access.token.claim" : "true",
        "user.attribute.locality" : "locality"
      }
    } ]
  }, {
    "id" : "5a6bef20-7c3d-4859-8c6f-981d07f22aa0",
    "name" : "acr",
    "description" : "OpenID Connect scope for add acr (authentication context class reference) to the token",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "false",
      "display.on.consent.screen" : "false"
    },
    "protocolMappers" : [ {
      "id" : "e1cba4cc-131c-4a64-b124-6d68607ff28e",
      "name" : "acr loa level",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-acr-mapper",
      "consentRequired" : false,
      "config" : {
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "userinfo.token.claim" : "true"
      }
    } ]
  }, {
    "id" : "e0e149a9-a1ae-4895-9a9b-460cd9e94ef6",
    "name" : "email",
    "description" : "OpenID Connect built-in scope: email",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "true",
      "consent.screen.text" : "${emailScopeConsentText}",
      "display.on.consent.screen" : "true"
    },
    "protocolMappers" : [ {
      "id" : "99c12568-e357-4ee4-94a6-17d65beedd1e",
      "name" : "email",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-property-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "email",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "email",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    }, {
      "id" : "e4a52ea1-05f1-4077-8af9-2d2927494349",
      "name" : "email verified",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-property-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "emailVerified",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "email_verified",
        "jsonType.label" : "boolean",
        "userinfo.token.claim" : "true"
      }
    } ]
  }, {
    "id" : "630c98a0-ead0-4009-b054-59caebc3fa1a",
    "name" : "basic",
    "description" : "OpenID Connect scope for add all basic claims to the token",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "false",
      "display.on.consent.screen" : "false"
    },
    "protocolMappers" : [ {
      "id" : "7a1dd6c2-f675-420d-af86-6de00b3a0d9f",
      "name" : "sub",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-sub-mapper",
      "consentRequired" : false,
      "config" : {
        "introspection.token.claim" : "true",
        "access.token.claim" : "true"
      }
    }, {
      "id" : "1394a41b-5f40-486e-ae8a-4fbd87dea16e",
      "name" : "auth_time",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usersessionmodel-note-mapper",
      "consentRequired" : false,
      "config" : {
        "user.session.note" : "AUTH_TIME",
        "id.token.claim" : "true",
        "introspection.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "auth_time",
        "jsonType.label" : "long"
      }
    } ]
  }, {
    "id" : "4bb74939-dc35-4d81-928d-f3c28f21983f",
    "name" : "microprofile-jwt",
    "description" : "Microprofile - JWT built-in scope",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "true",
      "display.on.consent.screen" : "false"
    },
    "protocolMappers" : [ {
      "id" : "cdd94e9d-0345-401f-867a-a304fa07eef0",
      "name" : "groups",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-realm-role-mapper",
      "consentRequired" : false,
      "config" : {
        "multivalued" : "true",
        "userinfo.token.claim" : "true",
        "user.attribute" : "foo",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "groups",
        "jsonType.label" : "String"
      }
    }, {
      "id" : "aed7f2ba-4100-4327-8fe9-502e8e5d52d0",
      "name" : "upn",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-usermodel-property-mapper",
      "consentRequired" : false,
      "config" : {
        "user.attribute" : "username",
        "id.token.claim" : "true",
        "access.token.claim" : "true",
        "claim.name" : "upn",
        "jsonType.label" : "String",
        "userinfo.token.claim" : "true"
      }
    } ]
  }, {
    "id" : "398008e8-7c2b-4b48-9ed3-89f4f684f848",
    "name" : "web-origins",
    "description" : "OpenID Connect scope for add allowed web origins to the access token",
    "protocol" : "openid-connect",
    "attributes" : {
      "include.in.token.scope" : "false",
      "consent.screen.text" : "",
      "display.on.consent.screen" : "false"
    },
    "protocolMappers" : [ {
      "id" : "6d1a2167-ba3d-4961-acd8-4cebe9603047",
      "name" : "allowed web origins",
      "protocol" : "openid-connect",
      "protocolMapper" : "oidc-allowed-origins-mapper",
      "consentRequired" : false,
      "config" : { }
    } ]
  } ],
  "defaultDefaultClientScopes" : [ "role_list", "profile", "email", "roles", "web-origins", "acr", "basic" ],
  "defaultOptionalClientScopes" : [ "offline_access", "address", "phone", "microprofile-jwt" ],
  "browserSecurityHeaders" : {
    "contentSecurityPolicyReportOnly" : "",
    "xContentTypeOptions" : "nosniff",
    "referrerPolicy" : "no-referrer",
    "xRobotsTag" : "none",
    "xFrameOptions" : "DENY",
    "contentSecurityPolicy" : "frame-src 'self'; frame-ancestors 'self'; object-src 'none';",
    "xXSSProtection" : "1; mode=block",
    "strictTransportSecurity" : "max-age=31536000; includeSubDomains"
  },
  "smtpServer" : { },
  "loginTheme" : "opentalk",
  "accountTheme" : "opentalk",
  "eventsEnabled" : true,
  "eventsExpiration" : 7889238,
  "eventsListeners" : [ "jboss-logging" ],
  "enabledEventTypes" : [ ],
  "adminEventsEnabled" : true,
  "adminEventsDetailsEnabled" : true,
  "identityProviders" : [ ],
  "identityProviderMappers" : [ ],
  "components" : {
    "org.keycloak.services.clientregistration.policy.ClientRegistrationPolicy" : [ {
      "id" : "599676e9-4d60-45b6-9b06-9dddb8d1ae92",
      "name" : "Trusted Hosts",
      "providerId" : "trusted-hosts",
      "subType" : "anonymous",
      "subComponents" : { },
      "config" : {
        "host-sending-registration-request-must-match" : [ "true" ],
        "client-uris-must-match" : [ "true" ]
      }
    }, {
      "id" : "a8a11706-3cb2-4113-9838-14edf72e0f75",
      "name" : "Consent Required",
      "providerId" : "consent-required",
      "subType" : "anonymous",
      "subComponents" : { },
      "config" : { }
    }, {
      "id" : "b58d2693-0b6a-472f-9be3-7b85b87a61e8",
      "name" : "Allowed Client Scopes",
      "providerId" : "allowed-client-templates",
      "subType" : "authenticated",
      "subComponents" : { },
      "config" : {
        "allow-default-scopes" : [ "true" ]
      }
    }, {
      "id" : "31265691-298d-4f9d-b808-3827b6fed9db",
      "name" : "Full Scope Disabled",
      "providerId" : "scope",
      "subType" : "anonymous",
      "subComponents" : { },
      "config" : { }
    }, {
      "id" : "1d4c21cd-4366-4fe4-8270-a22e0071a010",
      "name" : "Max Clients Limit",
      "providerId" : "max-clients",
      "subType" : "anonymous",
      "subComponents" : { },
      "config" : {
        "max-clients" : [ "200" ]
      }
    }, {
      "id" : "e37c518b-f42b-4967-8b95-49f0ec8ae805",
      "name" : "Allowed Protocol Mapper Types",
      "providerId" : "allowed-protocol-mappers",
      "subType" : "anonymous",
      "subComponents" : { },
      "config" : {
        "allowed-protocol-mapper-types" : [ "oidc-usermodel-attribute-mapper", "oidc-sha256-pairwise-sub-mapper", "saml-role-list-mapper", "oidc-full-name-mapper", "oidc-address-mapper", "saml-user-property-mapper", "saml-user-attribute-mapper", "oidc-usermodel-property-mapper" ]
      }
    }, {
      "id" : "b1d4baf2-2387-4988-865b-caa3134da216",
      "name" : "Allowed Client Scopes",
      "providerId" : "allowed-client-templates",
      "subType" : "anonymous",
      "subComponents" : { },
      "config" : {
        "allow-default-scopes" : [ "true" ]
      }
    }, {
      "id" : "bbf7fd57-8d4e-404d-b6a2-72676ab087ec",
      "name" : "Allowed Protocol Mapper Types",
      "providerId" : "allowed-protocol-mappers",
      "subType" : "authenticated",
      "subComponents" : { },
      "config" : {
        "allowed-protocol-mapper-types" : [ "oidc-full-name-mapper", "oidc-usermodel-attribute-mapper", "saml-role-list-mapper", "oidc-usermodel-property-mapper", "saml-user-attribute-mapper", "oidc-sha256-pairwise-sub-mapper", "oidc-address-mapper", "saml-user-property-mapper" ]
      }
    } ],
    "org.keycloak.userprofile.UserProfileProvider" : [ {
      "id" : "af291d4f-34b2-4403-b3c2-4b2cca4c540c",
      "providerId" : "declarative-user-profile",
      "subComponents" : { },
      "config" : {
        "kc.user.profile.config" : [ "{\"attributes\":[{\"name\":\"username\",\"displayName\":\"${username}\",\"validations\":{\"length\":{\"min\":3,\"max\":255},\"username-prohibited-characters\":{},\"up-username-not-idn-homograph\":{}},\"permissions\":{\"view\":[\"admin\",\"user\"],\"edit\":[\"admin\",\"user\"]},\"multivalued\":false},{\"name\":\"email\",\"displayName\":\"${email}\",\"validations\":{\"email\":{},\"length\":{\"max\":255}},\"required\":{\"roles\":[\"user\"]},\"permissions\":{\"view\":[\"admin\",\"user\"],\"edit\":[\"admin\",\"user\"]},\"multivalued\":false},{\"name\":\"firstName\",\"displayName\":\"${firstName}\",\"validations\":{\"length\":{\"max\":255},\"person-name-prohibited-characters\":{}},\"required\":{\"roles\":[\"user\"]},\"permissions\":{\"view\":[\"admin\",\"user\"],\"edit\":[\"admin\",\"user\"]},\"multivalued\":false},{\"name\":\"lastName\",\"displayName\":\"${lastName}\",\"validations\":{\"length\":{\"max\":255},\"person-name-prohibited-characters\":{}},\"required\":{\"roles\":[\"user\"]},\"permissions\":{\"view\":[\"admin\",\"user\"],\"edit\":[\"admin\",\"user\"]},\"multivalued\":false}],\"groups\":[{\"name\":\"user-metadata\",\"displayHeader\":\"User metadata\",\"displayDescription\":\"Attributes, which refer to user metadata\"}],\"unmanagedAttributePolicy\":\"ENABLED\"}" ]
      }
    } ],
    "org.keycloak.keys.KeyProvider" : [ {
      "id" : "bda4e1f0-9eb3-4ef9-9538-909ab5c7cf99",
      "name" : "rsa-generated",
      "providerId" : "rsa-generated",
      "subComponents" : { },
      "config" : {
        "privateKey" : [ "MIIEpAIBAAKCAQEA7veCv/7JzYu/WpL+PaUkhN7y5EaAR+akwnUWej8Zs/UTmNPOFIwdSLWItkmuiv7DG9DukUKov7Klq54OcAdzpfSoxIO6Ck6TbXhywrVHiXgOAWpA4trVJKpo/b7lxwLL57YnIm4AjeyPetQ0SJYv58E2jlmrGbpzGZ2VzSgMICsSgH8MMeg9MvtRZlnoln5/X0Cl4rT6ifURztDa2ZEvswTzIDr7NRdn6V2ZP1u9I8UsrFJbdsXS/btY551eYedKbiU3iYFi2tQF/8Hw1w5fHbwDUXUIIaNdVzQPpm4ENK5P5Ccm8713UM00FzAx5wI6xE3U2iCTL4guFGE4uHhbBQIDAQABAoIBACw9UKykKe5+XmLRfzVg3xGlJ7DYl3OYMKzaWdqwV10QNDxRC+AUb3BwQRrEKcbMMskxxRTJ0y1IELEknfYh/wg8cskMvPKfCN6Vk6NVxaRewDMFXXVR/28eCDlfr7DH5QFk4LpYRIyBvK9H3VUJzt0YETZJxeoJsTjPCnZUYWhLzW4EZSYxMt5QgIBVLnW4UuAt7c0sPNVnFAUiZa8GnPd5lA537xgZbNyLrtOvjXtIm7y8ru6oZK/RmisZ42eMiNQOqPRnc3dYLpo/z9sOuiSrA2NFSjQwolfpc5htb4QJzsanNE2nrGLnPn8lBKTVUSgyLwpa2k6RYn4NFjgjo/kCgYEA/wQYtVxxfouXVH8AQbKtlmv8ZjaabssdN+qX6WlDEOCceaHPKjD8KWv/GlUM7gXpfuMZU+5bkWWzp6Lceide74ro2y8czcFwRJXRLk8j9gAydlUyiXbK0WekmlfAPf+Q9ba3PWiS8sKOjgavhWcgOfLrmATFTFq9tgdJFukpU/kCgYEA7+OPmhJ44TyBpHeHAACQYQ5iVRN0nbLUIO4wSRLThldtIbjz2KdMTAVJ34+YuUrPQX509ybym27OJeItwv4JAlL3IbW1xbkUzP5terGYyAZnOaqYE5tYnKKpKb2UTXVfdlNap7A/6uIinpulTW6GENfP4Nuq7Opkpmi1W4+O6m0CgYEA9I54J4y0ppe1k2mL1HVGtSdv0JHyved5aMNH1aaFk2zjgl2jbyG04QBZj2Ya1TTEPFQLWQsPszrl9BBiAZ86EtyS/IeMxuEICi3mgdORd/uuaP7BTUSnsfQDzaPzj0FEQe9sgK6FpdrgnOFad8NzCz+joZCJqPSoVCVeeWM2p/kCgYB0rmYZ1eRQAgM5AR58BMo1Oy7fHMB7OBFwFwsLcAnM7VdasMCC4BT/YQ8mm4z4i4e13pF76cjVi5bf3FzP3Ip7Q+mKzvykOLsCXrrXk+yxVlP9O5oO51+5iHDpqZmp925Aepi8ucDNOcMHDMPhnuG87YviXlaMOBJO0t1rtK8aHQKBgQCr5TZP5MqsXtxXRhWAtFqpSu+UK730Xtrw4JUkCBua0/9aXKicDfXrW7Ztsk3sTghOl8B2vWUVc8Gw3PIbLJSASqFEK0ConYUntEL+YEBQ2dk4qWxKxNoFtZxDE0tsAgQsm6QpFx27Hqzbl6P+dddAJcb17cPLuH2uLi06fywh7w==" ],
        "certificate" : [ "MIICnzCCAYcCBgGSLueRZDANBgkqhkiG9w0BAQsFADATMREwDwYDVQQDDAhvcGVudGFsazAeFw0yNDA5MjYxNTE0NDBaFw0zNDA5MjYxNTE2MjBaMBMxETAPBgNVBAMMCG9wZW50YWxrMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA7veCv/7JzYu/WpL+PaUkhN7y5EaAR+akwnUWej8Zs/UTmNPOFIwdSLWItkmuiv7DG9DukUKov7Klq54OcAdzpfSoxIO6Ck6TbXhywrVHiXgOAWpA4trVJKpo/b7lxwLL57YnIm4AjeyPetQ0SJYv58E2jlmrGbpzGZ2VzSgMICsSgH8MMeg9MvtRZlnoln5/X0Cl4rT6ifURztDa2ZEvswTzIDr7NRdn6V2ZP1u9I8UsrFJbdsXS/btY551eYedKbiU3iYFi2tQF/8Hw1w5fHbwDUXUIIaNdVzQPpm4ENK5P5Ccm8713UM00FzAx5wI6xE3U2iCTL4guFGE4uHhbBQIDAQABMA0GCSqGSIb3DQEBCwUAA4IBAQCFp9jCAUulewCfPNPKK81+KaU/oG6qxKVPc13oAmoXMsDiOgGuiyboerRCtrFCOu0fi+DpGJSKLk6ZQLboC49ezrofIyl0D2KW30+gavvbk2Rn3jAAEBuwCP4/W3v1QAWhCymBMhlyXeuqDsob28EWMH+YvkQJJHKmJOzJFOEI0ih973Ffyz7elxnxnzlId294UdBZth32199Qd08F/lBYjtJPI+uq8YBwe4BkVHq/ICeZVRT6mCJj3uhHWiS7fBiaFbiXHujIt0zF0o0JdDH56IxwJ36DD3QXKHMynOdJwxf6qMTM6zDn/0qpPLLKiNu0cS6HqCVDOJGe+5V08yn0" ],
        "priority" : [ "100" ]
      }
    }, {
      "id" : "e4d0764e-9f3d-4385-887d-caf428e2001f",
      "name" : "rsa-enc-generated",
      "providerId" : "rsa-enc-generated",
      "subComponents" : { },
      "config" : {
        "privateKey" : [ "MIIEpAIBAAKCAQEAw96qVy7w26Gy/FhhAyrC7E5y4U/Wx11rGhC6LyHl87K5Yp4R78Prshx+jQdsG5xRKy693PPCvd7xyNzZoUG6jFtzCJNyI20ows3glATaXiPqIqunyZny7VZ6yEKPuR7F6pC2W535tYBu1XLSq97JTROF4Z3zi2W6/NvmFX9FQwrF2x7PS9YMb0X4xxLp4AmbMs0R5ozCqp3jHa7Q2K0aPkPClLOwGey7oNQZDkw0wmPjntxd/DsQ4bx8CSaWAzh1h9oAXmT7kIhra99NmG8TPwGDISTPbetz2Ne/0m5dNSpbXxd/ctyjJzVNAVxpSRHEY7up5Yjs+T83+qbABJ/JTwIDAQABAoIBAF2kpHLHJe8TWTOs+0Cc3pFElkbjOU+aeZx+F8112u8OaPRdlzy2wgYm+lfpjwWKUAIhuDMdNWM6ZgU+lSuz9YCqLg6RE7YdN6oe7LpHsbDlYciOocrQN+TBfjUlIXqDXGMfKZgWqG5eh160JUV3X21iopTxw6DHE8VVSqTHcrEwbGLBNA0EJbGKicoTgTKFMq0/SnA/7m2ArRMGRwZynshdHpHgSZv0gtRovNW9oVkbm9b7F4qGrUDHwUZW/EEkT0tge7PUAAyEVEFdq/J2O4DhH3m8xNXUs+Re8nRsCOliDE7NF9XTjloZDcksMva8FO+XRzmGBjrrxhNKMIggpR0CgYEA7LDbHgNp/chzTBsF8dHLn72Xro1vIyzEU9vKV2tYaoDiFw4zj4k+1OX6J4fmca5M0tL/3SYGLNsRWlaJgUERKiZIJ2Pw7OFebD0lSJnqZ9PIzkFbXBJqH8HvBEoB6TNV0m31wW/s7q0at9W6sQqWlbEf1dPRKcKV2GxQl0wWtksCgYEA09lJXDwvbUWY1mCJgjN4trqH5D2KpMldO86b4jxKax/ny90uh3xvot3wTtoWZuO6zCCRnwBCDhqnPfd0nLeodhg4tBUL47QbsCs1ubX1kq6T1zg/BLd80hbUJxTTC+iLBiWWNvWBStxTT5n5NzmwWJPxfiaM5f3Mqlwfff9C5o0CgYEA3y4ULsTBWI9+SDDi9hYAjutS70BlnyFDLl9899k8UZCdn2fAroOXP2Bq9INsg5z6y4AxyvFjuUAHqMsYdUxJRxk5yAqwzmvjgORXF5IqOSS1n+Z4ouSqNK/FFKJEEwRY1EbjCDbljlijbNyXYQC+5P9TKWjFPT9B1ASuKTuR3ukCgYBTeEOdYeFV7pwfq33CnUEsziYI2e6squJfVDK7WwH/X1zXSFiy7WaXB1dN2kQFWikZkKOPwGxGXl4S3lNGBF8N2NFOsxQyLZ7TNNNATt9pYfvCStkZpKo7pN8GXwAgqwlL7LdZBDZjJph0vrgg+EvxOqlmCQ5u+CcR4KZP4KmdaQKBgQCP0HLl/AK5D+RrvCoTPIrOB4X0atkNkKfFdIDyV97vsdc7SaPaS7JgySc5J0ylQNl0fejbIN2+cMtjkBtxKx3/c2AJp1mcK2I8Ue//E3ggOcpQZfv/192zPDH7Z7O6a1L657CDFaLq6STCY1hN7tDyLIiQRlRNRxPr6aREptXwdA==" ],
        "certificate" : [ "MIICnzCCAYcCBgGSLueSADANBgkqhkiG9w0BAQsFADATMREwDwYDVQQDDAhvcGVudGFsazAeFw0yNDA5MjYxNTE0NDFaFw0zNDA5MjYxNTE2MjFaMBMxETAPBgNVBAMMCG9wZW50YWxrMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAw96qVy7w26Gy/FhhAyrC7E5y4U/Wx11rGhC6LyHl87K5Yp4R78Prshx+jQdsG5xRKy693PPCvd7xyNzZoUG6jFtzCJNyI20ows3glATaXiPqIqunyZny7VZ6yEKPuR7F6pC2W535tYBu1XLSq97JTROF4Z3zi2W6/NvmFX9FQwrF2x7PS9YMb0X4xxLp4AmbMs0R5ozCqp3jHa7Q2K0aPkPClLOwGey7oNQZDkw0wmPjntxd/DsQ4bx8CSaWAzh1h9oAXmT7kIhra99NmG8TPwGDISTPbetz2Ne/0m5dNSpbXxd/ctyjJzVNAVxpSRHEY7up5Yjs+T83+qbABJ/JTwIDAQABMA0GCSqGSIb3DQEBCwUAA4IBAQAjcx0oP0vYaLEgdLFEaNCbeDDD571fPxyH1W/BEEW8rtCVHRaqoPWjtcJST4NUheVM9+WxOW1xFAfWeA3Ck8ZxoUXE8RjlApZN1kgqGIkxwfTEch6Xa5vniWqqarEmP5azR3PO0KCj+AOSMmSDOhKKAgbLTbzMmBMTKc7cMWKAQWqMBiEbRIn0IJuRG6LdLy7gvYf5tNkZvvs4ECOlB8YA6Fcsi4gm7XGiMaFUOX6sOQ08ZKyfhHChsmp3tv5lHL1dpmTl6JCYTjTwQK0AxQVYCdPfqQPOPuHuc/QO1n1Cali8BdCntCR+UvaBfmGaNPfleTTEViAaUquz+CQasbD+" ],
        "priority" : [ "100" ],
        "algorithm" : [ "RSA-OAEP" ]
      }
    }, {
      "id" : "0773083a-c078-4751-8c74-155560aa406b",
      "name" : "hmac-generated-hs512",
      "providerId" : "hmac-generated",
      "subComponents" : { },
      "config" : {
        "kid" : [ "87809860-df67-4fb6-94f9-08ea09662fc4" ],
        "secret" : [ "LUUUu6cnsd5su9cbE1mdg2uT8RLg6YDOzf8tevutWnEtI_LrKlduoZBtxdy1iaaCJOXflpOMRJCOUnqH8l_t0TxlRo0pecIqmAyfEt-ZkD6NBhOcsyTCwQvuM77EBJC_gN6EhE6f0OB7YdgJE6LajjG-lbiFR3QyVzFan523vxE" ],
        "priority" : [ "100" ],
        "algorithm" : [ "HS512" ]
      }
    }, {
      "id" : "d24882c0-eabe-49a5-98ed-0b2a0b1a6555",
      "name" : "hmac-generated",
      "providerId" : "hmac-generated",
      "subComponents" : { },
      "config" : {
        "kid" : [ "7c169523-fbac-4c96-ae0f-183be3f9fe01" ],
        "secret" : [ "nnPyTI0UxzktOP2Slz6O-h4AaUgfpNgKMm8pmv3YxB4-6SB_MYh-THLylxwQq9Snq2VsXmmKG5xV9sghbDLstlDgElGKEdN9LptDfXCKtKFoQhARVxKOBvVQm8qBSnIjOzZbscEgJH-ftD0mVSt9ZpHJcD3kwGWbmuHiCqvTJpU" ],
        "priority" : [ "100" ],
        "algorithm" : [ "HS256" ]
      }
    }, {
      "id" : "eff5081d-7e1d-41bf-a466-e17fe3c6ec16",
      "name" : "aes-generated",
      "providerId" : "aes-generated",
      "subComponents" : { },
      "config" : {
        "kid" : [ "a1c425e7-fbda-4b87-897e-7a119cd88ba5" ],
        "secret" : [ "W6SnzUaLkD1f9l6bsRRoGw" ],
        "priority" : [ "100" ]
      }
    } ]
  },
  "internationalizationEnabled" : false,
  "supportedLocales" : [ ],
  "defaultLocale" : "",
  "authenticationFlows" : [ {
    "id" : "e76bd55b-d7d4-4247-a434-7a489968392a",
    "alias" : "2FA subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "webauthn-authenticator",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 1,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 2,
      "autheticatorFlow" : true,
      "flowAlias" : "OTP Default Subflow",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "d0dc06cc-d215-4827-8ec5-2e2bfde8e7fc",
    "alias" : "Account verification options",
    "description" : "Method with which to verity the existing account",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "idp-email-verification",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 20,
      "autheticatorFlow" : true,
      "flowAlias" : "Verify Existing Account by Re-authentication",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "46cd1d35-303a-4e59-8198-b3809a788bd9",
    "alias" : "Browser - Conditional OTP",
    "description" : "Flow to determine if the OTP is required for the authentication",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "conditional-user-configured",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "a6810147-9978-454a-8150-94ef7c2f937d",
    "alias" : "Conditional Reset Credentials 2FA Subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "conditional-user-configured",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "REQUIRED",
      "priority" : 1,
      "autheticatorFlow" : true,
      "flowAlias" : "Reset Credentials 2FA Subflow",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "49f4733d-810c-4ccc-bd27-f950e9fbf283",
    "alias" : "Direct Grant - Conditional OTP",
    "description" : "Flow to determine if the OTP is required for the authentication",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "conditional-user-configured",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "direct-grant-validate-otp",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "638e36fd-c80d-4d75-9682-c0edd096ce01",
    "alias" : "First broker login - Conditional OTP",
    "description" : "Flow to determine if the OTP is required for the authentication",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "conditional-user-configured",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "0ad1c8a1-43eb-4a46-b08c-702b70aa1a33",
    "alias" : "Handle Existing Account",
    "description" : "Handle what to do if there is existing account with same email/username like authenticated identity provider",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "idp-confirm-link",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : true,
      "flowAlias" : "Account verification options",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "14dbe097-ff2e-4a67-930e-6fc8c337ca34",
    "alias" : "IPR OTP Default Subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "da448f24-7d5b-418e-b234-c7d3561596dd",
    "alias" : "OTP Default Subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "8a1a4ebf-3acd-4191-b17f-4fee16d9369f",
    "alias" : "Password and 2FA subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-username-password-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "DISABLED",
      "priority" : 1,
      "autheticatorFlow" : true,
      "flowAlias" : "2FA subflow",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "9733bc35-ef92-4e70-9199-d3610d4cd1dc",
    "alias" : "Reset - Conditional OTP",
    "description" : "Flow to determine if the OTP should be reset or not. Set to REQUIRED to force.",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "conditional-user-configured",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "reset-otp",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "aa6d8098-5df5-4a96-8dc6-8d20f4402748",
    "alias" : "Reset Credentials 2FA Subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "webauthn-authenticator",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 1,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 2,
      "autheticatorFlow" : true,
      "flowAlias" : "Reset Credentials OTP Default Subflow",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "2918853c-2161-48a7-8235-4ab97df38be2",
    "alias" : "Reset Credentials OTP Default Subflow",
    "description" : "",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "1866b461-79ec-4043-b89d-3f45ac4b4593",
    "alias" : "User creation or linking",
    "description" : "Flow for the existing/non-existing user alternatives",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticatorConfig" : "create unique user config",
      "authenticator" : "idp-create-user-if-unique",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 20,
      "autheticatorFlow" : true,
      "flowAlias" : "Handle Existing Account",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "8c876db3-7d8f-4898-86e0-69bcf9d3086c",
    "alias" : "Verify Existing Account by Re-authentication",
    "description" : "Reauthentication of existing account",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "idp-username-password-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "CONDITIONAL",
      "priority" : 20,
      "autheticatorFlow" : true,
      "flowAlias" : "First broker login - Conditional OTP",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "b8993ca4-e567-4495-a777-c6af97b871d6",
    "alias" : "{{ .domain }} Browser",
    "description" : "Customized Browser flow that forces 2FA.",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-cookie",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "identity-provider-redirector",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 1,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 2,
      "autheticatorFlow" : true,
      "flowAlias" : "Password and 2FA subflow",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "a807b502-d773-423d-b169-287437d45737",
    "alias" : "{{ .domain }} Post IPR Flow",
    "description" : "Post IPR login flow that forces 2FA.",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "webauthn-authenticator",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "auth-otp-form",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 1,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 2,
      "autheticatorFlow" : true,
      "flowAlias" : "IPR OTP Default Subflow",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "e2822599-7110-4ba5-a2a5-7fa4c7e11f7c",
    "alias" : "{{ .domain }} Reset Credentials",
    "description" : "Reset credentials flow that forces 2FA verification before password reset.",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : false,
    "authenticationExecutions" : [ {
      "authenticator" : "reset-credentials-choose-user",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 0,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "CONDITIONAL",
      "priority" : 1,
      "autheticatorFlow" : true,
      "flowAlias" : "Conditional Reset Credentials 2FA Subflow",
      "userSetupAllowed" : false
    }, {
      "authenticator" : "reset-credential-email",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 2,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "reset-password",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 3,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "11f8664a-5663-484f-8e26-9655cd54280d",
    "alias" : "browser",
    "description" : "browser based authentication",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-cookie",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "auth-spnego",
      "authenticatorFlow" : false,
      "requirement" : "DISABLED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "identity-provider-redirector",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 25,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "ALTERNATIVE",
      "priority" : 30,
      "autheticatorFlow" : true,
      "flowAlias" : "forms",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "b4d963e9-ecac-44cb-b9e3-f47bc2236dbf",
    "alias" : "clients",
    "description" : "Base authentication for clients",
    "providerId" : "client-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "client-secret",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "client-jwt",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "client-secret-jwt",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 30,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "client-x509",
      "authenticatorFlow" : false,
      "requirement" : "ALTERNATIVE",
      "priority" : 40,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "e0112b3a-0378-43d0-b1df-ff0410399011",
    "alias" : "direct grant",
    "description" : "OpenID Connect Resource Owner Grant",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "direct-grant-validate-username",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "direct-grant-validate-password",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "CONDITIONAL",
      "priority" : 30,
      "autheticatorFlow" : true,
      "flowAlias" : "Direct Grant - Conditional OTP",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "22c8a6da-a990-4f5f-a855-42f0c6369449",
    "alias" : "docker auth",
    "description" : "Used by Docker clients to authenticate against the IDP",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "docker-http-basic-authenticator",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "78d84dd6-09a1-4498-ad32-c30ac64cfeba",
    "alias" : "first broker login",
    "description" : "Actions taken after first broker login with identity provider account, which is not yet linked to any Keycloak account",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticatorConfig" : "review profile config",
      "authenticator" : "idp-review-profile",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : true,
      "flowAlias" : "User creation or linking",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "f647c182-2368-4cdc-969e-8f6c4251b891",
    "alias" : "forms",
    "description" : "Username, password, otp and other auth forms.",
    "providerId" : "basic-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "auth-username-password-form",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "CONDITIONAL",
      "priority" : 20,
      "autheticatorFlow" : true,
      "flowAlias" : "Browser - Conditional OTP",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "5fb34ba4-53a4-46da-a7f9-4a3eb88765e5",
    "alias" : "registration",
    "description" : "registration flow",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "registration-page-form",
      "authenticatorFlow" : true,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : true,
      "flowAlias" : "registration form",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "47d8ca43-5eae-458e-a7d5-cefcf0b19bb1",
    "alias" : "registration form",
    "description" : "registration form",
    "providerId" : "form-flow",
    "topLevel" : false,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "registration-user-creation",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "registration-password-action",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 50,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "registration-recaptcha-action",
      "authenticatorFlow" : false,
      "requirement" : "DISABLED",
      "priority" : 60,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "479210af-fb85-492c-9c0f-bcb3055b57f2",
    "alias" : "reset credentials",
    "description" : "Reset credentials for a user if they forgot their password or something",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "reset-credentials-choose-user",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "reset-credential-email",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 20,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticator" : "reset-password",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 30,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    }, {
      "authenticatorFlow" : true,
      "requirement" : "CONDITIONAL",
      "priority" : 40,
      "autheticatorFlow" : true,
      "flowAlias" : "Reset - Conditional OTP",
      "userSetupAllowed" : false
    } ]
  }, {
    "id" : "d994ca24-38b9-464f-8788-4df42533fcdd",
    "alias" : "saml ecp",
    "description" : "SAML ECP Profile Authentication Flow",
    "providerId" : "basic-flow",
    "topLevel" : true,
    "builtIn" : true,
    "authenticationExecutions" : [ {
      "authenticator" : "http-basic-authenticator",
      "authenticatorFlow" : false,
      "requirement" : "REQUIRED",
      "priority" : 10,
      "autheticatorFlow" : false,
      "userSetupAllowed" : false
    } ]
  } ],
  "authenticatorConfig" : [ {
    "id" : "dab50222-913e-458c-9580-7f56b616fe52",
    "alias" : "create unique user config",
    "config" : {
      "require.password.update.after.registration" : "false"
    }
  }, {
    "id" : "1e5fa012-b4c0-45b1-95eb-4fec75b1fdda",
    "alias" : "review profile config",
    "config" : {
      "update.profile.on.first.login" : "missing"
    }
  } ],
  "requiredActions" : [ {
    "alias" : "CONFIGURE_TOTP",
    "name" : "Configure OTP",
    "providerId" : "CONFIGURE_TOTP",
    "enabled" : true,
    "defaultAction" : false,
    "priority" : 0,
    "config" : { }
  }, {
    "alias" : "TERMS_AND_CONDITIONS",
    "name" : "Terms and Conditions",
    "providerId" : "TERMS_AND_CONDITIONS",
    "enabled" : false,
    "defaultAction" : false,
    "priority" : 20,
    "config" : { }
  }, {
    "alias" : "UPDATE_PASSWORD",
    "name" : "Update Password",
    "providerId" : "UPDATE_PASSWORD",
    "enabled" : true,
    "defaultAction" : false,
    "priority" : 20,
    "config" : { }
  }, {
    "alias" : "UPDATE_PROFILE",
    "name" : "Update Profile",
    "providerId" : "UPDATE_PROFILE",
    "enabled" : true,
    "defaultAction" : false,
    "priority" : 30,
    "config" : { }
  }, {
    "alias" : "VERIFY_EMAIL",
    "name" : "Verify Email",
    "providerId" : "VERIFY_EMAIL",
    "enabled" : false,
    "defaultAction" : false,
    "priority" : 40,
    "config" : { }
  }, {
    "alias" : "update_user_locale",
    "name" : "Update User Locale",
    "providerId" : "update_user_locale",
    "enabled" : true,
    "defaultAction" : false,
    "priority" : 50,
    "config" : { }
  }, {
    "alias" : "delete_account",
    "name" : "Delete Account",
    "providerId" : "delete_account",
    "enabled" : false,
    "defaultAction" : false,
    "priority" : 60,
    "config" : { }
  }, {
    "alias" : "webauthn-register",
    "name" : "Webauthn Register",
    "providerId" : "webauthn-register",
    "enabled" : true,
    "defaultAction" : false,
    "priority" : 60,
    "config" : { }
  }, {
    "alias" : "delete_credential",
    "name" : "Delete Credential",
    "providerId" : "delete_credential",
    "enabled" : true,
    "defaultAction" : false,
    "priority" : 100,
    "config" : { }
  } ],
  "browserFlow" : "{{ .domain }} Browser",
  "registrationFlow" : "registration",
  "directGrantFlow" : "direct grant",
  "resetCredentialsFlow" : "{{ .domain }} Reset Credentials",
  "clientAuthenticationFlow" : "clients",
  "dockerAuthenticationFlow" : "docker auth",
  "firstBrokerLoginFlow" : "first broker login",
  "attributes" : {
    "cibaBackchannelTokenDeliveryMode" : "poll",
    "cibaExpiresIn" : "120",
    "cibaAuthRequestedUserHint" : "login_hint",
    "oauth2DeviceCodeLifespan" : "600",
    "clientOfflineSessionMaxLifespan" : "0",
    "oauth2DevicePollingInterval" : "5",
    "clientSessionIdleTimeout" : "0",
    "parRequestUriLifespan" : "60",
    "clientSessionMaxLifespan" : "0",
    "clientOfflineSessionIdleTimeout" : "0",
    "cibaInterval" : "5",
    "realmReusableOtpCode" : "false"
  },
  "keycloakVersion" : "25.0.6",
  "userManagedAccessAllowed" : false,
  "organizationsEnabled" : false,
  "clientProfiles" : {
    "profiles" : [ ]
  },
  "clientPolicies" : {
    "policies" : [ ]
  }
}
{{- end }}
{{- end }}
{{- end }}