//! Core configuration operations
//!
//! This module provides functions for reading, writing, and syncing
//! MCP configuration files.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::models::{ClientInstance, McpConfigFile, McpServer, McpServerEntry};

use super::client_paths::client_requires_merge_write;
use super::file_io::{atomic_write, validate_path_security, with_file_lock};

/// Check if a config file exists
pub fn config_exists(path: &PathBuf) -> bool {
    path.exists() && path.is_file()
}

/// Read and parse an MCP configuration file
pub fn read_config_file(path: &PathBuf) -> Result<McpConfigFile, String> {
    // Validate path to prevent path traversal attacks
    let validated_path = validate_path_security(path)?;

    if !config_exists(&validated_path) {
        return Ok(McpConfigFile {
            mcp_servers: HashMap::new(),
        });
    }

    let content = fs::read_to_string(&validated_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    // Handle empty files
    if content.trim().is_empty() {
        return Ok(McpConfigFile {
            mcp_servers: HashMap::new(),
        });
    }

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
}

/// Write MCP configuration to a file (overwrites entire file)
pub fn write_config_file(path: &PathBuf, config: &McpConfigFile) -> Result<(), String> {
    // Validate path to prevent path traversal attacks
    let validated_path = validate_path_security(path)?;

    // Ensure parent directory exists
    if let Some(parent) = validated_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    atomic_write(&validated_path, &content)
}

/// Write MCP servers to a config file, preserving other fields in the file
/// This is used for config files like ~/.claude.json that contain other settings
///
/// Uses file locking to prevent race conditions when multiple processes
/// attempt to modify the same config file concurrently.
pub fn write_mcp_servers_preserving_config(
    path: &PathBuf,
    mcp_servers: &HashMap<String, McpServerEntry>,
) -> Result<(), String> {
    // Validate path to prevent path traversal attacks
    let validated_path = validate_path_security(path)?;

    // Ensure parent directory exists (before acquiring lock)
    if let Some(parent) = validated_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Clone for use in closure (borrow checker requires separate ownership)
    let servers_clone = mcp_servers.clone();
    let path_clone = validated_path.clone();

    // Wrap the read-modify-write operation in a file lock to prevent race conditions
    with_file_lock(&validated_path, move || {
        // Read existing content or start with empty object
        let mut existing: serde_json::Value = if path_clone.exists() {
            let content = fs::read_to_string(&path_clone)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            if content.trim().is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse config file: {}", e))?
            }
        } else {
            serde_json::json!({})
        };

        // Ensure we have an object at the root
        let obj = existing.as_object_mut()
            .ok_or_else(|| "Config file is not a JSON object".to_string())?;

        // Update only the mcpServers field
        let servers_value = serde_json::to_value(&servers_clone)
            .map_err(|e| format!("Failed to serialize MCP servers: {}", e))?;
        obj.insert("mcpServers".to_string(), servers_value);

        // Write back the merged config
        let content = serde_json::to_string_pretty(&existing)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        atomic_write(&path_clone, &content)
    })
}

