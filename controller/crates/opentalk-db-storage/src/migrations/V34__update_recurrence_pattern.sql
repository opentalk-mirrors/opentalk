UPDATE
    events
SET
    recurrence_pattern = REGEXP_REPLACE(
        recurrence_pattern,
        ';UNTIL=[0-9]{8}T[0-9]{6}Z',
        ''
    )
WHERE
    recurrence_pattern LIKE 'RRULE:FREQ=%;UNTIL=%';
