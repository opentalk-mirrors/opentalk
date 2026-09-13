// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub(crate) enum ConnectionUpgrade {
    Upgrade,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[schema(rename_all = "snake_case")]
pub(crate) enum WebsocketUpgrade {
    Websocket,
}

/// This is a dummy type to define the structure of the headers required for
/// upgrading a request to a signaling websocket connection.
#[derive(utoipa::IntoParams)]
#[into_params(
    parameter_in = Header,
)]
#[expect(dead_code)]
pub(crate) struct SignalingProtocolHeaders {
    #[param(
        rename = "Sec-WebSocket-Protocol",
        pattern = "^opentalk-signaling-json-v1.0, ticket#.*$",
        required = true,
        example = "opentalk-signaling-json-v1.0, ticket#eyJpc3MiOiJodHRwczovL2V4YW1wbGUuYXV0aDAuY29tLy"
    )]
    pub protocol: String,

    #[param(inline, required = true)]
    pub connection: ConnectionUpgrade,

    #[param(inline, required = true)]
    pub upgrade: WebsocketUpgrade,
}
