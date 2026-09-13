// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_internal::module_resources::{
    ModuleResource, ModuleResourceFilter, NewModuleResource, PatchModuleResourceBody,
};
use opentalk_types_api_v1::error::ApiError;

use crate::api::{
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::{ApiResponse, DefaultApiResult},
};

#[utoipa::path(
    context_path = "/internal",
    operation_id = "internal_create_module_resource",
    request_body(
        content = NewModuleResource,
        content_type = "application/json",
        description = "The module resource to create",
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The module resource has been created",
            body = ModuleResource,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[post("/module_resources")]
pub async fn create(
    service: Data<dyn OpenTalkControllerService>,
    resource: Json<NewModuleResource>,
) -> DefaultApiResult<ModuleResource> {
    Ok(ApiResponse::new(
        service
            .create_module_resource(resource.into_inner())
            .await?,
    ))
}

#[utoipa::path(
    context_path = "/internal",
    operation_id = "internal_get_module_resource",
    request_body(
        content = ModuleResourceFilter,
        content_type = "application/json",
        description = "The filter that is applied to the module resource search",
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The module resources matching the provided filter",
            body = Vec<ModuleResource>,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "The provided resource filter is empty or invalid",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/module_resources")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    filter: Json<ModuleResourceFilter>,
) -> DefaultApiResult<Vec<ModuleResource>> {
    let filter = filter.into_inner();
    verify_filter(&filter)?;

    Ok(ApiResponse::new(
        service.get_module_resources(filter).await?,
    ))
}

#[utoipa::path(
    context_path = "/internal",
    operation_id = "internal_patch_module_resource",
    request_body(
        content = PatchModuleResourceBody,
        content_type = "application/json",
        description = "The module resource filter and a list of patch operations",
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The patched module resources",
            body = Vec<ModuleResource>,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "The provided resource filter is empty or invalid"
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/module_resources")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    Json(PatchModuleResourceBody {
        filter,
        patch_operations,
    }): Json<PatchModuleResourceBody>,
) -> DefaultApiResult<Vec<ModuleResource>> {
    verify_filter(&filter)?;

    Ok(ApiResponse::new(
        service
            .patch_module_resources(filter, patch_operations)
            .await?,
    ))
}

#[utoipa::path(
    context_path = "/internal",
    operation_id = "internal_delete_module_resource",
    request_body(
        content = ModuleResourceFilter,
        content_type = "application/json",
        description = "The module resource filter",
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The deleted module resources",
            body = Vec<ModuleResource>,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "The provided resource filter is empty or invalid",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[delete("/module_resources")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    filter: Json<ModuleResourceFilter>,
) -> DefaultApiResult<Vec<ModuleResource>> {
    let filter = filter.into_inner();
    verify_filter(&filter)?;

    Ok(ApiResponse::new(
        service.delete_module_resources(filter).await?,
    ))
}

fn verify_filter(filter: &ModuleResourceFilter) -> Result<(), ApiError> {
    if filter.is_empty() {
        return Err(
            ApiError::bad_request().with_message("Empty module resource filter is prohibited")
        );
    }

    if filter.room_id.is_none() && filter.id.is_none() && filter.created_by.is_none() {
        return Err(ApiError::bad_request().with_message(
            "The module resource filter must have at least 'id', 'room_id' or 'created_by' set",
        ));
    }

    Ok(())
}
