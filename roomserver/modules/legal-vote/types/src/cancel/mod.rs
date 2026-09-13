// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling cancel types for the `legal-vote` namespace.

mod cancel_reason;
mod custom_cancel_reason;

pub use cancel_reason::CancelReason;
pub use custom_cancel_reason::{CustomCancelReason, MAX_CUSTOM_CANCEL_REASON_LENGTH};
