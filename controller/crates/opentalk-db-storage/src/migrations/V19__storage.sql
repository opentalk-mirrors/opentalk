CREATE TABLE assets (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    namespace varchar(255),
    kind VARCHAR(255) NOT NULL,
    filename VARCHAR(512) NOT NULL
);

CREATE TABLE room_assets (
    room_id UUID REFERENCES rooms(id) NOT NULL,
    asset_id UUID REFERENCES assets(id) ON DELETE CASCADE NOT NULL UNIQUE,
    PRIMARY KEY(room_id, asset_id)
);
