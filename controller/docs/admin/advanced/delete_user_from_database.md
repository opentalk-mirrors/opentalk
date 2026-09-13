# Deleting a User From the Database by Hand

In some cases it may be necessary to delete a user's information from the
database by hand. The development team is working on providing a
[job](../cli/jobs.md) in an upcoming version, until that is available this
manual can be used to perform the deletion.

## Remarks

- This manual works for `v24.1.0` and later versions. No promises are made for
  any earlier releases.
- While the user is only deleted from the database, no data will be deleted from
  external services such as the file storage ([MinIO](../core/minio.md)) or
  a configured [shared folder system](./additional_services/shared_folder.md). You will get a list
  of these entries though, so these can be deleted by hand on the external
  systems.
- When following the manual, database entries created by the affected user will
  be deleted as well. This includes events, rooms, references to file assets or
  shared folders.
- Some database tables contain a `updated_by` field which can reference users.
  This will be set to the value of the `created_by` field in the same tables.
- If you need some background information on the database tables and their
  relations, you can find them in the corresponding section of the
  [developer documentation](https://docs.opentalk.eu/developer/controller/database/).
- If any errors come up during execution of the snippet, no changes are
  committed to the database because the code is executed in a transaction that
  gets rolled back in that case.

## Instructions

This is the `SQL` snippet that needs to be executed in the
[Postgres database](../core/database.md) of the {{ product_name }} controller.

Please note that you need to perform a few changes before executing it, these
are explained below. It is best to copy the whole block into a text editor,
change it as needed, and then paste it into the Postgres cli or any other tool
that can execute `SQL` statements on the database.

### SQL

```sql
BEGIN;

-- Declare the user_id to be deleted
DO $$
DECLARE
    -- ----------------------------------------------------------------- --
    -- Change the `<userid>` to the real id of the user in the database.
    -- This would then e.g. look like:
    --
    -- target_user_id UUID := '5f5008b7-cd51-436b-a724-6a3709366726';
    -- ----------------------------------------------------------------- --
    target_user_id UUID := '<userid>';

BEGIN
    -- Create temporary tables to store rows before deletion
    CREATE TEMP TABLE temp_room_assets AS
    SELECT * FROM room_assets WHERE room_id IN (SELECT id FROM rooms WHERE created_by = target_user_id);

    CREATE TEMP TABLE temp_event_shared_folders AS
    SELECT * FROM event_shared_folders WHERE event_id IN (SELECT id FROM events where created_by = target_user_id);

    -- Update and delete entries related to the target user
    UPDATE events SET updated_by = created_by WHERE updated_by = target_user_id;
    DELETE FROM event_favorites WHERE user_id = target_user_id;
    DELETE FROM event_exceptions WHERE created_by = target_user_id;
    DELETE FROM event_invites WHERE invitee = target_user_id OR created_by = target_user_id;
    DELETE FROM event_email_invites WHERE created_by = target_user_id;
    DELETE FROM events WHERE created_by = target_user_id;

    UPDATE invites SET updated_by = created_by WHERE updated_by = target_user_id;
    DELETE FROM invites WHERE created_by = target_user_id;

    DELETE FROM module_resources WHERE created_by = target_user_id;
    DELETE FROM user_groups WHERE user_id = target_user_id;

    DELETE FROM room_assets WHERE room_id IN (SELECT id FROM rooms WHERE created_by = target_user_id);
    DELETE FROM rooms WHERE created_by = target_user_id;

    DELETE FROM users WHERE id = target_user_id;
END $$;

-- Select the stored data for review
SELECT * FROM temp_room_assets;
SELECT * FROM temp_event_shared_folders;

-- ----------------------------------------------------------------- --
-- Change this to `ROLLBACK` if you just want to do a dry run, or if you
-- only want to see the list of referenced assets and shared folders
-- without deleting the user.
-- This would then look like:
--
-- ROLLBACK;
-- ----------------------------------------------------------------- --
COMMIT;
```

### Changes needed in the SQL stataments

- Change the `<userid>` to the real user id as commented inline inside the script.
  The required user id can be retrieved by an appropriate `SELECT` statement on the
  `users` table.
- If you want to perform a dry-run or only see the list of the referenced assets,
  change the last line containing `COMMIT;` to `ROLLBACK;`.

### Explanation of the SQL statemants

1. **Declaration of the user ID:** The user ID to be deleted is declared as `target_user_id`.
    - Example: `target_user_id := '8c8da1d3-62f2-4f04-a9f8-4a4145d95d1e'`
2. **Creation of temporary tables:** Two temporary tables, `temp_room_assets` and `temp_event_shared_folders`, are created to store relevant data before deletion.
3. **Update and deletion of database contents:** Entries related to the target user are updated or deleted in several tables, including `events`, `event_favorites`, `event_exceptions`, `event_invites`, `event_email_invites`, `invites`, `module_resources`, `user_groups`, `rooms`, `room_assets` and `users`.
4. **Selection of stored data:** The data stored in temporary tables is returned for review.
5. **Commit:** If no errors are detected, the changes are written to the database.
