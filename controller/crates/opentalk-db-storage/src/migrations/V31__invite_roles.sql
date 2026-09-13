CREATE TYPE invite_role AS ENUM ('user', 'moderator');
ALTER TABLE event_invites ADD COLUMN role invite_role DEFAULT 'user' NOT NULL;