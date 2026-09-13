// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, sync::Once, time::Instant};

use axum_prometheus::metrics::{
    Gauge, Unit, counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram,
};
use opentalk_roomserver_types::{
    client_parameters::ParticipationKind, connection_id::ConnectionId,
};

const SCOPE_KEY: &str = "otel_scope_name";
const SCOPE_VALUE: &str = "ot-roomserver";
const BUCKET_KEY: &str = "bucket";

const CREATED_ROOMS: &str = "signaling.created_rooms_count";
const DESTROYED_ROOMS: &str = "signaling.destroyed_rooms_count";
const CREATED_BREAKOUT_ROOMS: &str = "signaling.created_breakout_rooms_count";
const DESTROYED_BREAKOUT_ROOMS: &str = "signaling.destroyed_breakout_rooms_count";
pub const ROOM_LIFE_TIME: &str = "signaling.room_life_time";
pub const ROOM_LIFE_TIME_BUCKETS: &[f64] = &[
    2.0 * 60.0,        // 2 minutes
    5.0 * 60.0,        // 5 minutes
    30.0 * 60.0,       // 30 minutes
    60.0 * 60.0,       // 1 hour
    3.0 * 60.0 * 60.0, // 3 hours
    8.0 * 60.0 * 60.0, // 8 hours
];
const CONNECTION_COUNT: &str = "signaling.connection_count";
const CONNECTIONS_PER_ROOM: &str = "signaling.connections_per_room";
const CONNECTIONS_PER_ROOM_BUCKETS: [usize; 7] = [2, 10, 25, 50, 100, 200, 300];
pub const CONNECTION_MEETING_TIME: &str = "signaling.connection_meeting_time";
pub const CONNECTION_MEETING_TIME_BUCKETS: &[f64] = &[
    2.0 * 60.0,        // 2 minutes
    5.0 * 60.0,        // 5 minutes
    30.0 * 60.0,       // 30 minutes
    60.0 * 60.0,       // 1 hour
    3.0 * 60.0 * 60.0, // 3 hours
    8.0 * 60.0 * 60.0, // 8 hours
];
const CONGESTED_CONNECTIONS: &str = "signaling.congested_connections";

static DESCRIBE_ONCE: Once = Once::new();

pub struct Metrics {
    connections: HashMap<ConnectionId, (Instant, ParticipationKind)>,
    room_created_at: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        DESCRIBE_ONCE.call_once(|| {
            describe_gauge!(CONNECTION_COUNT, Unit::Count, "Number of connections");
            describe_histogram!(
                CONNECTION_MEETING_TIME,
                Unit::Seconds,
                "Time a connection was connected to a meeting room"
            );
            describe_counter!(CREATED_ROOMS, Unit::Count, "Number of created rooms");
            describe_counter!(DESTROYED_ROOMS, Unit::Count, "Number of destroyed rooms");
            describe_counter!(
                CREATED_BREAKOUT_ROOMS,
                Unit::Count,
                "Number of created breakout rooms"
            );
            describe_counter!(
                DESTROYED_BREAKOUT_ROOMS,
                Unit::Count,
                "Number of destroyed breakout rooms"
            );
            describe_histogram!(ROOM_LIFE_TIME, Unit::Seconds, "Time rooms were active");
            describe_gauge!(CONNECTIONS_PER_ROOM, Unit::Count, "Connections per room");
            describe_counter!(
                CONGESTED_CONNECTIONS,
                Unit::Count,
                "Number of connections that were disconnected due to congestion"
            );
        });

        Self::increment_created_rooms_counter();

