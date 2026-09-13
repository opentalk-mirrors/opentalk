// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::{ResultExt, Whatever};

fn main() -> Result<(), Whatever> {
    opentalk_version::collect_build_information()
        .with_whatever_context(|e| format!("Failed to collect build information: {e}"))
}
