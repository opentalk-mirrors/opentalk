-- Change oidc_uuid to oidc_sub and add oidc_issuer field
-- Change id to id_serial and add new id fields which by default
-- gets the value of oidc_sub (former oidc_uuid).
ALTER TABLE users ALTER COLUMN oidc_uuid TYPE VARCHAR(255);
ALTER TABLE users RENAME COLUMN oidc_uuid TO oidc_sub;
ALTER TABLE users DROP CONSTRAINT users_oidc_uuid_key;
ALTER TABLE users RENAME COLUMN id TO id_serial;
ALTER TABLE users ADD COLUMN id UUID DEFAULT gen_random_uuid() UNIQUE NOT NULL;
UPDATE users SET id = oidc_sub::uuid;
ALTER TABLE users ADD COLUMN oidc_issuer TEXT;
ALTER TABLE users DROP CONSTRAINT users_id_key;
ALTER TABLE users DROP CONSTRAINT users_pkey CASCADE;
ALTER TABLE users ADD PRIMARY KEY (id);

-- Build UNIQUE index over the oidc_issuer and oidc_sub.
-- These make queries faster trying to identify a user 
-- from an id or access token. Also the unique contraint
-- over these both ensure that no user is duplicated
CREATE UNIQUE INDEX users_oidc_sub_issuer_key ON users (oidc_issuer, oidc_sub);

-- Change groups from 
-- old - { id: TEXT }
-- to 
-- new - { id: UUID, id_serial: BIGSERIAL, issuer: TEXT, name: TEXT }
-- where new.name = old.id
ALTER TABLE groups RENAME id TO old_id;
ALTER TABLE groups ADD COLUMN id UUID DEFAULT gen_random_uuid() NOT NULL;
ALTER TABLE groups ADD COLUMN id_serial BIGSERIAL NOT NULL;
ALTER TABLE groups ADD COLUMN oidc_issuer TEXT;
ALTER TABLE groups ADD COLUMN name TEXT;
UPDATE groups SET name = old_id::text;
ALTER TABLE groups ALTER name SET NOT NULL;
ALTER TABLE groups DROP COLUMN old_id CASCADE;
ALTER TABLE groups ADD PRIMARY KEY (id);

-- Change user_id from type
-- BIGINT REFERENCES users(id_serial)
-- to
-- UUID REFERENCES users(id)
ALTER TABLE user_groups RENAME user_id TO user_id_serial;
ALTER TABLE user_groups ADD COLUMN user_id UUID REFERENCES users(id);
UPDATE user_groups SET user_id = users.id FROM users WHERE users.id_serial = user_id_serial;
ALTER TABLE user_groups ALTER user_id SET NOT NULL;
ALTER TABLE user_groups DROP COLUMN user_id_serial;

-- Change group_id from type
-- VARCHAR which references the group name
-- to
-- UUID REFERENCES groups(id)
ALTER TABLE user_groups RENAME group_id to old_group_id;
ALTER TABLE user_groups ADD COLUMN group_id UUID REFERENCES groups(id);
UPDATE user_groups SET group_id = groups.id FROM groups WHERE groups.name = old_group_id;
ALTER TABLE user_groups ALTER group_id SET NOT NULL;
ALTER TABLE user_groups DROP COLUMN old_group_id;

-- Add back the primary key for user_groups
ALTER TABLE user_groups ADD PRIMARY KEY (user_id, group_id);

-- Change the id to id_serial, add id field
-- and change the primary key to id
ALTER TABLE rooms RENAME id TO id_serial;
ALTER TABLE rooms RENAME uuid TO id;
ALTER TABLE rooms DROP CONSTRAINT rooms_pkey;
ALTER TABLE rooms DROP CONSTRAINT rooms_uuid_key CASCADE;
ALTER TABLE rooms ALTER id SET DEFAULT gen_random_uuid();
ALTER TABLE rooms ADD  PRIMARY KEY (id);

