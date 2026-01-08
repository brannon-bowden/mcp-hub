//! Application directory paths
//!
//! This module provides functions to get platform-specific
//! application data directories.

use std::path::PathBuf;

/// Get the app data directory
pub fn get_app_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| home.join("Library/Application Support/MCP Hub"))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("MCP Hub"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("mcp-hub"))
    }
}

/// Get the backup directory
pub fn get_backup_dir() -> Option<PathBuf> {
    get_app_data_dir().map(|dir| dir.join("backups"))
}

/// Get the database path
pub fn get_database_path() -> Option<PathBuf> {
    get_app_data_dir().map(|dir| dir.join("mcp-hub.db"))
}
