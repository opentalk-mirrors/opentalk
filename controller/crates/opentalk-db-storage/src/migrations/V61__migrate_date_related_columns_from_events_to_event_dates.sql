-- Abort entire migration if constraints from the event_dates table are not met
BEGIN;

-- Create event_dates table 
CREATE TABLE event_dates (
    event_id uuid NOT null,
    starts_at timestamptz NOT null,
    starts_at_tz varchar(255) NOT null,
    ends_at timestamptz NOT null,
    ends_at_tz varchar(255) NOT null,
    is_all_day boolean NOT null,
    duration_secs int,
    recurrence_pattern varchar(4092),
    PRIMARY KEY (event_id),
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

-- Migrate date related columns from events if any date related field is not null
INSERT INTO event_dates (
    event_id, 
    starts_at,
    starts_at_tz,
    ends_at,
    ends_at_tz,
    is_all_day,
    duration_secs,
    recurrence_pattern
)
SELECT 
    id,
    starts_at,
    starts_at_tz,
    ends_at,
    ends_at_tz,
    is_all_day,
    duration_secs,
    recurrence_pattern
FROM 
    events
WHERE 
    starts_at IS NOT null
    OR starts_at_tz IS NOT null
    OR ends_at IS NOT null
    OR ends_at_tz IS NOT null
    OR is_all_day IS NOT null
    OR duration_secs IS NOT null
    OR recurrence_pattern IS NOT null; 

-- Drop migrated columns from events table
ALTER TABLE events 
    DROP COLUMN starts_at,
    DROP COLUMN starts_at_tz,
    DROP COLUMN ends_at, 
    DROP COLUMN ends_at_tz, 
    DROP COLUMN is_all_day,
    DROP COLUMN duration_secs, 
    DROP COLUMN recurrence_pattern;

COMMIT;
