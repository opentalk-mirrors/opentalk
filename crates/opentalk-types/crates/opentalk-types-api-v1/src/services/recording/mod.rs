// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/services/recording`.

mod get_recording_upload_query;
mod post_recording_start_request_body;

pub use get_recording_upload_query::GetRecordingUploadQuery;
pub use post_recording_start_request_body::PostRecordingStartRequestBody;
