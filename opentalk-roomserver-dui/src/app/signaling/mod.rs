// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{InnerResponse, Response, RichText, TextEdit, Widget as _, style::ScrollAnimation};
use opentalk_roomserver_client::api::command::{SignalingCommand, SignalingModuleCommand};
use opentalk_roomserver_types::core::CoreCommand;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError},
};

use super::shortcuts::{
    FILTER_SHORTCUT, FOCUS_MESSAGE_INPUT_SHORTCUT, PREVIOUS_SHORTCUT, SUCCESSOR_SHORTCUT,
    TOGGLE_HISTORY_PANEL_SHORTCUT,
};
use crate::{
    app::{
        TransitionToView,
        error::RunnerGoneError,
        event_widget::EventWidget,
        json_edit::json_editor,
        shortcuts::{
            CLEAR_SHORTCUT, DISCONNECT_SHORTCUT, ENTER_ROOM_SHORTCUT, ENTER_WAITING_ROOM_SHORTCUT,
            SEND_SHORTCUT,
        },
        signaling::{
            breakout::BreakoutPlugin,
            filtered_vec::FilteredVec,
            livekit::LiveKitPlugin,
            moderator_tools::ModeratorToolsPlugin,
            plugin::{Received, SignalingPlugin},
            spam_amount::SpamAmountPlugin,
            timer::TimerPlugin,
            waiting_room::WaitingRoomPlugin,
        },
        style::{delete_btn, delete_mode_btn},
    },
    client::{RunnerCommand, RunnerEvent, RunnerEventType, SignalingState},
    settings::{DuiSettings, HistoryEntry, MessageHistory},
};

mod breakout;
pub mod filtered_vec;
mod livekit;
mod moderator_tools;
mod plugin;
pub mod spam_amount;
mod timer;
mod waiting_room;

#[derive(Debug)]
pub struct HistorySelectState {
    /// The content of the message edit field before entering history-select-state
    unsent_message: String,
    history_index: usize,
}

#[derive(Debug)]
pub struct SignalingView {
    messages: FilteredVec<EventWidget>,
    show_plain_messages: bool,

    edit_message: String,

    /// Indicates if we are in history-select-state.
    ///
    /// While in this state, we search previously sent messages. This state is automatically exited
    /// when the currently displayed message is edited or when a message is send.
    ///
    /// In case that a message is send while in history-select-state, the selected message is moved
    /// to the front of the history.
    historic_message_state: Option<HistorySelectState>,

    /// Force focus to the message input.
    force_focus: bool,

    show_history_panel: bool,

    delete_mode: bool,

    receive_suspended: bool,

    plugins: Vec<(bool, Box<dyn SignalingPlugin>)>,
}

