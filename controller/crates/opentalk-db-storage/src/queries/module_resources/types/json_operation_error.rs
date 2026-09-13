// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use snafu::Snafu;

use super::JsonPatchError;

#[derive(Debug, Snafu)]
pub enum JsonOperationError {
    #[snafu(transparent)]
    JsonPatch { source: JsonPatchError },
    #[snafu(transparent)]
    Database { source: DatabaseError },
}

impl From<diesel::result::Error> for JsonOperationError {
    fn from(value: diesel::result::Error) -> Self {
        match JsonPatchError::try_from_diesel_error(&value) {
            Some(json_patch_error) => Self::JsonPatch {
                source: json_patch_error,
            },
            None => Self::Database {
                source: value.into(),
            },
        }
    }
}
