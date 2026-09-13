// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Display, net::SocketAddr};

use axum::serve::Listener;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

// This code is adapted from the [axum-listener](https://crates.io/crates/axum-listener) crate's MultiListener implementation.

// MIT License

// Copyright (c) 2025 Uttarayan Mondal

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// A listener that can accept connections on multiple underlying listeners simultaneously.
///
/// This struct allows you to bind to multiple TCP addresses and accept connections from any of
/// them. When multiple listeners are ready to accept connections, there is no guarantee which one
/// will be selected first.
///
/// # Implementation Details
///
/// Internally, this uses [`futures::future::select_all`] to wait on all listeners
/// simultaneously, ensuring efficient polling of all underlying listeners.
///
/// # Examples
///
/// ```rust,no_run
/// # tokio_test::block_on(async {
/// use axum::{Router, routing::get};
/// use axum_listener::multi::MultiListener;
///
/// let router = Router::new().route("/", get(|| async { "Hello, World!" }));
///
/// // Bind to multiple TCP addresses
/// let addresses = ["127.0.0.1:8080", "127.0.0.1:8081"];
/// let listener = MultiListener::bind(addresses).await.unwrap();
/// axum::serve(listener, router).await.unwrap();
/// # });
/// ```
pub struct MultiListener {
    /// The underlying listeners that this multi-listener manages
    pub listeners: Vec<TcpListener>,
}

/// An address collection representing the local addresses of a [`MultiListener`].
///
/// This struct contains all the addresses that the multi-listener is bound to.
/// It's returned by the [`axum::serve::Listener::local_addr`] method implementation
/// for [`MultiListener`].
///
/// # Examples
///
/// ```rust,no_run
/// # tokio_test::block_on(async {
/// use axum_listener::multi::MultiListener;
/// use axum::serve::Listener;
///
/// let addresses = ["127.0.0.1:8080", "127.0.0.1:8081"];
/// let listener = MultiListener::bind(addresses).await.unwrap();
/// let multi_addr = listener.local_addr().unwrap();
/// println!("Bound to {} addresses", multi_addr.addrs.len());
/// # });
/// ```
#[derive(Debug, Clone)]
pub struct MultiAddr {
    /// The collection of addresses that the multi-listener is bound to
    pub addrs: Vec<SocketAddr>,
}

impl Display for MultiAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.addrs
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
            .fmt(f)
    }
}

impl MultiListener {
    /// Creates a new [`MultiListener`] bound to multiple addresses.
    ///
    /// This method attempts to bind to all provided addresses simultaneously.
    /// If any of the bindings fail, the entire operation fails and returns an error.
    /// All addresses must be successfully bound for this method to succeed.
    ///
    /// # Arguments
    ///
    /// * `addresses` - An iterable collection of addresses that implement [`ToSocketAddrs`]
    ///
    /// # Returns
    ///
    /// Returns a [`MultiListener`] bound to all specified addresses, or an error
    /// if any binding fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # tokio_test::block_on(async {
    /// use axum_listener::multi::MultiListener;
    ///
    /// // Bind to multiple TCP ports
    /// let tcp_addresses = ["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"];
    /// let listener = MultiListener::bind(tcp_addresses).await.unwrap();
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - Any address format is invalid
    /// - Any address is already in use
    /// - Permission is denied for any requested address
    /// - The provided iterator is empty (no addresses to bind to)
    pub async fn bind<I: IntoIterator<Item = impl ToSocketAddrs>>(
        addresses: I,
    ) -> Result<Self, std::io::Error> {
        let listeners = futures::future::join_all(addresses.into_iter().map(TcpListener::bind))
            .await
            .into_iter()
            .inspect(|result| {
                if let Err(err) = result {
                    tracing::error!("Failed to bind TCP listener: {err}");
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(MultiListener { listeners })
    }
}

impl axum::serve::Listener for MultiListener {
    type Io = TcpStream;
    type Addr = MultiAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (out, _index, _rest) = futures::future::select_all(
            self.listeners
                .iter_mut()
                .map(|listener| Box::pin(async move { Listener::accept(listener).await })),
        )
        .await;
        tracing::trace!("Accepted connection on multi-listener from {}", _index);
        (out.0, MultiAddr { addrs: vec![out.1] })
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.listeners
            .iter()
            .map(|listener| listener.local_addr())
            .collect::<Result<Vec<_>, _>>()
            .map(|addrs| MultiAddr { addrs })
    }
}

const _: () = {
    use axum::extract::connect_info::Connected;
    impl Connected<MultiAddr> for MultiAddr {
        fn connect_info(remote_addr: MultiAddr) -> Self {
            remote_addr
        }
    }
    use axum::serve;

    impl Connected<serve::IncomingStream<'_, MultiListener>> for MultiAddr {
        fn connect_info(stream: serve::IncomingStream<'_, MultiListener>) -> Self {
            stream.remote_addr().clone()
        }
    }
};
