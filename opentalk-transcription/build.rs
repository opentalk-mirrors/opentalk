// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    opentalk_version::collect_build_information().context("Failed to collect build information")
}
