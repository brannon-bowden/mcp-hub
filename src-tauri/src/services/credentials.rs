use keyring::Entry;

const SERVICE_NAME: &str = "mcp-hub";

/// Store a credential securely using the OS keyring
pub fn store_credential(key: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(value)
        .map_err(|e| format!("Failed to store credential: {}", e))
}

/// Retrieve a credential from the OS keyring
pub fn get_credential(key: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve credential: {}", e)),
    }
}

/// Delete a credential from the OS keyring
pub fn delete_credential(key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(format!("Failed to delete credential: {}", e)),
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
