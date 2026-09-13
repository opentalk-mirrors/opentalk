// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use refinery::{Report, embed_migrations};
use refinery_core::tokio_postgres::{Config, NoTls};
use snafu::Snafu;
use tokio::sync::oneshot;
use tracing::Instrument;

embed_migrations!(".");

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to connect to database: {}", source), context(false))]
    DatabaseConnect {
        source: refinery_core::tokio_postgres::Error,
    },

    #[snafu(display("Migration failed: {}", source), context(false))]
    MigrationFailed { source: refinery::Error },

    #[snafu(context(false))]
    SenderDropped {
        source: tokio::sync::oneshot::error::RecvError,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[tracing::instrument(skip(config))]
async fn migrate(config: Config) -> Result<Report> {
    tracing::debug!("config: {:?}", config);

    let (mut client, conn) = config.connect(NoTls).await?;

    let (tx, rx) = oneshot::channel();

    tokio::spawn(
        async move {
            if let Err(e) = conn.await {
                tracing::error!("connection error: {}", snafu::Report::from_error(e))
            }

            tx.send(()).expect("Channel unexpectedly dropped");
        }
        .instrument(tracing::Span::current()),
    );

    // The runner is specified through the `include_migration_mods` macro
    let report = migrations::runner().run_async(&mut client).await?;

    if !report.applied_migrations().is_empty() {
        let applied_migration_names = report
            .applied_migrations()
            .iter()
            .map(|m| m.name().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        tracing::info!("Applied migration(s): {applied_migration_names}");
    }

    drop(client);

    // wait for the connection to close
    rx.await?;

    Ok(report)
}

pub async fn migrate_from_url(url: &str) -> Result<Report> {
    let config = url.parse::<Config>()?;
    migrate(config).await
}

mod type_polyfills {
    use barrel::types::{BaseType, Type};

    /// An SQL datetime type
    ///
    /// Barrel 0.6.5 is missing datetime and 0.6.6 is not out yet, furthermore 0.6.6 only support TIMESTAMP which is without any timezone information
    pub fn datetime() -> Type {
        Type {
            nullable: false,
            unique: false,
            increments: false,
            indexed: false,
            primary: false,
            default: None,
            size: None,
            inner: BaseType::Custom("TIMESTAMPTZ"),
        }
    }
}
#[cfg(test)]
mod migration_tests {
    use super::Result;

    /// Tests the refinery database migration.
    ///
    /// A dedicated postgres [testcontainer] is started automatically, so a working docker (or compatible) environment
    /// is required. See [`opentalk_test_util::database::DatabaseContext`] for details.
    ///
    /// [testcontainer]: https://testcontainers.com/
    #[tokio::test]
    async fn test_migration() -> Result<()> {
        // This will create a database and migrate it
        opentalk_test_util::database::DatabaseContext::new().await;

        Ok(())
    }
}
