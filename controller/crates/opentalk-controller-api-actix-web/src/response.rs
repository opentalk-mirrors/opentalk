// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Custom general purpose response types for HTTP endpoints.

use actix_web::{HttpResponse, Responder, body::BoxBody};

/// Represents a 201 Created HTTP Response
#[derive(Debug)]
pub struct Created;

impl Responder for Created {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::Created().finish()
    }
}

/// Represents a 204 No Content HTTP Response
#[derive(Debug)]
pub struct NoContent;

impl Responder for NoContent {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::NoContent().finish()
    }
}

/// Represents a 304 Not Modified HTTP Response
#[derive(Debug)]
pub struct NotModified;

impl Responder for NotModified {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::NotModified().finish()
    }
}

/// Represents a 410 Gone HTTP Response for endpoints that have been removed
#[derive(Debug)]
pub struct Gone;

impl Responder for Gone {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::Gone().finish()
    }
}
