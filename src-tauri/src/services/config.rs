use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::models::{ClientInstance, ClientType, McpConfigFile, McpServer, McpServerEntry};

/// Get the default configuration path for a client type on the current platform
pub fn get_default_config_path(client_type: &ClientType) -> Option<PathBuf> {
    match client_type {
        ClientType::ClaudeDesktop => get_claude_desktop_config_path(),
        ClientType::ClaudeCode => get_claude_code_config_path(),
        ClientType::Cursor => get_cursor_config_path(),
        ClientType::Windsurf => get_windsurf_config_path(),
        ClientType::Vscode => get_vscode_config_path(),
        ClientType::VscodeInsiders => get_vscode_insiders_config_path(),
        ClientType::Zed => get_zed_config_path(),
        ClientType::Continue => get_continue_config_path(),
        ClientType::Cody => get_cody_config_path(),
        ClientType::Cline => get_cline_config_path(),
        ClientType::RooCode => get_roo_code_config_path(),
        ClientType::KiloCode => get_kilo_code_config_path(),
        ClientType::Amp => get_amp_config_path(),
        ClientType::Augment => get_augment_config_path(),
        ClientType::Antigravity => get_antigravity_config_path(),
        ClientType::Jetbrains => get_jetbrains_config_path(),
        ClientType::GeminiCli => get_gemini_cli_config_path(),
        ClientType::QwenCoder => get_qwen_coder_config_path(),
        ClientType::Opencode => get_opencode_config_path(),
        ClientType::OpenaiCodex => get_openai_codex_config_path(),
        ClientType::Kiro => get_kiro_config_path(),
        ClientType::Trae => get_trae_config_path(),
        ClientType::LmStudio => get_lm_studio_config_path(),
        ClientType::VisualStudio => get_visual_studio_config_path(),
        ClientType::Crush => get_crush_config_path(),
        ClientType::Boltai => get_boltai_config_path(),
        ClientType::RovoDev => get_rovo_dev_config_path(),
        ClientType::Zencoder => get_zencoder_config_path(),
        ClientType::QodoGen => get_qodo_gen_config_path(),
        ClientType::Perplexity => get_perplexity_config_path(),
        ClientType::Factory => get_factory_config_path(),
        ClientType::Emdash => get_emdash_config_path(),
        ClientType::AmazonQ => get_amazon_q_config_path(),
        ClientType::Warp => get_warp_config_path(),
        ClientType::CopilotAgent => get_copilot_agent_config_path(),
        ClientType::CopilotCli => get_copilot_cli_config_path(),
        ClientType::Smithery => get_smithery_config_path(),
        ClientType::Custom => None,
    }
}

fn get_claude_desktop_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Claude/claude_desktop_config.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("Claude/claude_desktop_config.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("Claude/claude_desktop_config.json"))
    }
}

fn get_claude_code_config_path() -> Option<PathBuf> {
    // User-scoped config
    dirs::home_dir().map(|home| home.join(".claude.json"))
}

fn get_cursor_config_path() -> Option<PathBuf> {
    // Cursor uses ~/.cursor/mcp.json
    dirs::home_dir().map(|home| home.join(".cursor/mcp.json"))
}

fn get_windsurf_config_path() -> Option<PathBuf> {
    // Windsurf uses ~/.codeium/windsurf/mcp_config.json
    dirs::home_dir().map(|home| home.join(".codeium/windsurf/mcp_config.json"))
}

fn get_vscode_config_path() -> Option<PathBuf> {
    // VS Code native MCP support (user-level config)
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/mcp.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("Code/User/mcp.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("Code/User/mcp.json"))
    }
}

fn get_vscode_insiders_config_path() -> Option<PathBuf> {
    // VS Code Insiders native MCP support (user-level config)
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code - Insiders/User/mcp.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("Code - Insiders/User/mcp.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("Code - Insiders/User/mcp.json"))
    }
}

fn get_zed_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join(".config/zed/settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Zed/settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|home| {
            home.join(".config/zed/settings.json")
        })
    }
}

fn get_continue_config_path() -> Option<PathBuf> {
    // Continue.dev extension config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join(".continue/config.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::home_dir().map(|home| {
            home.join(".continue/config.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|home| {
            home.join(".continue/config.json")
        })
    }
}