-- Restore all foreign keys from dropping the rooms primary key
ALTER TABLE sip_configs ADD CONSTRAINT sip_config_room_fkey FOREIGN KEY (room) REFERENCES rooms(id); 
ALTER TABLE invites ADD CONSTRAINT invite_room_fkey FOREIGN KEY (room) REFERENCES rooms(id); 
ALTER TABLE legal_votes ADD CONSTRAINT legal_votes_room_fkey FOREIGN KEY (room_id) REFERENCES rooms(id); 

-- Change room's owner from type
-- BIGINT REFERENCES users(id_serial)
-- to
-- UUID REFERENCES users(id)
ALTER TABLE rooms RENAME owner TO old_owner;
ALTER TABLE rooms ADD COLUMN owner UUID REFERENCES users(id);
UPDATE rooms SET owner = users.id FROM users WHERE users.id_serial = old_owner;
ALTER TABLE rooms DROP COLUMN old_owner;
ALTER TABLE rooms ALTER COLUMN owner SET NOT NULL;

-- Change the id to id_serial, add id field
-- and change the primary key to id
ALTER TABLE invites RENAME COLUMN id TO id_serial;
ALTER TABLE invites RENAME COLUMN uuid TO id;
ALTER TABLE invites ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE invites DROP CONSTRAINT invites_pkey;
ALTER TABLE invites DROP CONSTRAINT invites_uuid_key;
ALTER TABLE invites ADD  PRIMARY KEY (id);

-- Change created_by/updated_by reference types from
-- BIGINT REFERENCES users(id_serial)
-- to
-- UUID REFERENCES users(id)
ALTER TABLE invites RENAME COLUMN created_by TO old_created_by;
ALTER TABLE invites RENAME COLUMN updated_by TO old_updated_by;
ALTER TABLE invites ADD COLUMN created_by UUID REFERENCES users(id);
ALTER TABLE invites ADD COLUMN updated_by UUID REFERENCES users(id);
UPDATE invites SET created_by = users.id FROM users WHERE users.id_serial = old_created_by;
UPDATE invites SET updated_by = users.id FROM users WHERE users.id_serial = old_updated_by;
ALTER TABLE invites DROP COLUMN old_created_by;
ALTER TABLE invites DROP COLUMN old_updated_by;
ALTER TABLE invites ALTER COLUMN created_by SET NOT NULL;
ALTER TABLE invites ALTER COLUMN updated_by SET NOT NULL;

-- Set default values for created/updated
ALTER TABLE invites ALTER COLUMN created SET DEFAULT now();
ALTER TABLE invites ALTER COLUMN updated SET DEFAULT now();


-- Add default for id and add id_serial field
ALTER TABLE legal_votes ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE legal_votes ADD COLUMN id_serial BIGSERIAL NOT NULL;

-- Change initiator reference types from
-- BIGINT REFERENCES users(id_serial)
-- to
-- UUID REFERENCES users(id)
ALTER TABLE legal_votes RENAME COLUMN initiator TO old_initiator;
ALTER TABLE legal_votes ADD COLUMN initiator UUID REFERENCES users(id);
UPDATE legal_votes SET initiator = users.id FROM users WHERE users.id_serial = old_initiator;
ALTER TABLE legal_votes DROP COLUMN old_initiator;
ALTER TABLE legal_votes ALTER COLUMN initiator SET NOT NULL;


-- Change the old stringified group id (string, now groups.name) in v0 and v1 in casbin_rule to the new group identifier
UPDATE casbin_rule
SET v1 = CONCAT('group:', subquery.id) 
FROM (SELECT id, name FROM groups) AS subquery
WHERE casbin_rule.v1 like 'group:%' AND SPLIT_PART(casbin_rule.v1, ':', 3) = subquery.name;

UPDATE casbin_rule
SET v0 = CONCAT('group:', subquery.id) 
FROM (SELECT id, name FROM groups) AS subquery      
WHERE casbin_rule.v0 like 'group:%' AND SPLIT_PART(casbin_rule.v0, ':', 3) = subquery.name;

