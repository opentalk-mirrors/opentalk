ALTER TABLE rooms DROP COLUMN wait_for_moderator;
ALTER TABLE rooms DROP COLUMN listen_only;
ALTER TABLE rooms ALTER password DROP NOT NULL;
UPDATE rooms SET password = NULL WHERE password = '';
