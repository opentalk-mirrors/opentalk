// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Helper binary that creates the schema.rs file
use std::path::PathBuf;

use devx_cmd::cmd;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use opentalk_database::query_helper;
use opentalk_db_storage::migrations::migrate_from_url;
use rand::RngExt;
use snafu::{Report, whatever};
use unified_diff::diff;

use crate::{Result, locate_project_root};

const SCHEMA_RS_PATH: &str = "crates/opentalk-db-storage/src/schema.rs";
const DATABASE_DEFAULT_URL: &str = "postgres://postgres:password123@localhost:5432";

pub async fn generate_db_schema(
    postgres_url: Option<url::Url>,
    database_name: Option<String>,
) -> Result<()> {
    let (drop_db, postgres_url) = connect_and_migrate(postgres_url, database_name).await?;

    log::info!("Generate schema.rs:");
    let outcome = |postgres_url| -> Result<PathBuf> {
        let diesel_output = run_diesel_cli(postgres_url)?;

        let rustfmt_output = run_rustfmt(diesel_output)?;
        write_schema(rustfmt_output)
    }(&postgres_url);

    match outcome {
        Ok(output_path) => log::info!("Generated db schema file: {} ", output_path.display()),
        Err(e) => log::error!("Failed to generate db schema: {}", Report::from_error(e)),
    };

    if drop_db {
        log::info!("Dropping temporary database");

        let (database, postgres_url) = change_database_of_url(&postgres_url, "postgres");
        let mut conn = diesel_async::AsyncPgConnection::establish(&postgres_url).await?;
        query_helper::drop_database(&database)
            .execute(&mut conn)
            .await?;
    } else {
        log::warn!("Did not drop database, as it was specified in an env var");
    }
    Ok(())
}

pub async fn verify_db_schema(
    postgres_url: Option<url::Url>,
    database_name: Option<String>,
) -> Result<()> {
    let (drop_db, postgres_url) = connect_and_migrate(postgres_url, database_name).await?;

    log::info!("Verify local schema.rs:");
    let outcome = |postgres_url| -> Result<()> {
        let diesel_output = run_diesel_cli(postgres_url)?;

        let rustfmt_output = run_rustfmt(diesel_output)?;

        let schema_rs_path =
            std::env::var("SCHEMA_RS_PATH").unwrap_or_else(|_| SCHEMA_RS_PATH.to_string());
        let schema_rs_on_disk = std::fs::read(locate_project_root().join(schema_rs_path))?;
        if rustfmt_output != schema_rs_on_disk {
            whatever!(
                "schema.rs on disk differs.\n{}",
                String::from_utf8_lossy(&diff(
                    &rustfmt_output,
                    "expected",
                    &schema_rs_on_disk,
                    "actual",
                    3
                )),
            );
        }
        Ok(())
    }(&postgres_url);

    if drop_db {
        log::info!("Dropping temporary database");

        let (database, postgres_url) = change_database_of_url(&postgres_url, "postgres");
        let mut conn = AsyncPgConnection::establish(&postgres_url).await?;
        query_helper::drop_database(&database)
            .execute(&mut conn)
            .await?;
    } else {
        log::warn!("Did not drop database, as it was specified in an env var");
    }

    outcome
}

async fn connect_and_migrate(
    postgres_url: Option<url::Url>,
    database_name: Option<String>,
) -> Result<(bool, String)> {
    log::info!("Connecting to database");
    let base_url = postgres_url
        .map(|u| u.as_str().to_string())
        .unwrap_or_else(|| DATABASE_DEFAULT_URL.to_owned());

    let random: u8 = rand::rng().random();
    let (db_name, drop_db) = database_name
        .map(|var| (var, false))
        .unwrap_or_else(|| (format!("opentalk_migration_{random}"), true));
    let postgres_url = format!("{base_url}/{db_name}");

    if AsyncPgConnection::establish(&postgres_url).await.is_err() {
        let (database, postgres_url) = change_database_of_url(&postgres_url, "postgres");
        log::info!("Creating database: {database}");
        let mut conn = AsyncPgConnection::establish(&postgres_url).await?;
        query_helper::create_database(&database)
            .execute(&mut conn)
            .await?;
    }

    log::info!("Applying migrations to database:");
    migrate_from_url(&postgres_url)
        .await
        .expect("Unable to migrate database");
    Ok((drop_db, postgres_url))
}

fn change_database_of_url(database_url: &str, default_database: &str) -> (String, String) {
    let base = ::url::Url::parse(database_url).unwrap();
    let database = base
        .path_segments()
        .unwrap()
        .next_back()
        .unwrap()
        .to_owned();
    let mut new_url = base.join(default_database).unwrap();
    new_url.set_query(base.query());
    (database, new_url.into())
}

fn run_diesel_cli(db_url: &str) -> Result<Vec<u8>> {
    let mut cmd = cmd!("diesel");
    cmd.arg("--database-url")
        .arg(db_url)
        .arg("print-schema")
        .arg("--import-types")
        .arg("crate::sql_types::*")
        .log_cmd(None)
        .log_err(log::Level::Warn);

    Ok(cmd.read_bytes()?)
}

fn run_rustfmt(stdin: Vec<u8>) -> Result<Vec<u8>> {
    let mut cmd = cmd!("rustfmt");
    cmd.stdin_bytes(stdin)
        .log_cmd(None)
        .log_err(log::Level::Warn);

    Ok(cmd.read_bytes()?)
}

fn write_schema(content: Vec<u8>) -> Result<PathBuf> {
    let target_file = locate_project_root().join(SCHEMA_RS_PATH);

    let file_exist = match std::fs::metadata(&target_file) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }?;

    if file_exist {
        log::info!("{} exists. Deleting it first.", target_file.display())
    }

    std::fs::write(&target_file, content)?;
    Ok(target_file)
}
