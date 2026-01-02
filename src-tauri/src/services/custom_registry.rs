use crate::db::{CustomRegistry, Database};
use crate::services::credentials;
use crate::services::registry::RegistryServer;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

/// JSON structure for custom registry files
#[derive(Debug, Deserialize, Serialize)]
pub struct CustomRegistryFile {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub servers: Vec<RegistryServer>,
}

/// Result of fetching a custom registry
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub servers: Vec<RegistryServer>,
    pub from_cache: bool,
    pub cached_at: Option<String>,
}

/// Add a new custom registry
pub fn add_custom_registry(
    db: &Database,
    url: &str,
    name_override: Option<&str>,
    token: Option<&str>,
) -> Result<CustomRegistry, String> {
    // First, try to fetch and validate the registry
    let registry_file = fetch_registry_file(url, token)?;

    let id = Uuid::new_v4().to_string();
    let name = name_override.unwrap_or(&registry_file.name).to_string();
    let requires_auth = token.is_some();

    // Store token in keychain if provided
    if let Some(token) = token {
        let credential_key = format!("custom-registry:{}", id);
        credentials::store_credential(&credential_key, token)
            .map_err(|e| format!("Failed to store token: {}", e))?;
    }

    // Cache the fetched data
    let cached_data = serde_json::to_string(&registry_file.servers)
        .map_err(|e| format!("Failed to serialize servers: {}", e))?;
    let cached_at = Utc::now().to_rfc3339();

    let registry = CustomRegistry {
        id: id.clone(),
        name,
        url: url.to_string(),
        description: registry_file.description,
        icon: registry_file.icon,
        requires_auth,
        cached_data: Some(cached_data),
        cached_at: Some(cached_at),
        created_at: Utc::now().to_rfc3339(),
    };

    db.create_custom_registry(&registry)
        .map_err(|e| format!("Failed to save registry: {}", e))?;

    Ok(registry)
}

/// Update an existing custom registry
pub fn update_custom_registry(
    db: &Database,
    id: &str,
    url: Option<&str>,
    name_override: Option<&str>,
    token: Option<&str>,
) -> Result<CustomRegistry, String> {
    let mut registry = db
        .get_custom_registry(id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Registry not found".to_string())?;

    // Update URL if provided
    if let Some(new_url) = url {
        registry.url = new_url.to_string();
    }

    // Update name if provided
    if let Some(new_name) = name_override {
        registry.name = new_name.to_string();
    }

    // Update token if provided
    if let Some(new_token) = token {
        let credential_key = format!("custom-registry:{}", id);
        credentials::store_credential(&credential_key, new_token)
            .map_err(|e| format!("Failed to store token: {}", e))?;
        registry.requires_auth = true;
    }

    // Re-fetch to update cache
    let token_for_fetch = if registry.requires_auth {
        get_registry_token(id).ok().flatten()
    } else {
        None
    };

    if let Ok(registry_file) = fetch_registry_file(&registry.url, token_for_fetch.as_deref()) {
        let cached_data = serde_json::to_string(&registry_file.servers)
            .map_err(|e| format!("Failed to serialize servers: {}", e))?;
        registry.cached_data = Some(cached_data);
        registry.cached_at = Some(Utc::now().to_rfc3339());
    }

    db.update_custom_registry(&registry)
        .map_err(|e| format!("Failed to update registry: {}", e))?;

    Ok(registry)
}

/// Delete a custom registry
pub fn delete_custom_registry(db: &Database, id: &str) -> Result<(), String> {
    // Remove token from keychain if it exists
    let credential_key = format!("custom-registry:{}", id);
    let _ = credentials::delete_credential(&credential_key);

    db.delete_custom_registry(id)
        .map_err(|e| format!("Failed to delete registry: {}", e))
}

/// Fetch servers from a custom registry
pub fn fetch_custom_registry_servers(
    db: &Database,
    id: &str,
    force_refresh: bool,
) -> Result<FetchResult, String> {
    let registry = db
        .get_custom_registry(id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Registry not found".to_string())?;

    // Return cached data if available and not forcing refresh
    if !force_refresh {
        if let Some(cached_data) = &registry.cached_data {
            let servers: Vec<RegistryServer> = serde_json::from_str(cached_data)
                .map_err(|e| format!("Failed to parse cached data: {}", e))?;
            return Ok(FetchResult {
                servers,
                from_cache: true,
                cached_at: registry.cached_at.clone(),
            });
        }
    }

    // Fetch fresh data
    let token = if registry.requires_auth {
        get_registry_token(id).ok().flatten()
    } else {
        None
    };

    let registry_file = fetch_registry_file(&registry.url, token.as_deref())?;

    // Update cache
    let cached_data = serde_json::to_string(&registry_file.servers)
        .map_err(|e| format!("Failed to serialize servers: {}", e))?;
    let cached_at = Utc::now().to_rfc3339();

    db.update_custom_registry_cache(id, &cached_data, &cached_at)
        .map_err(|e| format!("Failed to update cache: {}", e))?;

    Ok(FetchResult {
        servers: registry_file.servers,
        from_cache: false,
        cached_at: Some(cached_at),
    })
}

/// Get all custom registries
pub fn get_all_custom_registries(db: &Database) -> Result<Vec<CustomRegistry>, String> {
    db.get_custom_registries()
        .map_err(|e| format!("Database error: {}", e))
}

/// Fetch and parse a registry file from URL or local path
fn fetch_registry_file(url: &str, token: Option<&str>) -> Result<CustomRegistryFile, String> {
    let content = if url.starts_with("http://") || url.starts_with("https://") {
        fetch_remote_registry(url, token)?
    } else {
        fetch_local_registry(url)?
    };

    serde_json::from_str(&content).map_err(|e| format!("Invalid registry JSON: {}", e))
}

/// Fetch registry from remote URL
fn fetch_remote_registry(url: &str, token: Option<&str>) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut request = client.get(url);

    // Add auth header for GitHub URLs if token provided
    if let Some(token) = token {
        if url.contains("github.com") || url.contains("raw.githubusercontent.com") {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
    }

    request = request.header("User-Agent", "MCP-Hub");

    let response = request
        .send()
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 || response.status() == 403 {
        return Err("Authentication failed. Check your GitHub token.".to_string());
    }

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))
}

/// Fetch registry from local file
fn fetch_local_registry(path: &str) -> Result<String, String> {
    // Handle file:// prefix
    let path = path.strip_prefix("file://").unwrap_or(path);

    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Get token for a registry from keychain
fn get_registry_token(id: &str) -> Result<Option<String>, String> {
    let credential_key = format!("custom-registry:{}", id);
    credentials::get_credential(&credential_key)
}

/// Test a registry URL without saving it
pub fn test_registry_url(url: &str, token: Option<&str>) -> Result<CustomRegistryFile, String> {
    fetch_registry_file(url, token)
}
