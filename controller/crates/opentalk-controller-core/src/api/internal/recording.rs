// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::ws::Codec;
use actix_web::{
    HttpRequest, HttpResponse, get, post,
    web::{self, Data, Json, Query},
};
use actix_web_actors::ws;
use bytes::Bytes;
use opentalk_asset_storage::{
    ChunkFormat, ObjectStorage, ObjectStorageError, StorageNotifier, save_asset,
};
use opentalk_controller_api_actix_web::{
    host::Host,
    utoipa::responses::{InternalServerError, Unauthorized},
};
use opentalk_controller_service_facade::{NewAssetFileName, OpenTalkControllerService};
use opentalk_inventory::InventoryProvider;
use opentalk_roomserver_modules::RECORDING_MODULE_ID;
use opentalk_types_api_internal::recording::RecordingTarget;
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    rooms::by_room_id::RoomserverStartResponseBody,
    services::recording::GetRecordingUploadQuery,
};
use tokio::{sync::mpsc, task};

use crate::api::{
    headers::{ConnectionUpgrade, WebsocketUpgrade},
    upload::{MAXIMUM_WEBSOCKET_BUFFER_SIZE, UploadWebSocketActor},
};

/// This is a dummy type to define the structure of the headers required for
/// upgrading a request to a recording upload websocket connection.
#[derive(utoipa::IntoParams)]
#[into_params(
    parameter_in = Header,
)]
#[expect(dead_code)]
pub(crate) struct RecordingUploadWebSocketHeaders {
    #[param(inline, required = true)]
    pub connection: ConnectionUpgrade,

    #[param(inline, required = true)]
    pub upgrade: WebsocketUpgrade,
}

/// Starts a signaling session for recording
///
/// This endpoint is provided for participation of recording and streaming clients
/// which will join incognito and receive all the information and media streams required
/// for creating a recording or livestream of the meeting.
#[utoipa::path(
    context_path = "/recording",
    request_body = RecordingTarget,
    operation_id = "start_recording",
    responses(
        (
            status = StatusCode::OK,
            description = "The recording participant has successfully \
                authenticated for the room. Information needed for connecting to the signaling \
                is contained in the response",
            body = RoomserverStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Recording has not been configured",
            body = ErrorBody,
            example = json!(ApiError::not_found().body),
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
#[post("/recording/start")]
pub async fn post_start(
    service: Data<dyn OpenTalkControllerService>,
    body: Json<RecordingTarget>,
    host: Host,
) -> Result<Json<RoomserverStartResponseBody>, ApiError> {
    let response = service
        .start_recording(body.into_inner(), host.into_inner())
        .await?;

    Ok(Json(response))
}

/// Streaming upload of a rendered recording
///
/// This is a WebSocket endpoint, all the data that is sent in binary messages
/// is stored in the destination file.
#[utoipa::path(
    context_path = "/recording",
    params(
        GetRecordingUploadQuery,
        RecordingUploadWebSocketHeaders,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "WebSocket connection successfully established",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
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
#[get("/recording/upload")]
pub(crate) async fn get_upload(
    storage_connection_provider: Data<dyn InventoryProvider>,
    storage: Data<ObjectStorage>,
    notifier: Data<dyn StorageNotifier>,
    request: HttpRequest,
    Query(GetRecordingUploadQuery {
        room_id,
        file_extension,
        timestamp,
    }): Query<GetRecordingUploadQuery>,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse> {
    // Finish websocket handshake
    let (sender, receiver) = mpsc::unbounded_channel::<Result<Bytes, ObjectStorageError>>();
    let receiver_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
    let (_addr, response) =
        ws::WsResponseBuilder::new(UploadWebSocketActor::new(sender), &request, stream)
            .codec(Codec::new().max_size(100_000_000))
            .frame_size(MAXIMUM_WEBSOCKET_BUFFER_SIZE)
            .start_with_addr()?;

    // Spawn the runner task
    task::spawn_local({
        async move {
            let kind = "recording"
                .parse()
                .expect("Must be parseable as AssetFileKind");
            let filename = NewAssetFileName::new(kind, timestamp, file_extension);

            let result = save_asset(
                &storage,
                storage_connection_provider.as_ref(),
                notifier.as_ref(),
                &room_id.into(),
                Some(RECORDING_MODULE_ID),
                filename,
                receiver_stream,
                ChunkFormat::SequenceNumberAndData,
                None,
            )
            .await;

            if let Err(e) = result {
                tracing::error!("Error saving asset, {e}");
            }
        }
    });

    Ok(response)
}
