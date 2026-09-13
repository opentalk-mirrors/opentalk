// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{AccessMethod, Resource, SubjectCollection};

/// The target of an authorization request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorizationTarget {
    /// The subject which requests authorization
    pub authenticated_subjects: SubjectCollection,

    /// The resource for which authorization is requested
    pub resource: Resource,

    /// The access mode that is requested for the resource
    pub access_method: AccessMethod,
}

#[cfg(feature = "actix-web")]
mod actix_web_impls {
    use actix_web::dev::ServiceRequest;
    use snafu::{ResultExt, Snafu};

    use super::AuthorizationTarget;
    use crate::authorization::{
        access_method::TryFromHttpMethodError, resource::actix_web_impls::TryFromResourceError,
    };

    #[derive(Debug, Snafu)]
    pub enum TryFromServiceRequestError {
        #[snafu(display("Unable to load resource from request"))]
        Resource {
            source: TryFromResourceError,
        },

        HttpMethod {
            source: TryFromHttpMethodError,
        },
    }

    impl TryFrom<&ServiceRequest> for AuthorizationTarget {
        type Error = TryFromServiceRequestError;

        fn try_from(request: &ServiceRequest) -> Result<Self, Self::Error> {
            let authenticated_subjects = request.into();
            let resource = request.try_into().context(ResourceSnafu)?;
            let access_method = request.try_into().context(HttpMethodSnafu)?;

            Ok(Self {
                authenticated_subjects,
                resource,
                access_method,
            })
        }
    }
}
