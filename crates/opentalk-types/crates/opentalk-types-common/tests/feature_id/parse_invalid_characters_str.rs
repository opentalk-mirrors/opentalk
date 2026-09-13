// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common_macros::feature_id;

fn main() {
    let _feature_id = feature_id!("hello+world$123");
}
