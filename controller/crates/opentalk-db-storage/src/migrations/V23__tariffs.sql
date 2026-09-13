CREATE TABLE tariffs(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    quotas JSONB NOT NULL,
    disabled_modules TEXT[] NOT NULL
);

CREATE TABLE external_tariffs(
    external_id TEXT PRIMARY KEY,
    tariff_id UUID REFERENCES tariffs(id) NOT NULL
);

DO $$
DECLARE
    DefaultTariffId UUID := gen_random_uuid();
BEGIN
    -- Create a new default tarrif all current users are assigned to 
    INSERT INTO tariffs VALUES (DefaultTariffId, 'OpenTalkDefaultTariff', DEFAULT, DEFAULT, '{}', '{}');

    ALTER TABLE users ADD COLUMN tariff_id UUID REFERENCES tariffs(id);
    UPDATE users SET tariff_id = DefaultTariffId;
    ALTER TABLE users ALTER COLUMN tariff_id SET NOT NULL;
END $$
