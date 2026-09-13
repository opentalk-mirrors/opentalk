// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use fluent_templates::{fluent_bundle::FluentValue, static_loader};

static_loader! {
    pub static LOCALES = {
        locales: "./resources/locales",
        fallback_language: "en-US",
        customise: |bundle| {
            // Outlook/Word just doesn't handle unicode control characters
            // So this turns off the insertion of of RTL/LTR isolation characters
            bundle.set_use_isolating(false);
            // And to compensate for the missing isolation:
            // filter out all direction-changing characters, so no user input can mess up the emails
            bundle.set_formatter(Some(formatter))
        },
    };
}

fn formatter<M>(v: &FluentValue, _: &M) -> Option<String> {
    if let FluentValue::String(v) = v {
        // Filter out following characters from strings

        // - U+2066 LEFT-TO-RIGHT ISOLATE
        // - U+2067 RIGHT-TO-LEFT ISOLATE
        // - U+2068 FIRST-STRONG ISOLATE
        // - U+2069 POP DIRECTIONAL ISOLATE

        // - U+202A LEFT-TO-RIGHT EMBEDDING
        // - U+202B RIGHT-TO-LEFT EMBEDDING
        // - U+202C POP DIRECTIONAL FORMATTING
        // - U+202D LEFT-TO-RIGHT OVERRIDE
        // - U+202E RIGHT-TO-LEFT OVERRIDE
        Some(v.replace(
            |c| matches!(c, '\u{2066}'..='\u{2069}' | '\u{202A}'..='\u{202E}'),
            "",
        ))
    } else {
        None
    }
}
