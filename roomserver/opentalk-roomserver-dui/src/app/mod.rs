// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::Path;

use connecting::ConnectingView;
use connection_config::ConnectionConfigView;
use eframe::CreationContext;
use egui::{Button, RichText};
use error::RunnerGoneError;
use error_view::ErrorView;
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;
use settings::SettingsView;
use shortcuts::{ERROR_SHORTCUT, EXIT_SHORTCUT, SETTINGS_SHORTCUT};
use signaling::SignalingView;
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        watch,
    },
};

use crate::{
    app::about::AboutView,
    client::{RoomServerRunner, RunnerCommand, RunnerEvent, SignalingState},
    settings::DuiSettings,
};

mod about;
mod connecting;
mod connection_config;
mod error;
pub mod error_view;
pub mod event_widget;
pub mod json_edit;
mod settings;
pub mod shortcuts;
mod signaling;
pub mod style;

pub struct RoomServerApp {
    app_id: String,

    runtime: Runtime,
    settings: DuiSettings,

    event_rx: UnboundedReceiver<RunnerEvent>,
    command_tx: UnboundedSender<RunnerCommand>,
    signaling_state_rx: watch::Receiver<SignalingState>,

    view: CentralAppView,
    settings_view: Option<SettingsView>,
    about_view: Option<AboutView>,
}

impl RoomServerApp {
    pub fn new(
        cc: &CreationContext,
        roomserver_config: Option<&Path>,
        app_id: String,
    ) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let settings = crate::settings::load(cc, roomserver_config)?;

        let (event_rx, command_tx, signaling_state_rx) = RoomServerRunner::spawn(
            &runtime,
            cc.egui_ctx.clone(),
            settings.roomserver_url.clone(),
            settings.roomserver_api_key.clone(),
        )?;

        // Open the settings dialog if there where no settings provided
        let settings_view = if settings.is_default() {
            Some(SettingsView::new(&settings))
        } else {
            None
        };

        let view = CentralAppView::ConnectionConfig(Box::new(ConnectionConfigView::new(&settings)));

