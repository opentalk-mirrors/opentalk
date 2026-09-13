// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Access types to a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Access {
    /// Read access to the resource.
    Read,

    /// Write access to the resource.
    Write,
}
