CREATE TYPE guest_access AS ENUM ('disabled', 'waiting_room', 'direct_access');

ALTER TABLE rooms ADD COLUMN guest_access guest_access NOT NULL DEFAULT 'direct_access';

UPDATE rooms SET guest_access = 'waiting_room' WHERE waiting_room = true;

ALTER TABLE rooms ADD CONSTRAINT room_access_check CHECK (NOT(waiting_room = true AND guest_access = 'direct_access'));