CREATE TABLE event_email_invites (
    event_id UUID REFERENCES events(id) ON DELETE CASCADE NOT NULL,
    email VARCHAR(255) NOT NULL,
    created_by UUID REFERENCES users(id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    PRIMARY KEY(event_id, email)
);
