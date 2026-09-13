// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    post,
    web::{Json, Path},
};
use chrono::{DateTime, Utc};
use opentalk_types_common::{events::EventId, time::RecurrencePattern};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EventRescheduleBody {
    _from: DateTime<Utc>,
    _is_all_day: Option<bool>,
    _starts_at: Option<bool>,
    _ends_at: Option<bool>,
    _recurrence_pattern: RecurrencePattern,
}

#[post("/events/{event_id}/reschedule")]
pub async fn event_reschedule(
    _event_id: Path<EventId>,
    _body: Json<EventRescheduleBody>,
) -> actix_web::HttpResponse {
    actix_web::HttpResponse::NotImplemented().finish()
}
