// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context as _;
use egui::{Color32, RichText, Widget};
use opentalk_roomserver_client::api::{
    command::{LiveKitCommand, SignalingCommand, SignalingModuleCommand},
    event::{Credentials, LiveKitError, LiveKitEvent, LiveKitState, SignalingModuleEvent},
};
use opentalk_roomserver_types::{
    breakout::event::BreakoutEvent, core::CoreEvent, join::join_success::JoinSuccess,
};
use tokio::runtime::Runtime;

use crate::{
    app::{
        shortcuts::TOGGLE_LIVEKIT_WINDOW_SHORTCUT,
        signaling::{
            livekit::{handle::RunnerHandle, runner::LiveKitRunner},
            plugin::{Received, SignalingPlugin},
        },
    },
    settings::DuiSettings,
};

mod handle;
mod runner;

#[derive(Debug)]
pub struct LiveKitPlugin {
    events: Vec<String>,
    credentials: Option<Credentials>,

    handle: Option<RunnerHandle>,

    /// Automatically join breakout rooms when switching rooms.
    breakout_auto_connect: bool,
}

impl LiveKitPlugin {
    pub fn new(runtime: &Runtime, ctx: egui::Context, settings: &DuiSettings) -> Self {
        let handle = LiveKitRunner::spawn(runtime, ctx);
        Self {
            events: Vec::new(),
            credentials: None,
            handle: Some(handle),
            breakout_auto_connect: settings.livekit.breakout_auto_connect,
        }
    }

    fn handle_join_success(&mut self, join: &JoinSuccess) {
        log::debug!("Handle join event");

        match join.get_module::<LiveKitState>() {
            Ok(Some(state)) => {
                self.events.push("Got initial credential".to_string());
                self.handle_credentials(state.credentials, "Joined");
            }
            Ok(None) => {
                self.events
                    .push("No LiveKit state found in JoinSuccess".to_string());
            }
            Err(e) => {
                self.events
                    .push(format!("Failed to parse LiveKit state: {e}"));
            }
        }
    }

    fn handle_livekit_error(&mut self, e: &LiveKitError) {
        let Some(handle) = self.handle.as_ref() else {
            return;
        };

        self.events.push(format!("LiveKit Error: {e:?}"));
        let _ = handle.disconnect();
    }

    fn handle_livekit_events(&mut self) {
        let Some(handle) = self.handle.as_mut() else {
            return;
        };

        let mut received = handle.recv_event();
        while let Ok(Some(event)) = received {
            self.events.push(format!("LiveKit Event: {event:?}"));
            received = handle.recv_event();
        }

        if let Err(e) = received {
            log::error!("LiveKitRunner gone! {e:?}");
            self.events.push(format!("LiveKit Runner Error: {e:?}"));
            self.handle = None;
        }
    }

    fn connection_status_ui(&mut self, ui: &mut egui::Ui) {
        let Some(handle) = self.handle.as_ref() else {
            ui.label(RichText::new("LiveKit Runner Error").color(Color32::RED));
            return;
        };

        match &*handle.status_rx.borrow() {
            handle::Status::Disconnected => {
                ui.label(RichText::new("Disconnected").color(Color32::RED))
            }
            handle::Status::Connected { room_name } => {
                ui.label(RichText::new(format!("Connected to {room_name}")).color(Color32::GREEN))
            }
        };
    }

    fn toggle_connection_ui(
        &mut self,
        messages: &mut Vec<String>,
        ui: &mut egui::Ui,
    ) -> anyhow::Result<()> {
        let Some(handle) = self.handle.as_ref() else {
            return Ok(());
        };

        if (*handle.status_rx.borrow()).is_disconnected() {
            if let Some(credentials) = &self.credentials {
                if ui.button("connect").clicked() {
                    handle
                        .connect(credentials.clone())
                        .context("Failed to connect")?;
                }
            } else if ui.button("Request Credentials").clicked() {
                messages.push(
                    serde_json::to_string(&SignalingCommand::from(
                        SignalingModuleCommand::LiveKit(LiveKitCommand::CreateNewAccessToken),
                    ))
                    .context("LiveKit command failed to serializable")?,
                );
            }
        } else if ui.button("disconnect").clicked() {
            handle.disconnect().context("Failed to disconnect")?;
        }
        Ok(())
    }

