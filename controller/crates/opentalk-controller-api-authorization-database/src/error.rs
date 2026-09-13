// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::AuthorizationError;

/// Type alias for [`std::result::Result`] returning an [`AuthorizationError`].
pub type Result<T, E = AuthorizationError> = std::result::Result<T, E>;
