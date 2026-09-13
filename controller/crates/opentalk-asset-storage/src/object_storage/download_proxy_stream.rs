// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use bytes::Bytes;
use futures::Stream;

/// A struct containing a stream of a proxied object storage download.
pub struct DownloadProxyStream {
    /// The HTTP status to return for the response.
    pub status: u16,

    /// The HTTP headers to be added to the response.
    pub headers: BTreeMap<String, Bytes>,

    /// The stream containing the data which should be sent in the response.
    pub stream:
        Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Unpin>,
}

impl std::fmt::Debug for DownloadProxyStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadProxyStream")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("stream", &())
            .finish()
    }
}
