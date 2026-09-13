-- Checks the properties of a single tariff
CREATE OR REPLACE PROCEDURE ot_check_values_for_tariff(tariff RECORD)
    LANGUAGE PLPGSQL
AS
$$
DECLARE
    quota_list JSONB;
    quota RECORD;
    modules TEXT[];
    features TEXT[];
BEGIN
    quota_list := tariff.quotas;
    modules := tariff.disabled_modules;
    features := tariff.disabled_features;

    -- Check the quota names and values
    FOR quota IN SELECT key, value FROM jsonb_each_text(quota_list) LOOP
        -- Only well-known quotas are allowed
        IF quota.key NOT IN ('max_storage', 'room_time_limit_secs', 'room_participant_limit') THEN
            RAISE 'ot_invalid_quota'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = format(
                        'quota "%s" of tariff "%s" is invalid, see https://docs.opentalk.eu/admin/controller/advanced/tariffs',
                        quota.key, tariff.name
                    );
        END IF;

        -- Only unsigned integers are allowed
        IF NOT regexp_like(quota.value, '^[0-9]+$') THEN
            RAISE 'ot_invalid_quota_value'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = format('quota "%s" of tariff "%s" has an invalid value of "%s"', quota.key, tariff.name, quota.value);
        END IF;
    END LOOP;

    -- Check the disabled modules
    FOR i IN 1..cardinality(modules) LOOP
        IF NOT regexp_like(modules[i], '^[_0-9a-z]+$') THEN
            RAISE 'ot_invalid_module'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = format(
                        'module "%s" of tariff "%s" is invalid, see https://docs.opentalk.eu/admin/controller/advanced/modules',
                        modules[i], tariff.name
                    );
        END IF;
    END LOOP;

    -- Check the disabled features
    FOR i IN 1..cardinality(features) LOOP
        IF NOT regexp_like(features[i], '^[_0-9a-z]+::[_0-9a-z]+$') THEN
            RAISE 'ot_invalid_feature'
                USING
                    ERRCODE = 'OTALK',
                    DETAIL = format(
                        'feature "%s" of tariff "%s" is invalid, see https://docs.opentalk.eu/admin/controller/advanced/modules',
                        features[i], tariff.name
                    );
        END IF;
    END LOOP;
END;
$$;

-- Provides a trigger that checks the properties of a single tariff
CREATE OR REPLACE FUNCTION ot_check_values_for_tariff_trigger()
    RETURNS trigger
    LANGUAGE PLPGSQL
AS $$
BEGIN
    CALL ot_check_values_for_tariff(NEW);
    RETURN NEW;
END;
$$;

-- Attach the above trigger to the tariffs
CREATE OR REPLACE TRIGGER check_values_for_tariff BEFORE INSERT OR UPDATE ON tariffs
    FOR EACH ROW EXECUTE FUNCTION ot_check_values_for_tariff_trigger();

-- Check the values of all existing tariffs immediately during the database migration
DO $$
DECLARE
    tariff RECORD;
BEGIN
    FOR tariff IN SELECT name, id, quotas, disabled_modules, disabled_features  FROM tariffs LOOP
        CALL ot_check_values_for_tariff(tariff);
    END LOOP;
END;
$$;