/// Create a backup of a config file
pub fn backup_config_file(path: &PathBuf, backup_dir: &PathBuf) -> Result<PathBuf, String> {
    // Validate both paths to prevent path traversal attacks
    let validated_source = validate_path_security(path)?;
    let validated_backup_dir = validate_path_security(backup_dir)?;

    if !config_exists(&validated_source) {
        return Err("Config file does not exist".to_string());
    }

    fs::create_dir_all(&validated_backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = validated_source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config");
    let backup_filename = format!("{}_{}.backup", filename, timestamp);
    let backup_path = validated_backup_dir.join(backup_filename);

    fs::copy(&validated_source, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    Ok(backup_path)
}

/// Convert servers to MCP config format and write to instance config file.
pub fn sync_servers_to_instance(
    instance: &ClientInstance,
    servers: &[McpServer],
    backup_dir: Option<&PathBuf>,
) -> Result<Option<PathBuf>, String> {
    let config_path = PathBuf::from(&instance.config_path);
    let mut backup_path = None;

    // Create backup if requested and file exists
    if let Some(dir) = backup_dir {
        if config_exists(&config_path) {
            backup_path = Some(backup_config_file(&config_path, dir)?);
        }
    }

    // Build the MCP servers map
    let mut mcp_servers = HashMap::new();

    // Sync all enabled servers
    for server in servers {
        if instance.enabled_servers.contains(&server.id) {
            let entry = McpServerEntry {
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.clone(),
            };
            // Use server name as the key (sanitized)
            let key = sanitize_server_name(&server.name);
            mcp_servers.insert(key, entry);
        }
    }

    // Use merge-aware write for clients that have other settings in their config file
    if client_requires_merge_write(&instance.client_type) {
        write_mcp_servers_preserving_config(&config_path, &mcp_servers)?;
    } else {
        let config = McpConfigFile { mcp_servers };
        write_config_file(&config_path, &config)?;
    }

    Ok(backup_path)
}

/// Sanitize server name for use as a config key
fn sanitize_server_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Import servers from an existing config file
pub fn import_servers_from_config(path: &PathBuf) -> Result<Vec<McpServer>, String> {
    let config = read_config_file(path)?;
    let mut servers = Vec::new();

    for (name, entry) in config.mcp_servers {
        let mut server = McpServer::new(name.clone(), entry.command, entry.args);
        server.env = entry.env;
        server.source = Some(crate::models::ServerSource {
            source_type: crate::models::SourceType::Imported,
            url: Some(path.to_string_lossy().to_string()),
        });
        servers.push(server);
    }

    Ok(servers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    /// Create a test directory under the home directory to satisfy path validation.
    /// Path validation requires paths to be under home or app data directory.
    fn create_test_dir() -> PathBuf {
        let home = dirs::home_dir().expect("Could not get home directory");
        let test_dir = home.join(".mcp-hub-tests").join(Uuid::new_v4().to_string());
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");
        test_dir
    }

    /// Clean up a test directory
    fn cleanup_test_dir(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_sanitize_server_name() {
        assert_eq!(sanitize_server_name("My Server"), "my-server");
        assert_eq!(sanitize_server_name("server_123"), "server_123");
        assert_eq!(sanitize_server_name("  test  "), "test");
        assert_eq!(sanitize_server_name("hello@world!"), "hello-world");
    }

    #[test]
    fn test_sanitize_server_name_edge_cases() {
        // Empty and whitespace
        assert_eq!(sanitize_server_name(""), "");
        assert_eq!(sanitize_server_name("   "), "");

        // Special characters
        assert_eq!(sanitize_server_name("@@@"), "");
        assert_eq!(sanitize_server_name("a@b@c"), "a-b-c");

        // Numbers only
        assert_eq!(sanitize_server_name("123"), "123");

        // Unicode characters - non-alphanumeric becomes dash
        let result = sanitize_server_name("héllo");
        assert!(result.contains("h") && result.contains("llo"));

        // Leading/trailing dashes
        assert_eq!(sanitize_server_name("-test-"), "test");
        assert_eq!(sanitize_server_name("---test---"), "test");
    }

    #[test]
    fn test_config_exists_returns_false_for_nonexistent() {
        let test_dir = create_test_dir();
        let path = test_dir.join("nonexistent.json");
        assert!(!config_exists(&path));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_config_exists_returns_true_for_existing_file() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("config.json");
        fs::write(&config_path, "{}").unwrap();

        assert!(config_exists(&config_path));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_config_exists_returns_false_for_directory() {
        let test_dir = create_test_dir();
        // The test_dir itself is a directory, not a file
        assert!(!config_exists(&test_dir));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_config_file_returns_empty_for_nonexistent() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("nonexistent.json");

        let result = read_config_file(&config_path);
        assert!(result.is_ok());
        assert!(result.unwrap().mcp_servers.is_empty());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_config_file_returns_empty_for_empty_file() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("empty.json");
        fs::write(&config_path, "").unwrap();

        let result = read_config_file(&config_path);
        assert!(result.is_ok());
        assert!(result.unwrap().mcp_servers.is_empty());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_config_file_returns_empty_for_whitespace_file() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("whitespace.json");
        fs::write(&config_path, "   \n\t  ").unwrap();

        let result = read_config_file(&config_path);
        assert!(result.is_ok());
        assert!(result.unwrap().mcp_servers.is_empty());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_config_file_parses_valid_config() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("valid.json");
        let content = r#"{
            "mcpServers": {
                "test-server": {
                    "command": "npx",
                    "args": ["-y", "@test/server"]
                }
            }
        }"#;
        fs::write(&config_path, content).unwrap();

        let result = read_config_file(&config_path);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("test-server"));

        let server = &config.mcp_servers["test-server"];
        assert_eq!(server.command, "npx");
        assert_eq!(server.args, vec!["-y", "@test/server"]);
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_config_file_returns_error_for_invalid_json() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("invalid.json");
        fs::write(&config_path, "{ not valid json }").unwrap();

        let result = read_config_file(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse config file"));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_config_file_creates_file() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("new_config.json");

        let mut mcp_servers = HashMap::new();
        mcp_servers.insert(
            "test-server".to_string(),
            McpServerEntry {
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@test/server".to_string()],
                env: HashMap::new(),
            },
        );
        let config = McpConfigFile { mcp_servers };

        let result = write_config_file(&config_path, &config);
        assert!(result.is_ok());

        // Verify file was created and content is correct
        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: McpConfigFile = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.mcp_servers.len(), 1);
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_config_file_creates_parent_directories() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("nested/dir/config.json");

        let config = McpConfigFile {
            mcp_servers: HashMap::new(),
        };

        let result = write_config_file(&config_path, &config);
        assert!(result.is_ok());
        assert!(config_path.exists());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_mcp_servers_preserving_config_creates_new_file() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("new.json");

        let mut servers = HashMap::new();
        servers.insert(
            "my-server".to_string(),
            McpServerEntry {
                command: "node".to_string(),
                args: vec!["server.js".to_string()],
                env: HashMap::new(),
            },
        );

        let result = write_mcp_servers_preserving_config(&config_path, &servers);
        assert!(result.is_ok());

        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("mcpServers").is_some());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_mcp_servers_preserving_config_preserves_other_fields() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("existing.json");

        // Write existing config with other fields
        let initial_content = r#"{
            "someOtherSetting": "value",
            "nested": {
                "key": "data"
            },
            "mcpServers": {
                "old-server": {
                    "command": "old",
                    "args": []
                }
            }
        }"#;
        fs::write(&config_path, initial_content).unwrap();

        // Write new servers
        let mut servers = HashMap::new();
        servers.insert(
            "new-server".to_string(),
            McpServerEntry {
                command: "new".to_string(),
                args: vec!["arg".to_string()],
                env: HashMap::new(),
            },
        );

        let result = write_mcp_servers_preserving_config(&config_path, &servers);
        assert!(result.is_ok());

        // Verify other fields are preserved
        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["someOtherSetting"], "value");
        assert_eq!(parsed["nested"]["key"], "data");

        // Verify mcpServers was replaced (not merged)
        let mcp_servers = parsed["mcpServers"].as_object().unwrap();
        assert_eq!(mcp_servers.len(), 1);
        assert!(mcp_servers.contains_key("new-server"));
        assert!(!mcp_servers.contains_key("old-server"));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_mcp_servers_preserving_config_handles_empty_file() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("empty.json");
        fs::write(&config_path, "").unwrap();

        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            McpServerEntry {
                command: "cmd".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        let result = write_mcp_servers_preserving_config(&config_path, &servers);
        assert!(result.is_ok());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_mcp_servers_preserving_config_rejects_non_object_root() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("array.json");
        fs::write(&config_path, "[1, 2, 3]").unwrap();

        let servers = HashMap::new();
        let result = write_mcp_servers_preserving_config(&config_path, &servers);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a JSON object"));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_import_servers_from_config() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("import.json");
        let content = r#"{
            "mcpServers": {
                "server1": {
                    "command": "npx",
                    "args": ["-y", "@test/one"],
                    "env": {"KEY": "value"}
                },
                "server2": {
                    "command": "node",
                    "args": ["index.js"]
                }
            }
        }"#;
        fs::write(&config_path, content).unwrap();

        let result = import_servers_from_config(&config_path);
        assert!(result.is_ok());

        let servers = result.unwrap();
        assert_eq!(servers.len(), 2);

        // Verify imported servers have correct source type
        for server in &servers {
            assert!(server.source.is_some());
            let source = server.source.as_ref().unwrap();
            assert_eq!(source.source_type, crate::models::SourceType::Imported);
        }
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_import_servers_from_nonexistent_config() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("nonexistent.json");

        let result = import_servers_from_config(&config_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_backup_config_file_creates_backup() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("config.json");
        let backup_dir = test_dir.join("backups");
        fs::write(&config_path, r#"{"mcpServers": {}}"#).unwrap();

        let result = backup_config_file(&config_path, &backup_dir);
        assert!(result.is_ok());

        let backup_path = result.unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains(".backup"));

        // Verify backup content matches original
        let original = fs::read_to_string(&config_path).unwrap();
        let backup = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(original, backup);
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_backup_config_file_errors_for_nonexistent_source() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("nonexistent.json");
        let backup_dir = test_dir.join("backups");

        let result = backup_config_file(&config_path, &backup_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_sync_servers_to_instance_writes_enabled_servers() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("sync.json");

        let instance = ClientInstance {
            id: "inst-1".to_string(),
            name: "Test Instance".to_string(),
            client_type: crate::models::ClientType::Custom,
            config_path: config_path.to_string_lossy().to_string(),
            enabled_servers: vec!["srv-1".to_string(), "srv-2".to_string()],
            is_default: false,
            last_synced: None,
            last_modified: None,
            created_at: chrono::Utc::now(),
        };

        let mut server1 = McpServer::new(
            "Server One".to_string(),
            "npx".to_string(),
            vec!["-y".to_string(), "@test/one".to_string()],
        );
        server1.id = "srv-1".to_string();

        let mut server2 = McpServer::new(
            "Server Two".to_string(),
            "node".to_string(),
            vec!["index.js".to_string()],
        );
        server2.id = "srv-2".to_string();

        let mut server3 = McpServer::new(
            "Server Three".to_string(),
            "python".to_string(),
            vec!["server.py".to_string()],
        );
        server3.id = "srv-3".to_string(); // Not in enabled list

        let servers = vec![server1, server2, server3];

        let result = sync_servers_to_instance(&instance, &servers, None);
        assert!(result.is_ok());

        // Verify only enabled servers were written
        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: McpConfigFile = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.mcp_servers.len(), 2);
        assert!(parsed.mcp_servers.contains_key("server-one"));
        assert!(parsed.mcp_servers.contains_key("server-two"));
        assert!(!parsed.mcp_servers.contains_key("server-three"));
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_sync_servers_to_instance_creates_backup_when_requested() {
        let test_dir = create_test_dir();
        let config_path = test_dir.join("sync.json");
        let backup_dir = test_dir.join("backups");

        // Create initial config
        fs::write(
            &config_path,
            r#"{"mcpServers": {"old": {"command": "old", "args": []}}}"#,
        )
        .unwrap();

        let instance = ClientInstance {
            id: "inst-1".to_string(),
            name: "Test".to_string(),
            client_type: crate::models::ClientType::Custom,
            config_path: config_path.to_string_lossy().to_string(),
            enabled_servers: vec![],
            is_default: false,
            last_synced: None,
            last_modified: None,
            created_at: chrono::Utc::now(),
        };

        let result = sync_servers_to_instance(&instance, &[], Some(&backup_dir));
        assert!(result.is_ok());

        // Verify backup was created
        let backup_path = result.unwrap();
        assert!(backup_path.is_some());
        assert!(backup_path.unwrap().exists());
        cleanup_test_dir(&test_dir);
    }
}
