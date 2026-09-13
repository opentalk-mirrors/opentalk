// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText};
use opentalk_roomserver_types::{
    api::RoomServerAccess, client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;
use tokio::sync::{mpsc::UnboundedSender, oneshot::Receiver};
use url::Url;

use super::{TransitionToView, error::RunnerGoneError, shortcuts::SUBMIT_SHORTCUT};
use crate::{
    client::{RunnerCommand, RunnerResponse},
    settings::DuiSettings,
};

type ResponseReceiver<T> = Receiver<RunnerResponse<T>>;

#[derive(Debug, Default)]
struct Progress {
    push_server_settings: SetupProgress<(Url, ResponseReceiver<()>), Url>,
    request_token: SetupProgress<ResponseReceiver<RoomServerAccess>, RoomServerAccess>,
    signaling_connection: SetupProgress<ResponseReceiver<()>, ()>,
}

#[derive(Debug)]
pub struct ConnectingView {
    room_id: RoomId,
    room_parameters: RoomParameters,
    client_parameters: ClientParameters,
    progress: Progress,
}

impl ConnectingView {
    pub(crate) fn new(
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: RoomParameters,
    ) -> Self {
        Self {
            room_id,
            room_parameters,
            client_parameters,

            progress: Progress::default(),
        }
    }

    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        settings: &DuiSettings,
    ) -> Option<TransitionToView> {
        let roomserver_url = if let SetupProgress::Done(url) = &self.progress.push_server_settings {
            url
        } else {
            &settings.roomserver_url
        };

        ui.horizontal(|ui| {
            ui.label("Connecting to");
            ui.label(RichText::new(roomserver_url.to_string()).color(Color32::LIGHT_BLUE));
        });

        ui.horizontal(|ui| {
            self.progress.push_server_settings.ui(ui);
            ui.label("Configure RoomServer connection");
        });
        if !self.progress.push_server_settings.is_finished()
            && self.do_push_server_settings(command_tx, settings).is_err()
        {
            return Some(TransitionToView::Error {
                message: RichText::new("RoomServer Connection task crashed"),
            });
        }

        ui.horizontal(|ui| {
            self.progress.request_token.ui(ui);
            ui.label("Request Room Token");
        });
        if !self.progress.request_token.is_finished()
            && self.progress.push_server_settings.is_finished()
        {
            self.do_request_token(command_tx);
        }

        ui.horizontal(|ui| {
            self.progress.signaling_connection.ui(ui);
            ui.label("Connect signaling");
        });
        if self.progress.request_token.is_done() {
            self.do_signaling_connection(command_tx);
        }

        let transition = ui
            .horizontal(|ui| {
                if ui.button("Back").clicked() {
                    if command_tx.send(RunnerCommand::Close).is_err() {
                        return Some(TransitionToView::Error {
                            message: RichText::new("RoomServer Connection task crashed"),
                        });
                    }
                    return Some(TransitionToView::ConnectionConfig);
                }

                let retry_btn = egui::Button::new("Retry")
                    .shortcut_text(ui.ctx().format_shortcut(&SUBMIT_SHORTCUT));
                if ui.add_enabled(self.has_failed(), retry_btn).clicked()
                    || ui.ctx().input_mut(|i| i.consume_shortcut(&SUBMIT_SHORTCUT))
                {
                    self.reset();
                }

                None
            })
            .inner;

        if let Some(transition) = transition {
            Some(transition)
        } else if let SetupProgress::Done(()) = self.progress.signaling_connection {
            Some(TransitionToView::Signaling)
        } else {
            None
        }
    }

    fn do_push_server_settings(
        &mut self,
        command_tx: &UnboundedSender<RunnerCommand>,
        settings: &DuiSettings,
    ) -> Result<(), RunnerGoneError> {
        match &mut self.progress.push_server_settings {
            SetupProgress::NotStarted => {
                let roomserver_url = settings.roomserver_url.clone();
                let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                command_tx.send(RunnerCommand::RoomServerAccess {
                    response_tx,
                    url: roomserver_url.clone(),
                    api_key: settings.roomserver_api_key.clone(),
                })?;
                self.progress.push_server_settings =
                    SetupProgress::Ongoing((roomserver_url, response_rx));
            }
            SetupProgress::Ongoing((url, rx)) => match rx.try_recv() {
                Ok(Ok(())) => {
                    self.progress.push_server_settings = SetupProgress::Done(url.clone());
                }
                Ok(Err(err)) => {
                    self.progress.request_token = SetupProgress::Failed(format!("{err:?}"));
                }
                Err(_) => {}
            },

            SetupProgress::Failed(_) | SetupProgress::Done(_) => {}
        }

        Ok(())
    }

    fn do_request_token(&mut self, command_tx: &UnboundedSender<RunnerCommand>) {
        match &mut self.progress.request_token {
            SetupProgress::NotStarted => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let command = RunnerCommand::RequestToken {
                    response_tx: tx,
                    room_id: self.room_id,
                    room_parameters: Box::new(Some(self.room_parameters.clone())),
                    client_parameters: self.client_parameters.clone(),
                };
                if command_tx.send(command).is_ok() {
                    self.progress.request_token = SetupProgress::Ongoing(rx);
                } else {
                    self.progress.request_token = SetupProgress::Failed(
                        "Internal Error: Failed to send command to RoomServer runner".to_string(),
                    );
                }
            }
            SetupProgress::Ongoing(rx) => match rx.try_recv() {
                Ok(Ok(res)) => {
                    self.progress.request_token = SetupProgress::Done(res);
                }
                Ok(Err(err)) => {
                    self.progress.request_token = SetupProgress::Failed(format!("{err:?}"));
                }
                Err(_) => {}
            },

            SetupProgress::Failed(_) | SetupProgress::Done(_) => {}
        }
    }

    fn do_signaling_connection(&mut self, command_tx: &UnboundedSender<RunnerCommand>) {
        let SetupProgress::Done(RoomServerAccess { token, public_url }) =
            &self.progress.request_token
        else {
            self.progress.signaling_connection =
                SetupProgress::Failed("Internal Error: invalid state".to_string());
            return;
        };

        match &mut self.progress.signaling_connection {
            SetupProgress::NotStarted => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let command = RunnerCommand::ConnectSignaling {
                    response_tx: tx,
                    token: *token,
                    url: public_url.clone(),
                };
                if command_tx.send(command).is_ok() {
                    self.progress.signaling_connection = SetupProgress::Ongoing(rx);
                } else {
                    self.progress.signaling_connection = SetupProgress::Failed(
                        "Internal Error: Failed to send command to RoomServer runner".to_string(),
                    );
                }
            }
            SetupProgress::Ongoing(rx) => match rx.try_recv() {
                Ok(Ok(())) => {
                    self.progress.signaling_connection = SetupProgress::Done(());
                }
                Ok(Err(err)) => {
                    self.progress.signaling_connection = SetupProgress::Failed(format!("{err:?}"));
                }
                Err(_) => {}
            },

            SetupProgress::Failed(_) | SetupProgress::Done(()) => {}
        }
    }

    fn has_failed(&self) -> bool {
        self.progress.request_token.is_failed() || self.progress.signaling_connection.is_failed()
    }

    fn reset(&mut self) {
        self.progress = Progress::default();
    }
}

#[derive(Debug, Clone, Default)]
enum SetupProgress<S, R> {
    #[default]
    NotStarted,
    Ongoing(S),
    Failed(String),
    Done(R),
}

impl<S, R> SetupProgress<S, R> {
    fn ui(&self, ui: &mut egui::Ui) {
        match self {
            Self::NotStarted => {
                ui.label("⌛");
            }
            Self::Ongoing(_) => {
                ui.spinner();
            }
            Self::Failed(message) => {
                ui.label("🚫");
                ui.label(RichText::new(message).color(Color32::RED));
            }
            Self::Done(_) => {
                ui.label("✅");
            }
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self, Self::Failed(_) | Self::Done(_))
    }

    /// Returns `true` if the setup progress is [`Failed`].
    ///
    /// [`Failed`]: SetupProgress::Failed
    #[must_use]
    fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(..))
    }

    /// Returns `true` if the setup progress is [`Done`].
    ///
    /// [`Done`]: SetupProgress::Done
    #[must_use]
    fn is_done(&self) -> bool {
        matches!(self, Self::Done(..))
    }
}
