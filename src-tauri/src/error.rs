//! Typed error handling for MCP Hub
//!
//! This module provides structured error types that can be serialized to the frontend,
//! enabling proper error handling with error codes and contextual information.
//!
//! Security: Error messages are sanitized to prevent information disclosure.
//! Detailed errors are logged internally but generic messages are returned to users.

use serde::Serialize;
use thiserror::Error;

/// Error codes for frontend error handling and i18n
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Database errors (DB_*)
    DbReadError,
    DbWriteError,
    DbLockError,
    DbNotFound,

    // Server errors (SERVER_*)
    ServerNotFound,
    ServerValidationError,
    ServerCommandBlocked,

    // Instance errors (INSTANCE_*)
    InstanceNotFound,
    InstanceConfigInvalid,

    // Config errors (CONFIG_*)
    ConfigReadError,
    ConfigWriteError,
    ConfigParseError,
    ConfigPathInvalid,

    // Sync errors (SYNC_*)
    SyncFailed,
    SyncBackupFailed,

    // Registry errors (REGISTRY_*)
    RegistryFetchError,
    RegistryParseError,
    RegistryNotFound,
    RegistrySsrfBlocked,

    // Credential errors (CRED_*)
    CredentialStorageUnavailable,
    CredentialNotFound,
    CredentialStoreError,

    // Discovery errors (DISCOVERY_*)
    DiscoveryStartFailed,
    DiscoveryPortUnavailable,

    // Validation errors (VALIDATION_*)
    ValidationFailed,

    // Internal errors (INTERNAL_*)
    InternalError,
}

/// Structured error response for the frontend
#[derive(Debug, Error, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpHubError {
    /// Human-readable error message
    pub message: String,

    /// Machine-readable error code for frontend handling
    pub code: ErrorCode,

    /// Optional additional context (e.g., which server, which instance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ErrorContext>,
}

