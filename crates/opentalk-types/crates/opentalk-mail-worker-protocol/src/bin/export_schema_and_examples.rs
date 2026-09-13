// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use opentalk_mail_worker_protocol::{
    MailTask,
    v1::{
        ExternalEventCancellation, ExternalEventInvite, ExternalEventUninvite, ExternalEventUpdate,
        Message, RegisteredEventCancellation, RegisteredEventInvite, RegisteredEventUninvite,
        RegisteredEventUpdate, UnregisteredEventCancellation, UnregisteredEventInvite,
        UnregisteredEventUninvite, UnregisteredEventUpdate,
    },
};
use opentalk_types_common::utils::ExampleData;
use serde::Serialize;
use utoipa::{OpenApi as _, openapi::Components};

fn export_json_file<T>(base_path: &Option<PathBuf>, stem: &str, data: &T) -> Result<(), String>
where
    T: Serialize,
{
    let file_name = format!("{stem}.json");

    let file_path = if let Some(base_path) = base_path {
        std::fs::create_dir_all(base_path)
            .map_err(|e| format!("Couldn't create directory {base_path:?}: {e}"))?;

        base_path.join(&file_name)
    } else {
        Path::new(&file_name).to_path_buf()
    };

    print!("writing example file {file_path:?}…");
    let mut file = File::create(&file_path).map_err(|e| {
        println!(" failed!");
        format!("Couldn't create file {file_path:?}: {e:?}")
    })?;

    serde_json::to_writer_pretty(&mut file, data).map_err(|e| {
        println!(" failed!");
        format!("Couldn't serialize JSON to file {file_path:?}: {e:?}")
    })?;

    println!(" done.");

    Ok(())
}

fn main() -> Result<(), String> {
    #[derive(utoipa::OpenApi)]
    #[openapi(components(schemas(
        MailTask,
        Message,
        RegisteredEventInvite,
        UnregisteredEventInvite,
        ExternalEventInvite,
        RegisteredEventUpdate,
        UnregisteredEventUpdate,
        ExternalEventUpdate,
        RegisteredEventCancellation,
        UnregisteredEventCancellation,
        ExternalEventCancellation,
        RegisteredEventUninvite,
        UnregisteredEventUninvite,
        ExternalEventUninvite,
    )))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    let components = openapi.components.unwrap();

    #[derive(serde::Serialize)]
    pub struct SchemaExport {
        pub components: Components,
    }

    let base_path = std::env::var("MAIL_WORKER_PROTOCOL_EXAMPLE_OUTPUT_DIR")
        .map(|v| Path::new(&v).to_path_buf())
        .ok();

    export_json_file(&base_path, "schema", &SchemaExport { components })?;

    export_json_file(
        &base_path,
        "registered_event_invite",
        &MailTask::V1(RegisteredEventInvite::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "unregistered_event_invite",
        &MailTask::V1(UnregisteredEventInvite::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "external_event_invite",
        &MailTask::V1(ExternalEventInvite::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "registered_event_update",
        &MailTask::V1(RegisteredEventUpdate::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "unregistered_event_update",
        &MailTask::V1(UnregisteredEventUpdate::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "external_event_update",
        &MailTask::V1(ExternalEventUpdate::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "registered_event_cancellation",
        &MailTask::V1(RegisteredEventCancellation::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "unregistered_event_cancellation",
        &MailTask::V1(UnregisteredEventCancellation::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "external_event_cancellation",
        &MailTask::V1(ExternalEventCancellation::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "registered_event_uninvite",
        &MailTask::V1(RegisteredEventUninvite::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "unregistered_event_uninvite",
        &MailTask::V1(UnregisteredEventUninvite::example_data().into()),
    )?;
    export_json_file(
        &base_path,
        "external_event_uninvite",
        &MailTask::V1(ExternalEventUninvite::example_data().into()),
    )?;

    Ok(())
}
