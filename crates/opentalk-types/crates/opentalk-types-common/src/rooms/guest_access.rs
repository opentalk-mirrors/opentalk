// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{sql_enum, utils::ExampleData};

sql_enum!(
    feature_gated:

    #[derive( Eq, PartialEq)]
    #[cfg_attr(
        feature = "serde",
        derive(serde::Serialize, serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(GuestAccess::example_data()))
    )]
    GuestAccess,
    "guest_access",
    GuestAccessType,
    {
        Disabled = b"disabled",
        WaitingRoom = b"waiting_room",
        DirectAccess = b"direct_access",
    }
);

impl GuestAccess {
    /// Check whether the value is [`GuestAccess::Disabled`].
    pub const fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    /// Check whether the value is [`GuestAccess::WaitingRoom`].
    pub const fn is_waiting_room(&self) -> bool {
        matches!(self, Self::WaitingRoom)
    }

    /// Check whether the value is [`GuestAccess::DirectAccess`].
    pub const fn is_direct_access(&self) -> bool {
        matches!(self, Self::DirectAccess)
    }
}

#[allow(clippy::derivable_impls)]
impl Default for GuestAccess {
    fn default() -> Self {
        Self::WaitingRoom
    }
}

impl ExampleData for GuestAccess {
    fn example_data() -> Self {
        Self::WaitingRoom
    }
}

#[cfg(test)]
mod tests {
    use super::GuestAccess;

    #[test]
    fn is_disabled() {
        assert!(GuestAccess::Disabled.is_disabled());
        assert!(!GuestAccess::WaitingRoom.is_disabled());
        assert!(!GuestAccess::DirectAccess.is_disabled());
    }

    #[test]
    fn is_waiting_room() {
        assert!(!GuestAccess::Disabled.is_waiting_room());
        assert!(GuestAccess::WaitingRoom.is_waiting_room());
        assert!(!GuestAccess::DirectAccess.is_waiting_room());
    }

    #[test]
    fn is_direct_access() {
        assert!(!GuestAccess::Disabled.is_direct_access());
        assert!(!GuestAccess::WaitingRoom.is_direct_access());
        assert!(GuestAccess::DirectAccess.is_direct_access());
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::GuestAccess;

    #[test]
    fn serialize() {
        assert_eq!(
            serde_json::to_value(GuestAccess::Disabled).unwrap(),
            json!("disabled")
        );
        assert_eq!(
            serde_json::to_value(GuestAccess::WaitingRoom).unwrap(),
            json!("waiting_room")
        );
        assert_eq!(
            serde_json::to_value(GuestAccess::DirectAccess).unwrap(),
            json!("direct_access")
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::from_value::<GuestAccess>(json!("disabled")).unwrap(),
            GuestAccess::Disabled
        );
        assert_eq!(
            serde_json::from_value::<GuestAccess>(json!("waiting_room")).unwrap(),
            GuestAccess::WaitingRoom
        );
        assert_eq!(
            serde_json::from_value::<GuestAccess>(json!("direct_access")).unwrap(),
            GuestAccess::DirectAccess
        );
    }
}
