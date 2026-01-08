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

    #[test]
    fn test_sanitize_server_name() {
        assert_eq!(sanitize_server_name("My Server"), "my-server");
        assert_eq!(sanitize_server_name("server_123"), "server_123");
        assert_eq!(sanitize_server_name("  test  "), "test");
        assert_eq!(sanitize_server_name("hello@world!"), "hello-world");
    }
}
