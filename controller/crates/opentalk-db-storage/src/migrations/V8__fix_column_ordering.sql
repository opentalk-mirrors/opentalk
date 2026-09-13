-- Fix Ordering of users
-- id_serial, oidc_sub, email, title, firstname, lastname, id_token_exp, theme, language, id, oidc_issuer
-- to
-- id, id_serial, oidc_sub, oidc_issuer, email, title, firstname, lastname, id_token_exp, theme, language
-- Add copies of columns after the id field.
ALTER TABLE users
ADD COLUMN id_serial_new BIGINT DEFAULT nextval('users_id_seq'::regclass),
    ADD COLUMN oidc_sub_new VARCHAR(255),
    ADD COLUMN oidc_issuer_new TEXT,
    ADD COLUMN email_new VARCHAR(255),
    ADD COLUMN title_new VARCHAR(255),
    ADD COLUMN firstname_new VARCHAR(255),
    ADD COLUMN lastname_new VARCHAR(255),
    ADD COLUMN id_token_exp_new BIGINT,
    ADD COLUMN theme_new VARCHAR(255),
    ADD COLUMN language_new VARCHAR(35);
--
-- Update values
UPDATE users
SET id_serial_new = id_serial,
    oidc_sub_new = oidc_sub,
    oidc_issuer_new = oidc_issuer,
    email_new = email,
    title_new = title,
    firstname_new = firstname,
    lastname_new = lastname,
    id_token_exp_new = id_token_exp,
    theme_new = theme,
    language_new = language;
--
-- Set everything to not null (except oidc_issuer)
ALTER TABLE users
ALTER COLUMN id_serial_new
SET NOT NULL,
    ALTER COLUMN oidc_sub_new
SET NOT NULL,
    ALTER COLUMN email_new
SET NOT NULL,
    ALTER COLUMN title_new
SET NOT NULL,
    ALTER COLUMN firstname_new
SET NOT NULL,
    ALTER COLUMN lastname_new
SET NOT NULL,
    ALTER COLUMN id_token_exp_new
SET NOT NULL,
    ALTER COLUMN theme_new
SET NOT NULL,
    ALTER COLUMN language_new
SET NOT NULL;
--
-- Move sequence to id_serial_new
ALTER SEQUENCE users_id_seq OWNED BY users.id_serial_new;
--
-- Drop old columns
ALTER TABLE users DROP COLUMN id_serial,
    DROP COLUMN oidc_sub,
    DROP COLUMN email,
    DROP COLUMN title,
    DROP COLUMN firstname,
    DROP COLUMN lastname,
    DROP COLUMN id_token_exp,
    DROP COLUMN theme,
    DROP COLUMN language,
    DROP COLUMN oidc_issuer;
--
-- Rename all the columns
ALTER TABLE users
    RENAME COLUMN id_serial_new TO id_serial;
ALTER TABLE users
    RENAME COLUMN oidc_sub_new TO oidc_sub;
ALTER TABLE users
    RENAME COLUMN oidc_issuer_new TO oidc_issuer;
ALTER TABLE users
    RENAME COLUMN email_new TO email;
ALTER TABLE users
    RENAME COLUMN title_new TO title;
ALTER TABLE users
    RENAME COLUMN firstname_new TO firstname;
ALTER TABLE users
    RENAME COLUMN lastname_new TO lastname;
ALTER TABLE users
    RENAME COLUMN id_token_exp_new TO id_token_exp;
ALTER TABLE users
    RENAME COLUMN theme_new TO theme;
ALTER TABLE users
    RENAME COLUMN language_new TO language;
--
-- Rebuild Indexes for user search suggestion/autocompletion
CREATE INDEX firstname_lastname_txt_like ON users (
    lower(firstname || ' ' || lastname) text_pattern_ops
);
CREATE INDEX firstname_lastname_txt_soundex ON users (soundex(lower(firstname || ' ' || lastname)));
-- readd unique index for oidc_sub and oidc_issuer
CREATE UNIQUE INDEX users_oidc_sub_issuer_key ON users (oidc_issuer, oidc_sub);
--
-- ##################### ROOMS #####################
-- Fix Ordering of rooms
-- id_serial, id, password, wait_for_moderator, listen_only, owner
-- to
-- id, id_serial, created_by, created_at, password, wait_for_moderator, listen_only
-- Add copies of columns after the id field.
ALTER TABLE rooms
ADD COLUMN id_serial_new BIGINT DEFAULT nextval('rooms_id_seq'::regclass),
    ADD COLUMN created_by UUID REFERENCES users(id),
    ADD COLUMN created_at TIMESTAMPTZ DEFAULT now(),
    ADD COLUMN password_new VARCHAR(255),
    ADD COLUMN wait_for_moderator_new BOOLEAN,
    ADD COLUMN listen_only_new BOOLEAN;
--
-- Update values
UPDATE rooms
SET id_serial_new = id_serial,
    created_by = owner,
    created_at = now(),
    password_new = password,
    wait_for_moderator_new = wait_for_moderator,
    listen_only_new = listen_only;
