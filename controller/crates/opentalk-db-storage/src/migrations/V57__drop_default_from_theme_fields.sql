ALTER TABLE users
  ALTER COLUMN dashboard_theme DROP DEFAULT,
  ALTER COLUMN dashboard_theme DROP NOT NULL,
  ALTER COLUMN conference_theme DROP DEFAULT,
  ALTER COLUMN conference_theme DROP NOT NULL;

DO $$ BEGIN
	CREATE TYPE theme AS ENUM ( 'light', 'dark', 'system');
	EXCEPTION
	    WHEN duplicate_object THEN null;
END $$;

ALTER TABLE users
    ALTER COLUMN dashboard_theme TYPE theme
        USING (NULL::theme),
    ALTER COLUMN conference_theme TYPE theme
        USING (NULL::theme);
