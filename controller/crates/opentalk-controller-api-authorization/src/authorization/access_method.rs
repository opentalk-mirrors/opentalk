// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::events::invites::InviteRole;
use snafu::Snafu;

/// Access methods to a resource.
///
/// These match a set of well-known HTTP methods, unknown methods are not
/// represented by this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessMethod {
    /// GET request access to the resource.
    Get,

    /// POST request access to the resource.
    Post,

    /// PUT request access to the resource.
    Put,

    /// DELETE request access to the resource.
    Delete,

    /// HEAD request access to the resource.
    Head,

    /// OPTIONS request access to the resource.
    Options,

    /// CONNECT request access to the resource.
    Connect,

    /// PATCH request access to the resource.
    Patch,

    /// TRACE request access to the resource.
    Trace,
}

#[derive(Debug, Snafu)]
pub enum TryFromHttpMethodError {
    #[snafu(display("Unknown HTTP method {method:?}"))]
    UnknownHttpMethod { method: String },
}

impl TryFrom<&http0::Method> for AccessMethod {
    type Error = TryFromHttpMethodError;

    fn try_from(value: &http0::Method) -> Result<Self, Self::Error> {
        use http0::Method;

        match value {
            &Method::GET => Ok(Self::Get),
            &Method::POST => Ok(Self::Post),
            &Method::PUT => Ok(Self::Put),
            &Method::DELETE => Ok(Self::Delete),
            &Method::HEAD => Ok(Self::Head),
            &Method::OPTIONS => Ok(Self::Options),
            &Method::CONNECT => Ok(Self::Connect),
            &Method::PATCH => Ok(Self::Patch),
            &Method::TRACE => Ok(Self::Trace),
            m => Err(TryFromHttpMethodError::UnknownHttpMethod {
                method: m.to_string(),
            }),
        }
    }
}

impl AccessMethod {
    /// The method is safe with regard to not performing any modifications on the server
    /// according to <https://developer.mozilla.org/en-US/docs/Glossary/Safe/HTTP>.
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options | Self::Trace)
    }

    /// The method only requires read permission on the endpoint resource.
    pub const fn is_read_only(&self) -> bool {
        self.is_safe()
    }

    /// The method requires write permission on the endpoint resource.
    pub const fn requires_write_access(&self) -> bool {
        !self.is_read_only()
    }

    /// Get the required invite role for accessing a resource.
    pub const fn required_invite_role(&self) -> Option<InviteRole> {
        if self.is_read_only() {
            Some(InviteRole::User)
        } else {
            None
        }
    }
}

#[cfg(feature = "actix-web")]
pub mod actix_web_impls {
    use actix_web::dev::ServiceRequest;

    use super::*;

    impl TryFrom<&ServiceRequest> for AccessMethod {
        type Error = TryFromHttpMethodError;

        fn try_from(req: &ServiceRequest) -> Result<Self, Self::Error> {
            AccessMethod::try_from(req.method())
        }
    }
}
