// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `whiteboard` namespace

use opentalk_roomserver_signaling::storage::assets::StorageError;
use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_types_common::assets::AssetId;
use serde::{Deserialize, Serialize};
use url::Url;

/// Events sent out by the `whiteboard` module.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum WhiteboardEvent {
    /// Initialization of spacedeck has started.
    InitializationStarted,

    /// Spacedeck has been initialized.
    Initialized {
        /// URL to the space.
        url: Url,
    },

    /// The whiteboard was exported as PDF and stored in the asset store.
    PdfCreated {
        /// The file name of the PDF asset.
        filename: String,

        /// The asset id for the PDF asset.
        asset_id: AssetId,
    },

    /// An error happened when executing a `whiteboard` command.
    Error(WhiteboardError),
}

impl From<WhiteboardError> for WhiteboardEvent {
    fn from(value: WhiteboardError) -> Self {
        Self::Error(value)
    }
}

/// Error from the `whiteboard` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum WhiteboardError {
    /// The requesting user has insufficient permissions for the operation.
    InsufficientPermissions,
    /// Spacedeck has not been initialized yet.
    NotInitialized,
    /// Spacedeck is already initializing.
    CurrentlyInitializing,
    /// The spacedeck initialization failed.
    InitializationFailed,
    /// Spacedeck is already initialized.
    AlreadyInitialized,
    /// The requesting user has exceeded their storage.
    StorageExceeded,
    /// An internal error occurred while saving the whiteboard pdf.
    InternalStorage,
}

impl ModuleError for WhiteboardError {}

impl From<StorageError> for WhiteboardError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::QuotaExceeded => Self::StorageExceeded,
            StorageError::Internal(..) | StorageError::ReadAsset(..) => Self::InternalStorage,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_common::assets::AssetId;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use url::Url;

    use super::WhiteboardEvent;
    use crate::event::WhiteboardError;

    #[test]
    fn serialize_initialization_started() {
        let event = WhiteboardEvent::InitializationStarted;
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "initialization_started"
        }
        "#);
    }

    #[test]
    fn deserialize_initialization_started() {
        let json = json!({
            "message": "initialization_started"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();

        assert_eq!(produced, WhiteboardEvent::InitializationStarted);
    }

    #[test]
    fn serialize_initialized() {
        let event = WhiteboardEvent::Initialized {
            url: Url::parse("https://example.com").unwrap(),
        };
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "initialized",
          "url": "https://example.com/"
        }
        "#);
    }

    #[test]
    fn deserialize_initialized() {
        let json = json!({
            "message": "initialized",
            "url": "https://example.com/"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();

        assert_eq!(
            produced,
            WhiteboardEvent::Initialized {
                url: Url::parse("https://example.com").unwrap(),
            }
        );
    }

    #[test]
    fn serialize_pdf_asset() {
        let event = WhiteboardEvent::PdfCreated {
            filename: "asset.pdf".into(),
            asset_id: AssetId::from_u128(0x1),
        };
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "pdf_created",
          "filename": "asset.pdf",
          "asset_id": "00000000-0000-0000-0000-000000000001"
        }
        "#);
    }

    #[test]
    fn deserialize_pdf_asset() {
        let json = json!({
            "message": "pdf_created",
            "filename": "asset.pdf",
            "asset_id": "00000000-0000-0000-0000-000000000001",
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();

        assert_eq!(
            produced,
            WhiteboardEvent::PdfCreated {
                filename: "asset.pdf".into(),
                asset_id: AssetId::from_u128(0x1),
            }
        );
    }

    #[test]
    fn serialize_insufficient_permissions_error() {
        let event = WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions);
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_insufficient_permissions_error() {
        let json = json!({
            "message": "error",
            "error": "insufficient_permissions"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions)
        );
    }

    #[test]
    fn serialize_currently_initializing_error() {
        let event = WhiteboardEvent::Error(WhiteboardError::CurrentlyInitializing);
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "currently_initializing"
        }
        "#);
    }

    #[test]
    fn deserialize_currently_initializing_error() {
        let json = json!({
            "message": "error",
            "error": "currently_initializing"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            WhiteboardEvent::Error(WhiteboardError::CurrentlyInitializing)
        );
    }

    #[test]
    fn serialize_initialization_failed_error() {
        let event = WhiteboardEvent::Error(WhiteboardError::InitializationFailed);
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "initialization_failed"
        }
        "#);
    }

    #[test]
    fn deserialize_initialization_failed_error() {
        let json = json!({
            "message": "error",
            "error": "initialization_failed"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            WhiteboardEvent::Error(WhiteboardError::InitializationFailed)
        );
    }

    #[test]
    fn serialize_already_initialized_error() {
        let event = WhiteboardEvent::Error(WhiteboardError::AlreadyInitialized);
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "already_initialized"
        }
        "#);
    }

    #[test]
    fn deserialize_already_initialized_error() {
        let json = json!({
            "message": "error",
            "error": "already_initialized"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            WhiteboardEvent::Error(WhiteboardError::AlreadyInitialized)
        );
    }

    #[test]
    fn serialize_storage_exceeded_error() {
        let event = WhiteboardEvent::Error(WhiteboardError::StorageExceeded);
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "storage_exceeded"
        }
        "#);
    }

    #[test]
    fn deserialize_storage_exceeded_error() {
        let json = json!({
            "message": "error",
            "error": "storage_exceeded"
        });
        let produced: WhiteboardEvent = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            WhiteboardEvent::Error(WhiteboardError::StorageExceeded)
        );
    }
}
