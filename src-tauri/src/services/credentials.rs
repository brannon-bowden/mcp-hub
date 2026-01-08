use keyring::Entry;
use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::OnceLock;

const SERVICE_NAME: &str = "mcp-hub";
const INSTALLATION_SECRET_KEY: &str = "_mcp_hub_installation_secret";

/// Per-installation secret for key obfuscation
/// This is generated once per installation and stored in the keyring itself
static INSTALLATION_SECRET: OnceLock<String> = OnceLock::new();

/// Get or create the installation secret
fn get_installation_secret() -> Result<String, String> {
    // Try to get from cache first
    if let Some(secret) = INSTALLATION_SECRET.get() {
        return Ok(secret.clone());
    }

    // Try to retrieve from keyring
    let entry = Entry::new(SERVICE_NAME, INSTALLATION_SECRET_KEY).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        "Failed to access credential storage".to_string()
    })?;

    match entry.get_password() {
        Ok(secret) => {
            let _ = INSTALLATION_SECRET.set(secret.clone());
            Ok(secret)
        }
        Err(keyring::Error::NoEntry) => {
            // Generate new secret
            let secret = format!(
                "{:x}{:x}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
                rand_u64()
            );

            // Store in keyring
            entry.set_password(&secret).map_err(|e| {
                log::error!("Failed to store installation secret: {}", e);
                "Failed to initialize credential storage".to_string()
            })?;

            let _ = INSTALLATION_SECRET.set(secret.clone());
            Ok(secret)
        }
        Err(e) => {
            log::error!("Failed to retrieve installation secret: {}", e);
            Err("Failed to access credential storage".to_string())
        }
    }
}

/// Generate a pseudo-random u64 using system entropy
fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    let state = RandomState::new();
    let mut hasher = state.build_hasher();
    std::time::Instant::now().hash(&mut hasher);
    hasher.finish()
}

/// Derive an obfuscated key from a logical key
/// This prevents credential enumeration by making keys unpredictable
fn derive_storage_key(logical_key: &str) -> Result<String, String> {
    let secret = get_installation_secret()?;

    let mut hasher = DefaultHasher::new();
    secret.hash(&mut hasher);
    logical_key.hash(&mut hasher);
    let hash = hasher.finish();

    // Use a prefix of the hash to keep keys reasonably short
    // but long enough to prevent collisions
    Ok(format!("k_{:016x}", hash))
}

/// Store a credential securely using the OS keyring
/// Keys are obfuscated to prevent enumeration attacks
pub fn store_credential(key: &str, value: &str) -> Result<(), String> {
    let storage_key = derive_storage_key(key)?;
    let entry = Entry::new(SERVICE_NAME, &storage_key).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        "Failed to access credential storage".to_string()
    })?;

    entry.set_password(value).map_err(|e| {
        log::error!("Failed to store credential: {}", e);
        "Failed to store credential".to_string()
    })
}

/// Retrieve a credential from the OS keyring
/// Keys are obfuscated to prevent enumeration attacks
pub fn get_credential(key: &str) -> Result<Option<String>, String> {
    let storage_key = derive_storage_key(key)?;
    let entry = Entry::new(SERVICE_NAME, &storage_key).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        "Failed to access credential storage".to_string()
    })?;

    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => {
            log::error!("Failed to retrieve credential: {}", e);
            Err("Failed to retrieve credential".to_string())
        }
    }
}

/// Delete a credential from the OS keyring
/// Keys are obfuscated to prevent enumeration attacks
pub fn delete_credential(key: &str) -> Result<(), String> {
    let storage_key = derive_storage_key(key)?;
    let entry = Entry::new(SERVICE_NAME, &storage_key).map_err(|e| {
        log::error!("Failed to create keyring entry: {}", e);
        "Failed to access credential storage".to_string()
    })?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => {
            log::error!("Failed to delete credential: {}", e);
            Err("Failed to delete credential".to_string())
        }
    }
}

/// Generate a unique key for storing server environment variable credentials
#[allow(dead_code)]
pub fn get_server_env_key(server_id: &str, env_var: &str) -> String {
    format!("server:{}:env:{}", server_id, env_var)
}

/// Store all environment variable credentials for a server
#[allow(dead_code)]
pub fn store_server_credentials(
    server_id: &str,
    env_vars: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    for (key, value) in env_vars {
        let credential_key = get_server_env_key(server_id, key);
        store_credential(&credential_key, value)?;
    }
    Ok(())
}

/// Retrieve all stored credentials for a server
/// Returns a map of env var names to their values
#[allow(dead_code)]
pub fn get_server_credentials(
    server_id: &str,
    env_var_names: &[String],
) -> Result<std::collections::HashMap<String, String>, String> {
    let mut credentials = std::collections::HashMap::new();

    for name in env_var_names {
        let credential_key = get_server_env_key(server_id, name);
        if let Some(value) = get_credential(&credential_key)? {
            credentials.insert(name.clone(), value);
        }
    }

    Ok(credentials)
}

/// Delete all credentials for a server
#[allow(dead_code)]
pub fn delete_server_credentials(server_id: &str, env_var_names: &[String]) -> Result<(), String> {
    for name in env_var_names {
        let credential_key = get_server_env_key(server_id, name);
        delete_credential(&credential_key)?;
    }
    Ok(())
}

/// Check if credential storage is available on this system
pub fn is_credential_storage_available() -> bool {
    // Try to create a test entry
    match Entry::new(SERVICE_NAME, "test-availability") {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_server_env_key() {
        let key = get_server_env_key("server-123", "API_KEY");
        assert_eq!(key, "server:server-123:env:API_KEY");
    }
}
