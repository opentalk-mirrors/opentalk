ALTER TABLE users DROP COLUMN theme;
ALTER TABLE users
ADD COLUMN display_name VARCHAR(511); -- firstname + lastname + ' ' = 511
ALTER TABLE users
ADD COLUMN dashboard_theme VARCHAR(128) DEFAULT 'system' NOT NULL;
ALTER TABLE users
ADD COLUMN conference_theme VARCHAR(128) DEFAULT 'system' NOT NULL;
-- Set display_name to firstname + ' ' + lastname by default
UPDATE users SET display_name=firstname||' '||lastname;
ALTER TABLE users ALTER COLUMN display_name SET NOT NULL;

-- For /users/find create index on the display name
CREATE INDEX display_name_txt_soundex ON users (soundex(lower(display_name)));
