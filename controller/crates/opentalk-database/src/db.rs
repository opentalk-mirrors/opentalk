// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use deadpool_runtime::Runtime;
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use snafu::Report;

use crate::{
    DatabaseError, DbConnection,
    metrics::{DatabaseMetrics, MetricsConnection},
};

type DbPool = Pool<AsyncPgConnection>;

/// Db container that uses a connection pool to hand out connections
///
/// Uses an deadpool connection pool to manage multiple established connections.
pub struct Db {
    metrics: Option<Arc<DatabaseMetrics>>,
    pool: DbPool,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Db")
    }
}

impl Db {
    /// Creates a new Db instance from the specified database settings.
    #[tracing::instrument(skip(db_settings))]
    pub fn connect(db_settings: &opentalk_controller_settings::Database) -> crate::Result<Self> {
        Self::connect_url(&db_settings.url, db_settings.max_connections)
    }

    /// Creates a new Db instance from the specified database url.
    pub fn connect_url(db_url: &str, max_conns: u32) -> crate::Result<Self> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);

        let pool = Pool::builder(manager)
            .max_size(max_conns as usize)
            .create_timeout(Some(Duration::from_secs(10)))
            .runtime(Runtime::Tokio1)
            .build()?;

        Ok(Self {
            metrics: None,
            pool,
        })
    }

    /// Set the metrics to use for this database pool
    pub fn set_metrics(&mut self, metrics: Arc<DatabaseMetrics>) {
        self.metrics = Some(metrics);
    }

    /// Returns an established connection from the connection pool
    #[tracing::instrument(skip_all)]
    pub async fn get_conn(&self) -> crate::Result<DbConnection> {
        let res = self.pool.get().await;
        let state = self.pool.status();

        if let Some(metrics) = &self.metrics {
            metrics.dbpool_connections.record(state.size as u64, &[]);

            metrics
                .dbpool_connections_idle
                .record(u64::try_from(state.available).unwrap_or_default(), &[]);
        }

        match res {
            Ok(conn) => {
                let conn = MetricsConnection {
                    metrics: self.metrics.clone(),
                    conn,
                    instrumentation: Arc::new(std::sync::Mutex::new(
                        diesel::connection::get_default_instrumentation(),
                    )),
                };

                Ok(conn)
            }
            Err(e) => {
                let state = self.pool.status();
                tracing::error!(
                    "Unable to get connection from connection pool.
                                Error: {}
                                Pool State:
                                    {state:?}",
                    Report::from_error(&e)
                );
                Err(DatabaseError::DeadpoolError { source: e })
            }
        }
    }
}