impl SignalingView {
    pub fn new(runtime: &Runtime, ctx: egui::Context, settings: &DuiSettings) -> Self {
        Self {
            messages: FilteredVec::new(),
            show_plain_messages: false,
            edit_message: String::new(),
            historic_message_state: None,
            show_history_panel: true,
            force_focus: true,
            delete_mode: false,
            receive_suspended: false,
            plugins: vec![
                (false, Box::new(LiveKitPlugin::new(runtime, ctx, settings))),
                (false, Box::new(BreakoutPlugin::new())),
                (false, Box::new(TimerPlugin::new())),
                (false, Box::new(SpamAmountPlugin::new())),
                (false, Box::new(WaitingRoomPlugin::new())),
                (false, Box::new(ModeratorToolsPlugin::new())),
            ],
        }
    }
    pub fn menu_ui(
        &mut self,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        signaling_state: SignalingState,
        settings: &DuiSettings,
    ) -> Result<Option<TransitionToView>, RunnerGoneError> {
        ui.menu_button("Message", |ui| {
            let btn_clear =
                egui::Button::new("Clear").shortcut_text(ui.format_shortcut(&CLEAR_SHORTCUT));
            if ui.add(btn_clear).clicked() {
                self.clear_messages(ui);
            }

            let btn_previous =
                egui::Button::new("Previous").shortcut_text(ui.format_shortcut(&PREVIOUS_SHORTCUT));
            if ui.add(btn_previous).clicked() {
                self.load_previous_message(&settings.history);
            }

            let btn_successor =
                egui::Button::new("Successor").shortcut_text(ui.format_shortcut(&CLEAR_SHORTCUT));
            if ui
                .add_enabled(self.historic_message_state.is_some(), btn_successor)
                .clicked()
            {
                self.load_successor_message(&settings.history);
            }

            let btn_plain_toggle_txt = if self.show_plain_messages {
                "Parse Messages"
            } else {
                "Plain Messages"
            };
            let btn_plain_toggle = egui::Button::new(btn_plain_toggle_txt);
            if ui.add(btn_plain_toggle).clicked() {
                self.show_plain_messages = !self.show_plain_messages;
            }

            let btn_receive_toggle_txt = if self.receive_suspended {
                "Resume receive"
            } else {
                "Suspend receive"
            };
            let btn_receive_toggle = egui::Button::new(btn_receive_toggle_txt);
            if ui.add(btn_receive_toggle).clicked() {
                self.receive_suspended = !self.receive_suspended;
                if self.receive_suspended {
                    command_tx.send(RunnerCommand::SuspendReceive)?;
                } else {
                    command_tx.send(RunnerCommand::ResumeReceive)?;
                }
            }

            delete_mode_btn(ui, &mut self.delete_mode);

            Ok::<(), RunnerGoneError>(())
        })
        .inner
        .transpose()?;

        match signaling_state {
            SignalingState::Connected => {
                let button = egui::Button::new("Disconnect")
                    .shortcut_text(ui.format_shortcut(&DISCONNECT_SHORTCUT));
                if ui.add(button).clicked()
                    || ui.input_mut(|i| i.consume_shortcut(&DISCONNECT_SHORTCUT))
                {
                    command_tx.send(RunnerCommand::Close)?;
                }

                // Quick access to the lobby join flow, so the conference can be entered without
                // hand-crafting the core commands.
                let enter_room_btn = egui::Button::new("Enter Room")
                    .shortcut_text(ui.format_shortcut(&ENTER_ROOM_SHORTCUT));
                if ui.add(enter_room_btn).clicked()
                    || ui.input_mut(|i| i.consume_shortcut(&ENTER_ROOM_SHORTCUT))
                {
                    Self::send_core_command(
                        command_tx,
                        CoreCommand::EnterRoom { display_name: None },
                    )?;
                }

                let enter_waiting_room_btn = egui::Button::new("Enter Waiting Room")
                    .shortcut_text(ui.format_shortcut(&ENTER_WAITING_ROOM_SHORTCUT));
                if ui.add(enter_waiting_room_btn).clicked()
                    || ui.input_mut(|i| i.consume_shortcut(&ENTER_WAITING_ROOM_SHORTCUT))
                {
                    Self::send_core_command(
                        command_tx,
                        CoreCommand::EnterWaitingRoom { display_name: None },
                    )?;
                }
            }
            SignalingState::Disconnect => {
                let button = egui::Button::new("Connect")
                    .shortcut_text(ui.format_shortcut(&DISCONNECT_SHORTCUT));
                if ui.add(button).clicked() {
                    return Ok(Some(TransitionToView::ConnectionConfig));
                }
                if ui.input_mut(|i| i.consume_shortcut(&DISCONNECT_SHORTCUT)) {
                    return Ok(Some(TransitionToView::ConnectionConfig));
                }
            }
        }

        let show_history_txt = if self.show_history_panel {
            "Hide History Panel"
        } else {
            "Show History Panel"
        };
        let btn_show_history = egui::Button::new(show_history_txt)
            .shortcut_text(ui.format_shortcut(&TOGGLE_HISTORY_PANEL_SHORTCUT));
        if ui.add(btn_show_history).clicked()
            || ui.input_mut(|i| i.consume_shortcut(&TOGGLE_HISTORY_PANEL_SHORTCUT))
        {
            self.show_history_panel = !self.show_history_panel;
        }

        ui.menu_button("Plugins", |ui| {
            for (open, plugin) in &mut self.plugins {
                let show_plugin_txt = if *open {
                    format!("Hide {} Window", plugin.name())
                } else {
                    format!("Show {} Window", plugin.name())
                };
                let mut btn_show_plugin = egui::Button::new(show_plugin_txt);
                if let Some(shortcut) = plugin.shortcut() {
                    btn_show_plugin = btn_show_plugin.shortcut_text(ui.format_shortcut(shortcut));
                }
                if ui.add(btn_show_plugin).clicked() {
                    *open = !*open;
                }
            }
        });

        // we cannot handle the shortcuts inside the menu button ui since this is not executed if
        // the menu is closed
        for (open, plugin) in &mut self.plugins {
            if plugin
                .shortcut()
                .is_some_and(|shortcut| ui.input_mut(|i| i.consume_shortcut(shortcut)))
            {
                *open = !*open;
            }
        }

        delete_mode_btn(ui, &mut self.delete_mode);

        if ui.input_mut(|i| i.consume_shortcut(&CLEAR_SHORTCUT)) {
            self.clear_messages(ui);
        }

        if ui.input_mut(|i| i.consume_shortcut(&PREVIOUS_SHORTCUT)) {
            self.load_previous_message(&settings.history);
        }
        if ui.input_mut(|i| i.consume_shortcut(&SUCCESSOR_SHORTCUT)) {
            self.load_successor_message(&settings.history);
        }
        if ui.input_mut(|i| i.consume_shortcut(&FOCUS_MESSAGE_INPUT_SHORTCUT)) {
            self.force_focus = true;
            log::trace!("request repaint: change focus to message input");
            ui.request_repaint();
        }

        Ok(None)
    }

