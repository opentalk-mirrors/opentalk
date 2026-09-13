// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common_macros::asset_file_kind;

fn main() {
    let _asset_file_kind = asset_file_kind!("hello_world_123");
}
