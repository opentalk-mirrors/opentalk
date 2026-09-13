-- Add the column (nullable first, so existing rows don't violate a NOT NULL constraint)
ALTER TABLE event_dates 
    ADD COLUMN duration_secs int;

-- Populate from event_recurrences where a match exists
UPDATE event_dates
    SET duration_secs = event_recurrences.duration_secs
    FROM event_recurrences
    WHERE event_recurrences.event_id = event_dates.event_id; 

-- Drop the duration_secs from event_recurrences
ALTER TABLE event_recurrences
    DROP COLUMN duration_secs;

-- Set fallback duration_secs to 0
--
-- This is fine under the assumption that we only use the duration_secs to calculate the ends_at
-- of recurring evnts. If we change the event to be recurring we calculate the correct  
-- duration_secs as part of our business logic, so setting it to 0 for nonrecurring events is fine 
-- for now.
UPDATE event_dates
    SET duration_secs = 0 
    WHERE duration_secs IS null;

-- Now that every row is populated, enforce NOT NULL
ALTER TABLE event_dates 
    ALTER COLUMN duration_secs SET NOT null;