fn get_cody_config_path() -> Option<PathBuf> {
    // Sourcegraph Cody config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/globalStorage/sourcegraph.cody-ai/cody_mcp_settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/sourcegraph.cody-ai/cody_mcp_settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/sourcegraph.cody-ai/cody_mcp_settings.json")
        })
    }
}

fn get_cline_config_path() -> Option<PathBuf> {
    // Cline VS Code extension config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json")
        })
    }
}

fn get_roo_code_config_path() -> Option<PathBuf> {
    // Roo Code extension config (VS Code extension)
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json")
        })
    }
}

fn get_kilo_code_config_path() -> Option<PathBuf> {
    // Kilo Code uses mcp_settings.json global or .kilocode/mcp.json project-level
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/globalStorage/kilocode.kilocode/mcp_settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/kilocode.kilocode/mcp_settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/kilocode.kilocode/mcp_settings.json")
        })
    }
}

fn get_amp_config_path() -> Option<PathBuf> {
    // Amp uses ~/.amp/mcp.json
    dirs::home_dir().map(|home| home.join(".amp/mcp.json"))
}

fn get_augment_config_path() -> Option<PathBuf> {
    // Augment Code uses VS Code settings with augment.advanced
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/settings.json")
        })
    }
}

fn get_antigravity_config_path() -> Option<PathBuf> {
    // Google Antigravity uses ~/.gemini/antigravity/mcp_config.json
    dirs::home_dir().map(|home| home.join(".gemini/antigravity/mcp_config.json"))
}

fn get_jetbrains_config_path() -> Option<PathBuf> {
    // JetBrains IDEs with Junie use ~/.junie/mcp/mcp.json for global config
    // Note: JetBrains AI Assistant configures MCP via IDE settings, not a file
    dirs::home_dir().map(|home| home.join(".junie/mcp/mcp.json"))
}

fn get_gemini_cli_config_path() -> Option<PathBuf> {
    // Gemini CLI uses ~/.gemini/settings.json
    dirs::home_dir().map(|home| home.join(".gemini/settings.json"))
}

fn get_qwen_coder_config_path() -> Option<PathBuf> {
    // Qwen Coder config path (estimated)
    dirs::home_dir().map(|home| home.join(".qwen-coder/mcp.json"))
}

fn get_opencode_config_path() -> Option<PathBuf> {
    // Opencode config path
    dirs::home_dir().map(|home| home.join(".opencode/mcp.json"))
}

fn get_openai_codex_config_path() -> Option<PathBuf> {
    // OpenAI Codex CLI config
    dirs::home_dir().map(|home| home.join(".codex/mcp.json"))
}

fn get_kiro_config_path() -> Option<PathBuf> {
    // Kiro uses ~/.kiro/settings/mcp.json
    dirs::home_dir().map(|home| home.join(".kiro/settings/mcp.json"))
}

fn get_trae_config_path() -> Option<PathBuf> {
    // Trae config path
    dirs::home_dir().map(|home| home.join(".trae/mcp.json"))
}

fn get_lm_studio_config_path() -> Option<PathBuf> {
    // LM Studio MCP config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/LM Studio/mcp.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("LM Studio/mcp.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("LM Studio/mcp.json")
        })
    }
}

fn get_visual_studio_config_path() -> Option<PathBuf> {
    // Visual Studio 2022 MCP config
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Microsoft/VisualStudio/mcp.json")
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

fn get_crush_config_path() -> Option<PathBuf> {
    // Crush config path
    dirs::home_dir().map(|home| home.join(".crush/mcp.json"))
}

fn get_boltai_config_path() -> Option<PathBuf> {
    // BoltAI MCP config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/BoltAI/mcp.json")
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        dirs::config_dir().map(|config| {
            config.join("BoltAI/mcp.json")
        })
    }
}

fn get_rovo_dev_config_path() -> Option<PathBuf> {
    // Rovo Dev CLI config
    dirs::home_dir().map(|home| home.join(".rovo/mcp.json"))
}

fn get_zencoder_config_path() -> Option<PathBuf> {
    // Zencoder config path
    dirs::home_dir().map(|home| home.join(".zencoder/mcp.json"))
}

