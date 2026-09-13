// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for call-in functionality.

/// The length of a numeric dial-in identifier
pub const DIAL_IN_NUMERIC_ID_LENGTH: usize = 10;

mod call_in_id;
mod call_in_info;
mod call_in_password;
mod numeric_id;

pub use call_in_id::CallInId;
pub use call_in_info::CallInInfo;
pub use call_in_password::CallInPassword;
pub use numeric_id::NumericId;
