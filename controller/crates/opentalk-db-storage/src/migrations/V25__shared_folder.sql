CREATE TABLE event_shared_folders(
    event_id UUID PRIMARY KEY REFERENCES events(id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    path TEXT NOT NULL,
    write_share_id TEXT NOT NULL,
    write_url TEXT NOT NULL,
    write_password TEXT NOT NULL,
    read_share_id TEXT NOT NULL,
    read_url TEXT NOT NULL,
    read_password TEXT NOT NULL
);
