// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::{
    future::Future,
    pin::Pin,
    task::{Poll, ready},
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use diesel::{
    connection::{CacheSize, Instrumentation, InstrumentationEvent},
    query_builder::{AsQuery, QueryFragment, QueryId},
    result::{ConnectionResult, QueryResult},
};
use diesel_async::{
    AnsiTransactionManager, AsyncConnection, AsyncConnectionCore, AsyncPgConnection,
    SimpleAsyncConnection, TransactionManager, pooled_connection::deadpool::Object,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use opentelemetry::{
    Key, KeyValue,
    metrics::{Counter, Histogram, Meter},
};
use opentelemetry_sdk::metrics::{
    Aggregation, Instrument as OtlInstrument, MeterProviderBuilder, Stream,
};

type Parent = Object<AsyncPgConnection>;

const ERROR_KEY: Key = Key::from_static_str("error");
const EXEC_TIME: &str = "sql.execution_time_seconds";
const POOL_CONNECTIONS: &str = "sql.dbpool_connections";
const POOL_CONNECTIONS_IDLE: &str = "sql.dbpool_connections_idle";
const ERRORS_TOTAL: &str = "sql.errors_total";

#[derive(Debug)]
pub struct DatabaseMetrics {
    pub sql_execution_time: Histogram<f64>,
    pub sql_error: Counter<u64>,
    pub dbpool_connections: Histogram<u64>,
    pub dbpool_connections_idle: Histogram<u64>,
}

impl DatabaseMetrics {
    pub fn append_views(provider_builder: MeterProviderBuilder) -> MeterProviderBuilder {
        provider_builder
            .with_view(|instrument: &OtlInstrument| {
                if instrument.name() == EXEC_TIME {
                    Stream::builder()
                        .with_aggregation(Aggregation::ExplicitBucketHistogram {
                            boundaries: vec![0.01, 0.05, 0.1, 0.25, 0.5],
                            record_min_max: false,
                        })
                        .build()
                        .ok()
                } else {
                    None
                }
            })
            .with_view(|instrument: &OtlInstrument| {
                if instrument.name() == POOL_CONNECTIONS {
                    Stream::builder()
                        .with_aggregation(Aggregation::Default)
                        .build()
                        .ok()
                } else {
                    None
                }
            })
            .with_view(|instrument: &OtlInstrument| {
                if instrument.name() == POOL_CONNECTIONS_IDLE {
                    Stream::builder()
                        .with_aggregation(Aggregation::Default)
                        .build()
                        .ok()
                } else {
                    None
                }
            })
    }

    pub fn new(meter: &Meter) -> Self {
        Self {
            sql_execution_time: meter
                .f64_histogram(EXEC_TIME)
                .with_description("SQL execution time for a single diesel query")
                .with_unit("seconds")
                .build(),
            sql_error: meter
                .u64_counter(ERRORS_TOTAL)
                .with_description("Counter for total SQL query errors")
                .build(),
            dbpool_connections: meter
                .u64_histogram(POOL_CONNECTIONS)
                .with_description("Number of currently non-idling db connections")
                .build(),
            dbpool_connections_idle: meter
                .u64_histogram(POOL_CONNECTIONS_IDLE)
                .with_description("Number of currently idling db connections")
                .build(),
        }
    }
}

pub struct MetricsConnection<Conn> {
    pub(crate) metrics: Option<Arc<DatabaseMetrics>>,
    pub(crate) conn: Conn,
    pub(crate) instrumentation: Arc<Mutex<Option<Box<dyn Instrumentation>>>>,
}

impl<Conn> std::fmt::Debug for MetricsConnection<Conn> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsConnection")
            .field("metrics", &())
            .field("conn", &())
            .field("instrumentation", &())
            .finish()
    }
}

fn get_metrics_label_for_error(error: &diesel::result::Error) -> &'static str {
    match error {
        diesel::result::Error::InvalidCString(_) => "invalid_c_string",
        diesel::result::Error::DatabaseError(e, _) => match e {
            diesel::result::DatabaseErrorKind::UniqueViolation => "unique_violation",
            diesel::result::DatabaseErrorKind::ForeignKeyViolation => "foreign_key_violation",
            diesel::result::DatabaseErrorKind::UnableToSendCommand => "unable_to_send_command",
            diesel::result::DatabaseErrorKind::SerializationFailure => "serialization_failure",
            _ => "unknown",
        },
        diesel::result::Error::NotFound => unreachable!(),
        diesel::result::Error::QueryBuilderError(_) => "query_builder_error",
        diesel::result::Error::DeserializationError(_) => "deserialization_error",
        diesel::result::Error::SerializationError(_) => "serialization_error",
        diesel::result::Error::RollbackTransaction => "rollback_transaction",
        diesel::result::Error::AlreadyInTransaction => "already_in_transaction",
        _ => "unknown",
    }
}

