-- Migration 002: Add Indexes
-- Add indexes for better query performance

-- Index for server lookups by name (case-insensitive searches)
CREATE INDEX IF NOT EXISTS idx_servers_name ON servers(name);

-- Index for filtering servers by source type
CREATE INDEX IF NOT EXISTS idx_servers_source_type ON servers(source_type);

-- Index for filtering client instances by type
CREATE INDEX IF NOT EXISTS idx_client_instances_type ON client_instances(client_type);

-- Index for backup lookups by instance
CREATE INDEX IF NOT EXISTS idx_backups_instance ON backups(instance_id);

-- Index for instance_servers foreign key lookups
CREATE INDEX IF NOT EXISTS idx_instance_servers_server ON instance_servers(server_id);

-- Index for custom registry lookups by name
CREATE INDEX IF NOT EXISTS idx_custom_registries_name ON custom_registries(name);
