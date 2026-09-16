// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    // FIXME: Allow duplicate symbols for webrtc-sys shenanigans
    if std::env::var_os("CARGO_CFG_TARGET_OS").is_some_and(|os| os == "linux") {
        println!("cargo::rustc-link-arg=-Wl,--allow-multiple-definition");
    }

    opentalk_version::collect_build_information().context("Failed to collect build information")
}