    fn auto_connect_ui(&mut self, settings: &mut DuiSettings, ui: &mut egui::Ui) {
        if ui
            .checkbox(&mut settings.livekit.breakout_auto_connect, "auto")
            .on_hover_text("Automatically connect to new LiveKit room when switching rooms")
            .changed()
        {
            self.breakout_auto_connect = settings.livekit.breakout_auto_connect;
        }
    }

    fn handle_credentials(&mut self, credentials: Credentials, reason: &str) {
        self.events
            .push(format!("Received new credentials, {reason}"));
        self.credentials.replace(credentials.clone());
        if let Some(handle) = &self.handle
            && self.breakout_auto_connect
        {
            let res = handle.connect(credentials);
            if res.is_err() {
                self.events
                    .push("runner error, failed to send command".to_string());
            }
        }
    }

    fn handle_breakout(&mut self, event: &BreakoutEvent) {
        if let BreakoutEvent::SwitchedRoom {
            own_data: module_data,
            new_room,
            old_room,
            ..
        } = event
        {
            match module_data.get::<LiveKitState>() {
                Ok(Some(state)) => {
                    self.handle_credentials(
                        state.credentials,
                        &format!("Switched room {old_room:?} => {new_room:?}"),
                    );
                }
                Ok(None) => {
                    self.events
                        .push("Missing LiveKit credentials in SwitchRoom event".to_string());
                }
                Err(e) => {
                    self.events.push(
                        format!("Failed to parse LiveKit ModuleData in SwitchRoom event: {e}")
                            .to_string(),
                    );
                }
            }
        }
    }

    fn handle_opentalk_events(&mut self, received: &[Received]) {
        for msg in received {
            let Received::SignalingEvent(event) = msg else {
                continue;
            };
            match &event.payload {
                SignalingModuleEvent::Core(CoreEvent::JoinSuccess(join)) => {
                    self.handle_join_success(join);
                }
                SignalingModuleEvent::LiveKit(LiveKitEvent::Error(e)) => {
                    self.handle_livekit_error(e);
                }
                SignalingModuleEvent::LiveKit(LiveKitEvent::Credentials(credentials)) => {
                    self.handle_credentials(credentials.clone(), "Credentials event");
                }
                SignalingModuleEvent::Breakout(event) => {
                    self.handle_breakout(event);
                }
                _ => {}
            }
        }
    }
}

impl SignalingPlugin for LiveKitPlugin {
    fn handle_events(&mut self, _settings: &mut DuiSettings, received: &[Received]) -> Vec<String> {
        self.handle_opentalk_events(received);

        self.handle_livekit_events();

        Vec::new()
    }

    fn ui(&mut self, ui: &mut egui::Ui, settings: &mut DuiSettings) -> Vec<String> {
        let mut messages = Vec::new();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.connection_status_ui(ui);
                if let Err(e) = self.toggle_connection_ui(&mut messages, ui) {
                    log::error!("{e:?}");
                }
                self.auto_connect_ui(settings, ui);
            });

            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_width(ui.available_width())
                .show(ui, |ui| {
                    egui::Grid::new("Message Grid")
                        .striped(true)
                        .num_columns(1)
                        .show(ui, |ui| {
                            for msg in &self.events {
                                egui::Label::new(msg).wrap().ui(ui);
                                ui.end_row();
                            }
                        });
                });
        });
        messages
    }

    fn name(&self) -> &'static str {
        "LiveKit"
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_LIVEKIT_WINDOW_SHORTCUT)
    }
}
