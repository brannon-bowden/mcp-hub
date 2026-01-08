use crate::db::{CustomRegistry, Database};
use crate::services::credentials;
use crate::services::registry::RegistryServer;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::{IpAddr, ToSocketAddrs};
use url::Url;
use uuid::Uuid;

/// Maximum response size for custom registry fetches (1MB)
/// Prevents memory exhaustion from malicious or misconfigured endpoints
const MAX_RESPONSE_SIZE: u64 = 1_048_576;

/// Allowed URL schemes for custom registries
const ALLOWED_SCHEMES: &[&str] = &["https"];

/// Calculate SHA-256 hash of content for integrity verification
fn calculate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify content integrity by comparing hash
/// Returns Ok(()) if hash matches or no hash stored (legacy data)
/// Returns Err with message if hash mismatch detected
fn verify_content_integrity(content: &str, stored_hash: Option<&str>) -> Result<(), String> {
    if let Some(expected_hash) = stored_hash {
        let computed_hash = calculate_content_hash(content);
        if computed_hash != expected_hash {
            log::warn!(
                "Content integrity check failed: expected {}, computed {}",
                expected_hash,
                computed_hash
            );
            return Err("Cached data integrity check failed. Data may be corrupted.".to_string());
        }
        log::debug!("Content integrity verified: {}", computed_hash);
    }
    Ok(())
}

/// Check if an IP address is in a private/internal range (SSRF protection)
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Loopback: 127.0.0.0/8
            ipv4.is_loopback()
            // Private: 10.0.0.0/8
            || ipv4.octets()[0] == 10
            // Private: 172.16.0.0/12
            || (ipv4.octets()[0] == 172 && (16..=31).contains(&ipv4.octets()[1]))
            // Private: 192.168.0.0/16
            || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168)
            // Link-local: 169.254.0.0/16 (includes cloud metadata at 169.254.169.254)
            || (ipv4.octets()[0] == 169 && ipv4.octets()[1] == 254)
            // Broadcast
            || ipv4.is_broadcast()
            // Documentation ranges
            || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2)
            || (ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100)
            || (ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113)
            // Unspecified
            || ipv4.is_unspecified()
        }
        IpAddr::V6(ipv6) => {
            // Loopback ::1
            ipv6.is_loopback()
            // Unspecified ::
            || ipv6.is_unspecified()
            // IPv4-mapped addresses - check the embedded IPv4
            || {
                let segments = ipv6.segments();
                // ::ffff:x.x.x.x format
                if segments[0..5] == [0, 0, 0, 0, 0] && segments[5] == 0xffff {
                    let ipv4 = std::net::Ipv4Addr::new(
                        (segments[6] >> 8) as u8,
                        segments[6] as u8,
                        (segments[7] >> 8) as u8,
                        segments[7] as u8,
                    );
                    is_private_ip(&IpAddr::V4(ipv4))
                } else {
                    false
                }
            }
            // Link-local fe80::/10
            || (ipv6.segments()[0] & 0xffc0) == 0xfe80
            // Unique local fc00::/7
            || (ipv6.segments()[0] & 0xfe00) == 0xfc00
        }
    }
}