    pub fn center_ui(
        &mut self,
        ui: &mut egui::Ui,
        event_rx: &mut UnboundedReceiver<RunnerEvent>,
        command_tx: &UnboundedSender<RunnerCommand>,
        settings: &mut DuiSettings,
    ) -> Option<TransitionToView> {
        // receive events from RoomServerRunner and forward them to the plugins.
        let res = self.receive_runner_events(event_rx).and_then(|received| {
            self.plugin_ui(ui, command_tx, settings, &received)?;
            Ok(())
        });
        if let Err(_e) = res {
            return Some(TransitionToView::Error {
                message: RichText::new("Failed to get new messages from RoomServer task."),
            });
        }

        ui.vertical(|ui| {
            self.filter_message_ui(ui);

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    egui::Grid::new("Message Grid")
                        .striped(true)
                        .num_columns(2)
                        .show(ui, |ui| {
                            for msg in &mut self.messages.iter_mut().filter(|i| i.visible()) {
                                msg.inner.control_ui(ui);
                                msg.inner.content_ui(
                                    ui,
                                    &settings.event_widget_layout,
                                    self.show_plain_messages,
                                );
                                ui.end_row();
                            }
                        });
                });
        });
        None
    }

    fn plugin_ui(
        &mut self,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        settings: &mut DuiSettings,
        received: &[Received],
    ) -> Result<(), RunnerGoneError> {
        for (open, plugin) in &mut self.plugins {
            for message in plugin.handle_events(settings, received) {
                command_tx.send(RunnerCommand::Send { message })?;
            }

            let res = egui::Window::new(plugin.name())
                .open(open)
                .show(ui.ctx(), |ui| plugin.ui(ui, settings));
            if let Some(InnerResponse {
                inner: Some(messages),
                ..
            }) = res
            {
                for message in messages {
                    command_tx.send(RunnerCommand::Send { message })?;
                }
            }
        }
        Ok(())
    }

    fn filter_message_ui(&mut self, ui: &mut egui::Ui) {
        let filter_edit_res = TextEdit::singleline(self.messages.filter_string())
            .hint_text("Message Filter")
            .desired_width(f32::INFINITY)
            .ui(ui);
        if filter_edit_res.changed() {
            self.messages.update();
        }
        if ui.ctx().input_mut(|i| i.consume_shortcut(&FILTER_SHORTCUT)) {
            ui.memory_mut(|memory| memory.request_focus(filter_edit_res.id));
        }
    }

    pub fn bottom_ui(
        &mut self,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        signaling_state: SignalingState,
        history: &mut MessageHistory,
    ) -> Result<(), RunnerGoneError> {
        let button =
            egui::Button::new("Send").shortcut_text(ui.ctx().format_shortcut(&SEND_SHORTCUT));
        let button_res = ui.add_enabled(signaling_state.is_connected(), button);

        let res = json_editor(ui, &mut self.edit_message);

        if self.force_focus {
            res.request_focus();
            self.force_focus = false;
        }

        // The message that was edited before searching the history will be lost.
        if self.historic_message_state.is_some() && res.changed() {
            self.historic_message_state.take();
        }

        if button_res.clicked() || ui.input_mut(|i| i.consume_shortcut(&SEND_SHORTCUT)) {
            self.record_sent_message(history);
            self.send_websocket_message(command_tx)
        } else {
            Ok(())
        }
    }

    pub fn left_panel_ui(
        &mut self,
        ui: &mut egui::Ui,
        command_tx: &UnboundedSender<RunnerCommand>,
        history: &mut MessageHistory,
    ) -> Result<(), RunnerGoneError> {
        ui.heading("Message History");
        let mut delete_entry = None;
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                egui::Grid::new("Message Grid")
                    .striped(true)
                    .num_columns(3)
                    .min_col_width(15.)
                    .show(ui, |ui| {
                        for (index, msg) in history.iter().enumerate().rev() {
                            if ui.button("send").clicked() {
                                // we don't want to mess with the "history-select-state" or the
                                // message input. Just send it!
                                let message = msg.text().to_string();
                                command_tx.send(RunnerCommand::Send { message })?;
                            }
                            if delete_btn(ui, self.delete_mode).clicked() {
                                delete_entry.replace(index);
                            }
                            let res = ui.vertical(|ui| self.history_item_ui(index, msg, ui)).inner;
                            if res.clicked() {
                                self.set_historic_state(history, index);
                            }
                            ui.end_row();
                        }

                        if let Some(index) = delete_entry {
                            history.remove(index);
                            if let Some(state) = self.historic_message_state.as_mut()
                                && state.history_index > index
                            {
                                state.history_index -= 1;
                            }
                        }
                        Ok(())
                    })
                    .inner
            })
            .inner
    }

    fn history_item_ui(&mut self, index: usize, msg: &HistoryEntry, ui: &mut egui::Ui) -> Response {
        let text = RichText::new(msg.text());

        let highlight = self
            .historic_message_state
            .as_ref()
            .is_some_and(|state| state.history_index == index);
        let mut res = ui.label(text);
        if highlight || res.hovered() {
            res = res.highlight();
        }
        if highlight {
            res.scroll_to_me_animation(None, ScrollAnimation::duration(0.1));
        }
        res
    }

    fn clear_messages(&mut self, ui: &egui::Ui) {
        self.messages.clear();
        self.historic_message_state.take();
        log::trace!("request repaint: clear messages");
        ui.request_repaint();
    }

    fn receive_runner_events(
        &mut self,
        event_rx: &mut UnboundedReceiver<RunnerEvent>,
    ) -> Result<Vec<Received>, RunnerGoneError> {
        let mut received = Vec::new();
        loop {
            match event_rx.try_recv() {
                Ok(msg) => {
                    let mut known_type = false;
                    if let RunnerEventType::Received { message } = &msg.event_type {
                        let recv: Received = message.clone().into();
                        known_type = !recv.is_invalid();
                        received.push(recv);
                    }
                    let mut event_widget = EventWidget::from(msg);
                    event_widget.set_type_known(known_type);
                    self.messages.push(event_widget);
                }
                Err(TryRecvError::Empty) => return Ok(received),
                Err(TryRecvError::Disconnected) => {
                    log::error!("Failed to receive runner event, channel closed.");
                    return Err(RunnerGoneError);
                }
            }
        }
    }

    fn send_websocket_message(
        &mut self,
        command_tx: &UnboundedSender<RunnerCommand>,
    ) -> Result<(), RunnerGoneError> {
        let mut message = String::new();
        std::mem::swap(&mut message, &mut self.edit_message);
        command_tx.send(RunnerCommand::Send { message })?;

        Ok(())
    }

    /// Serialize a [`CoreCommand`] and send it over the signaling connection.
    fn send_core_command(
        command_tx: &UnboundedSender<RunnerCommand>,
        command: CoreCommand,
    ) -> Result<(), RunnerGoneError> {
        let message = match serde_json::to_string(&SignalingCommand::from(
            SignalingModuleCommand::Core(command),
        )) {
            Ok(message) => message,
            Err(error) => {
                log::error!("Failed to serialize core command: {error}");
                return Ok(());
            }
        };
        command_tx.send(RunnerCommand::Send { message })?;

        Ok(())
    }

    /// Record the current [`Self::edit_message`] in history and leave the history-select-state
    /// if necessary.
    fn record_sent_message(&mut self, history: &mut MessageHistory) {
        if let Some(historic) = self.historic_message_state.take() {
            history.move_front(historic.history_index);
        } else {
            history.push(self.edit_message.clone());
        }
    }

    pub fn show_side_panel(&self) -> bool {
        self.show_history_panel
    }

    fn load_previous_message(&mut self, history: &MessageHistory) {
        let new_previous_index = self
            .historic_message_state
            .as_ref()
            .map_or(0, |history_state| history_state.history_index + 1);

        self.set_historic_state(history, new_previous_index);
    }

    fn load_successor_message(&mut self, history: &MessageHistory) {
        // subtract 1 from the history-message-index. If we go below 0 or we
        // are not in history-select-state, this is none.
        let successor_index = self
            .historic_message_state
            .as_ref()
            .and_then(|history_state| history_state.history_index.checked_sub(1));

        if let Some(successor_index) = successor_index {
            self.set_historic_state(history, successor_index);
        } else {
            self.restore_edit_state();
        }
    }

    fn set_historic_state(&mut self, history: &MessageHistory, history_index: usize) {
        let Some(new_historic) = history.get(history_index) else {
            return;
        };

        let history_state = self
            .historic_message_state
            .get_or_insert_with(|| HistorySelectState {
                unsent_message: self.edit_message.clone(),
                history_index: 0,
            });

        history_state.history_index = history_index;
        self.edit_message = new_historic.text().to_string();
    }

    // leave history-select-state and restore the message that was edited before entering the state
    fn restore_edit_state(&mut self) {
        if let Some(historic_state) = self.historic_message_state.take() {
            self.edit_message = historic_state.unsent_message;
        }
    }
}
