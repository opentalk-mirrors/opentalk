// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use opentalk_roomserver_signaling::storage::{
    assets::provider::AssetStorageProvider, module_resources::provider::ModuleResourceProvider,
};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    core::RoomCloseReason,
    livekit_proxy::{LiveKitProxyRequest, PreparedSocket, websocket::LiveKitSocket},
    room_parameters::RoomParameters,
    room_parameters_patch::RoomParametersPatch,
    signaling::websocket::SignalingSocket,
};
use opentalk_types_api_internal::{error::ApiError, module_assets::Quota};
use opentalk_types_common::users::UserId;
use tokio::sync::{
    mpsc,
    oneshot::{self, Receiver},
};
use tracing::Span;
use url::Url;

use super::RoomTaskApiError;

#[derive(Debug, thiserror::Error)]
pub enum RoomTaskHandleError<Socket: SignalingSocket> {
    /// The room task is gone.
    ///
    /// If the room task received the request, [`request`](RoomTaskHandleError::Gone::request) will
    /// be `None`. If the room task did not receive the request, it's returned to the sender by
    /// setting it here.
    #[error("The room task is no longer available")]
    Gone {
        /// If the request couldn't be dispatched to the room task, it's returned here.
        request: Box<Option<Request<Socket>>>,
    },

    #[error("API request failed: {0}")]
    ApiError(#[from] RoomTaskApiError),
}

impl<Socket: SignalingSocket> From<RoomTaskHandleError<Socket>> for ApiError {
    fn from(error: RoomTaskHandleError<Socket>) -> Self {
        match error {
            RoomTaskHandleError::Gone { request: _ } => {
                Self::not_found().with_message("The requested room could not be found")
            }
            RoomTaskHandleError::ApiError(ref room_task_api_error) => match room_task_api_error {
                RoomTaskApiError::NotImplemented => {
                    ApiError::internal().with_message(error.to_string())
                }
                RoomTaskApiError::NotFound => ApiError::not_found().with_message(error.to_string()),
                RoomTaskApiError::Unauthorized => {
                    ApiError::unauthorized().with_message(error.to_string())
                }
                RoomTaskApiError::FailedToApplyPatch(inner_err) => {
                    ApiError::unprocessable_entity().with_message(format!("{error}: {inner_err}"))
                }
                RoomTaskApiError::Closing => {
                    ApiError::service_unavailable().with_message(error.to_string())
                }
                RoomTaskApiError::Internal => ApiError::internal().with_message(error.to_string()),
            },
        }
    }
}

impl<Socket: SignalingSocket> RoomTaskHandleError<Socket> {
    /// Take the request from the error.
    ///
    /// If the message could not be delivered to the room task, the request
    /// might be required for error handling (e.g. closing the WebSocket).
    pub fn take_request(&mut self) -> Option<Request<Socket>> {
        match self {
            RoomTaskHandleError::Gone { request } => request.take(),
            RoomTaskHandleError::ApiError(_) => None,
        }
    }
}

/// A handle for the a [`super::RoomTask`]
///
/// Is used for communication between the room task and the web server API
#[derive(Debug)]
pub struct RoomTaskHandle<Socket: SignalingSocket> {
    pub(super) assets: Arc<dyn AssetStorageProvider>,
    pub(super) module_resources: Arc<dyn ModuleResourceProvider>,
    pub(super) sender: mpsc::Sender<TaskMessage<Socket>>,
}

// Manually implementing clone so that we don't require [`Socket`] to be
// Clone as well.
impl<Socket: SignalingSocket> Clone for RoomTaskHandle<Socket> {
    fn clone(&self) -> Self {
        Self {
            assets: Arc::clone(&self.assets),
            module_resources: Arc::clone(&self.module_resources),
            sender: self.sender.clone(),
        }
    }
}

impl<Socket: SignalingSocket> RoomTaskHandle<Socket> {
    async fn send_request(
        &self,
        request: Request<Socket>,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let msg = TaskMessage::new(request);
        self.sender
            .send(msg)
            .await
            .map_err(|e| RoomTaskHandleError::Gone {
                request: Box::new(Some(e.0.request)),
            })?;

        Ok(())
    }

    async fn receive_response<T>(
        rx: Receiver<Result<T, RoomTaskApiError>>,
    ) -> Result<T, RoomTaskHandleError<Socket>> {
        let response = rx.await.map_err(|_| RoomTaskHandleError::Gone {
            request: Box::new(None),
        })??;

        Ok(response)
    }

