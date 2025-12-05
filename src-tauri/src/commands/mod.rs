use chrono::Utc;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

use crate::db::Database;
use crate::models::{
    AppSettings, ClientInstance, ClientType, ConfigBackup, McpServer, ServerHealth, HealthStatus,
};
use crate::services::{self, config};

pub struct AppState {
    pub db: Mutex<Database>,
}

// ==================== Server Commands ====================

#[tauri::command]
pub fn get_servers(state: State<AppState>) -> Result<Vec<McpServer>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_servers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_server(state: State<AppState>, id: String) -> Result<Option<McpServer>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_server(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_server(state: State<AppState>, server: McpServer) -> Result<McpServer, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_server(&server).map_err(|e| e.to_string())?;
    Ok(server)
}

#[tauri::command]
pub fn update_server(state: State<AppState>, server: McpServer) -> Result<McpServer, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_server(&server).map_err(|e| e.to_string())?;
    Ok(server)
}

#[tauri::command]
pub fn delete_server(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_server(&id).map_err(|e| e.to_string())
}

// ==================== Instance Commands ====================

#[tauri::command]
pub fn get_instances(state: State<AppState>) -> Result<Vec<ClientInstance>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_instances().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_instance(state: State<AppState>, id: String) -> Result<Option<ClientInstance>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_instance(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_instance(
    state: State<AppState>,
    instance: ClientInstance,
) -> Result<ClientInstance, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_instance(&instance).map_err(|e| e.to_string())?;
    Ok(instance)
}

#[tauri::command]
pub fn update_instance(
    state: State<AppState>,
    instance: ClientInstance,
) -> Result<ClientInstance, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_instance(&instance).map_err(|e| e.to_string())?;
    Ok(instance)
}

#[tauri::command]
pub fn delete_instance(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_instance(&id).map_err(|e| e.to_string())
}

// ==================== Server-Instance Mapping Commands ====================

