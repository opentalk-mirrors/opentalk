// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[test]
fn example() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/module_id/parse_invalid_empty_str.rs");
    t.compile_fail("tests/module_id/parse_invalid_characters_str.rs");
    t.pass("tests/module_id/parse_valid.rs");
}
