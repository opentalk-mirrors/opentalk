// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use sip_core::{Endpoint, IncomingRequest, MayTake};
use sip_types::{Method, StatusCode, header::typed::ContentType};
use sip_ua::dialog::Usage;
use std::str::from_utf8;
use tokio::sync::broadcast;

/// Simple Dialog Usage which reacts to `picture_fast_update` messages in the `media_control` protocol
pub(super) struct MediaControlUsage {
    pub(super) notify_picture_fast_update: broadcast::Sender<()>,
}

#[async_trait]
impl Usage for MediaControlUsage {
    fn name(&self) -> &'static str {
        "Media Control"
    }

    async fn receive(&self, endpoint: &Endpoint, request: MayTake<'_, IncomingRequest>) {
        if request.line.method != Method::INFO {
            return;
        }

        self.receive_info_request(endpoint, request).await;
    }
}

impl MediaControlUsage {
    async fn receive_info_request(
        &self,
        endpoint: &Endpoint,
        request: MayTake<'_, IncomingRequest>,
    ) {
        let is_media_control = request
            .headers
            .get_named::<ContentType>()
            .is_ok_and(|ct| ct.0.starts_with("application/media_control"));

        if !is_media_control {
            return;
        }

        let is_picture_fast_update =
            from_utf8(request.body.as_ref()).is_ok_and(|str| str.contains("picture_fast_update"));

        let code = if !is_picture_fast_update {
            StatusCode::NOT_FOUND
        } else if self.notify_picture_fast_update.send(()).is_ok() {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        };

        let mut request = request.take();

        let tsx = endpoint.create_server_tsx(&mut request);

        let response = endpoint.create_response(&request, code, None);
        if let Err(e) = tsx.respond(response).await {
            log::warn!("Failed to respond to INFO request, {e:?}");
        }
    }
}
