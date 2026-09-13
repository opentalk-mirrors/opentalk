UPDATE events
SET 
    ends_at = starts_at + INTERVAL '100 years',
    updated_at = NOW()
WHERE 
    is_recurring = true
    AND recurrence_pattern NOT LIKE '%UNTIL%'
    AND ends_at < NOW()
    AND starts_at IS NOT NULL;
