use chrono::{DateTime, Utc};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OpenFlags, Result as SqlResult};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::models::{
    ClientInstance, ClientType, ConfigBackup, McpServer, ServerSource, SourceType,
};

pub mod migrations;

/// A custom registry added by the user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRegistry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub requires_auth: bool,
    pub cached_data: Option<String>,
    pub cached_at: Option<String>,
    /// SHA-256 hash of cached_data for integrity verification
    pub content_hash: Option<String>,
    pub created_at: String,
}

/// Connection pool size for the database
const POOL_SIZE: u32 = 10;

/// Connection timeout in seconds
const CONNECTION_TIMEOUT_SECS: u64 = 30;

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(path: PathBuf) -> SqlResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // Create connection manager with optimized SQLite flags
        // SQLITE_OPEN_NO_MUTEX allows concurrent access since we're using r2d2 for pooling
        let manager = SqliteConnectionManager::file(&path)
            .with_flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            );

        // Build the connection pool
        let pool = Pool::builder()
            .max_size(POOL_SIZE)
            .connection_timeout(std::time::Duration::from_secs(CONNECTION_TIMEOUT_SECS))
            .build(manager)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        // Set restrictive file permissions on Unix (0600 = owner read/write only)
        // This prevents other users from reading the database which may contain
        // sensitive configuration data
        #[cfg(unix)]
        {
            if let Ok(metadata) = std::fs::metadata(&path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&path, perms);
            }
        }

        let db = Self { pool };
        db.init_schema()?;
        Ok(db)
    }

    /// Get a connection from the pool
    fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, rusqlite::Error> {
        self.pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
    }

    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.get_conn()?;

        // Run all pending migrations using the versioned migration system
        // This replaces the previous inline schema creation with a proper
        // migration system that tracks schema versions
        migrations::run_migrations(&conn)?;

        Ok(())
    }

    // ==================== Server CRUD ====================

    pub fn create_server(&self, server: &McpServer) -> SqlResult<()> {
        let conn = self.get_conn()?;

        let args_json = serde_json::to_string(&server.args).unwrap_or_default();
        let env_json = serde_json::to_string(&server.env).unwrap_or_default();
        let tags_json = serde_json::to_string(&server.tags).unwrap_or_default();
        let source_type = server
            .source
            .as_ref()
            .map(|s| match s.source_type {
                SourceType::Manual => "manual",
                SourceType::Imported => "imported",
                SourceType::Registry => "registry",
            })
            .unwrap_or("manual");
        let source_url = server.source.as_ref().and_then(|s| s.url.clone());

        conn.execute(
            "INSERT INTO servers (id, name, description, command, args, env, tags, source_type, source_url, parent_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                server.id,
                server.name,
                server.description,
                server.command,
                args_json,
                env_json,
                tags_json,
                source_type,
                source_url,
                server.parent_id,
                server.created_at.to_rfc3339(),
                server.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn get_server(&self, id: &str) -> SqlResult<Option<McpServer>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, command, args, env, tags, source_type, source_url, parent_id, created_at, updated_at
             FROM servers WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(Self::row_to_server(row)?)
        });

        match result {
            Ok(server) => Ok(Some(server)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_all_servers(&self) -> SqlResult<Vec<McpServer>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, command, args, env, tags, source_type, source_url, parent_id, created_at, updated_at
             FROM servers ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| Self::row_to_server(row))?;

        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }

        Ok(servers)
    }

    pub fn update_server(&self, server: &McpServer) -> SqlResult<()> {
        let conn = self.get_conn()?;

        let args_json = serde_json::to_string(&server.args).unwrap_or_default();
        let env_json = serde_json::to_string(&server.env).unwrap_or_default();
        let tags_json = serde_json::to_string(&server.tags).unwrap_or_default();
        let source_type = server
            .source
            .as_ref()
            .map(|s| match s.source_type {
                SourceType::Manual => "manual",
                SourceType::Imported => "imported",
                SourceType::Registry => "registry",
            })
            .unwrap_or("manual");
        let source_url = server.source.as_ref().and_then(|s| s.url.clone());

        conn.execute(
            "UPDATE servers SET name = ?2, description = ?3, command = ?4, args = ?5, env = ?6,
             tags = ?7, source_type = ?8, source_url = ?9, parent_id = ?10, updated_at = ?11 WHERE id = ?1",
            params![
                server.id,
                server.name,
                server.description,
                server.command,
                args_json,
                env_json,
                tags_json,
                source_type,
                source_url,
                server.parent_id,
                server.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn delete_server(&self, id: &str) -> SqlResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn row_to_server(row: &rusqlite::Row) -> SqlResult<McpServer> {
        let args_str: String = row.get(4)?;
        let env_str: String = row.get(5)?;
        let tags_str: Option<String> = row.get(6)?;
        let source_type: Option<String> = row.get(7)?;
        let source_url: Option<String> = row.get(8)?;
        let parent_id: Option<String> = row.get(9)?;
        let created_at_str: String = row.get(10)?;
        let updated_at_str: String = row.get(11)?;

        Ok(McpServer {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            command: row.get(3)?,
            args: serde_json::from_str(&args_str).unwrap_or_default(),
            env: serde_json::from_str(&env_str).unwrap_or_default(),
            tags: tags_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            source: source_type.map(|st| ServerSource {
                source_type: match st.as_str() {
                    "imported" => SourceType::Imported,
                    "registry" => SourceType::Registry,
                    _ => SourceType::Manual,
                },
                url: source_url,
            }),
            parent_id,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    // ==================== Client Instance CRUD ====================

    pub fn create_instance(&self, instance: &ClientInstance) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "INSERT INTO client_instances (id, name, client_type, config_path, is_default, last_synced, last_modified, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                instance.id,
                instance.name,
                instance.client_type.as_str(),
                instance.config_path,
                instance.is_default as i32,
                instance.last_synced.map(|dt| dt.to_rfc3339()),
                instance.last_modified.map(|dt| dt.to_rfc3339()),
                instance.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn get_instance(&self, id: &str) -> SqlResult<Option<ClientInstance>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, client_type, config_path, is_default, last_synced, last_modified, created_at
             FROM client_instances WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| self.row_to_instance(row));

        match result {
            Ok(instance) => Ok(Some(instance)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_all_instances(&self) -> SqlResult<Vec<ClientInstance>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, client_type, config_path, is_default, last_synced, last_modified, created_at
             FROM client_instances ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| self.row_to_instance(row))?;

        let mut instances = Vec::new();
        for row in rows {
            instances.push(row?);
        }

        // Load enabled servers for each instance
        drop(stmt);
        drop(conn);

        let mut instances_with_servers = Vec::new();
        for mut instance in instances {
            instance.enabled_servers = self.get_enabled_servers_for_instance(&instance.id)?;
            instances_with_servers.push(instance);
        }

        Ok(instances_with_servers)
    }

    pub fn update_instance(&self, instance: &ClientInstance) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "UPDATE client_instances SET name = ?2, client_type = ?3, config_path = ?4,
             is_default = ?5, last_synced = ?6, last_modified = ?7 WHERE id = ?1",
            params![
                instance.id,
                instance.name,
                instance.client_type.as_str(),
                instance.config_path,
                instance.is_default as i32,
                instance.last_synced.map(|dt| dt.to_rfc3339()),
                instance.last_modified.map(|dt| dt.to_rfc3339()),
            ],
        )?;

        Ok(())
    }

    pub fn delete_instance(&self, id: &str) -> SqlResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM client_instances WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn row_to_instance(&self, row: &rusqlite::Row) -> SqlResult<ClientInstance> {
        let client_type_str: String = row.get(2)?;
        let is_default: i32 = row.get(4)?;
        let last_synced_str: Option<String> = row.get(5)?;
        let last_modified_str: Option<String> = row.get(6)?;
        let created_at_str: String = row.get(7)?;

        Ok(ClientInstance {
            id: row.get(0)?,
            name: row.get(1)?,
            client_type: ClientType::from_str(&client_type_str).unwrap_or(ClientType::Custom),
            config_path: row.get(3)?,
            enabled_servers: Vec::new(), // Loaded separately
            is_default: is_default != 0,
            last_synced: last_synced_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            last_modified: last_modified_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    // ==================== Instance-Server Mapping ====================

    pub fn set_server_enabled_for_instance(
        &self,
        instance_id: &str,
        server_id: &str,
        enabled: bool,
    ) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "INSERT INTO instance_servers (instance_id, server_id, enabled) VALUES (?1, ?2, ?3)
             ON CONFLICT(instance_id, server_id) DO UPDATE SET enabled = ?3",
            params![instance_id, server_id, enabled as i32],
        )?;

        // Update last_modified timestamp on the instance
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE client_instances SET last_modified = ?1 WHERE id = ?2",
            params![now, instance_id],
        )?;

        Ok(())
    }

    pub fn get_enabled_servers_for_instance(&self, instance_id: &str) -> SqlResult<Vec<String>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT server_id FROM instance_servers WHERE instance_id = ?1 AND enabled = 1",
        )?;

        let rows = stmt.query_map(params![instance_id], |row| row.get(0))?;

        let mut server_ids = Vec::new();
        for row in rows {
            server_ids.push(row?);
        }

        Ok(server_ids)
    }

    #[allow(dead_code)]
    pub fn remove_server_from_instance(&self, instance_id: &str, server_id: &str) -> SqlResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "DELETE FROM instance_servers WHERE instance_id = ?1 AND server_id = ?2",
            params![instance_id, server_id],
        )?;
        Ok(())
    }

    // ==================== Backups ====================

    pub fn create_backup(&self, backup: &ConfigBackup) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "INSERT INTO backups (id, instance_id, backup_path, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                backup.id,
                backup.instance_id,
                backup.backup_path,
                backup.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn get_backups_for_instance(&self, instance_id: &str) -> SqlResult<Vec<ConfigBackup>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, instance_id, backup_path, created_at FROM backups
             WHERE instance_id = ?1 ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![instance_id], |row| {
            let created_at_str: String = row.get(3)?;
            Ok(ConfigBackup {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                backup_path: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut backups = Vec::new();
        for row in rows {
            backups.push(row?);
        }

        Ok(backups)
    }

    #[allow(dead_code)]
    pub fn delete_old_backups(&self, instance_id: &str, keep_count: usize) -> SqlResult<()> {
        let conn = self.get_conn()?;

        // Get all backups sorted by date
        let mut stmt = conn.prepare(
            "SELECT id FROM backups WHERE instance_id = ?1 ORDER BY created_at DESC",
        )?;

        let backup_ids: Vec<String> = stmt
            .query_map(params![instance_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Delete old ones
        if backup_ids.len() > keep_count {
            for id in backup_ids.into_iter().skip(keep_count) {
                conn.execute("DELETE FROM backups WHERE id = ?1", params![id])?;
            }
        }

        Ok(())
    }

    // ==================== Settings ====================

    pub fn get_setting(&self, key: &str) -> SqlResult<Option<String>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;

        match stmt.query_row(params![key], |row| row.get(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;

        Ok(())
    }

    // ==================== Custom Registries ====================

    pub fn create_custom_registry(&self, registry: &CustomRegistry) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "INSERT INTO custom_registries (id, name, url, description, icon, requires_auth, cached_data, cached_at, content_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                registry.id,
                registry.name,
                registry.url,
                registry.description,
                registry.icon,
                registry.requires_auth as i32,
                registry.cached_data,
                registry.cached_at,
                registry.content_hash,
                registry.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_custom_registries(&self) -> SqlResult<Vec<CustomRegistry>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, url, description, icon, requires_auth, cached_data, cached_at, content_hash, created_at
             FROM custom_registries ORDER BY name",
        )?;

        let registries = stmt.query_map([], |row| {
            Ok(CustomRegistry {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                requires_auth: row.get::<_, i32>(5)? != 0,
                cached_data: row.get(6)?,
                cached_at: row.get(7)?,
                content_hash: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        registries.collect()
    }

    pub fn get_custom_registry(&self, id: &str) -> SqlResult<Option<CustomRegistry>> {
        let conn = self.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, url, description, icon, requires_auth, cached_data, cached_at, content_hash, created_at
             FROM custom_registries WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(CustomRegistry {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                requires_auth: row.get::<_, i32>(5)? != 0,
                cached_data: row.get(6)?,
                cached_at: row.get(7)?,
                content_hash: row.get(8)?,
                created_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_custom_registry(&self, registry: &CustomRegistry) -> SqlResult<()> {
        let conn = self.get_conn()?;

        conn.execute(
            "UPDATE custom_registries
             SET name = ?2, url = ?3, description = ?4, icon = ?5, requires_auth = ?6, cached_data = ?7, cached_at = ?8, content_hash = ?9
             WHERE id = ?1",
            params![
                registry.id,
                registry.name,
                registry.url,
                registry.description,
                registry.icon,
                registry.requires_auth as i32,
                registry.cached_data,
                registry.cached_at,
                registry.content_hash,
            ],
        )?;
        Ok(())
    }

    pub fn delete_custom_registry(&self, id: &str) -> SqlResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM custom_registries WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_custom_registry_cache(
        &self,
        id: &str,
        cached_data: &str,
        cached_at: &str,
        content_hash: &str,
    ) -> SqlResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE custom_registries SET cached_data = ?2, cached_at = ?3, content_hash = ?4 WHERE id = ?1",
            params![id, cached_data, cached_at, content_hash],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test database in a temporary directory
    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).expect("Failed to create database");
        (db, temp_dir)
    }

    // ==================== Server CRUD Tests ====================

    #[test]
    fn test_create_and_get_server() {
        let (db, _temp) = create_test_db();

        let server = McpServer::new(
            "Test Server".to_string(),
            "npx".to_string(),
            vec!["-y".to_string(), "@test/server".to_string()],
        );

        db.create_server(&server).expect("Failed to create server");

        let retrieved = db
            .get_server(&server.id)
            .expect("Failed to get server")
            .expect("Server not found");

        assert_eq!(retrieved.id, server.id);
        assert_eq!(retrieved.name, "Test Server");
        assert_eq!(retrieved.command, "npx");
        assert_eq!(retrieved.args, vec!["-y", "@test/server"]);
    }

    #[test]
    fn test_get_nonexistent_server() {
        let (db, _temp) = create_test_db();

        let result = db
            .get_server("nonexistent-id")
            .expect("Query should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn test_get_all_servers() {
        let (db, _temp) = create_test_db();

        // Create multiple servers
        let server1 = McpServer::new(
            "Alpha Server".to_string(),
            "npx".to_string(),
            vec!["-y".to_string(), "@test/alpha".to_string()],
        );
        let server2 = McpServer::new(
            "Beta Server".to_string(),
            "node".to_string(),
            vec!["server.js".to_string()],
        );

        db.create_server(&server1).unwrap();
        db.create_server(&server2).unwrap();

        let servers = db.get_all_servers().expect("Failed to get servers");

        assert_eq!(servers.len(), 2);
        // Should be ordered by name
        assert_eq!(servers[0].name, "Alpha Server");
        assert_eq!(servers[1].name, "Beta Server");
    }

    #[test]
    fn test_update_server() {
        let (db, _temp) = create_test_db();

        let mut server = McpServer::new(
            "Original Name".to_string(),
            "npx".to_string(),
            vec![],
        );
        db.create_server(&server).unwrap();

        // Update the server
        server.name = "Updated Name".to_string();
        server.description = Some("New description".to_string());
        server.args = vec!["new-arg".to_string()];

        db.update_server(&server).expect("Failed to update server");

        let retrieved = db
            .get_server(&server.id)
            .unwrap()
            .expect("Server not found");

        assert_eq!(retrieved.name, "Updated Name");
        assert_eq!(retrieved.description, Some("New description".to_string()));
        assert_eq!(retrieved.args, vec!["new-arg"]);
    }

    #[test]
    fn test_delete_server() {
        let (db, _temp) = create_test_db();

        let server = McpServer::new(
            "To Delete".to_string(),
            "npx".to_string(),
            vec![],
        );
        db.create_server(&server).unwrap();

        // Verify it exists
        assert!(db.get_server(&server.id).unwrap().is_some());

        // Delete it
        db.delete_server(&server.id).expect("Failed to delete server");

        // Verify it's gone
        assert!(db.get_server(&server.id).unwrap().is_none());
    }

    #[test]
    fn test_server_with_env_vars() {
        let (db, _temp) = create_test_db();

        let mut server = McpServer::new(
            "Server With Env".to_string(),
            "node".to_string(),
            vec!["server.js".to_string()],
        );
        server.env.insert("API_KEY".to_string(), "secret".to_string());
        server.env.insert("DEBUG".to_string(), "true".to_string());

        db.create_server(&server).unwrap();

        let retrieved = db.get_server(&server.id).unwrap().unwrap();

        assert_eq!(retrieved.env.len(), 2);
        assert_eq!(retrieved.env.get("API_KEY"), Some(&"secret".to_string()));
        assert_eq!(retrieved.env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_server_with_tags() {
        let (db, _temp) = create_test_db();

        let mut server = McpServer::new(
            "Tagged Server".to_string(),
            "npx".to_string(),
            vec![],
        );
        server.tags = vec!["database".to_string(), "sql".to_string()];

        db.create_server(&server).unwrap();

        let retrieved = db.get_server(&server.id).unwrap().unwrap();

        assert_eq!(retrieved.tags, vec!["database", "sql"]);
    }

    #[test]
    fn test_server_source_types() {
        let (db, _temp) = create_test_db();

        // Test manual source (default)
        let manual = McpServer::new("Manual".to_string(), "cmd".to_string(), vec![]);
        db.create_server(&manual).unwrap();
        let retrieved = db.get_server(&manual.id).unwrap().unwrap();
        assert_eq!(
            retrieved.source.as_ref().unwrap().source_type,
            SourceType::Manual
        );

        // Test registry source
        let mut registry = McpServer::new("Registry".to_string(), "cmd".to_string(), vec![]);
        registry.source = Some(ServerSource {
            source_type: SourceType::Registry,
            url: Some("https://registry.example.com/server".to_string()),
        });
        db.create_server(&registry).unwrap();
        let retrieved = db.get_server(&registry.id).unwrap().unwrap();
        assert_eq!(
            retrieved.source.as_ref().unwrap().source_type,
            SourceType::Registry
        );
        assert_eq!(
            retrieved.source.as_ref().unwrap().url,
            Some("https://registry.example.com/server".to_string())
        );

        // Test imported source
        let mut imported = McpServer::new("Imported".to_string(), "cmd".to_string(), vec![]);
        imported.source = Some(ServerSource {
            source_type: SourceType::Imported,
            url: Some("/path/to/config.json".to_string()),
        });
        db.create_server(&imported).unwrap();
        let retrieved = db.get_server(&imported.id).unwrap().unwrap();
        assert_eq!(
            retrieved.source.as_ref().unwrap().source_type,
            SourceType::Imported
        );
    }

    // ==================== Instance CRUD Tests ====================

    #[test]
    fn test_create_and_get_instance() {
        let (db, _temp) = create_test_db();

        let mut instance = ClientInstance::new(
            "Test Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path/to/config.json".to_string(),
        );
        instance.is_default = true;

        db.create_instance(&instance)
            .expect("Failed to create instance");

        let retrieved = db
            .get_instance(&instance.id)
            .expect("Failed to get instance")
            .expect("Instance not found");

        assert_eq!(retrieved.id, instance.id);
        assert_eq!(retrieved.name, "Test Instance");
        assert_eq!(retrieved.client_type, ClientType::ClaudeDesktop);
        assert_eq!(retrieved.config_path, "/path/to/config.json");
        assert!(retrieved.is_default);
    }

    #[test]
    fn test_get_all_instances() {
        let (db, _temp) = create_test_db();

        let instance1 = ClientInstance::new(
            "Alpha Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path/a.json".to_string(),
        );
        let instance2 = ClientInstance::new(
            "Beta Instance".to_string(),
            ClientType::Cursor,
            "/path/b.json".to_string(),
        );

        db.create_instance(&instance1).unwrap();
        db.create_instance(&instance2).unwrap();

        let instances = db.get_all_instances().expect("Failed to get instances");

        assert_eq!(instances.len(), 2);
        // Should be ordered by name
        assert_eq!(instances[0].name, "Alpha Instance");
        assert_eq!(instances[1].name, "Beta Instance");
    }

    #[test]
    fn test_update_instance() {
        let (db, _temp) = create_test_db();

        let mut instance = ClientInstance::new(
            "Original".to_string(),
            ClientType::ClaudeDesktop,
            "/old/path.json".to_string(),
        );
        db.create_instance(&instance).unwrap();

        // Update
        instance.name = "Updated".to_string();
        instance.config_path = "/new/path.json".to_string();
        instance.is_default = true;

        db.update_instance(&instance).expect("Failed to update");

        let retrieved = db.get_instance(&instance.id).unwrap().unwrap();

        assert_eq!(retrieved.name, "Updated");
        assert_eq!(retrieved.config_path, "/new/path.json");
        assert!(retrieved.is_default);
    }

    #[test]
    fn test_delete_instance() {
        let (db, _temp) = create_test_db();

        let instance = ClientInstance::new(
            "To Delete".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );
        db.create_instance(&instance).unwrap();

        assert!(db.get_instance(&instance.id).unwrap().is_some());

        db.delete_instance(&instance.id)
            .expect("Failed to delete");

        assert!(db.get_instance(&instance.id).unwrap().is_none());
    }

    // ==================== Instance-Server Mapping Tests ====================

    #[test]
    fn test_enable_server_for_instance() {
        let (db, _temp) = create_test_db();

        let server = McpServer::new("Server".to_string(), "cmd".to_string(), vec![]);
        let instance = ClientInstance::new(
            "Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );

        db.create_server(&server).unwrap();
        db.create_instance(&instance).unwrap();

        // Enable server for instance
        db.set_server_enabled_for_instance(&instance.id, &server.id, true)
            .expect("Failed to enable server");

        let enabled = db
            .get_enabled_servers_for_instance(&instance.id)
            .expect("Failed to get enabled servers");

        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0], server.id);
    }

    #[test]
    fn test_disable_server_for_instance() {
        let (db, _temp) = create_test_db();

        let server = McpServer::new("Server".to_string(), "cmd".to_string(), vec![]);
        let instance = ClientInstance::new(
            "Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );

        db.create_server(&server).unwrap();
        db.create_instance(&instance).unwrap();

        // Enable then disable
        db.set_server_enabled_for_instance(&instance.id, &server.id, true)
            .unwrap();
        db.set_server_enabled_for_instance(&instance.id, &server.id, false)
            .unwrap();

        let enabled = db
            .get_enabled_servers_for_instance(&instance.id)
            .unwrap();

        assert!(enabled.is_empty());
    }

    #[test]
    fn test_multiple_servers_for_instance() {
        let (db, _temp) = create_test_db();

        let server1 = McpServer::new("Server1".to_string(), "cmd".to_string(), vec![]);
        let server2 = McpServer::new("Server2".to_string(), "cmd".to_string(), vec![]);
        let server3 = McpServer::new("Server3".to_string(), "cmd".to_string(), vec![]);
        let instance = ClientInstance::new(
            "Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );

        db.create_server(&server1).unwrap();
        db.create_server(&server2).unwrap();
        db.create_server(&server3).unwrap();
        db.create_instance(&instance).unwrap();

        // Enable two servers
        db.set_server_enabled_for_instance(&instance.id, &server1.id, true)
            .unwrap();
        db.set_server_enabled_for_instance(&instance.id, &server3.id, true)
            .unwrap();

        let enabled = db
            .get_enabled_servers_for_instance(&instance.id)
            .unwrap();

        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&server1.id));
        assert!(!enabled.contains(&server2.id));
        assert!(enabled.contains(&server3.id));
    }

    #[test]
    fn test_enabled_servers_included_in_get_all_instances() {
        let (db, _temp) = create_test_db();

        let server1 = McpServer::new("Server1".to_string(), "cmd".to_string(), vec![]);
        let server2 = McpServer::new("Server2".to_string(), "cmd".to_string(), vec![]);
        let instance = ClientInstance::new(
            "Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );

        db.create_server(&server1).unwrap();
        db.create_server(&server2).unwrap();
        db.create_instance(&instance).unwrap();

        db.set_server_enabled_for_instance(&instance.id, &server1.id, true)
            .unwrap();
        db.set_server_enabled_for_instance(&instance.id, &server2.id, true)
            .unwrap();

        let instances = db.get_all_instances().unwrap();

        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].enabled_servers.len(), 2);
    }

    // ==================== Backup Tests ====================

    #[test]
    fn test_create_and_get_backups() {
        let (db, _temp) = create_test_db();

        let instance = ClientInstance::new(
            "Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );
        db.create_instance(&instance).unwrap();

        let backup = ConfigBackup::new(instance.id.clone(), "/backup/path.json".to_string());
        db.create_backup(&backup).expect("Failed to create backup");

        let backups = db
            .get_backups_for_instance(&instance.id)
            .expect("Failed to get backups");

        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].instance_id, instance.id);
        assert_eq!(backups[0].backup_path, "/backup/path.json");
    }

    #[test]
    fn test_backups_ordered_by_date() {
        let (db, _temp) = create_test_db();

        let instance = ClientInstance::new(
            "Instance".to_string(),
            ClientType::ClaudeDesktop,
            "/path.json".to_string(),
        );
        db.create_instance(&instance).unwrap();

        // Create multiple backups
        let backup1 = ConfigBackup::new(instance.id.clone(), "/backup/1.json".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let backup2 = ConfigBackup::new(instance.id.clone(), "/backup/2.json".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        let backup3 = ConfigBackup::new(instance.id.clone(), "/backup/3.json".to_string());

        db.create_backup(&backup1).unwrap();
        db.create_backup(&backup2).unwrap();
        db.create_backup(&backup3).unwrap();

        let backups = db.get_backups_for_instance(&instance.id).unwrap();

        assert_eq!(backups.len(), 3);
        // Should be ordered by date descending (newest first)
        assert_eq!(backups[0].backup_path, "/backup/3.json");
        assert_eq!(backups[2].backup_path, "/backup/1.json");
    }

    // ==================== Settings Tests ====================

    #[test]
    fn test_get_and_set_setting() {
        let (db, _temp) = create_test_db();

        // Setting doesn't exist initially
        let result = db.get_setting("theme").expect("Query should succeed");
        assert!(result.is_none());

        // Set the setting
        db.set_setting("theme", "dark").expect("Failed to set");

        // Get it back
        let value = db.get_setting("theme").unwrap().unwrap();
        assert_eq!(value, "dark");
    }

    #[test]
    fn test_update_setting() {
        let (db, _temp) = create_test_db();

        db.set_setting("key", "value1").unwrap();
        db.set_setting("key", "value2").unwrap();

        let value = db.get_setting("key").unwrap().unwrap();
        assert_eq!(value, "value2");
    }

    // ==================== Custom Registry Tests ====================

    #[test]
    fn test_create_and_get_custom_registry() {
        let (db, _temp) = create_test_db();

        let registry = CustomRegistry {
            id: "reg-1".to_string(),
            name: "My Registry".to_string(),
            url: "https://registry.example.com".to_string(),
            description: Some("Test registry".to_string()),
            icon: None,
            requires_auth: false,
            cached_data: None,
            cached_at: None,
            content_hash: None,
            created_at: Utc::now().to_rfc3339(),
        };

        db.create_custom_registry(&registry)
            .expect("Failed to create registry");

        let retrieved = db
            .get_custom_registry("reg-1")
            .expect("Query failed")
            .expect("Registry not found");

        assert_eq!(retrieved.name, "My Registry");
        assert_eq!(retrieved.url, "https://registry.example.com");
        assert_eq!(retrieved.description, Some("Test registry".to_string()));
    }

    #[test]
    fn test_update_custom_registry() {
        let (db, _temp) = create_test_db();

        let mut registry = CustomRegistry {
            id: "reg-1".to_string(),
            name: "Original".to_string(),
            url: "https://old.url".to_string(),
            description: None,
            icon: None,
            requires_auth: false,
            cached_data: None,
            cached_at: None,
            content_hash: None,
            created_at: Utc::now().to_rfc3339(),
        };

        db.create_custom_registry(&registry).unwrap();

        registry.name = "Updated".to_string();
        registry.url = "https://new.url".to_string();
        registry.requires_auth = true;

        db.update_custom_registry(&registry)
            .expect("Failed to update");

        let retrieved = db.get_custom_registry("reg-1").unwrap().unwrap();

        assert_eq!(retrieved.name, "Updated");
        assert_eq!(retrieved.url, "https://new.url");
        assert!(retrieved.requires_auth);
    }

    #[test]
    fn test_delete_custom_registry() {
        let (db, _temp) = create_test_db();

        let registry = CustomRegistry {
            id: "reg-1".to_string(),
            name: "To Delete".to_string(),
            url: "https://example.com".to_string(),
            description: None,
            icon: None,
            requires_auth: false,
            cached_data: None,
            cached_at: None,
            content_hash: None,
            created_at: Utc::now().to_rfc3339(),
        };

        db.create_custom_registry(&registry).unwrap();
        assert!(db.get_custom_registry("reg-1").unwrap().is_some());

        db.delete_custom_registry("reg-1").expect("Failed to delete");

        assert!(db.get_custom_registry("reg-1").unwrap().is_none());
    }

    #[test]
    fn test_update_registry_cache() {
        let (db, _temp) = create_test_db();

        let registry = CustomRegistry {
            id: "reg-1".to_string(),
            name: "Cached".to_string(),
            url: "https://example.com".to_string(),
            description: None,
            icon: None,
            requires_auth: false,
            cached_data: None,
            cached_at: None,
            content_hash: None,
            created_at: Utc::now().to_rfc3339(),
        };

        db.create_custom_registry(&registry).unwrap();

        let cached_data = r#"{"servers": []}"#;
        let cached_at = Utc::now().to_rfc3339();
        let content_hash = "abc123";

        db.update_custom_registry_cache("reg-1", cached_data, &cached_at, content_hash)
            .expect("Failed to update cache");

        let retrieved = db.get_custom_registry("reg-1").unwrap().unwrap();

        assert_eq!(retrieved.cached_data, Some(cached_data.to_string()));
        assert_eq!(retrieved.cached_at, Some(cached_at));
        assert_eq!(retrieved.content_hash, Some(content_hash.to_string()));
    }

    #[test]
    fn test_get_all_custom_registries() {
        let (db, _temp) = create_test_db();

        let registry1 = CustomRegistry {
            id: "reg-1".to_string(),
            name: "Alpha".to_string(),
            url: "https://alpha.com".to_string(),
            description: None,
            icon: None,
            requires_auth: false,
            cached_data: None,
            cached_at: None,
            content_hash: None,
            created_at: Utc::now().to_rfc3339(),
        };
        let registry2 = CustomRegistry {
            id: "reg-2".to_string(),
            name: "Beta".to_string(),
            url: "https://beta.com".to_string(),
            description: None,
            icon: None,
            requires_auth: false,
            cached_data: None,
            cached_at: None,
            content_hash: None,
            created_at: Utc::now().to_rfc3339(),
        };

        db.create_custom_registry(&registry1).unwrap();
        db.create_custom_registry(&registry2).unwrap();

        let registries = db.get_custom_registries().expect("Failed to get registries");

        assert_eq!(registries.len(), 2);
        // Should be ordered by name
        assert_eq!(registries[0].name, "Alpha");
        assert_eq!(registries[1].name, "Beta");
    }
}
