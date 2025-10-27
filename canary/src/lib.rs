// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#[test]
fn test() {
    use build_info::BuildInfo;
    use opentalk_version::InfoArgs;

    opentalk_version::build_info!();

    let args = InfoArgs {
        version: true,
        license: false,
    };

    let build_info = BuildInfo::new();
    let text = build_info.format(&args).unwrap();
    println!("{text}");
}
