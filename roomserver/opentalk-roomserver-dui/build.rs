// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

fn main() -> anyhow::Result<()> {
    opentalk_version::collect_build_information()
}
