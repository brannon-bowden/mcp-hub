//! Configuration management for MCP Hub
//!
//! This module provides functionality for managing MCP configuration files
//! across different client applications. It handles:
//!
//! - Reading and writing MCP configuration files
//! - Client-specific configuration paths
//! - Backup management
//! - Client detection
//!
//! # Module Structure
//!
//! - `app_dirs` - Application data directory paths
//! - `client_paths` - Client-specific configuration paths
//! - `detection` - Client detection utilities
//! - `file_io` - Low-level file I/O with security and atomicity
//! - `operations` - Core configuration operations

mod app_dirs;
mod client_paths;
mod detection;
mod file_io;
mod operations;

// Re-export public API
pub use app_dirs::{get_app_data_dir, get_backup_dir, get_database_path};
pub use client_paths::{client_requires_merge_write, get_default_config_path};
pub use detection::detect_installed_clients;
pub use operations::{
    backup_config_file, config_exists, import_servers_from_config, read_config_file,
    sync_servers_to_instance, write_config_file, write_mcp_servers_preserving_config,
};
