CREATE TYPE tariff_status AS ENUM ('default', 'paid', 'downgraded');
ALTER TABLE users ADD COLUMN tariff_status tariff_status DEFAULT 'default' NOT NULL;
