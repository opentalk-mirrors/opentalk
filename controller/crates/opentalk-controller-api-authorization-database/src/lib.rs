// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! WebAPI endpoint authorization based on permissions stored in the database.
//!
//! This crate provides [`OpenTalkAuthorizerBackend`], the production implementation of the
//! [`AuthorizerBackend`] trait defined by [`opentalk_controller_api_authorization`]. The controller
//! plugs this backend into an [`Authorizer`], which the actix-web authorization middleware then
//! uses to admit or refuse each incoming request to the WebAPI.
//!
//! # How an authorization decision is made
//!
//! Each request from the middleware arrives as an [`AuthorizationTarget`], which bundles three
//! pieces of information:
//!
//! - a [`SubjectCollection`] of authenticated subjects — a request can carry an authenticated user,
//!   and access is granted if *any* subject is admitted;
//! - the [`Resource`] being accessed (identified by its `/v1/…` URL pattern);
//! - the [`AccessMethod`] (an HTTP verb, classified into read-only vs. read/write).
//!
//! [`AuthorizerBackend::authorize`] dispatches on the [`Resource`] variant to a per-resource
//! handler in the private `event`, `room` or `user` module. Each handler declares an ACL table that
//! pairs the four well-known subject categories with the access level the resource grants them, and
//! then evaluates the request against that table using the shared helpers in the private `common`
//! module:
//!
//! | Subject          | Meaning                                                     |
//! | ---------------- | ----------------------------------------------------------- |
//! | **Owner**        | The user that created the event or room.                    |
//! | **Moderator**    | An invited user with the `Moderator` invite role.           |
//! | **Invited-User** | An invited user with the `User` invite role.                |
//! | **Guest**        | A guest (subject to tariff).                                |
//!
//! Handlers translate a subject to its category by asking the [`AuthorizationInventory`] — obtained
//! through the injected [`InventoryProvider`] — for either the user's role on the event/room or the
//! validity of guest access which depends on the tariff's disabled features and the module feature
//! registry.
//!
//! Two lightweight helpers handle resources that don't fit the Owner/Moderator/Invited/Guest
//! model: `require_read_write_user` and `require_read_user`, used by endpoints such as `/v1/rooms`,
//! `/v1/events`, `/v1/users/find` and the `/v1/users/me/…` family, which only distinguish "any
//! registered user" from "guest".
//!
//! # Change notifications
//!
//! Because every decision is recomputed from the database, there is no cache to invalidate.
//! [`OpenTalkAuthorizerBackend`]'s [`AuthorizerBackend::apply_changes`] implementation is therefore
//! intentionally empty. A future backend that maintains a cache would need to provide its own
//! synchronization; see the note on [`AuthorizerBackend::apply_changes`] in the parent crate.
//!
//! [`AccessMethod`]: opentalk_controller_api_authorization::authorization::AccessMethod
//! [`Authorizer`]: opentalk_controller_api_authorization::authorization::Authorizer
//! [`AuthorizerBackend`]: opentalk_controller_api_authorization::authorization::AuthorizerBackend
//! [`AuthorizerBackend::authorize`]: opentalk_controller_api_authorization::authorization::AuthorizerBackend::authorize
//! [`AuthorizerBackend::apply_changes`]: opentalk_controller_api_authorization::authorization::AuthorizerBackend::apply_changes
//! [`AuthorizationInventory`]: opentalk_inventory::AuthorizationInventory
//! [`AuthorizationTarget`]: opentalk_controller_api_authorization::authorization::AuthorizationTarget
//! [`InventoryProvider`]: opentalk_inventory::InventoryProvider
//! [`Resource`]: opentalk_controller_api_authorization::authorization::Resource
//! [`SubjectCollection`]: opentalk_controller_api_authorization::authorization::SubjectCollection

mod auth;
mod authorizer_backend;
mod common;
mod error;
mod event;
mod invite_verify;
mod room;
mod signaling;
mod turn;
mod user;

pub use authorizer_backend::OpenTalkAuthorizerBackend;
pub use error::Result;
