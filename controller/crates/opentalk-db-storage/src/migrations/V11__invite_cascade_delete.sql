--- Set the room fkey on invites to DELETE CASCADE. If you can delete the room you certainly have the right to delete the invites for that room.
ALTER TABLE invites DROP CONSTRAINT invites_room_new_fkey;

ALTER TABLE invites ADD CONSTRAINT invites_room_new_fkey FOREIGN KEY (room) REFERENCES rooms (id) ON DELETE CASCADE;
ALTER TABLE invites ALTER room SET NOT NULL;

--- Delete all rules mentioning /invites/:inviteid as the main resource id for the invites is now /room/:roomid/invites/:inviteid
DELETE FROM casbin_rule WHERE v1 LIKE '/invites/%'
