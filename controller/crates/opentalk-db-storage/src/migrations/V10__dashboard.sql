--
-- Tables for Event planning/calendar on the dashboard
CREATE TABLE events (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    id_serial BIGSERIAL UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(4096) NOT NULL,
    room UUID REFERENCES rooms(id) NOT NULL,
    created_by UUID REFERENCES users(id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_by UUID REFERENCES users(id) NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    is_time_independent BOOL NOT NULL,
    is_all_day BOOL,
    starts_at TIMESTAMPTZ,
    starts_at_tz VARCHAR(255),
    ends_at TIMESTAMPTZ,
    ends_at_tz VARCHAR(255),
    duration_secs INTEGER,
    is_recurring BOOLEAN,
    recurrence_pattern VARCHAR(4094)
);
CREATE TYPE event_exception_kind AS ENUM ('modified', 'cancelled');
CREATE TABLE event_exceptions(
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    event_id UUID REFERENCES events(id) ON DELETE CASCADE NOT NULL,
    exception_date TIMESTAMPTZ NOT NULL,
    exception_date_tz VARCHAR(255) NOT NULL,
    created_by UUID REFERENCES users(id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    kind event_exception_kind NOT NULL,
    title VARCHAR(255),
    description VARCHAR(4096),
    is_all_day BOOL,
    starts_at TIMESTAMPTZ,
    starts_at_tz VARCHAR(255),
    ends_at TIMESTAMPTZ,
    ends_at_tz VARCHAR(255),
    UNIQUE (event_id, exception_date, exception_date_tz)
);
CREATE INDEX ON event_exceptions (event_id, exception_date);
CREATE TYPE event_invite_status AS ENUM ('pending', 'accepted', 'tentative', 'declined');
CREATE TABLE event_invites(
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    event_id UUID REFERENCES events(id) ON DELETE CASCADE NOT NULL,
    invitee UUID REFERENCES users(id) NOT NULL,
    created_by UUID REFERENCES users(id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    status event_invite_status DEFAULT 'pending' NOT NULL,
    UNIQUE (event_id, invitee)
);
CREATE INDEX ON event_invites (event_id, invitee);
CREATE TABLE event_favorites(
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    event_id UUID REFERENCES events(id) ON DELETE CASCADE NOT NULL,
    PRIMARY KEY(user_id, event_id)
);
