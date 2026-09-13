// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::Snafu;

/// Error performing a change in the authorization dataset.
#[derive(Debug, Snafu)]
pub enum AuthorizationChangeError {}