impl<Conn> SimpleAsyncConnection for MetricsConnection<Conn>
where
    Conn: SimpleAsyncConnection + Send,
{
    async fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        Instrument {
            metrics: self.metrics.clone(),
            future: self.conn.batch_execute(query),
            start: None,
        }
        .await
    }
}

impl AsyncConnectionCore for MetricsConnection<Parent> {
    type ExecuteFuture<'conn, 'query> = Instrument<BoxFuture<'conn, QueryResult<usize>>>;
    type LoadFuture<'conn, 'query> =
        Instrument<BoxFuture<'conn, QueryResult<Self::Stream<'conn, 'query>>>>;
    type Stream<'conn, 'query> = BoxStream<'static, QueryResult<Self::Row<'conn, 'query>>>;
    type Row<'conn, 'query> = <Parent as AsyncConnectionCore>::Row<'conn, 'query>;
    type Backend = <Parent as AsyncConnectionCore>::Backend;

    fn execute_returning_count<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> Self::ExecuteFuture<'conn, 'query>
    where
        T: QueryFragment<Self::Backend> + QueryId + 'query,
    {
        tracing::trace!(
            "SQL Query:\n{}",
            diesel::debug_query::<Self::Backend, _>(&source)
        );

        Instrument {
            metrics: self.metrics.clone(),
            future: self.conn.execute_returning_count(source),
            start: None,
        }
    }

    #[doc(hidden)]
    fn load<'conn, 'query, T>(&'conn mut self, source: T) -> Self::LoadFuture<'conn, 'query>
    where
        T: AsQuery + 'query,
        T::Query: QueryFragment<Self::Backend> + QueryId + 'query,
    {
        let query = source.as_query();
        tracing::trace!(
            "SQL Query:\n{}",
            diesel::debug_query::<Self::Backend, _>(&query)
        );

        Instrument {
            metrics: self.metrics.clone(),
            future: self.conn.load(query),
            start: None,
        }
    }
}

impl AsyncConnection for MetricsConnection<Parent> {
    type TransactionManager = AnsiTransactionManager;

    async fn establish(database_url: &str) -> ConnectionResult<Self> {
        let mut instrumentation = diesel::connection::get_default_instrumentation();
        instrumentation.on_connection_event(InstrumentationEvent::start_establish_connection(
            database_url,
        ));

        Parent::establish(database_url).await.map(|conn| Self {
            metrics: None,
            conn,
            instrumentation: Arc::new(Mutex::new(instrumentation)),
        })
    }

    /// Get access to the current transaction state of this connection
    ///
    /// Hidden in `diesel` behind the
    /// `i-implement-a-third-party-backend-and-opt-into-breaking-changes` feature flag,
    /// therefore not generally visible in the `diesel` generated docs.
    fn transaction_state(
        &mut self,
    ) -> &mut <Self::TransactionManager as TransactionManager<Self>>::TransactionStateData {
        self.conn.transaction_state()
    }

    fn instrumentation(&mut self) -> &mut dyn Instrumentation {
        let Some(instrumentation) = Arc::get_mut(&mut self.instrumentation) else {
            panic!("Cannot access shared instrumentation")
        };

        instrumentation.get_mut().unwrap_or_else(|p| p.into_inner())
    }

    fn set_instrumentation(&mut self, instrumentation: impl Instrumentation) {
        self.instrumentation = Arc::new(std::sync::Mutex::new(Some(Box::new(instrumentation))));
    }

    fn set_prepared_statement_cache_size(&mut self, size: CacheSize) {
        self.conn.set_prepared_statement_cache_size(size);
    }
}

pin_project_lite::pin_project! {
    pub struct Instrument<F> {
        metrics: Option<Arc<DatabaseMetrics>>,
        #[pin]
        future: F,
        start: Option<Instant>,
    }
}

impl<F, T> Future for Instrument<F>
where
    F: Future<Output = diesel::result::QueryResult<T>>,
{
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        if let Some(metrics) = &this.metrics {
            let start = this.start.get_or_insert_with(Instant::now);

            match ready!(this.future.poll(cx)) {
                res @ (Ok(_) | Err(diesel::result::Error::NotFound)) => {
                    metrics
                        .sql_execution_time
                        .record(start.elapsed().as_secs_f64(), &[]);

                    Poll::Ready(res)
                }
                Err(e) => {
                    let labels = &[KeyValue::new(ERROR_KEY, get_metrics_label_for_error(&e))];
                    metrics.sql_error.add(1, labels);

                    Poll::Ready(Err(e))
                }
            }
        } else {
            this.future.poll(cx)
        }
    }
}
