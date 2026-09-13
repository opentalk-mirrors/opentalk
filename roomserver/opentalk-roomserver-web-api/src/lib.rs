// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! This crate provides a web API specification written as [Axum](https://docs.rs/axum/latest/axum/) endpoints.
//! It effectively provides a collection of traits that need to be implemented in order to provide
//! the web API. The [`Backend`][v1::Backend] Trait must be implemented by a project wanting to
//! provide the _RoomServer Web API_.

pub mod livekit_proxy;
pub mod v1;

pub(crate) type Router<Backend> = axum::Router<Backend>;
