// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Once;

static CRYPTO_PROVIDER: Once = Once::new();

/// Installs `aws-lc-rs` as the default crypto provider for `rustls` and `jsonwebtoken`.
///
/// `rustls` and `jsonwebtoken` depend on a `CryptoProvider` being configured.
/// If no provider was explicitly configured, a provider will be derived from
/// the enabled features. Since there are many crates that depend on rustls and
/// `jsonwebtoken`, we don't have complete control over the enabled features.
/// If the configuration via feature is ambiguous these crates will panic.
///
/// Call this function early in your program (or test setup) before any TLS or
/// JWT operations take place.
pub fn ensure_crypto_provider() {
    CRYPTO_PROVIDER.call_once(|| {
        rustls::crypto::CryptoProvider::install_default(
            rustls::crypto::aws_lc_rs::default_provider(),
        )
        .expect("valid default crypto provider expected");

        jsonwebtoken::crypto::CryptoProvider::install_default(
            &jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER,
        )
        .expect("valid default crypto provider expected");
    });
}