--
-- Set everything to not null
ALTER TABLE rooms
ALTER COLUMN id_serial_new
SET NOT NULL,
    ALTER COLUMN created_by
SET NOT NULL,
    ALTER COLUMN created_at
SET NOT NULL,
    ALTER COLUMN password_new
SET NOT NULL,
    ALTER COLUMN wait_for_moderator_new
SET NOT NULL,
    ALTER COLUMN listen_only_new
SET NOT NULL;
--
-- Move sequence to id_serial_new
ALTER SEQUENCE rooms_id_seq OWNED BY rooms.id_serial_new;
--
-- Drop old columns
ALTER TABLE rooms DROP COLUMN id_serial,
    DROP COLUMN owner,
    DROP COLUMN password,
    DROP COLUMN wait_for_moderator,
    DROP COLUMN listen_only;
--
-- Rename all the columns
ALTER TABLE rooms
    RENAME COLUMN id_serial_new TO id_serial;
ALTER TABLE rooms
    RENAME COLUMN password_new TO password;
ALTER TABLE rooms
    RENAME COLUMN wait_for_moderator_new TO wait_for_moderator;
ALTER TABLE rooms
    RENAME COLUMN listen_only_new TO listen_only;
--
-- ##################### INVITES #####################
-- Fix Ordering of invites
-- id_serial, id, created, updated, room, active, expiration, created_by, updated_by
-- to
-- id, id_serial, created_by, created_at, updated_by, updated_at, room, active, expiration
ALTER TABLE invites
ADD COLUMN id_serial_new BIGINT DEFAULT nextval('invites_id_seq'::regclass),
    ADD COLUMN created_by_new UUID REFERENCES users(id),
    ADD COLUMN created_at TIMESTAMPTZ DEFAULT now(),
    ADD COLUMN updated_by_new UUID REFERENCES users(id),
    ADD COLUMN updated_at TIMESTAMPTZ DEFAULT now(),
    ADD COLUMN room_new UUID REFERENCES rooms(id),
    ADD COLUMN active_new BOOLEAN,
    ADD COLUMN expiration_new TIMESTAMPTZ;
--
-- copy values into new_ columns
UPDATE invites
SET id_serial_new = id_serial,
    created_by_new = created_by,
    created_at = created,
    updated_by_new = updated_by,
    updated_at = updated,
    room_new = room,
    active_new = active,
    expiration_new = expiration;
--
-- Set everything to not null (except expiration)
ALTER TABLE invites
ALTER COLUMN id_serial_new
SET NOT NULL,
    ALTER COLUMN created_by_new
SET NOT NULL,
    ALTER COLUMN created_at
SET NOT NULL,
    ALTER COLUMN updated_at
SET NOT NULL,
    ALTER COLUMN updated_by_new
SET NOT NULL,
    ALTER COLUMN room_new
SET NOT NULL,
    ALTER COLUMN active_new
SET NOT NULL;
--
-- Move sequence to id_serial_new
ALTER SEQUENCE invites_id_seq OWNED BY invites.id_serial_new;
--
-- Drop old columns
ALTER TABLE invites DROP COLUMN id_serial,
    DROP COLUMN created,
    DROP COLUMN created_by,
    DROP COLUMN updated,
    DROP COLUMN updated_by,
    DROP COLUMN room,
    DROP COLUMN active,
    DROP COLUMN expiration;
--
-- Rename all the columns
ALTER TABLE invites
    RENAME COLUMN id_serial_new TO id_serial;
ALTER TABLE invites
    RENAME COLUMN created_by_new TO created_by;
ALTER TABLE invites
    RENAME COLUMN updated_by_new TO updated_by;
ALTER TABLE invites
    RENAME COLUMN room_new TO room;
ALTER TABLE invites
    RENAME COLUMN active_new TO active;
ALTER TABLE invites
    RENAME COLUMN expiration_new TO expiration;
--
-- ##################### LEGAL VOTES #####################
-- Fix Ordering of legal votes
ALTER TABLE legal_votes
ADD COLUMN created_by UUID REFERENCES users(id),
    ADD COLUMN created_at TIMESTAMPTZ DEFAULT now(),
    ADD COLUMN room UUID REFERENCES rooms(id),
    ADD COLUMN protocol_new JSONB;
--
-- Copy values into new columns
UPDATE legal_votes
SET created_by = initiator,
    created_at = now(),
    room = room_id,
    protocol_new = protocol;
--
-- Set new columns NOT NULL
ALTER TABLE legal_votes
ALTER COLUMN created_by
SET NOT NULL,
    ALTER COLUMN created_at
SET NOT NULL,
    ALTER COLUMN protocol_new
SET NOT NULL;
--
-- Drop old columns
ALTER TABLE legal_votes DROP COLUMN initiator,
    DROP COLUMN room_id,
    DROP COLUMN protocol;
--
-- Rename new columns
ALTER TABLE legal_votes
    RENAME COLUMN protocol_new TO protocol;
