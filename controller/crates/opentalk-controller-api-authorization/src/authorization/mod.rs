// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Interfaces and types for implementing authorization checks.

mod access_method;
mod admission;
mod authorization_change;
mod authorization_change_error;
mod authorization_error;
mod authorization_target;
mod authorizer;
mod authorizer_backend;
mod resource;
mod subject;
mod subject_collection;

pub use access_method::AccessMethod;
pub use admission::Admission;
pub use authorization_change::AuthorizationChange;
pub use authorization_change_error::AuthorizationChangeError;
pub use authorization_error::AuthorizationError;
pub use authorization_target::AuthorizationTarget;
pub use authorizer::Authorizer;
pub use authorizer_backend::AuthorizerBackend;
#[cfg(feature = "mockall")]
pub use authorizer_backend::MockAuthorizerBackend;
pub use resource::Resource;
pub use subject::Subject;
pub use subject_collection::SubjectCollection;
