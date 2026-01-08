-- Migration 001: Initial Schema
-- This establishes the base database schema for MCP Hub

-- Central server registry
CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    command TEXT NOT NULL,
    args TEXT NOT NULL,
    env TEXT NOT NULL,
    tags TEXT,
    source_type TEXT,
    source_url TEXT,
    parent_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES servers(id) ON DELETE SET NULL
);

-- Client instances
CREATE TABLE IF NOT EXISTS client_instances (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    client_type TEXT NOT NULL,
    config_path TEXT NOT NULL,
    is_default INTEGER DEFAULT 0,
    last_synced TEXT,
    last_modified TEXT,
    created_at TEXT NOT NULL
);

-- Server-to-instance mapping
CREATE TABLE IF NOT EXISTS instance_servers (
    instance_id TEXT NOT NULL,
    server_id TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    PRIMARY KEY (instance_id, server_id),
    FOREIGN KEY (instance_id) REFERENCES client_instances(id) ON DELETE CASCADE,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

-- Config file backups
CREATE TABLE IF NOT EXISTS backups (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    backup_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (instance_id) REFERENCES client_instances(id) ON DELETE CASCADE
);

-- App settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Custom registries
CREATE TABLE IF NOT EXISTS custom_registries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    requires_auth INTEGER NOT NULL DEFAULT 0,
    cached_data TEXT,
    cached_at TEXT,
    created_at TEXT NOT NULL
);
