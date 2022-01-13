use super::{MediaTaskMsg, TrackController};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Webrtc session of an event
pub enum Target {
    Publish,
    Subscribe(Uuid),
}

/// Event emitted by the media task received from the `MediaTaskHandle`
pub enum HandleEvent {
    /// WebRTC ICE Candidate to be sent via the websocket
    IceCandidate {
        target: Target,
        candidate: String,
        mline_index: u32,
    },

    /// End Of Candidates event of the session of the given target
    NoMoreIceCandidates(Target),

    /// The webrtc-publish webrtcbin  has created an SDP offer
    /// to be transmitted to the controller/janus via the websocket
    PublishOffer(String),

    /// The webrtcbin has created an SDP answer
    /// to the offer provided by the controller/janus
    SubscribeAnswer { uuid: Uuid, answer: String },

    /// Received a DTMF digit from the SIP side
    DtmfDigit(i32),

    /// Media task encountered a fatal error and will shut down
    Error,
}

/// Handle to a `MediaTask` used to send event to the task and received events from it
///
/// When dropped the task is stopped gracefully
pub struct MediaTaskHandle {
    pub(super) tx: Arc<mpsc::UnboundedSender<MediaTaskMsg>>,
    pub(super) rx: mpsc::UnboundedReceiver<HandleEvent>,
}

#[derive(Debug, thiserror::Error)]
#[error("media task receiver dropped")]
pub struct HandleError;

impl MediaTaskHandle {
    /// Receive a event from the media task
    pub async fn poll_event(&mut self) -> HandleEvent {
        match self.rx.recv().await {
            Some(HandleEvent::Error) | None => HandleEvent::Error,
            Some(evt) => evt,
        }
    }

    /// Send the SIP SDP offer to the media task and in return
    /// receive a `TrackController` and SDP answer as string
    pub async fn send_sip_sdp_offer(
        &self,
        offer: Bytes,
    ) -> Result<(TrackController, String), HandleError> {
        let (answer_tx, answer_rx) = oneshot::channel();

        self.tx
            .send(MediaTaskMsg::CreateSipSession { offer, answer_tx })
            .map_err(|_| HandleError)?;

        answer_rx.await.map_err(|_| HandleError)
    }

    /// Send the SIP SDP offer of an received ReINVITE and wait for the media task
    /// to generate a SDP response
    pub async fn send_sip_sdp_reinvite(&self, offer: Bytes) -> Result<String, HandleError> {
        let (answer_tx, answer_rx) = oneshot::channel();

        self.tx
            .send(MediaTaskMsg::SipReInvite { offer, answer_tx })
            .map_err(|_| HandleError)?;

        answer_rx.await.map_err(|_| HandleError)
    }

    /// Instruct the media task to create the webrtc-publish webrtcbin, shortly afterwards
    /// a `HandleEvent::PublishOffer` will be received with the SDP offer
    pub fn send_create_publish(&self) -> Result<(), HandleError> {
        self.tx
            .send(MediaTaskMsg::CreatePublish)
            .map_err(|_| HandleError)?;

        Ok(())
    }

    /// The websocket task received an SDP response to the publish SDP offer from controller/janus
    pub fn send_publish_response(&self, response: String) -> Result<(), HandleError> {
        self.tx
            .send(MediaTaskMsg::ReceivePublishResponse { response })
            .map_err(|_| HandleError)?;

        Ok(())
    }

    /// Instruct the media task to create the webrtc-subscribe:{uuid} webrtcbin with the given SDP offer
    /// which was received from controller/janus after sending them an subscribe/requestOffer
    ///
    /// Shortly afterwards a `HandleEvent::SubscribeAnswer` will be received with the SDP answer
    pub fn send_create_subscribe(&self, uuid: Uuid, offer: String) -> Result<(), HandleError> {
        self.tx
            .send(MediaTaskMsg::CreateSubscribe { uuid, offer })
            .map_err(|_| HandleError)?;

        Ok(())
    }

    /// Instruct the media task to remove the webrtc-subscribe:{uuid} webrtcbin from the pipeline
    pub fn send_remove_subscribe(&self, uuid: Uuid) -> Result<(), HandleError> {
        self.tx
            .send(MediaTaskMsg::RemoveSubscribe { uuid })
            .map_err(|_| HandleError)?;

        Ok(())
    }
}
