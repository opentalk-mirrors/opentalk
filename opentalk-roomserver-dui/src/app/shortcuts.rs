// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Key, KeyboardShortcut, Modifiers};

pub const EXIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);
pub const ERROR_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::E);
pub const SETTINGS_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Comma);
pub const DELETE_MODE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::R);

pub const BACK_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::L);

pub const SEND_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Enter);
pub const SUBMIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Enter);
pub const DISCONNECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::D);
pub const CLEAR_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::X);

pub const ENTER_ROOM_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::J);
pub const ENTER_WAITING_ROOM_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::K);

pub const PREVIOUS_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowUp);
pub const SUCCESSOR_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowDown);
pub const TOGGLE_HISTORY_PANEL_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::H);
pub const FILTER_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::F);
pub const FOCUS_MESSAGE_INPUT_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::M);

pub const TOGGLE_LIVEKIT_WINDOW_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::L);
pub const TOGGLE_BREAKOUT_WINDOW_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::B);
pub const TOGGLE_TIMER_WINDOW_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::T);
pub const TOGGLE_SPAM_AMOUNT_WINDOW_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::M);
pub const TOGGLE_WAITING_ROOM_WINDOW_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::W);
