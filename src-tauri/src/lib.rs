mod commands;
mod db;
mod models;
mod services;

use commands::AppState;
use db::Database;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::init();

    // Get database path
    let db_path = services::config::get_database_path()
        .expect("Failed to determine database path");

    // Initialize database
    let database = Database::new(db_path).expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            db: Mutex::new(database),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
