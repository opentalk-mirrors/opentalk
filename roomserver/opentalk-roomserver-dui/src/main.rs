// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use app::RoomServerApp;
use clap::Parser as _;
use opentalk_roomserver_crypto_provider::ensure_crypto_provider;

pub mod app;
mod cli;
pub mod client;
pub mod settings;

fn main() -> eframe::Result {
    const APP_NAME: &str = "OpenTalk RoomServer Developer UI";
    env_logger::init();

    ensure_crypto_provider();

    let args = cli::Args::parse();
    if args.run_tasks().should_exit() {
        return Ok(());
    }

    let viewport = egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]);
    let app_id = viewport.app_id.as_deref().unwrap_or(APP_NAME).to_string();
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            let app = RoomServerApp::new(cc, args.config.as_deref(), app_id)?;
            Ok(Box::new(app))
        }),
    )
}