    /// Refresh the room idle timeout to its original duration
    ///
    /// This can only fail if the room has reached its idle timeout or been removed by a user
    pub async fn refresh_idle_timeout(&self) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::RefreshIdleTimeout { response: tx })
            .await?;

        Self::receive_response(rx).await
    }

    /// Set the parameters for the room
    pub async fn set_parameters(
        &self,
        parameter: RoomParameters,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::SetParameters {
            response: tx,
            parameters: Box::new(parameter),
        })
        .await?;

        Self::receive_response(rx).await
    }

    pub async fn patch_parameters(
        &self,
        patch: RoomParametersPatch,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::PatchParameters {
            response: tx,
            patch: Box::new(patch),
        })
        .await?;

        Self::receive_response(rx).await?;

        Ok(())
    }

    pub async fn close(&self, reason: RoomCloseReason) {
        let (tx, rx) = oneshot::channel();

        // When the room task is already gone, we do not want to respond with an error because the
        // room **is** closed.
        self.send_request(Request::Close {
            response: tx,
            reason,
        })
        .await
        .unwrap_or_else(|err| tracing::error!("Room task already gone: {err}"));

        Self::receive_response(rx)
            .await
            .unwrap_or_else(|err| tracing::error!("Room task already gone: {err}"));
    }

    pub async fn set_storage_quota(&self, quota: Quota) -> Result<(), RoomTaskHandleError<Socket>> {
        self.assets.set_storage_quota(quota.clone()).await;

        let (tx, rx) = oneshot::channel();
        self.send_request(Request::StorageQuotaChanged {
            response: tx,
            quota,
        })
        .await?;

        Self::receive_response(rx).await?;

        Ok(())
    }

    pub async fn accept_signaling_socket(
        &self,
        socket: Socket,
        client_parameters: ClientParameters,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::WsJoin {
            response: tx,
            socket,
            client_parameters,
        })
        .await?;

        Self::receive_response(rx).await
    }

    pub async fn reject_if_banned(&self, user_id: UserId) -> Result<(), ApiError> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::IsBanned {
            response: tx,
            user_id,
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to check ban status of participant {user_id}: {e}");
            ApiError::internal().with_message("Failed to check the users ban status")
        })?;

        let is_banned = Self::receive_response(rx).await.map_err(|e| {
            tracing::error!("Failed to check ban status of participant {user_id}: {e}");
            ApiError::internal().with_message("Failed to check the users ban status")
        })?;

        if is_banned {
            Err(ApiError::forbidden()
                .with_code("banned")
                .with_message("User is banned from this room"))
        } else {
            Ok(())
        }
    }

    pub async fn allowed_origins(&self) -> Option<Vec<String>> {
        let (tx, rx) = oneshot::channel();

        if self
            .send_request(Request::AllowedOrigins { response: tx })
            .await
            .is_err()
        {
            return None;
        }

        Self::receive_response(rx).await.ok()
    }

    pub async fn is_guest_access_allowed(&self) -> Option<bool> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::GuestAccess { response: tx })
            .await
            .inspect_err(|err| tracing::error!("Failed to check guest access: {err}"))
            .ok()?;

        Self::receive_response(rx).await.ok()
    }

    pub fn assets(&self) -> Arc<dyn AssetStorageProvider> {
        Arc::clone(&self.assets)
    }

    pub fn module_resources(&self) -> Arc<dyn ModuleResourceProvider> {
        Arc::clone(&self.module_resources)
    }

    pub async fn prepare_proxy_socket(
        &self,
        websocket_request: LiveKitProxyRequest,
    ) -> Result<PreparedSocket, RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::ConnectUpstreamLivekitSocket {
            response: tx,
            websocket_request,
        })
        .await?;

        Self::receive_response(rx).await
    }

    pub async fn accept_livekit_socket(
        &self,
        websocket_request: LiveKitProxyRequest,
        upstream_socket: PreparedSocket,
        downstream_socket: Box<dyn LiveKitSocket>,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::ConnectDownstreamLivekitSocket {
            response: tx,
            websocket_request,
            upstream_socket: Box::new(upstream_socket),
            downstream_socket,
        })
        .await?;

        Self::receive_response(rx).await
    }

    pub async fn livekit_service_url(&self) -> Result<Url, RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::GetLivekitServiceUrl { response: tx })
            .await?;

        Self::receive_response(rx).await
    }
}

