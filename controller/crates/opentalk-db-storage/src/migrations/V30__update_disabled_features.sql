DO $$
DECLARE
    tariff RECORD;
    features TEXT[];
    namespace_separator CONSTANT TEXT := '::';
    default_namespace CONSTANT TEXT := 'core';
BEGIN
    FOR tariff IN SELECT id, disabled_features  FROM tariffs LOOP
        features := tariff.disabled_features;

        FOR i IN 1..cardinality(features) LOOP
                        IF STRPOS(features[i], namespace_separator) = 0 THEN
                            features[i] := default_namespace || namespace_separator || features[i];
        END IF;
    END LOOP;

    UPDATE tariffs SET disabled_features = features where id = tariff.id;

END LOOP;

END;
$$;
