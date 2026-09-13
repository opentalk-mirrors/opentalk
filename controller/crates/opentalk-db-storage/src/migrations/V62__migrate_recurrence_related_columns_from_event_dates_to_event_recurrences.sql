-- Abort entire migration if constraints from the event_recurrences table are not met
BEGIN;

-- Create event_recurrences table 
CREATE TABLE event_recurrences (
    event_id uuid NOT null,
    duration_secs int NOT null,
    recurrence_pattern varchar(4092) NOT null,
    PRIMARY KEY (event_id),
    FOREIGN KEY (event_id) REFERENCES event_dates(event_id) ON DELETE CASCADE
);

-- Migrate recurrence related columns from event_dates if any date related field is not null
INSERT INTO event_recurrences (
    event_id,
    duration_secs,
    recurrence_pattern
)
SELECT 
    event_id,
    duration_secs,
    recurrence_pattern
FROM 
    event_dates
WHERE 
    duration_secs IS NOT null
    OR recurrence_pattern IS NOT null; 

-- Drop migrated columns from event_dates table
ALTER TABLE event_dates 
    DROP COLUMN duration_secs, 
    DROP COLUMN recurrence_pattern;

COMMIT;
