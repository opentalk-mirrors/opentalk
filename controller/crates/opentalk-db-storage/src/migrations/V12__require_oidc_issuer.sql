--- DELETE all user_group rows where a group without oidc issuer is referenced 
DELETE FROM user_groups USING groups 
WHERE user_groups.group_id = groups.id AND groups.oidc_issuer IS NULL;

--- DELETE all groups that have no oidc_issuer
DELETE FROM groups WHERE groups.oidc_issuer IS NULL;

--- Make the groups oidc_issuer non-nullable
ALTER TABLE groups ALTER oidc_issuer SET NOT NULL;

--- Make the users oidc_issuer non-nullable
--- NOTE: This may require manual fixing if users without oidc issuer exist
ALTER TABLE users ALTER oidc_issuer SET NOT NULL;
