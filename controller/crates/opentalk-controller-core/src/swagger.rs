// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    App, Error,
    dev::{ServiceFactory, ServiceRequest},
};

pub(super) trait WithSwagger {
    fn with_swagger_service_if(self, enabled: bool) -> Self;
}

impl<T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>> WithSwagger
    for App<T>
{
    fn with_swagger_service_if(self, enabled: bool) -> Self {
        if enabled {
            #[cfg(feature = "swagger")]
            {
                use utoipa::OpenApi as _;

                let mut openapi = crate::ApiDoc::openapi();
                openapi.servers = Some(vec![utoipa::openapi::Server::new("/v1")]);

                return self.service(
                    utoipa_swagger_ui::SwaggerUi::new("/swagger/{_:.*}")
                        .url("/v1/openapi.json", openapi),
                );
            }
        }
        self
    }
}
