// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Body of a *POST* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AuthLoginPostRequestBody {
    /// The id token to use for the login
    #[cfg_attr(
        feature = "utoipa",
        schema(example = "bG9yZW0gaXBzdW0sIHF1aWEgZG9sb3Igc2")
    )]
    pub id_token: String,
}