        Metrics {
            connections: HashMap::new(),
            room_created_at: Instant::now(),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn record_participant_joined(
        &mut self,
        connection_id: ConnectionId,
        kind: ParticipationKind,
    ) {
        if self.connections.contains_key(&connection_id) {
            return;
        }

        // Update connections per room metric
        // Remove the room from the previous bucket. Only do so if this is not the first connection
        // joining. This prevents the connection count going to a negative number.
        if !self.connections.is_empty() {
            let previous_bucket = Self::connection_count_bucket(self.connections.len());
            Self::decrement_connections_per_room_bucket(previous_bucket);
        }

        // The insert has to happen after we determine the previous bucket, otherwise we could end
        // up with the wrong one.
        self.connections
            .insert(connection_id, (Instant::now(), kind));

        // Add the room to the new bucket
        let new_bucket = Self::connection_count_bucket(self.connections.len());
        Self::increment_connections_per_room_bucket(new_bucket);

        // Update connection count metric
        Self::increment_connection_count(kind);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn record_participant_left(&mut self, connection_id: ConnectionId) {
        // Determine the bucket before removing the connection
        let previous_bucket = Self::connection_count_bucket(self.connections.len());

        let Some((joined_at, kind)) = self.connections.remove(&connection_id) else {
            tracing::warn!("connection left metrics invoked without join");
            return;
        };

        // Update meeting time metric
        Self::record_meeting_time(joined_at);

        // Update connections per room metric
        // Remove the room from the previous bucket
        Self::decrement_connections_per_room_bucket(previous_bucket);
        // Add the room to the new bucket. Only do so if there is at least one connection left, it
        // does not make sense to track empty rooms.
        if !self.connections.is_empty() {
            let new_bucket = Self::connection_count_bucket(self.connections.len());
            Self::increment_connections_per_room_bucket(new_bucket);
        }

        // Update connection count metric
        Self::decrement_connection_count(kind);
    }

    fn increment_created_rooms_counter() {
        counter!(CREATED_ROOMS, SCOPE_KEY => SCOPE_VALUE).increment(1);
    }

    fn increment_destroyed_rooms_counter() {
        counter!(DESTROYED_ROOMS, SCOPE_KEY => SCOPE_VALUE).increment(1);
    }

    fn record_room_life_time(created_at: Instant) {
        let life_time = Instant::now().duration_since(created_at).as_secs_f64();
        histogram!(ROOM_LIFE_TIME, SCOPE_KEY => SCOPE_VALUE).record(life_time);
    }

    pub fn increment_created_breakout_rooms_counter(value: u64) {
        counter!(CREATED_BREAKOUT_ROOMS, SCOPE_KEY => SCOPE_VALUE).increment(value);
    }

    pub fn increment_destroyed_breakout_rooms_counter(value: u64) {
        counter!(DESTROYED_BREAKOUT_ROOMS, SCOPE_KEY => SCOPE_VALUE).increment(value);
    }

    pub fn increment_connection_count(kind: ParticipationKind) {
        Self::connection_count(kind).increment(1);
    }

    pub fn decrement_connection_count(kind: ParticipationKind) {
        Self::connection_count(kind).decrement(1);
    }

    pub fn increment_congested_connections(value: u64) {
        counter!(CONGESTED_CONNECTIONS, SCOPE_KEY => SCOPE_VALUE).increment(value);
    }

    fn connection_count(kind: ParticipationKind) -> Gauge {
        const PARTICIPATION_KIND: &str = "participation_kind";
        let kind = match kind {
            ParticipationKind::Registered => "user",
            ParticipationKind::Guest => "guest",
            ParticipationKind::Recorder => "recorder",
            ParticipationKind::Transcription => "transcription",
            ParticipationKind::CallIn => "call_in",
            ParticipationKind::RegisteredCallIn => "registered_call_in",
        };

        gauge!(CONNECTION_COUNT, PARTICIPATION_KIND => kind, SCOPE_KEY => SCOPE_VALUE)
    }

    fn record_meeting_time(joined_at: Instant) {
        let meeting_time = Instant::now().duration_since(joined_at).as_secs_f64();
        histogram!(CONNECTION_MEETING_TIME, SCOPE_KEY => SCOPE_VALUE).record(meeting_time);
    }

    fn increment_connections_per_room_bucket(bucket: usize) {
        // The idea behind the connections per room metric is to track the number of connections
        // in a room in a histogram. However, a histogram is not suitable for this, because
        // in a histogram, a recorded value cannot change. E.g., in the connection meeting time
        // histogram, the meeting time for each connection is recorded once they leave the room. If
        // we were to implement the connections per room metric in the same manner, we'd record a
        // new observation each time the number of connections in a room changes and we'd end up
        // with the same room in multiple buckets.
        // Instead we must be able to "move" a room to a different bucket when its number of
        // connections changes. To do so, we implement this metric as a set of gauges, one per
        // bucket.
        gauge!(CONNECTIONS_PER_ROOM, BUCKET_KEY => bucket.to_string(), SCOPE_KEY => SCOPE_VALUE)
            .increment(1);
    }

    fn decrement_connections_per_room_bucket(bucket: usize) {
        gauge!(CONNECTIONS_PER_ROOM, BUCKET_KEY => bucket.to_string(), SCOPE_KEY => SCOPE_VALUE)
            .decrement(1);
    }

    fn connection_count_bucket(count: usize) -> usize {
        for bucket in CONNECTIONS_PER_ROOM_BUCKETS {
            if count <= bucket {
                return bucket;
            }
        }
        *CONNECTIONS_PER_ROOM_BUCKETS
            .last()
            .expect("Buckets must not be empty")
    }
}

impl Drop for Metrics {
    fn drop(&mut self) {
        Self::increment_destroyed_rooms_counter();
        Self::record_room_life_time(self.room_created_at);
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::CONNECTIONS_PER_ROOM_BUCKETS;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn connections_per_room_buckets_is_not_empty() {
        // `CONNECTIONS_PER_ROOM_BUCKETS` must not be empty, or the code will panic
        assert!(!CONNECTIONS_PER_ROOM_BUCKETS.is_empty());
    }
}
