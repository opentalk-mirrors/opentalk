-- For each event and upper-/lowercase-normalized email address, keep just one email invite and delete the others.
DELETE FROM event_email_invites
WHERE (email, event_id) NOT IN (
    SELECT
    DISTINCT ON (LOWER(email), event_id)
    email, event_id
    FROM event_email_invites
);

-- Normalize the email address of the kept email invite.
UPDATE event_email_invites SET email = lower(email);

-- Normalize the email address of the users.
-- This might lead to duplicates as there is no unique constraint yet but currently duplicates are allowed anyway.
UPDATE users SET email = lower(email);

-- Delete all email invites that a matching regular invite is already existing for.
DELETE FROM event_email_invites
WHERE (email, event_id) IN (
    SELECT
    users.email, event_invites.event_id
    FROM event_invites
    LEFT JOIN users ON event_invites.invitee = users.id
);

-- From now on, enforce lowercase email addresses.
ALTER TABLE users ADD CONSTRAINT check_email_lowercase CHECK (lower(email) = email);
ALTER TABLE event_email_invites ADD CONSTRAINT check_email_lowercase CHECK (lower(email) = email);