fn get_qodo_gen_config_path() -> Option<PathBuf> {
    // Qodo Gen (VS Code extension) config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/globalStorage/qodo-ai.qodo-gen/mcp_settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/qodo-ai.qodo-gen/mcp_settings.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Code/User/globalStorage/qodo-ai.qodo-gen/mcp_settings.json")
        })
    }
}

fn get_perplexity_config_path() -> Option<PathBuf> {
    // Perplexity Desktop config
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Perplexity/mcp.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| {
            config.join("Perplexity/mcp.json")
        })
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| {
            config.join("Perplexity/mcp.json")
        })
    }
}

fn get_factory_config_path() -> Option<PathBuf> {
    // Factory config path
    dirs::home_dir().map(|home| home.join(".factory/mcp.json"))
}

fn get_emdash_config_path() -> Option<PathBuf> {
    // Emdash config path
    dirs::home_dir().map(|home| home.join(".emdash/mcp.json"))
}

fn get_amazon_q_config_path() -> Option<PathBuf> {
    // Amazon Q Developer CLI config
    dirs::home_dir().map(|home| home.join(".aws/amazonq/mcp.json"))
}

fn get_warp_config_path() -> Option<PathBuf> {
    // Warp terminal configures MCP via Warp Drive sync, not a local config file
    // See: https://github.com/warpdotdev/Warp/issues/6602 for feature request
    None
}

fn get_copilot_agent_config_path() -> Option<PathBuf> {
    // GitHub Copilot Coding Agent config
    dirs::home_dir().map(|home| home.join(".github/copilot/mcp.json"))
}

fn get_copilot_cli_config_path() -> Option<PathBuf> {
    // GitHub Copilot CLI config
    dirs::home_dir().map(|home| home.join(".github/copilot-cli/mcp.json"))
}

fn get_smithery_config_path() -> Option<PathBuf> {
    // Smithery config path
    dirs::home_dir().map(|home| home.join(".smithery/mcp.json"))
}

/// Check if a config file exists
pub fn config_exists(path: &PathBuf) -> bool {
    path.exists() && path.is_file()
}

/// Read and parse an MCP configuration file
pub fn read_config_file(path: &PathBuf) -> Result<McpConfigFile, String> {
    if !config_exists(path) {
        return Ok(McpConfigFile {
            mcp_servers: HashMap::new(),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

    // Handle empty files
    if content.trim().is_empty() {
        return Ok(McpConfigFile {
            mcp_servers: HashMap::new(),
        });
    }

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
}

/// Write MCP configuration to a file (overwrites entire file)
pub fn write_config_file(path: &PathBuf, config: &McpConfigFile) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))
}

/// Write MCP servers to a config file, preserving other fields in the file
/// This is used for config files like ~/.claude.json that contain other settings
pub fn write_mcp_servers_preserving_config(
    path: &PathBuf,
    mcp_servers: &HashMap<String, McpServerEntry>,
) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Read existing content or start with empty object
    let mut existing: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        if content.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))?
        }
    } else {
        serde_json::json!({})
    };

    // Ensure we have an object at the root
    let obj = existing.as_object_mut()
        .ok_or_else(|| "Config file is not a JSON object".to_string())?;

    // Update only the mcpServers field
    let servers_value = serde_json::to_value(mcp_servers)
        .map_err(|e| format!("Failed to serialize MCP servers: {}", e))?;
    obj.insert("mcpServers".to_string(), servers_value);

    // Write back the merged config
    let content = serde_json::to_string_pretty(&existing)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))
}

