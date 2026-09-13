ALTER TABLE sip_configs DROP CONSTRAINT sip_config_room_fkey;

ALTER TABLE sip_configs ADD CONSTRAINT sip_config_room_fkey
    FOREIGN KEY (room)
    REFERENCES rooms (id)
    ON DELETE CASCADE;
