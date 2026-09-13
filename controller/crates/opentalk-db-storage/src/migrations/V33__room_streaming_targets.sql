CREATE TYPE streaming_kind AS enum ('custom');

CREATE TABLE room_streaming_targets (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    room_id UUID REFERENCES rooms(id) ON DELETE CASCADE NOT NULL,
    name TEXT NOT NULL,
    kind streaming_kind NOT NULL,
    streaming_endpoint TEXT NOT NULL,
    streaming_key TEXT NOT NULL,
    public_url TEXT NOT NULL
);
