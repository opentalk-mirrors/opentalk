ALTER TABLE users ALTER COLUMN language DROP NOT NULL;

UPDATE users SET language = NULL WHERE LENGTH(language) = 0;

ALTER TABLE users ADD CONSTRAINT language_not_empty CHECK (LENGTH(language) > 0);
