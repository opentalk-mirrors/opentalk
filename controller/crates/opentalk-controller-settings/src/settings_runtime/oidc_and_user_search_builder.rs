// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::ClientId;
use snafu::ensure;
use url::Url;

use super::{Oidc, OidcController, OidcFrontend};
use crate::{
    Result, SettingsError,
    settings_error::{
        OidcConfigurationMissingSnafu, OidcInvalidConfigurationSnafu,
        UsersFindBehaviorBackendMissingSnafu,
    },
    settings_file::{self, UsersFindBehavior},
    settings_runtime::{UserSearchBackend, UserSearchBackendKeycloak},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OidcAndUserSearchBuilder {
    pub oidc: Oidc,
    pub user_search_backend: Option<UserSearchBackend>,
    pub users_find_behavior: UsersFindBehavior,
}

impl OidcAndUserSearchBuilder {
    pub(super) fn load_from_settings_raw(
        keycloak: Option<settings_file::Keycloak>,
        oidc: Option<settings_file::Oidc>,
        user_search: Option<settings_file::UserSearch>,
        endpoints: Option<&settings_file::Endpoints>,
    ) -> Result<Self> {
        let disable_users_find = endpoints.and_then(|e| e.disable_users_find);
        let users_find_use_kc = endpoints.and_then(|e| e.users_find_use_kc);

        if let Some(oidc) = oidc {
            ensure!(
                keycloak.is_none(),
                OidcInvalidConfigurationSnafu {
                    conflicting_field: "keycloak"
                }
            );
            ensure!(
                disable_users_find.is_none(),
                OidcInvalidConfigurationSnafu {
                    conflicting_field: "endpoints.disable_users_find"
                }
            );
            ensure!(
                users_find_use_kc.is_none(),
                OidcInvalidConfigurationSnafu {
                    conflicting_field: "endpoints.users_find_use_kc"
                }
            );
            return Self::load_from_settings_raw_oidc(oidc, user_search);
        }

        let Some(keycloak) = keycloak else {
            return OidcConfigurationMissingSnafu.fail();
        };

        Self::load_from_settings_raw_with_keycloak(&keycloak, disable_users_find, users_find_use_kc)
    }

    fn load_from_settings_raw_oidc(
        settings_file::Oidc {
            authority,
            frontend,
            controller,
        }: settings_file::Oidc,
        user_search: Option<settings_file::UserSearch>,
    ) -> Result<Self> {
        let frontend = OidcFrontend {
            authority: frontend.authority.unwrap_or(authority.clone()),
            client_id: frontend.client_id,
        };
        let controller = OidcController {
            authority: controller.authority.unwrap_or(authority.clone()),
            client_id: controller.client_id,
            client_secret: controller.client_secret,
        };

        let (user_search_backend, users_find_behavior) = if let Some(settings_file::UserSearch {
            backend,
            users_find_behavior,
        }) = user_search
        {
            let backend = backend.map(
                |settings_file::UserSearchBackend::KeycloakWebapi(
                    settings_file::UserSearchBackendKeycloakWebapi {
                        api_base_url,
                        client_id,
                        client_secret,
                        external_id_user_attribute_name,
                    },
                )| {
                    UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                        api_base_url,
                        client_id: client_id.unwrap_or_else(|| controller.client_id.clone()),
                        client_secret: client_secret
                            .unwrap_or_else(|| controller.client_secret.clone()),
                        external_id_user_attribute_name,
                    })
                },
            );
            let users_find_behavior = if backend.is_some() {
                users_find_behavior.unwrap_or(UsersFindBehavior::FromUserSearchBackend)
            } else {
                let users_find_behavior =
                    users_find_behavior.unwrap_or(UsersFindBehavior::Disabled);
                ensure!(
                    users_find_behavior != UsersFindBehavior::FromUserSearchBackend,
                    UsersFindBehaviorBackendMissingSnafu
                );
                users_find_behavior
            };
            (backend, users_find_behavior)
        } else {
            (None, UsersFindBehavior::Disabled)
        };

        Ok(Self {
            oidc: Oidc {
                controller,
                frontend,
            },
            user_search_backend,
            users_find_behavior,
        })
    }

    fn load_from_settings_raw_with_keycloak(
        keycloak: &settings_file::Keycloak,
        disable_users_find: Option<bool>,
        users_find_use_kc: Option<bool>,
    ) -> Result<Self> {
        tracing::warn!(
            "Found a deprecated [keycloak] configuration. Please use the new [oidc] and [user_search] configuration instead."
        );

        let oidc = {
            let auth_base_url =
                Self::build_url(keycloak.base_url.clone(), ["realms", &keycloak.realm])?;
            let controller = OidcController {
                authority: auth_base_url.clone(),
                client_id: keycloak.client_id.clone(),
                client_secret: keycloak.client_secret.clone(),
            };
            let frontend = OidcFrontend {
                authority: auth_base_url.clone(),
                client_id: ClientId::new("default".to_string()),
            };
            Oidc {
                controller,
                frontend,
            }
        };

        let users_find_behavior = match (
            disable_users_find.unwrap_or_default(),
            users_find_use_kc.unwrap_or_default(),
        ) {
            (true, _) => UsersFindBehavior::Disabled,
            (false, false) => UsersFindBehavior::FromDatabase,
            (false, true) => UsersFindBehavior::FromUserSearchBackend,
        };
        let user_search_backend = {
            let api_base_url = Self::build_url(
                keycloak.base_url.clone(),
                ["admin", "realms", &keycloak.realm],
            )?;

            let backend = UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                api_base_url,
                client_id: keycloak.client_id.clone(),
                client_secret: keycloak.client_secret.clone(),
                external_id_user_attribute_name: keycloak.external_id_user_attribute_name.clone(),
            });

            Some(backend)
        };
        Ok(Self {
            oidc,
            user_search_backend,
            users_find_behavior,
        })
    }

    fn build_url<I>(base_url: Url, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let err_url = base_url.clone();
        let mut url = base_url;
        _ = url
            .path_segments_mut()
            .map_err(|_| SettingsError::NotBaseUrl { url: err_url })?
            .extend(path_segments);
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use openidconnect::{ClientId, ClientSecret};
    use pretty_assertions::{assert_eq, assert_matches};

    use super::OidcAndUserSearchBuilder;
    use crate::{
        Oidc, SettingsError, UserSearchBackend, UserSearchBackendKeycloak,
        settings_file::{UsersFindBehavior, settings_raw_minimal_example},
    };

    #[test]
    fn minimal_example() {
        let raw = settings_raw_minimal_example();
        let builder = OidcAndUserSearchBuilder::load_from_settings_raw(
            raw.keycloak,
            raw.oidc,
            raw.user_search,
            raw.endpoints.as_ref(),
        )
        .expect("must be a valid configuration");

        assert_eq!(
            builder,
            OidcAndUserSearchBuilder {
                oidc: Oidc {
                    controller: crate::OidcController {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("Controller".to_string()),
                        client_secret: ClientSecret::new("mysecret".to_string())
                    },
                    frontend: crate::OidcFrontend {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("Webapp".to_string()),
                    }
                },
                user_search_backend: None,
                users_find_behavior: UsersFindBehavior::Disabled,
            }
        );
    }

    #[test]
    fn minimal_example_without_user_search() {
        let mut raw = settings_raw_minimal_example();
        raw.user_search = None;
        let builder = OidcAndUserSearchBuilder::load_from_settings_raw(
            raw.keycloak,
            raw.oidc,
            raw.user_search,
            raw.endpoints.as_ref(),
        )
        .expect("must be a valid configuration");

        assert_eq!(
            builder,
            OidcAndUserSearchBuilder {
                oidc: Oidc {
                    controller: crate::OidcController {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("Controller".to_string()),
                        client_secret: ClientSecret::new("mysecret".to_string())
                    },
                    frontend: crate::OidcFrontend {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("Webapp".to_string()),
                    }
                },
                user_search_backend: None,
                users_find_behavior: UsersFindBehavior::Disabled,
            }
        );
    }

    #[test]
    fn keycloak_only() {
        let mut raw = settings_raw_minimal_example();
        raw.oidc = None;
        raw.user_search = None;
        raw.keycloak = Some(crate::settings_file::Keycloak {
            base_url: "http://localhost:8080/"
                .parse()
                .expect("valid url expected"),
            realm: "opentalk".to_string(),
            client_id: ClientId::new("Controller".to_string()),
            client_secret: ClientSecret::new("MySecret".to_string()),
            external_id_user_attribute_name: Some("TheExternalAttribute".to_string()),
        });

        let builder = OidcAndUserSearchBuilder::load_from_settings_raw(
            raw.keycloak,
            raw.oidc,
            raw.user_search,
            raw.endpoints.as_ref(),
        )
        .expect("must be a valid configuration");

        assert_eq!(
            builder,
            OidcAndUserSearchBuilder {
                oidc: Oidc {
                    controller: crate::OidcController {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("Controller".to_string()),
                        client_secret: ClientSecret::new("MySecret".to_string())
                    },
                    frontend: crate::OidcFrontend {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("default".to_string()),
                    }
                },
                user_search_backend: Some(UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                    api_base_url: "http://localhost:8080/admin/realms/opentalk"
                        .parse()
                        .expect("must be a valid url"),
                    client_id: ClientId::new("Controller".to_string()),
                    client_secret: ClientSecret::new("MySecret".to_string()),
                    external_id_user_attribute_name: Some("TheExternalAttribute".to_string())
                })),
                users_find_behavior: UsersFindBehavior::FromDatabase,
            }
        );
    }

    #[test]
    fn keycloak_with_search() {
        let mut raw = settings_raw_minimal_example();
        raw.oidc = None;
        raw.user_search = None;
        raw.keycloak = Some(crate::settings_file::Keycloak {
            base_url: "http://localhost:8080/"
                .parse()
                .expect("valid url expected"),
            realm: "opentalk".to_string(),
            client_id: ClientId::new("Controller".to_string()),
            client_secret: ClientSecret::new("MySecret".to_string()),
            external_id_user_attribute_name: Some("TheExternalAttribute".to_string()),
        });
        raw.endpoints = Some(crate::settings_file::Endpoints {
            users_find_use_kc: Some(true),
            ..Default::default()
        });

        let builder = OidcAndUserSearchBuilder::load_from_settings_raw(
            raw.keycloak,
            raw.oidc,
            raw.user_search,
            raw.endpoints.as_ref(),
        )
        .expect("must be a valid configuration");

        assert_eq!(
            builder,
            OidcAndUserSearchBuilder {
                oidc: Oidc {
                    controller: crate::OidcController {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("Controller".to_string()),
                        client_secret: ClientSecret::new("MySecret".to_string())
                    },
                    frontend: crate::OidcFrontend {
                        authority: "http://localhost:8080/realms/opentalk"
                            .parse()
                            .expect("valid url expected"),
                        client_id: ClientId::new("default".to_string()),
                    }
                },
                user_search_backend: Some(UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                    api_base_url: "http://localhost:8080/admin/realms/opentalk"
                        .parse()
                        .expect("must be a valid url"),
                    client_id: ClientId::new("Controller".to_string()),
                    client_secret: ClientSecret::new("MySecret".to_string()),
                    external_id_user_attribute_name: Some("TheExternalAttribute".to_string())
                })),
                users_find_behavior: UsersFindBehavior::FromUserSearchBackend,
            }
        );
    }

    #[test]
    fn invalid_keycloak_and_oidc() {
        let mut raw = settings_raw_minimal_example();
        raw.keycloak = Some(crate::settings_file::Keycloak {
            base_url: "http://localhost:8080/"
                .parse()
                .expect("valid url expected"),
            realm: "opentalk".to_string(),
            client_id: ClientId::new("Controller".to_string()),
            client_secret: ClientSecret::new("MySecret".to_string()),
            external_id_user_attribute_name: Some("TheExternalAttribute".to_string()),
        });
        raw.endpoints = Some(crate::settings_file::Endpoints {
            users_find_use_kc: Some(true),
            ..Default::default()
        });

        assert_matches!(
            OidcAndUserSearchBuilder::load_from_settings_raw(
                raw.keycloak,
                raw.oidc,
                raw.user_search,
                raw.endpoints.as_ref()
            ),
            Err(SettingsError::OidcInvalidConfiguration {
                conflicting_field: "keycloak"
            })
        );
    }

    #[test]
    fn invalid_neither_keycloak_nor_oidc() {
        let mut raw = settings_raw_minimal_example();
        raw.user_search = None;
        raw.oidc = None;

        assert_matches!(
            OidcAndUserSearchBuilder::load_from_settings_raw(
                raw.keycloak,
                raw.oidc,
                raw.user_search,
                raw.endpoints.as_ref()
            ),
            Err(SettingsError::OidcConfigurationMissing)
        );
    }

    #[test]
    fn invalid_oidc_with_endpoints_disable_users_find() {
        let mut raw = settings_raw_minimal_example();
        raw.endpoints = Some(crate::settings_file::Endpoints {
            disable_users_find: Some(true),
            ..Default::default()
        });

        assert_matches!(
            OidcAndUserSearchBuilder::load_from_settings_raw(
                raw.keycloak,
                raw.oidc,
                raw.user_search,
                raw.endpoints.as_ref()
            ),
            Err(SettingsError::OidcInvalidConfiguration {
                conflicting_field: "endpoints.disable_users_find"
            })
        );
    }

    #[test]
    fn invalid_oidc_with_endpoints_users_find_use_kc() {
        let mut raw = settings_raw_minimal_example();
        raw.endpoints = Some(crate::settings_file::Endpoints {
            users_find_use_kc: Some(true),
            ..Default::default()
        });

        assert_matches!(
            OidcAndUserSearchBuilder::load_from_settings_raw(
                raw.keycloak,
                raw.oidc,
                raw.user_search,
                raw.endpoints.as_ref()
            ),
            Err(SettingsError::OidcInvalidConfiguration {
                conflicting_field: "endpoints.users_find_use_kc"
            })
        );
    }
}