impl std::fmt::Display for McpHubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Additional context for errors
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorContext {
    /// ID of the resource involved (server_id, instance_id, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    /// Type of resource involved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,

    /// File path involved (for config errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl McpHubError {
    /// Create a new error with just a message and code
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code,
            context: None,
        }
    }

    /// Create an error with context
    pub fn with_context(code: ErrorCode, message: impl Into<String>, context: ErrorContext) -> Self {
        Self {
            message: message.into(),
            code,
            context: Some(context),
        }
    }

    // Database errors
    pub fn db_read(err: impl std::fmt::Display) -> Self {
        // Log detailed error internally, return sanitized message to user
        log::error!("Database read error: {}", err);
        Self::new(ErrorCode::DbReadError, "Failed to read from database")
    }

    pub fn db_write(err: impl std::fmt::Display) -> Self {
        log::error!("Database write error: {}", err);
        Self::new(ErrorCode::DbWriteError, "Failed to write to database")
    }

    pub fn db_lock() -> Self {
        Self::new(ErrorCode::DbLockError, "Failed to acquire database lock")
    }

    // Server errors
    pub fn server_not_found(server_id: &str) -> Self {
        Self::with_context(
            ErrorCode::ServerNotFound,
            format!("Server not found: {}", server_id),
            ErrorContext {
                resource_id: Some(server_id.to_string()),
                resource_type: Some("server".to_string()),
                path: None,
                details: None,
            },
        )
    }

    pub fn server_validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ServerValidationError, message)
    }

    pub fn server_command_blocked(command: &str, reason: &str) -> Self {
        Self::with_context(
            ErrorCode::ServerCommandBlocked,
            format!("Command '{}' is blocked: {}", command, reason),
            ErrorContext {
                resource_id: None,
                resource_type: Some("command".to_string()),
                path: None,
                details: Some(reason.to_string()),
            },
        )
    }

    // Instance errors
    pub fn instance_not_found(instance_id: &str) -> Self {
        Self::with_context(
            ErrorCode::InstanceNotFound,
            format!("Instance not found: {}", instance_id),
            ErrorContext {
                resource_id: Some(instance_id.to_string()),
                resource_type: Some("instance".to_string()),
                path: None,
                details: None,
            },
        )
    }

    // Config errors
    pub fn config_read(path: &str, err: impl std::fmt::Display) -> Self {
        // Log detailed error with full path internally
        log::error!("Config read error at {}: {}", path, err);
        Self::with_context(
            ErrorCode::ConfigReadError,
            "Failed to read configuration file",
            ErrorContext {
                resource_id: None,
                resource_type: Some("config".to_string()),
                // Only include filename, not full path (may contain username)
                path: std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string()),
                details: None,
            },
        )
    }

    pub fn config_write(path: &str, err: impl std::fmt::Display) -> Self {
        log::error!("Config write error at {}: {}", path, err);
        Self::with_context(
            ErrorCode::ConfigWriteError,
            "Failed to write configuration file",
            ErrorContext {
                resource_id: None,
                resource_type: Some("config".to_string()),
                path: std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string()),
                details: None,
            },
        )
    }

    pub fn config_parse(path: &str, err: impl std::fmt::Display) -> Self {
        log::error!("Config parse error at {}: {}", path, err);
        Self::with_context(
            ErrorCode::ConfigParseError,
            "Failed to parse configuration file",
            ErrorContext {
                resource_id: None,
                resource_type: Some("config".to_string()),
                path: std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string()),
                details: None,
            },
        )
    }

    pub fn config_path_invalid(path: &str, reason: &str) -> Self {
        log::warn!("Invalid config path {}: {}", path, reason);
        Self::with_context(
            ErrorCode::ConfigPathInvalid,
            "Invalid configuration path",
            ErrorContext {
                resource_id: None,
                resource_type: Some("config".to_string()),
                path: std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string()),
                details: None, // Don't expose reason which may contain paths
            },
        )
    }

    // Sync errors
    pub fn sync_failed(instance_id: &str, err: impl std::fmt::Display) -> Self {
        log::error!("Sync failed for instance {}: {}", instance_id, err);
        Self::with_context(
            ErrorCode::SyncFailed,
            "Failed to sync configuration",
            ErrorContext {
                resource_id: Some(instance_id.to_string()),
                resource_type: Some("instance".to_string()),
                path: None,
                details: None,
            },
        )
    }

    // Registry errors
    pub fn registry_fetch(registry_id: &str, err: impl std::fmt::Display) -> Self {
        // Don't expose network/HTTP errors which may contain URLs or tokens
        log::error!("Registry fetch failed for {}: {}", registry_id, err);
        Self::with_context(
            ErrorCode::RegistryFetchError,
            "Failed to fetch from registry",
            ErrorContext {
                resource_id: Some(registry_id.to_string()),
                resource_type: Some("registry".to_string()),
                path: None,
                details: None,
            },
        )
    }

    pub fn registry_not_found(registry_id: &str) -> Self {
        Self::with_context(
            ErrorCode::RegistryNotFound,
            format!("Registry not found: {}", registry_id),
            ErrorContext {
                resource_id: Some(registry_id.to_string()),
                resource_type: Some("registry".to_string()),
                path: None,
                details: None,
            },
        )
    }

    pub fn registry_ssrf_blocked(url: &str) -> Self {
        Self::with_context(
            ErrorCode::RegistrySsrfBlocked,
            "URL blocked for security reasons (SSRF protection)",
            ErrorContext {
                resource_id: None,
                resource_type: Some("url".to_string()),
                path: Some(url.to_string()),
                details: Some("Internal or private network addresses are not allowed".to_string()),
            },
        )
    }

    // Credential errors
    pub fn credential_unavailable() -> Self {
        Self::new(
            ErrorCode::CredentialStorageUnavailable,
            "Secure credential storage is not available on this system",
        )
    }

    pub fn credential_not_found(key: &str) -> Self {
        Self::with_context(
            ErrorCode::CredentialNotFound,
            format!("Credential not found: {}", key),
            ErrorContext {
                resource_id: Some(key.to_string()),
                resource_type: Some("credential".to_string()),
                path: None,
                details: None,
            },
        )
    }

    pub fn credential_store_error(err: impl std::fmt::Display) -> Self {
        // Log detailed keyring error internally, don't expose to user
        log::error!("Credential store error: {}", err);
        Self::new(
            ErrorCode::CredentialStoreError,
            "Failed to access secure credential storage",
        )
    }

    // Discovery errors
    pub fn discovery_start_failed(port: u16, err: impl std::fmt::Display) -> Self {
        log::error!("Discovery server start failed on port {}: {}", port, err);
        Self::with_context(
            ErrorCode::DiscoveryStartFailed,
            format!("Failed to start discovery server on port {}", port),
            ErrorContext {
                resource_id: None,
                resource_type: Some("discovery".to_string()),
                path: None,
                details: None, // Don't expose internal network errors
            },
        )
    }

    pub fn discovery_port_unavailable(port: u16) -> Self {
        Self::with_context(
            ErrorCode::DiscoveryPortUnavailable,
            format!("Port {} is not available", port),
            ErrorContext {
                resource_id: None,
                resource_type: Some("port".to_string()),
                path: None,
                details: Some(format!("Port {} is already in use", port)),
            },
        )
    }

    // Validation errors
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
    }

    // Internal errors
    pub fn internal(err: impl std::fmt::Display) -> Self {
        log::error!("Internal error: {}", err);
        Self::new(ErrorCode::InternalError, "An internal error occurred")
    }
}

/// Result type alias for commands
pub type McpResult<T> = Result<T, McpHubError>;

// Conversion from common error types
impl From<rusqlite::Error> for McpHubError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                Self::new(ErrorCode::DbNotFound, "Record not found")
            }
            _ => Self::db_read(err),
        }
    }
}

impl From<serde_json::Error> for McpHubError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorCode::ConfigParseError, format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for McpHubError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorCode::ConfigReadError, format!("I/O error: {}", err))
    }
}

// Allow converting string errors (for backward compatibility during migration)
impl From<String> for McpHubError {
    fn from(err: String) -> Self {
        Self::new(ErrorCode::InternalError, err)
    }
}

impl From<&str> for McpHubError {
    fn from(err: &str) -> Self {
        Self::new(ErrorCode::InternalError, err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let err = McpHubError::server_not_found("test-server-123");
        let json = serde_json::to_string(&err).unwrap();

        assert!(json.contains("SERVER_NOT_FOUND"));
        assert!(json.contains("test-server-123"));
    }

    #[test]
    fn test_error_with_context() {
        let err = McpHubError::config_read("/path/to/config.json", "file not found");

        assert!(err.context.is_some());
        let ctx = err.context.unwrap();
        assert_eq!(ctx.path, Some("/path/to/config.json".to_string()));
    }

    #[test]
    fn test_error_display() {
        let err = McpHubError::db_lock();
        assert_eq!(format!("{}", err), "Failed to acquire database lock");
    }
}
