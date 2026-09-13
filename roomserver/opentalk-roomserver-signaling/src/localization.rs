// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use fluent_langneg::NegotiationStrategy;
use icu_locid::LanguageIdentifier;

use crate::{module_context::ModuleContext, signaling_module::SignalingModule};

pub fn negotiate_languages<M: SignalingModule>(
    ctx: &ModuleContext<'_, M>,
    available_languages: &[LanguageIdentifier],
) -> Option<LanguageIdentifier> {
    let preferred_language = &ctx.room_task_info.room.preferred_language;
    let system_default = &ctx.room_task_info.room.fallback_language;

    fluent_langneg::negotiate_languages(
            &[preferred_language, system_default],
            available_languages,
            None,
            NegotiationStrategy::Lookup,
        )
        .into_iter()
        .next()
        .cloned()
        .or_else(|| {
            tracing::warn!(
                "Could not find a valid report language. System default: {system_default}, available: {available_languages:?}."
            );
            available_languages.first().cloned()
        })
}
