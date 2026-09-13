CREATE TYPE email_invite_role AS ENUM ('guest', 'moderator');
ALTER TABLE event_email_invites ADD COLUMN role email_invite_role DEFAULT 'guest' NOT NULL;
