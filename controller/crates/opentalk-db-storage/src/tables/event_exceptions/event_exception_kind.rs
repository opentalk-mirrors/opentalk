// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::sql_enum;

sql_enum!(
    EventExceptionKind,
    "event_exception_kind",
    EventExceptionKindType,
    {
        Modified = b"modified",
        Cancelled = b"cancelled",
    }
);

impl From<EventExceptionKind> for inventory::EventExceptionKind {
    fn from(value: EventExceptionKind) -> Self {
        match value {
            EventExceptionKind::Modified => Self::Modified,
            EventExceptionKind::Cancelled => Self::Cancelled,
        }
    }
}

impl From<inventory::EventExceptionKind> for EventExceptionKind {
    fn from(value: inventory::EventExceptionKind) -> Self {
        match value {
            inventory::EventExceptionKind::Modified => Self::Modified,
            inventory::EventExceptionKind::Cancelled => Self::Cancelled,
        }
    }
}
