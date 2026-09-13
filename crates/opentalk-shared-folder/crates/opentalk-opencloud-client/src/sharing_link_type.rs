// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Type of a sharing link as defined by the LibreGraph `SharingLinkType`.
///
/// The `Upload` and `CreateOnly` variants are only valid for folders.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Display, EnumString,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SharingLinkType {
    /// An internal link, only usable by people with existing access to the resource.
    Internal,
    /// A read-only link.
    #[default]
    View,
    /// An upload-only link for a folder (recipients can add but not see existing files).
    Upload,
    /// A link granting read and write access.
    Edit,
    /// A link that only allows creating new items in a folder.
    CreateOnly,
    /// A read-only link that additionally blocks downloads.
    BlocksDownload,
}

#[cfg(test)]
mod tests {
    use super::SharingLinkType;

    #[test]
    fn serializes_to_camel_case() {
        assert_eq!(
            serde_json::to_string(&SharingLinkType::View).unwrap(),
            "\"view\""
        );
        assert_eq!(
            serde_json::to_string(&SharingLinkType::CreateOnly).unwrap(),
            "\"createOnly\""
        );
        assert_eq!(
            serde_json::to_string(&SharingLinkType::BlocksDownload).unwrap(),
            "\"blocksDownload\""
        );
    }

    #[test]
    fn deserializes_from_camel_case() {
        let parsed: SharingLinkType = serde_json::from_str("\"edit\"").unwrap();
        assert_eq!(parsed, SharingLinkType::Edit);
    }

    #[test]
    fn parses_from_str() {
        assert_eq!(
            "upload".parse::<SharingLinkType>().unwrap(),
            SharingLinkType::Upload
        );
    }
}
