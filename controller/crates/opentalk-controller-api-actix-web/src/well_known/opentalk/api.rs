// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `.well-known/opentalk/api`

use actix_web::{get, web::Json};
use opentalk_types_common::api_well_known::{ApiDescription, OpenTalkApi, WellKnown};

#[get("/.well-known/opentalk/api")]
pub async fn get() -> Json<WellKnown> {
    Json(WellKnown {
        opentalk_api: OpenTalkApi {
            v1: Some(ApiDescription {
                base_url: "v1".to_string(),
            }),
        },
    })
}