#[cfg(any(test, feature = "mock"))]
impl Default for RoomTaskHandle<crate::mocking::socket::MockSocket> {
    fn default() -> Self {
        use crate::storage::{
            memory_asset_storage::MemoryAssetStorage,
            memory_module_storage::MemoryModuleResourceStorage,
        };

        let assets = Arc::new(MemoryAssetStorage::new(Quota {
            total: None,
            used: 0,
        }));
        let module_resources = Arc::new(MemoryModuleResourceStorage::new());
        let (sender, _) = mpsc::channel(1);

        Self {
            assets,
            module_resources,
            sender,
        }
    }
}

/// A message that can be send to a [`super::RoomTask`]
///
/// See [`Request`] for the possible messages.
pub(super) struct TaskMessage<Socket: SignalingSocket> {
    /// The specific request
    pub request: Request<Socket>,
    /// Parent span on the caller side, used to track spans across channels.
    pub span: Span,
}

impl<Socket: SignalingSocket> TaskMessage<Socket> {
    fn new(request: Request<Socket>) -> Self {
        let span = tracing::Span::current();
        Self { request, span }
    }
}

type ResponseSender<T> = oneshot::Sender<Result<T, RoomTaskApiError>>;

/// A request to a [`RoomTask`](super::RoomTask)
#[derive(Debug)]
pub enum Request<Socket: SignalingSocket> {
    /// Refresh the room tasks idle timeout
    RefreshIdleTimeout {
        response: ResponseSender<()>,
    },

    /// Set the parameters for the room
    SetParameters {
        response: ResponseSender<()>,
        parameters: Box<RoomParameters>,
    },

    /// Partially update the parameters for the room
    PatchParameters {
        response: ResponseSender<()>,
        patch: Box<RoomParametersPatch>,
    },

    /// Notify the room task that the storage quota changed
    StorageQuotaChanged {
        response: ResponseSender<()>,
        quota: Quota,
    },

    /// Check if a user with the given [`UserId`] is banned
    IsBanned {
        response: ResponseSender<bool>,
        user_id: UserId,
    },

    AllowedOrigins {
        response: ResponseSender<Vec<String>>,
    },

    /// Check if guests are allowed to participate in the meeting
    GuestAccess {
        response: ResponseSender<bool>,
    },

    /// Join the room with a given websocket stream and sink
    WsJoin {
        response: ResponseSender<()>,
        socket: Socket,
        client_parameters: ClientParameters,
    },

    ConnectUpstreamLivekitSocket {
        response: ResponseSender<PreparedSocket>,
        websocket_request: LiveKitProxyRequest,
    },

    ConnectDownstreamLivekitSocket {
        response: ResponseSender<()>,
        websocket_request: LiveKitProxyRequest,
        upstream_socket: Box<PreparedSocket>,
        downstream_socket: Box<dyn LiveKitSocket>,
    },

    GetLivekitServiceUrl {
        response: ResponseSender<Url>,
    },

    /// Close the room task with the given reason
    Close {
        response: ResponseSender<()>,
        /// The close reason sent to the participants in [`CoreEvent::Closing`]
        ///
        /// [`CoreEvent::Closing`]: opentalk_roomserver_types::core::CoreEvent::Closing
        reason: RoomCloseReason,
    },
}

impl<Socket: SignalingSocket> Request<Socket> {
    pub fn send_error(self, error: RoomTaskApiError) -> anyhow::Result<()> {
        match self {
            Request::RefreshIdleTimeout { response }
            | Request::SetParameters { response, .. }
            | Request::PatchParameters { response, .. }
            | Request::StorageQuotaChanged { response, .. }
            | Request::Close { response, .. }
            | Request::WsJoin { response, .. } => response
                .send(Err(error))
                .map_err(|_| anyhow::anyhow!("Failed to send response to client")),

            Request::IsBanned { response, .. } | Request::GuestAccess { response } => response
                .send(Err(error))
                .map_err(|_| anyhow::anyhow!("Failed to send response to client")),

            Request::AllowedOrigins { response, .. } => response
                .send(Err(error))
                .map_err(|_| anyhow::anyhow!("Failed to send response to client")),
            Request::ConnectUpstreamLivekitSocket { response, .. } => response
                .send(Err(error))
                .map_err(|_| anyhow::anyhow!("Failed to send response to client")),
            Request::ConnectDownstreamLivekitSocket { response, .. } => response
                .send(Err(error))
                .map_err(|_| anyhow::anyhow!("Failed to send response to client")),
            Request::GetLivekitServiceUrl { response } => response
                .send(Err(error))
                .map_err(|_| anyhow::anyhow!("Failed to send response to client")),
        }
    }
}
