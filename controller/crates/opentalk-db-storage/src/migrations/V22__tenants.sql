
-- Create new tenant table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    oidc_tenant_id TEXT NOT NULL UNIQUE
);
CREATE INDEX tenants_oidc_tenant_id_index ON tenants(oidc_tenant_id);

-- Migrate current users and and resources to a new default tenant
DO $$
DECLARE
    DefaultTenantId UUID := gen_random_uuid();
BEGIN
    -- Create a new tenant with a "default" oidc-id
    INSERT INTO tenants VALUES (DefaultTenantId, DEFAULT, DEFAULT, 'OpenTalkDefaultTenant');

    -- Migrate users
    ALTER TABLE users ADD COLUMN tenant_id UUID REFERENCES tenants(id);
    UPDATE users SET tenant_id = DefaultTenantId;
    ALTER TABLE users ALTER COLUMN tenant_id SET NOT NULL;
    -- Make the combination of oidc_sub + tenant_id unique as it identifies a user inside a tenant
    ALTER TABLE users ADD UNIQUE(oidc_sub, tenant_id);

    -- Migrate groups
    ALTER TABLE groups ADD COLUMN tenant_id UUID REFERENCES tenants(id);
    UPDATE groups SET tenant_id = DefaultTenantId;
    ALTER TABLE groups ALTER COLUMN tenant_id SET NOT NULL;
    ALTER TABLE groups ADD UNIQUE(tenant_id, name);

    -- Migrate rooms
    ALTER TABLE rooms ADD COLUMN tenant_id UUID REFERENCES tenants(id);
    UPDATE rooms SET tenant_id = DefaultTenantId;
    ALTER TABLE rooms ALTER COLUMN tenant_id SET NOT NULL;

    -- Migrate assets
    ALTER TABLE assets ADD COLUMN tenant_id UUID REFERENCES tenants(id);
    UPDATE assets SET tenant_id = DefaultTenantId;
    ALTER TABLE assets ALTER COLUMN tenant_id SET NOT NULL;

    -- Migrate legal-votes
    ALTER TABLE legal_votes ADD COLUMN tenant_id UUID REFERENCES tenants(id);
    UPDATE legal_votes SET tenant_id = DefaultTenantId;
    ALTER TABLE legal_votes ALTER COLUMN tenant_id SET NOT NULL;

    -- Migrate events
    ALTER TABLE events ADD COLUMN tenant_id UUID REFERENCES tenants(id);
    UPDATE events SET tenant_id = DefaultTenantId;
    ALTER TABLE events ALTER COLUMN tenant_id SET NOT NULL;
END $$
