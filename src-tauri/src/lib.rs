mod commands;
mod db;
mod models;
mod services;

use commands::AppState;
use db::Database;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::init();

    // Get database path
    let db_path = services::config::get_database_path()
        .expect("Failed to determine database path");

    // Initialize database
    let database = Database::new(db_path).expect("Failed to initialize database");

    // Create shared discovery server handle
    let discovery_server = Arc::new(RwLock::new(None));

    // Clone for setup hook
    let discovery_server_setup = discovery_server.clone();
    let db_for_setup = Database::new(
        services::config::get_database_path().expect("Failed to determine database path"),
    )
    .expect("Failed to initialize database for setup");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db: Mutex::new(database),
            discovery_server,
        })
        .setup(move |_app| {
            // Initialize discovery services based on saved settings
            let discovery_server = discovery_server_setup.clone();

            tauri::async_runtime::spawn(async move {
                // Load settings
                let settings_json = db_for_setup.get_setting("app_settings").ok().flatten();
                let settings: models::AppSettings = settings_json
                    .and_then(|json| serde_json::from_str(&json).ok())
                    .unwrap_or_default();

                // Get servers for discovery
                let servers = db_for_setup.get_all_servers().unwrap_or_default();

                // Initialize ~/.mcp directory if enabled
                if settings.discovery.mcp_directory_enabled {
                    if let Err(e) = services::discovery::write_mcp_directory(&servers) {
                        log::error!("Failed to initialize ~/.mcp directory: {}", e);
                    } else {
                        log::info!("MCP directory discovery initialized with {} servers", servers.len());
                    }
                }

                // Start HTTP discovery server if enabled
                if settings.discovery.http_server_enabled {
                    match services::discovery::start_discovery_server(
                        settings.discovery.http_server_port,
                        servers,
                    )
                    .await
                    {
                        Ok(handle) => {
                            let mut guard = discovery_server.write().await;
                            *guard = Some(handle);
                            log::info!(
                                "Discovery HTTP server started on port {}",
                                settings.discovery.http_server_port
                            );
                        }
                        Err(e) => {
                            log::error!("Failed to start discovery HTTP server: {}", e);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Server commands
            commands::get_servers,
            commands::get_server,
            commands::create_server,
            commands::update_server,
            commands::delete_server,
            // Instance commands
            commands::get_instances,
            commands::get_instance,
            commands::create_instance,
            commands::update_instance,
            commands::delete_instance,
            // Server-Instance mapping
            commands::set_server_enabled,
            commands::get_enabled_servers,
            // Sync commands
            commands::sync_instance,
            commands::sync_all_instances,
            // Import/Export
            commands::import_from_file,
            commands::detect_clients,
            // Credentials
            commands::store_credential,
            commands::get_credential,
            commands::delete_credential,
            commands::is_credential_storage_available,
            // Backups
            commands::get_backups,
            commands::restore_backup,
            // Settings
            commands::get_settings,
            commands::save_settings,
            // Health
            commands::check_server_health,
            // Utility
            commands::get_app_data_dir,
            commands::get_default_config_path,
            commands::read_config_file,
            // Registry
            commands::get_registries,
            commands::get_registry_servers,
            commands::import_from_registry,
            // Discovery
            commands::get_discovery_settings,
            commands::update_discovery_settings,
            commands::refresh_discovery,
            commands::get_discovery_status,
            commands::check_port_available,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
