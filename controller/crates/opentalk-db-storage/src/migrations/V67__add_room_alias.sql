-- Add the room alias (name and suffix) to the rooms table.
ALTER TABLE rooms
    ADD COLUMN name VARCHAR(40),
    ADD COLUMN suffix VARCHAR(64),
    -- Room names can exist without suffixes, but if a suffix is present, the name must also be present.
    ADD CONSTRAINT chk_name_when_suffix CHECK (suffix IS NULL OR name IS NOT NULL);

-- Room alias are the combination of the room name and (optional) suffix and must be unique.
--
-- `NULLS NOT DISTINCT` ensures that two named rooms without a suffix (suffix IS NULL) are still treated as a
-- collision; without it Postgres considers NULL suffixes distinct and the name-only alias would not be unique.
--
-- The partial `WHERE name IS NOT NULL` predicate keeps nameless rooms out of the index so any number of them can exist.
CREATE UNIQUE INDEX room_alias ON rooms(name, suffix) NULLS NOT DISTINCT WHERE name IS NOT NULL;