/// Create a backup of a config file
pub fn backup_config_file(path: &PathBuf, backup_dir: &PathBuf) -> Result<PathBuf, String> {
    if !config_exists(path) {
        return Err("Config file does not exist".to_string());
    }

    fs::create_dir_all(backup_dir).map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config");
    let backup_filename = format!("{}_{}.backup", filename, timestamp);
    let backup_path = backup_dir.join(backup_filename);

    fs::copy(path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    Ok(backup_path)
}

/// Check if a client type uses a config file that contains other settings
/// beyond just MCP servers (requiring merge-aware writes)
fn client_requires_merge_write(client_type: &ClientType) -> bool {
    matches!(
        client_type,
        ClientType::ClaudeCode
            | ClientType::Zed           // settings.json with other Zed settings
            | ClientType::Augment       // VS Code settings.json
            | ClientType::GeminiCli     // settings.json with other Gemini settings
    )
}

/// Convert servers to MCP config format and write to instance config file
pub fn sync_servers_to_instance(
    instance: &ClientInstance,
    servers: &[McpServer],
    backup_dir: Option<&PathBuf>,
) -> Result<Option<PathBuf>, String> {
    let config_path = PathBuf::from(&instance.config_path);
    let mut backup_path = None;

    // Create backup if requested and file exists
    if let Some(dir) = backup_dir {
        if config_exists(&config_path) {
            backup_path = Some(backup_config_file(&config_path, dir)?);
        }
    }

    // Build the MCP servers map
    let mut mcp_servers = HashMap::new();

    for server in servers {
        if instance.enabled_servers.contains(&server.id) {
            let entry = McpServerEntry {
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.clone(),
            };
            // Use server name as the key (sanitized)
            let key = sanitize_server_name(&server.name);
            mcp_servers.insert(key, entry);
        }
    }

    // Use merge-aware write for clients that have other settings in their config file
    if client_requires_merge_write(&instance.client_type) {
        write_mcp_servers_preserving_config(&config_path, &mcp_servers)?;
    } else {
        let config = McpConfigFile { mcp_servers };
        write_config_file(&config_path, &config)?;
    }

    Ok(backup_path)
}

/// Sanitize server name for use as a config key
fn sanitize_server_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Import servers from an existing config file
pub fn import_servers_from_config(path: &PathBuf) -> Result<Vec<McpServer>, String> {
    let config = read_config_file(path)?;
    let mut servers = Vec::new();

    for (name, entry) in config.mcp_servers {
        let mut server = McpServer::new(name.clone(), entry.command, entry.args);
        server.env = entry.env;
        server.source = Some(crate::models::ServerSource {
            source_type: crate::models::SourceType::Imported,
            url: Some(path.to_string_lossy().to_string()),
        });
        servers.push(server);
    }

    Ok(servers)
}

/// Detect installed MCP clients and return their config paths
pub fn detect_installed_clients() -> Vec<(ClientType, PathBuf)> {
    let mut clients = Vec::new();

    let client_types = [
        ClientType::ClaudeDesktop,
        ClientType::ClaudeCode,
        ClientType::Cursor,
        ClientType::Windsurf,
        ClientType::Vscode,
        ClientType::VscodeInsiders,
        ClientType::Zed,
        ClientType::Continue,
        ClientType::Cody,
        ClientType::Cline,
        ClientType::RooCode,
        ClientType::KiloCode,
        ClientType::Amp,
        ClientType::Augment,
        ClientType::Antigravity,
        ClientType::Jetbrains,
        ClientType::GeminiCli,
        ClientType::QwenCoder,
        ClientType::Opencode,
        ClientType::OpenaiCodex,
        ClientType::Kiro,
        ClientType::Trae,
        ClientType::LmStudio,
        ClientType::VisualStudio,
        ClientType::Crush,
        ClientType::Boltai,
        ClientType::RovoDev,
        ClientType::Zencoder,
        ClientType::QodoGen,
        ClientType::Perplexity,
        ClientType::Factory,
        ClientType::Emdash,
        ClientType::AmazonQ,
        ClientType::Warp,
        ClientType::CopilotAgent,
        ClientType::CopilotCli,
        ClientType::Smithery,
    ];

    for client_type in client_types {
        if let Some(path) = get_default_config_path(&client_type) {
            // Check if the parent directory exists (client might be installed even if no config yet)
            let exists = if let Some(parent) = path.parent() {
                parent.exists()
            } else {
                path.exists()
            };

            if exists || config_exists(&path) {
                clients.push((client_type, path));
            }
        }
    }

    clients
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_server_name() {
        assert_eq!(sanitize_server_name("My Server"), "my-server");
        assert_eq!(sanitize_server_name("server_123"), "server_123");
        assert_eq!(sanitize_server_name("  test  "), "test");
        assert_eq!(sanitize_server_name("hello@world!"), "hello-world");
    }
}
