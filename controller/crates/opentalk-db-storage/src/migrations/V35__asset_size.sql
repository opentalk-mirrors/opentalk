ALTER TABLE assets
	ADD COLUMN size bigint CHECK (size >= 0);

UPDATE
	assets
SET
	size = 0
WHERE
	size IS NULL;

ALTER TABLE assets ALTER COLUMN size SET NOT NULL;
