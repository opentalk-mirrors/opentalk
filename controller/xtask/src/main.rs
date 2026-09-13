// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use clap::Parser;
use snafu::Snafu;

mod db_schema;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    DatabaseConnection {
        source: diesel::result::ConnectionError,
    },

    #[snafu(context(false))]
    DatabaseOperation { source: diesel::result::Error },

    #[snafu(display("Failed to read file: {}", source), context(false))]
    FileRead { source: std::io::Error },

    #[snafu(display("Failed to execute command: {}", source), context(false))]
    CommandExecution { source: devx_cmd::Error },

    #[snafu(display("Custom error: {message}"), whatever)]
    CustomError {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error >, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "This binary defines auxiliary ad-hoc scripts."
)]
enum XTasks {
    /// Create the diesel DB schema file
    GenerateDbSchema {
        #[arg(long, env = "POSTGRES_URL")]
        postgres_url: Option<url::Url>,
        #[arg(long, env = "DATABASE_NAME")]
        database_name: Option<String>,
    },
    /// Runs the db-storage crates migrations and verifies if the present schema.rs is correct.
    VerifyDbSchema {
        #[arg(long, env = "POSTGRES_URL")]
        postgres_url: Option<url::Url>,
        #[arg(long, env = "DATABASE_NAME")]
        database_name: Option<String>,
    },
}

#[tokio::main]
#[snafu::report]
async fn main() -> Result<()> {
    let mut builder = env_logger::Builder::new();
    builder
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .parse_default_env();
    builder.init();

    let opt = XTasks::parse();
    match opt {
        XTasks::GenerateDbSchema {
            postgres_url,
            database_name,
        } => db_schema::generate_db_schema(postgres_url, database_name).await?,
        XTasks::VerifyDbSchema {
            postgres_url,
            database_name,
        } => db_schema::verify_db_schema(postgres_url, database_name).await?,
    };

    Ok(())
}

/// Searches for a project root dir, which is a directory that contains a
/// `Cargo.toml` file that defines the project's [cargo workspace][cargo-workspace]).
///
/// It uses the value of [`cargo metadata`][cargo-metadata] `workspace_root`.
///
/// [cargo-metadata]: https://doc.rust-lang.org/cargo/commands/cargo-metadata.html
/// [cargo-workspace]: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
pub fn locate_project_root() -> PathBuf {
    let cmd = cargo_metadata::MetadataCommand::new();

    let metadata = cmd.exec().unwrap();
    let workspace_root = metadata.workspace_root;

    workspace_root.into()
}
