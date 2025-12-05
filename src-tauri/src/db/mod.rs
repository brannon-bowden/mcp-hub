use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::{
    ClientInstance, ClientType, ConfigBackup, McpServer, ServerSource, SourceType,
};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: PathBuf) -> SqlResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
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
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
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
            ",
        )?;

        // Migration: Add last_modified column if it doesn't exist
        // Check if column exists first
        let has_last_modified: bool = {
            let mut stmt = conn.prepare("PRAGMA table_info(client_instances)")?;
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))?
                .filter_map(|r| r.ok())
                .collect();
            columns.contains(&"last_modified".to_string())
        };

        if !has_last_modified {
            conn.execute("ALTER TABLE client_instances ADD COLUMN last_modified TEXT", [])?;
        }

        Ok(())
    }

    // ==================== Server CRUD ====================

    pub fn create_server(&self, server: &McpServer) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

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
            "INSERT INTO servers (id, name, description, command, args, env, tags, source_type, source_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                server.created_at.to_rfc3339(),
                server.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn get_server(&self, id: &str) -> SqlResult<Option<McpServer>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, description, command, args, env, tags, source_type, source_url, created_at, updated_at
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
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, description, command, args, env, tags, source_type, source_url, created_at, updated_at
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
        let conn = self.conn.lock().unwrap();

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
             tags = ?7, source_type = ?8, source_url = ?9, updated_at = ?10 WHERE id = ?1",
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
                server.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn delete_server(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn row_to_server(row: &rusqlite::Row) -> SqlResult<McpServer> {
        let args_str: String = row.get(4)?;
        let env_str: String = row.get(5)?;
        let tags_str: Option<String> = row.get(6)?;
        let source_type: Option<String> = row.get(7)?;
        let source_url: Option<String> = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let updated_at_str: String = row.get(10)?;

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM instance_servers WHERE instance_id = ?1 AND server_id = ?2",
            params![instance_id, server_id],
        )?;
        Ok(())
    }

    // ==================== Backups ====================

    pub fn create_backup(&self, backup: &ConfigBackup) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

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
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;

        match stmt.query_row(params![key], |row| row.get(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;

        Ok(())
    }
}
