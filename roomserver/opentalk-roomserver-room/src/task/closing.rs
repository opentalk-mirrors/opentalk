// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use futures::StreamExt as _;
use opentalk_roomserver_signaling::event_origin::EventOrigin;
use opentalk_roomserver_types::{
    room_kind::RoomKind,
    signaling::{module_error::FatalError, websocket::SignalingSocket},
};

use crate::{
    RoomTaskApiError,
    signaling::DynBroadcastEvent,
    task::{RoomCloseReason, RoomTask},
};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    pub(super) async fn close(&mut self, close_reason: RoomCloseReason) -> Result<(), FatalError> {
        self.broadcast_closing_event(close_reason)?;

        self.cancel_loopback_guard();

        tracing::trace!("broadcast closing event to modules");
        let action_result = self
            .broadcast_event_to_modules(
                EventOrigin::Internal,
                RoomKind::Main,
                DynBroadcastEvent::Closing,
            )
            .handle_requested_messages(self);
        if let Err(e) = action_result {
            tracing::error!("Failed to handle requested messages while closing: {e:?}");
        }

        loop {
            tracing::trace!("Remaining loopback tasks: {}", self.loopback_futures.len());
            tokio::select! {
                msg = self.api_rx.recv() => {
                    let Some(msg) = msg else {
                        // TaskHandle dropped, exiting
                        tracing::warn!("Room tasks {} api channel was dropped, exiting", self.info.room_id);

                        // if the the TaskHandle was dropped, we can assume that RoomTaskRegistry was dropped
                        // and thus the whole application is shutting down.
                        break;
                    };

                    if let Err(e) = msg.request.send_error(RoomTaskApiError::Closing) {
                        tracing::error!("Failed to handle room task api request: {e:?}");
                    }
                },
                msg = self.loopback_futures.next() => {
                    if let Some(msg) = msg {
                        self.handle_loopback(msg)?;
                    } else {
                        tracing::trace!("All loopback tasks finished");
                        break;
                    }
                },
            };
        }

        Ok(())
    }

    fn cancel_loopback_guard(&mut self) {
        tracing::trace!("Cancel loopback guard");
        if let Some(loopback_cancel_tx) = self.loopback_cancel_tx.take()
            && loopback_cancel_tx.send(()).is_err()
        {
            tracing::warn!(
                "Failed to send loopback cancel signal, loopback futures may not finish"
            );
        }
    }
}
