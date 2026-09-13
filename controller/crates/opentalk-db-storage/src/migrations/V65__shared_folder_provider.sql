-- SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
--
-- SPDX-License-Identifier: EUPL-1.2

-- Replace the provider-specific `write_share_id`/`read_share_id` columns with a
-- single self-describing `provider_data` JSONB column.
--
-- The JSON payload is internally tagged through a `type` discriminator field. At
-- the time of this migration only the Nextcloud backend has been released, so
-- every existing row is a Nextcloud shared folder.

ALTER TABLE event_shared_folders
    ADD COLUMN provider_data JSONB;

UPDATE event_shared_folders
SET provider_data = jsonb_build_object(
        'type', 'nextcloud',
        'write_share_id', write_share_id,
        'read_share_id', read_share_id
    );

ALTER TABLE event_shared_folders
    ALTER COLUMN provider_data SET NOT NULL,
    DROP COLUMN write_share_id,
    DROP COLUMN read_share_id;
