CREATE TABLE module_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) NOT NULL,
    room_id UUID REFERENCES rooms(id) NOT NULL,
    created_by UUID REFERENCES users(id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    namespace varchar(255) NOT NULL,
    tag varchar(255),
    data jsonb NOT NULL
);

-- migrate all legal_votes into module_resources
DO $$
DECLARE
legal_vote RECORD;
BEGIN
    FOR legal_vote IN SELECT * FROM legal_votes WHERE room IS NOT NULL
    
    LOOP
        INSERT INTO module_resources
        VALUES
        (
            legal_vote.id,          -- id
            legal_vote.tenant_id,   -- tenant_id
            legal_vote.room,        -- room_id
            legal_vote.created_by,  -- created_by
            legal_vote.created_at,  -- created_at
            legal_vote.created_at,  -- updated_at
            'legal_vote',           -- namespace
            'protocol',             -- tag
            legal_vote.protocol     -- data
        );
    END LOOP;
END;
$$;

DROP TABLE legal_votes;

-- remove legal vote access rules
DELETE FROM casbin_rule WHERE v1 LIKE '/legal_vote/%';

-- This migration includes functions for a JSON Patch interface (RFC 6902)

CREATE FUNCTION ot_jsonb_add(target JSONB, path TEXT[], value JSONB, create_if_missing BOOLEAN)
RETURNS JSONB
LANGUAGE PLPGSQL
AS
$$
BEGIN
    -- check if the path is empty (aka the added value is the whole JSON element)
    IF cardinality(path) = 0 THEN
        RETURN value;
    ELSE
        -- check if the parent of the targeted path exits
        IF NOT (ot_jsonb_path_exists(target, trim_array(path, 1))) THEN
            RAISE 'ot_invalid_path'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = format('path "%s" does not exist', path);

        END IF;

        -- check if the final path part is numeric to determine if the operation is an array insert
        IF path[array_upper(path,1)] ~ '^\d+$' THEN
            return jsonb_insert(target, path, value);
        END IF;

        RETURN jsonb_set_lax(target, path, value, create_if_missing, 'use_json_null');
    END IF;
END;
$$;

CREATE FUNCTION ot_jsonb_remove(target JSONB, path TEXT[])
RETURNS JSONB
LANGUAGE PLPGSQL
AS
$$
BEGIN
    -- abort if the given path is empty or referenced element does not exist
    IF cardinality(path) = 0 THEN
        RAISE 'ot_invalid_path'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('path cannot be empty or non-existent in %s', path);
    ELSE
        IF NOT (ot_jsonb_path_exists(target, path)) THEN
            RAISE 'ot_invalid_path'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = format('path "%s" does not exist', path);
        END IF;
    END IF;

    RETURN target #- path;
END;
$$;

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

CREATE FUNCTION ot_jsonb_test(target JSONB, path TEXT[], value JSONB)
RETURNS JSONB
LANGUAGE PLPGSQL
AS
$$
BEGIN
    IF NOT ot_jsonb_path_exists(target, path) THEN
        RAISE 'ot_invalid_path'
            USING 
                ERRCODE = 'OTALK',
                DETAIL = format('path "%s" does not exist', path);
    END IF;

    -- logical compare two jsonb values
    IF (target #> path) != value THEN
        RAISE 'ot_value_not_equal'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('the value comparison at path "%s" returned false', path);
    END IF;

    RETURN target;
END;
$$;


CREATE FUNCTION ot_patch_json(target JSONB, changeset JSONB)
RETURNS JSONB
LANGUAGE PLPGSQL
AS
$$
DECLARE
    target_path TEXT[];
    from_path TEXT[];
    operation JSONB;

    index int = 0;
    err_msg TEXT;
    err_detail TEXT;
BEGIN
    FOR operation IN SELECT * FROM jsonb_array_elements(changeset) LOOP
        BEGIN
        index := index + 1;
        SELECT path_string_to_array(operation ->> 'path') INTO target_path;

            CASE operation ->> 'op'
                WHEN 'add' THEN target = ot_jsonb_add(target, target_path, operation -> 'value', true);

                WHEN 'remove' THEN target = ot_jsonb_remove(target, target_path);

                WHEN 'replace' THEN
                    target = ot_jsonb_remove(target, target_path);
                    target = ot_jsonb_add(target, target_path, operation -> 'value', true);

                WHEN 'move' THEN
                    SELECT path_string_to_array(operation ->> 'from') INTO from_path;
                    target = ot_jsonb_copy(target, from_path, target_path, true);

                WHEN 'copy' THEN
                    SELECT path_string_to_array(operation ->> 'from') INTO from_path;
                    target = ot_jsonb_copy(target, from_path, target_path, false);

                WHEN 'test' THEN target = ot_jsonb_test(target, target_path, operation -> 'value');
            END CASE;
        EXCEPTION
            -- catch any exception with error code 'OTALK' and overwrite the error details with the current array index
            WHEN SQLSTATE 'OTALK' THEN
                GET STACKED DIAGNOSTICS 
                    err_msg = MESSAGE_TEXT,
                    err_detail = PG_EXCEPTION_DETAIL;

                RAISE
                    USING
                        MESSAGE = err_msg,
                        DETAIL = err_detail,
                        HINT = index;

        END;
    END LOOP;
    RETURN target;
END;
$$;

CREATE FUNCTION ot_jsonb_path_exists(to_test JSONB, path TEXT[])
RETURNS BOOLEAN
LANGUAGE PLPGSQL
AS
$$
DECLARE
    RETURN_BOOLEAN BOOLEAN;
BEGIN
    SELECT COUNT((to_test #> path)::JSONB) > 0 INTO RETURN_BOOLEAN;
    RETURN RETURN_BOOLEAN;
END;
$$;

CREATE FUNCTION path_string_to_array(path_string TEXT)
RETURNS TEXT[]
LANGUAGE PLPGSQL
AS
$$
BEGIN
    -- error if the path does not start with a forward slash (/)
    IF NOT path_string ^@ '/' THEN
        RAISE 'ot_invalid_path'
            USING
                ERRCODE = 'OTALK',
                DETAIL = format('path "%s" needs to begin with a slash (/)', path_string);
    END IF;

    -- early return an empty array if the full path is the root element (just a slash)
    IF path_string = '/' THEN
        return '{}';
    END IF;

    -- trim the first slash to avoid empty array elements after the regex split
    path_string = trim(LEADING '/' FROM path_string);

    return regexp_split_to_array(path_string, '/');
END;
$$;