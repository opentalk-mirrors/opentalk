// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> anyhow::Result<()> {
    opentalk_version::collect_build_information()
}