#[tauri::command]
pub fn set_server_enabled(
    state: State<AppState>,
    instance_id: String,
    server_id: String,
    enabled: bool,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_server_enabled_for_instance(&instance_id, &server_id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_enabled_servers(
    state: State<AppState>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_enabled_servers_for_instance(&instance_id)
        .map_err(|e| e.to_string())
}

// ==================== Sync Commands ====================

#[tauri::command]
pub fn sync_instance(state: State<AppState>, instance_id: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Get instance
    let mut instance = db
        .get_instance(&instance_id)
        .map_err(|e| e.to_string())?
        .ok_or("Instance not found")?;

    // Get enabled servers for this instance
    instance.enabled_servers = db
        .get_enabled_servers_for_instance(&instance_id)
        .map_err(|e| e.to_string())?;

    // Get all servers
    let servers = db.get_all_servers().map_err(|e| e.to_string())?;

    // Get backup directory
    let backup_dir = config::get_backup_dir();

    // Sync configuration
    let backup_path = config::sync_servers_to_instance(
        &instance,
        &servers,
        backup_dir.as_ref(),
    )?;

    // Record backup if created
    if let Some(ref path) = backup_path {
        let backup = ConfigBackup::new(instance_id.clone(), path.to_string_lossy().to_string());
        db.create_backup(&backup).map_err(|e| e.to_string())?;
    }

    // Update last synced timestamp
    instance.last_synced = Some(Utc::now());
    db.update_instance(&instance).map_err(|e| e.to_string())?;

    Ok(backup_path.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn sync_all_instances(state: State<AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let instances = db.get_all_instances().map_err(|e| e.to_string())?;
    drop(db); // Release lock before calling sync_instance

    let mut synced = Vec::new();
    for instance in instances {
        // Re-acquire state for each instance
        match sync_instance(state.clone(), instance.id.clone()) {
            Ok(_) => synced.push(instance.id),
            Err(e) => log::error!("Failed to sync instance {}: {}", instance.id, e),
        }
    }

    Ok(synced)
}

// ==================== Import/Export Commands ====================

#[tauri::command]
pub fn import_from_file(state: State<AppState>, path: String) -> Result<Vec<McpServer>, String> {
    let path = PathBuf::from(path);
    let servers = config::import_servers_from_config(&path)?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    for server in &servers {
        db.create_server(server).map_err(|e| e.to_string())?;
    }

    Ok(servers)
}

#[tauri::command]
pub fn detect_clients() -> Result<Vec<DetectedClient>, String> {
    let detected = config::detect_installed_clients();

    Ok(detected
        .into_iter()
        .map(|(client_type, path)| DetectedClient {
            client_type,
            config_path: path.to_string_lossy().to_string(),
            has_config: path.exists(),
        })
        .collect())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedClient {
    pub client_type: ClientType,
    pub config_path: String,
    pub has_config: bool,
}

// ==================== Credential Commands ====================

#[tauri::command]
pub fn store_credential(key: String, value: String) -> Result<(), String> {
    services::credentials::store_credential(&key, &value)
}

#[tauri::command]
pub fn get_credential(key: String) -> Result<Option<String>, String> {
    services::credentials::get_credential(&key)
}

#[tauri::command]
pub fn delete_credential(key: String) -> Result<(), String> {
    services::credentials::delete_credential(&key)
}

#[tauri::command]
pub fn is_credential_storage_available() -> bool {
    services::credentials::is_credential_storage_available()
}

// ==================== Backup Commands ====================

#[tauri::command]
pub fn get_backups(state: State<AppState>, instance_id: String) -> Result<Vec<ConfigBackup>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_backups_for_instance(&instance_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_backup(backup_id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // This is a simplified restore - in production you'd want more validation
    let _backups = db
        .get_backups_for_instance(&backup_id)
        .map_err(|e| e.to_string())?;

    // TODO: Implement actual restore logic
    Err("Restore not yet implemented".to_string())
}

// ==================== Settings Commands ====================

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let settings_json = db
        .get_setting("app_settings")
        .map_err(|e| e.to_string())?;

    match settings_json {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        None => Ok(AppSettings::default()),
    }
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    db.set_setting("app_settings", &json)
        .map_err(|e| e.to_string())
}

// ==================== Health Check Commands ====================

#[tauri::command]
pub async fn check_server_health(server: McpServer) -> Result<ServerHealth, String> {
    use std::process::Command;
    use std::time::Duration;

    // Try to run the command with --version or --help to check if it exists
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        let output = Command::new(&server.command)
            .args(["--version"])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(ServerHealth {
                        server_id: server.id.clone(),
                        status: HealthStatus::Healthy,
                        error_message: None,
                        last_checked: Utc::now(),
                    })
                } else {
                    // Command exists but returned error - might still be healthy
                    Ok(ServerHealth {
                        server_id: server.id.clone(),
                        status: HealthStatus::Unknown,
                        error_message: Some("Command returned non-zero exit code".to_string()),
                        last_checked: Utc::now(),
                    })
                }
            }
            Err(e) => Ok(ServerHealth {
                server_id: server.id.clone(),
                status: HealthStatus::Error,
                error_message: Some(format!("Failed to execute command: {}", e)),
                last_checked: Utc::now(),
            }),
        }
    })
    .await;

    match result {
        Ok(health) => health,
        Err(_) => Ok(ServerHealth {
            server_id: server.id,
            status: HealthStatus::Error,
            error_message: Some("Health check timed out".to_string()),
            last_checked: Utc::now(),
        }),
    }
}

// ==================== Utility Commands ====================

#[tauri::command]
pub fn get_app_data_dir() -> Result<String, String> {
    config::get_app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "Could not determine app data directory".to_string())
}

#[tauri::command]
pub fn get_default_config_path(client_type: ClientType) -> Result<Option<String>, String> {
    Ok(config::get_default_config_path(&client_type).map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn read_config_file(path: String) -> Result<crate::models::McpConfigFile, String> {
    let path = PathBuf::from(path);
    config::read_config_file(&path)
}

// ==================== Registry Commands ====================

#[tauri::command]
pub fn get_registries() -> Vec<services::registry::RegistrySource> {
    services::registry::get_available_registries()
}

#[tauri::command]
pub async fn get_registry_servers(registry_id: String) -> Result<Vec<services::registry::RegistryServer>, String> {
    services::registry::fetch_registry_servers(&registry_id).await
}

#[tauri::command]
pub fn import_from_registry(
    state: State<AppState>,
    registry_id: String,
    servers: Vec<services::registry::RegistryServer>,
) -> Result<Vec<McpServer>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut imported = Vec::new();

    for registry_server in servers {
        let server = services::registry::registry_server_to_mcp_server(&registry_server, &registry_id);
        db.create_server(&server).map_err(|e| e.to_string())?;
        imported.push(server);
    }

    Ok(imported)
}