        Ok(Self {
            app_id,

            runtime,
            settings,

            event_rx,
            command_tx,
            signaling_state_rx,

            view,
            settings_view,
            about_view: None,
        })
    }

    fn left_panel_ui(&mut self, ui: &mut egui::Ui) -> Result<(), RunnerGoneError> {
        match &mut self.view {
            CentralAppView::Signaling(signaling_view) if signaling_view.show_side_panel() => {
                egui::Panel::left("Message Side Panel")
                    .show(ui, |ui| {
                        signaling_view.left_panel_ui(
                            ui,
                            &self.command_tx,
                            &mut self.settings.history,
                        )
                    })
                    .inner?;
            }
            _ => {}
        }

        Ok(())
    }

    fn central_panel_ui(&mut self, ui: &mut egui::Ui) {
        let request = match &mut self.view {
            CentralAppView::ConnectionConfig(view) => {
                view.center_ui(&mut self.settings, ui);
                None
            }
            CentralAppView::Connecting(view) => view.ui(ui, &self.command_tx, &self.settings),
            CentralAppView::Signaling(signaling_view) => signaling_view.center_ui(
                ui,
                &mut self.event_rx,
                &self.command_tx,
                &mut self.settings,
            ),
            CentralAppView::Error(error_view) => {
                error_view.ui(ui);
                None
            }
        };
        if let Some(request) = request {
            self.transition_to_view(request, ui.ctx());
        }
    }

    fn menu_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);

                ui.menu_button("File", |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    let settings_btn = Button::new("Settings")
                        .shortcut_text(ui.format_shortcut(&SETTINGS_SHORTCUT));
                    if ui.add(settings_btn).clicked() && self.settings_view.is_none() {
                        self.settings_view = Some(SettingsView::new(&self.settings));
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    egui::gui_zoom::zoom_menu_buttons(ui);
                    ui.weak(format!(
                        "Current zoom: {:.0}%",
                        100.0 * ui.ctx().zoom_factor()
                    ))
                    .on_hover_text(
                        "The UI zoom level, on top of the operating system's default value",
                    );
                    ui.separator();
                });

                let request = match &mut self.view {
                    CentralAppView::Signaling(signaling_view) => signaling_view
                        .menu_ui(
                            ui,
                            &self.command_tx,
                            self.signaling_state_rx.borrow().to_owned(),
                            &self.settings,
                        )
                        .expect("Fatal Error"),
                    CentralAppView::ConnectionConfig(..) => {
                        ConnectionConfigView::menu_ui(&mut self.settings, ui);
                        None
                    }
                    _ => None,
                };

                ui.menu_button("Help", |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    let about_btn = Button::new("About");
                    if ui.add(about_btn).clicked() && self.about_view.is_none() {
                        self.about_view = Some(AboutView::new(self.app_id.clone()));
                    }
                });

                if let Some(request) = request {
                    self.transition_to_view(request, ui);
                }
            })
        });
    }

    fn transition_to_view(&mut self, request: TransitionToView, ctx: &egui::Context) {
        match request {
            TransitionToView::ConnectionConfig => {
                self.view = CentralAppView::ConnectionConfig(Box::new(ConnectionConfigView::new(
                    &self.settings,
                )));
            }
            TransitionToView::Connecting {
                room_id,
                client_parameters,
                room_parameters,
            } => {
                self.view = CentralAppView::Connecting(Box::new(ConnectingView::new(
                    room_id,
                    *client_parameters,
                    *room_parameters,
                )));
            }
            TransitionToView::Signaling => {
                self.view = CentralAppView::Signaling(SignalingView::new(
                    &self.runtime,
                    ctx.clone(),
                    &self.settings,
                ));
            }
            TransitionToView::Error { message } => {
                self.view = CentralAppView::Error(ErrorView::new(message));
            }
        }
    }

    fn bottom_panel_ui(&mut self, ui: &mut egui::Ui) {
        match &mut self.view {
            CentralAppView::Signaling(signaling_view) => {
                signaling_view
                    .bottom_ui(
                        ui,
                        &self.command_tx,
                        self.signaling_state_rx.borrow().to_owned(),
                        &mut self.settings.history,
                    )
                    .expect("Fatal error");
            }
            CentralAppView::ConnectionConfig(view) => {
                let req = view.ui_bottom(ui, &mut self.settings);
                if let Some(transition) = req {
                    self.transition_to_view(transition, ui.ctx());
                }
            }
            _ => (),
        }
    }
}
impl eframe::App for RoomServerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.menu_ui(ui);
        egui::Panel::bottom("bottom-view")
            .resizable(true)
            .default_size(102.)
            .show(ui, |ui| {
                self.bottom_panel_ui(ui);
            });
        if let Err(RunnerGoneError) = self.left_panel_ui(ui) {
            self.transition_to_view(
                TransitionToView::Error {
                    message: RichText::new("Runner is gone"),
                },
                ui,
            );
        }
        egui::CentralPanel::default().show(ui, |ui| {
            self.central_panel_ui(ui);
        });
        if let Some(settings_view) = &mut self.settings_view {
            let modal = egui::Modal::new(egui::Id::new("Settings")).show(ui, |ui| {
                settings_view.ui(ui, &mut self.settings);
            });
            if modal.should_close() {
                self.settings_view.take();
            }
        }
        if let Some(view) = &mut self.about_view {
            let modal = egui::Modal::new(egui::Id::new("About")).show(ui, |ui| {
                view.ui(ui);
            });
            if modal.should_close() {
                self.about_view.take();
            }
        }
        if ui.input_mut(|i| i.consume_shortcut(&EXIT_SHORTCUT)) {
            let _ = self.command_tx.send(RunnerCommand::Close);
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ui.input_mut(|i| i.consume_shortcut(&SETTINGS_SHORTCUT)) && self.settings_view.is_none()
        {
            self.settings_view = Some(SettingsView::new(&self.settings));
        }
        if ui.input_mut(|i| i.consume_shortcut(&ERROR_SHORTCUT)) {
            self.transition_to_view(
                TransitionToView::Error {
                    message: RichText::new("This is a test"),
                },
                ui,
            );
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        crate::settings::save(&self.settings, storage);
    }
}

#[derive(Debug)]
pub enum CentralAppView {
    ConnectionConfig(Box<ConnectionConfigView>),
    Connecting(Box<ConnectingView>),
    Signaling(SignalingView),
    Error(ErrorView),
}

#[derive(Debug, Clone)]
pub enum TransitionToView {
    ConnectionConfig,
    Connecting {
        room_id: RoomId,
        client_parameters: Box<ClientParameters>,
        room_parameters: Box<RoomParameters>,
    },
    Signaling,
    Error {
        message: RichText,
    },
}