/// Validate a URL for SSRF protection
/// Returns Ok(()) if the URL is safe to fetch, Err with reason otherwise
fn validate_url_for_ssrf(url_str: &str) -> Result<(), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // Check scheme - only HTTPS allowed for remote registries
    let scheme = url.scheme().to_lowercase();
    if !ALLOWED_SCHEMES.contains(&scheme.as_str()) {
        return Err(format!(
            "Invalid URL scheme '{}'. Only HTTPS is allowed for remote registries.",
            scheme
        ));
    }

    // Get host
    let host = url.host_str().ok_or("URL has no host")?;

    // Block localhost variations
    let host_lower = host.to_lowercase();
    if host_lower == "localhost"
        || host_lower == "localhost.localdomain"
        || host_lower.ends_with(".localhost")
    {
        return Err("URLs pointing to localhost are not allowed".to_string());
    }

    // Block IP addresses directly in URL (force hostname usage for auditability)
    // Also prevents direct specification of internal IPs
    if url.host().map(|h| matches!(h, url::Host::Ipv4(_) | url::Host::Ipv6(_))).unwrap_or(false) {
        return Err("Direct IP addresses are not allowed. Please use a hostname.".to_string());
    }

    // Resolve hostname and check all resolved IPs
    let port = url.port().unwrap_or(443);
    let socket_addrs: Vec<_> = format!("{}:{}", host, port)
        .to_socket_addrs()
        .map_err(|e| format!("Failed to resolve hostname '{}': {}", host, e))?
        .collect();

    if socket_addrs.is_empty() {
        return Err(format!("Hostname '{}' did not resolve to any addresses", host));
    }

    // Check all resolved IPs - block if ANY resolve to private ranges
    for addr in &socket_addrs {
        if is_private_ip(&addr.ip()) {
            return Err(format!(
                "URL resolves to private/internal IP address ({}). This is not allowed for security reasons.",
                addr.ip()
            ));
        }
    }

    Ok(())
}

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

    // Cache the fetched data with integrity hash
    let cached_data = serde_json::to_string(&registry_file.servers)
        .map_err(|e| format!("Failed to serialize servers: {}", e))?;
    let content_hash = calculate_content_hash(&cached_data);
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
        content_hash: Some(content_hash),
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
        let content_hash = calculate_content_hash(&cached_data);
        registry.cached_data = Some(cached_data);
        registry.cached_at = Some(Utc::now().to_rfc3339());
        registry.content_hash = Some(content_hash);
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
            // Verify cached data integrity before using
            if let Err(e) = verify_content_integrity(cached_data, registry.content_hash.as_deref()) {
                log::error!("Registry {} cache integrity check failed: {}", id, e);
                // Fall through to fetch fresh data
            } else {
                let servers: Vec<RegistryServer> = serde_json::from_str(cached_data)
                    .map_err(|e| format!("Failed to parse cached data: {}", e))?;
                return Ok(FetchResult {
                    servers,
                    from_cache: true,
                    cached_at: registry.cached_at.clone(),
                });
            }
        }
    }

    // Fetch fresh data
    let token = if registry.requires_auth {
        get_registry_token(id).ok().flatten()
    } else {
        None
    };

    let registry_file = fetch_registry_file(&registry.url, token.as_deref())?;

    // Update cache with integrity hash
    let cached_data = serde_json::to_string(&registry_file.servers)
        .map_err(|e| format!("Failed to serialize servers: {}", e))?;
    let content_hash = calculate_content_hash(&cached_data);
    let cached_at = Utc::now().to_rfc3339();

    db.update_custom_registry_cache(id, &cached_data, &cached_at, &content_hash)
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
    let content = if url.starts_with("https://") {
        // Remote HTTPS URL - will be validated for SSRF in fetch_remote_registry
        fetch_remote_registry(url, token)?
    } else if url.starts_with("http://") {
        // Reject plain HTTP for security
        return Err("HTTP URLs are not allowed for security reasons. Please use HTTPS.".to_string());
    } else {
        // Local file path
        fetch_local_registry(url)?
    };

    serde_json::from_str(&content).map_err(|e| format!("Invalid registry JSON: {}", e))
}

/// Fetch registry from remote URL
fn fetch_remote_registry(url: &str, token: Option<&str>) -> Result<String, String> {
    // Validate URL for SSRF protection before making the request
    validate_url_for_ssrf(url)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        // Security: Explicit TLS configuration
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .https_only(true) // Prevent HTTP downgrade attacks (also enforced by SSRF validation)
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

    // Check Content-Length header if present to reject oversized responses early
    if let Some(content_length) = response.content_length() {
        if content_length > MAX_RESPONSE_SIZE {
            return Err(format!(
                "Response too large: {} bytes exceeds {} byte limit",
                content_length, MAX_RESPONSE_SIZE
            ));
        }
    }

    // Read response with size limit to prevent memory exhaustion
    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if bytes.len() as u64 > MAX_RESPONSE_SIZE {
        return Err(format!(
            "Response too large: {} bytes exceeds {} byte limit",
            bytes.len(),
            MAX_RESPONSE_SIZE
        ));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Invalid UTF-8 in response: {}", e))
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
