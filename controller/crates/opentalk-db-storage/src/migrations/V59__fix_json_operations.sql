-- Fixes the path handling for various json operations that were initially introduces in migration V27.
-- These changes allow empty path parameters to be used and changes root paths ("/") to be handled as an empty field 
-- as shown in https://www.rfc-editor.org/rfc/rfc6901#section-5

-- difference to V27: an empty path will reset the json document to '{}' instead of erroring
DROP FUNCTION ot_jsonb_remove;
CREATE FUNCTION ot_jsonb_remove(target JSONB, path TEXT[])
RETURNS JSONB
LANGUAGE PLPGSQL
AS
$$
BEGIN
    IF cardinality(path) = 0 THEN
        -- Removing an empty path would usually result in the deletion of the whole json document. This is not desired
        -- in the context of our database. As a compromise, we set the json to be an empty object
        target := '{}'::JSONB;
        RETURN target;
    END IF;

    IF NOT (ot_jsonb_path_exists(target, path)) THEN
        RAISE 'ot_invalid_path'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('path "%s" does not exist', path);
    END IF;

    RETURN target #- path;
END;
$$;

-- difference to V27: adds a null check for the value at the from_path
DROP FUNCTION ot_jsonb_copy;
CREATE FUNCTION ot_jsonb_copy(target JSONB, from_path TEXT[], target_path TEXT[], move_instead BOOLEAN)
RETURNS JSONB
LANGUAGE PLPGSQL
AS
$$
DECLARE
    tmp JSONB;
BEGIN
    IF NOT ot_jsonb_path_exists(target, trim_array(target_path, 1)) THEN
        RAISE 'ot_invalid_path'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('path "%s" does not exist', target_path);
    END IF;

    -- read the VALUE to copy/move into tmp
    SELECT target #> from_path INTO tmp;

    IF tmp IS NULL THEN
        RAISE 'ot_invalid_from_path'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('from path "%s" does not exist', from_path);
    END IF;

    IF move_instead THEN
        -- check if `from_path` is a prefix of `target_path`
        IF cardinality(from_path) = 0 OR target_path[:(array_length(from_path, 1))] = from_path THEN
            RAISE 'ot_invalid_from_path'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = 'from_path is a prefix of path';
        END IF;

        -- remove the VALUE to move from the target
        SELECT target #- from_path INTO target;
    END IF;

    -- Set tmp into target in 'target_path' and return it
    RETURN ot_jsonb_add(target, target_path, tmp, true);
END;
$$;


-- difference to V27: allow empty paths
DROP FUNCTION path_string_to_array;
CREATE FUNCTION path_string_to_array(path_string TEXT)
RETURNS TEXT[]
LANGUAGE PLPGSQL
AS
$$
BEGIN
    -- return an empty array if the path is empty
    IF path_string = '' THEN
        RETURN '{}'::TEXT[];
    END IF;

    -- error if the path does not start with a forward slash (/)
    IF NOT path_string ^@ '/' THEN
        RAISE 'ot_invalid_path'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('path "%s" needs to empty or begin with a slash (/)', path_string);
    END IF;

    -- trim the first slash to avoid empty array elements after the regex split
    path_string = trim(LEADING '/' FROM path_string);

    return regexp_split_to_array(path_string, '/');
END;
$$;

