-- Purge the casbin_rule table of the administrator role as its implementation is incomplete and will interfere with
-- a multi-tenancy implementation.
DELETE FROM casbin_rule WHERE
    v0 = 'role::administrator' OR -- remove any definitions of the role
    v1 = 'role::administrator';   -- remove any assignements to the role
