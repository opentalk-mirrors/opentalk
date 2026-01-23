// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use sip_core::{Endpoint, IncomingRequest, MayTake};
use sip_types::{Method, StatusCode, header::typed::Contact};
use sip_ua::dialog::Usage;

/// Simple Dialog Usage which handles OPTIONS requests inside a dialog
pub(super) struct OptionsUsage {
    pub(super) contact: Contact,
}

#[async_trait]
impl Usage for OptionsUsage {
    fn name(&self) -> &'static str {
        "options"
    }

    async fn receive(&self, endpoint: &Endpoint, request: MayTake<'_, IncomingRequest>) {
        if request.line.method != Method::OPTIONS {
            return;
        }

        handle_options_request(endpoint, request.take(), &self.contact).await;
    }
}

pub(super) async fn handle_options_request(
    endpoint: &Endpoint,
    mut request: IncomingRequest,
    contact: &Contact,
) {
    let mut response = endpoint.create_response(&request, StatusCode::OK, None);
    response.msg.headers.insert_named(contact);

    let tsx = endpoint.create_server_tsx(&mut request);

    if let Err(e) = tsx.respond(response).await {
        log::warn!("Failed to respond to OPTIONS request: {e}");
    }
}
