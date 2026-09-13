// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use http::StatusCode;

use super::{AuthenticationError, ErrorBody, ValidationErrorEntry};

/// The default REST API error
///
/// Can be build via the associated functions to represent various HTTP errors. Each
/// HTTP error has their default error code and message that get send in a JSON body.
/// The error code and message can be overwritten when creating an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// The HTTP status code of the error
    pub status: StatusCode,

    /// An optional authentication header value
    pub www_authenticate: Option<AuthenticationError>,

    /// The body of the error
    pub body: ErrorBody,
}

impl ApiError {
    /// Create a new 500 Internal Server Error
    pub fn internal() -> Self {
        Self::new_standard(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            "An internal server error occurred",
        )
    }

    fn new_standard<T>(status: StatusCode, code: T, message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            status,
            www_authenticate: None,
            body: ErrorBody::new(code, message),
        }
    }

    /// Override the default code for an error
    pub fn with_code<T>(mut self, code: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.body.code = code.into();
        self
    }

    /// Override the default message for an error
    pub fn with_message<T>(mut self, message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.body.message = message.into();
        self
    }

    /// Add the www_authenticate header to this error
    pub fn with_www_authenticate(mut self, authentication_error: AuthenticationError) -> Self {
        self.www_authenticate = Some(authentication_error);
        self
    }

    /// Create a new 400 Bad Request error
    pub fn bad_request() -> Self {
        Self::new_standard(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "Invalid request due to malformed syntax",
        )
    }

    /// Create a new 401 Unauthorized error
    pub fn unauthorized() -> Self {
        Self::new_standard(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Authentication failed",
        )
    }

    /// Create a new 403 Forbidden error
    pub fn forbidden() -> Self {
        Self::new_standard(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Access to the requested resource is forbidden",
        )
    }

    /// Create a new 404 Not Found error
    pub fn not_found() -> Self {
        Self::new_standard(
            StatusCode::NOT_FOUND,
            "not_found",
            "A requested resource could not be found",
        )
    }

    /// Create a new 409 Conflict error
    pub fn conflict() -> Self {
        Self::new_standard(
            StatusCode::CONFLICT,
            "conflict",
            "The request conflicts with the state of the resource",
        )
    }

    /// Create a new 422 Unprocessable Entity error
    ///
    /// see [`Self::unprocessable_entities()`]
    pub fn unprocessable_entity() -> Self {
        Self::unprocessable_entities::<ValidationErrorEntry, _>([])
    }

    /// Create a new 422 Unprocessable Entity error
    ///
    /// The JSON body for this error additionally contains a list of errors for each invalid field.
    pub fn unprocessable_entities<T, I>(errors: I) -> Self
    where
        T: Into<ValidationErrorEntry>,
        I: IntoIterator<Item = T>,
    {
        let validation_errors = errors.into_iter().map(|entry| entry.into()).collect();

        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            www_authenticate: None,
            body: ErrorBody {
                code: "validation_failed".into(),
                message: "Some provided values are invalid".into(),
                validation_errors,
            },
        }
    }

    /// Create a new 503 Service Unavailable error
    pub fn service_unavailable() -> Self {
        Self::new_standard(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            "The service is currently unavailable",
        )
    }

    /// Creates a new 413 Payload Too Large error that indicates that the storage quota was exceeded.
    pub fn storage_quota_exceeded() -> Self {
        Self::new_standard(
            StatusCode::PAYLOAD_TOO_LARGE,
            "storage_quota_exceeded",
            "Storage quota has been exceeded",
        )
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "status={}, code={}, message={}",
            self.status, self.body.code, self.body.message
        )
    }
}

#[cfg(feature = "actix")]
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> http0::StatusCode {
        let status_code = self.status.as_u16();
        http0::StatusCode::from_u16(status_code).unwrap_or_else(|_| {
            log::error!(
                "Received invalid status code {status_code} when converting from http 1 to http 0"
            );
            http0::StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut response = actix_web::HttpResponse::new(self.status_code());

        let _ = response.headers_mut().insert(
            http0::header::CONTENT_TYPE,
            http0::HeaderValue::from_static("text/json; charset=utf-8"),
        );

        if let Some(www_authenticate) = self.www_authenticate {
            let _ = response.headers_mut().insert(
                http0::header::WWW_AUTHENTICATE,
                www_authenticate
                    .header_value()
                    .try_into()
                    .expect("Unable to create www-authenticate bearer header-value"),
            );
        }

        let body = serde_json::to_string(&self.body).expect("Unable to serialize API error body");

        response.set_body(actix_web::body::BoxBody::new(body))
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let mut builder = axum::response::Response::builder();

        builder = builder.status(self.status.as_u16());

        builder = builder.header(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/json; charset=utf-8"),
        );

        let builder = if let Some(www_authenticate) = self.www_authenticate {
            builder.header(
                http::header::WWW_AUTHENTICATE,
                www_authenticate.header_value(),
            )
        } else {
            builder
        };

        let body = serde_json::to_string(&self.body).expect("Unable to serialize API error body");

        builder
            .body(axum::body::Body::new(body))
            .expect("Failed to build axum Response ")
    }
}
