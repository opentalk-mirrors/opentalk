// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common_macros::module_id;

fn main() {
    let _module_id = module_id!("hello+world$123");
}
