use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an MCP server configuration in the central registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ServerSource>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl McpServer {
    pub fn new(name: String, command: String, args: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            command,
            args,
            env: std::collections::HashMap::new(),
            tags: Vec::new(),
            source: Some(ServerSource {
                source_type: SourceType::Manual,
                url: None,
            }),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSource {
    pub source_type: SourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Manual,
    Imported,
    Registry,
}

/// Supported MCP client applications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ClientType {
    ClaudeDesktop,
    ClaudeCode,
    Cursor,
    Windsurf,
    Vscode,
    VscodeInsiders,
    Zed,
    Continue,
    Cody,
    Cline,
    RooCode,
    KiloCode,
    Amp,
    Augment,
    Antigravity,
    Jetbrains,
    GeminiCli,
    QwenCoder,
    Opencode,
    OpenaiCodex,
    Kiro,
    Trae,
    LmStudio,
    VisualStudio,
    Crush,
    Boltai,
    RovoDev,
    Zencoder,
    QodoGen,
    Perplexity,
    Factory,
    Emdash,
    AmazonQ,
    Warp,
    CopilotAgent,
    CopilotCli,
    Smithery,
    Custom,
}

impl ClientType {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            ClientType::ClaudeDesktop => "Claude Desktop",
            ClientType::ClaudeCode => "Claude Code",
            ClientType::Cursor => "Cursor",
            ClientType::Windsurf => "Windsurf",
            ClientType::Vscode => "VS Code",
            ClientType::VscodeInsiders => "VS Code Insiders",
            ClientType::Zed => "Zed",
            ClientType::Continue => "Continue",
            ClientType::Cody => "Sourcegraph Cody",
            ClientType::Cline => "Cline",
            ClientType::RooCode => "Roo Code",
            ClientType::KiloCode => "Kilo Code",
            ClientType::Amp => "Amp",
            ClientType::Augment => "Augment Code",
            ClientType::Antigravity => "Google Antigravity",
            ClientType::Jetbrains => "JetBrains AI",
            ClientType::GeminiCli => "Gemini CLI",
            ClientType::QwenCoder => "Qwen Coder",
            ClientType::Opencode => "Opencode",
            ClientType::OpenaiCodex => "OpenAI Codex",
            ClientType::Kiro => "Kiro",
            ClientType::Trae => "Trae",
            ClientType::LmStudio => "LM Studio",
            ClientType::VisualStudio => "Visual Studio 2022",
            ClientType::Crush => "Crush",
            ClientType::Boltai => "BoltAI",
            ClientType::RovoDev => "Rovo Dev CLI",
            ClientType::Zencoder => "Zencoder",
            ClientType::QodoGen => "Qodo Gen",
            ClientType::Perplexity => "Perplexity Desktop",
            ClientType::Factory => "Factory",
            ClientType::Emdash => "Emdash",
            ClientType::AmazonQ => "Amazon Q Developer",
            ClientType::Warp => "Warp",
            ClientType::CopilotAgent => "Copilot Coding Agent",
            ClientType::CopilotCli => "Copilot CLI",
            ClientType::Smithery => "Smithery",
            ClientType::Custom => "Custom",
        }
    }

    pub fn from_str(s: &str) -> Option<ClientType> {
        match s {
            "claude-desktop" => Some(ClientType::ClaudeDesktop),
            "claude-code" => Some(ClientType::ClaudeCode),
            "cursor" => Some(ClientType::Cursor),
            "windsurf" => Some(ClientType::Windsurf),
            "vscode" => Some(ClientType::Vscode),
            "vscode-insiders" => Some(ClientType::VscodeInsiders),
            "zed" => Some(ClientType::Zed),
            "continue" => Some(ClientType::Continue),
            "cody" => Some(ClientType::Cody),
            "cline" => Some(ClientType::Cline),
            "roo-code" => Some(ClientType::RooCode),
            "kilo-code" => Some(ClientType::KiloCode),
            "amp" => Some(ClientType::Amp),
            "augment" => Some(ClientType::Augment),
            "antigravity" => Some(ClientType::Antigravity),
            "jetbrains" => Some(ClientType::Jetbrains),
            "gemini-cli" => Some(ClientType::GeminiCli),
            "qwen-coder" => Some(ClientType::QwenCoder),
            "opencode" => Some(ClientType::Opencode),
            "openai-codex" => Some(ClientType::OpenaiCodex),
            "kiro" => Some(ClientType::Kiro),
            "trae" => Some(ClientType::Trae),
            "lm-studio" => Some(ClientType::LmStudio),
            "visual-studio" => Some(ClientType::VisualStudio),
            "crush" => Some(ClientType::Crush),
            "boltai" => Some(ClientType::Boltai),
            "rovo-dev" => Some(ClientType::RovoDev),
            "zencoder" => Some(ClientType::Zencoder),
            "qodo-gen" => Some(ClientType::QodoGen),
            "perplexity" => Some(ClientType::Perplexity),
            "factory" => Some(ClientType::Factory),
            "emdash" => Some(ClientType::Emdash),
            "amazon-q" => Some(ClientType::AmazonQ),
            "warp" => Some(ClientType::Warp),
            "copilot-agent" => Some(ClientType::CopilotAgent),
            "copilot-cli" => Some(ClientType::CopilotCli),
            "smithery" => Some(ClientType::Smithery),
            "custom" => Some(ClientType::Custom),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ClientType::ClaudeDesktop => "claude-desktop",
            ClientType::ClaudeCode => "claude-code",
            ClientType::Cursor => "cursor",
            ClientType::Windsurf => "windsurf",
            ClientType::Vscode => "vscode",
            ClientType::VscodeInsiders => "vscode-insiders",
            ClientType::Zed => "zed",
            ClientType::Continue => "continue",
            ClientType::Cody => "cody",
            ClientType::Cline => "cline",
            ClientType::RooCode => "roo-code",
            ClientType::KiloCode => "kilo-code",
            ClientType::Amp => "amp",
            ClientType::Augment => "augment",
            ClientType::Antigravity => "antigravity",
            ClientType::Jetbrains => "jetbrains",
            ClientType::GeminiCli => "gemini-cli",
            ClientType::QwenCoder => "qwen-coder",
            ClientType::Opencode => "opencode",
            ClientType::OpenaiCodex => "openai-codex",
            ClientType::Kiro => "kiro",
            ClientType::Trae => "trae",
            ClientType::LmStudio => "lm-studio",
            ClientType::VisualStudio => "visual-studio",
            ClientType::Crush => "crush",
            ClientType::Boltai => "boltai",
            ClientType::RovoDev => "rovo-dev",
            ClientType::Zencoder => "zencoder",
            ClientType::QodoGen => "qodo-gen",
            ClientType::Perplexity => "perplexity",
            ClientType::Factory => "factory",
            ClientType::Emdash => "emdash",
            ClientType::AmazonQ => "amazon-q",
            ClientType::Warp => "warp",
            ClientType::CopilotAgent => "copilot-agent",
            ClientType::CopilotCli => "copilot-cli",
            ClientType::Smithery => "smithery",
            ClientType::Custom => "custom",
        }
    }
}

/// Represents a client instance (profile) for an MCP client application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInstance {
    pub id: String,
    pub name: String,
    pub client_type: ClientType,
    pub config_path: String,
    pub enabled_servers: Vec<String>,
    pub is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ClientInstance {
    #[allow(dead_code)]
    pub fn new(name: String, client_type: ClientType, config_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            client_type,
            config_path,
            enabled_servers: Vec::new(),
            is_default: false,
            last_synced: None,
            last_modified: None,
            created_at: Utc::now(),
        }
    }
}

/// Mapping between server and instance with enabled state
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceServerMapping {
    pub instance_id: String,
    pub server_id: String,
    pub enabled: bool,
}

/// Configuration backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigBackup {
    pub id: String,
    pub instance_id: String,
    pub backup_path: String,
    pub created_at: DateTime<Utc>,
}

impl ConfigBackup {
    pub fn new(instance_id: String, backup_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            instance_id,
            backup_path,
            created_at: Utc::now(),
        }
    }
}

/// MCP configuration format for client config files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigFile {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: std::collections::HashMap<String, McpServerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub env: std::collections::HashMap<String, String>,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: Theme,
    pub auto_start: bool,
    pub create_backups: bool,
    pub backup_retention_days: u32,
    /// Discovery settings
    #[serde(default)]
    pub discovery: DiscoverySettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            auto_start: false,
            create_backups: true,
            backup_retention_days: 30,
            discovery: DiscoverySettings::default(),
        }
    }
}

/// MCP Discovery settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverySettings {
    /// Enable ~/.mcp/ directory discovery
    pub mcp_directory_enabled: bool,
    /// Enable local HTTP server discovery
    pub http_server_enabled: bool,
    /// Port for the local HTTP server (default: 24368)
    pub http_server_port: u16,
}

impl Default for DiscoverySettings {
    fn default() -> Self {
        Self {
            mcp_directory_enabled: false,
            http_server_enabled: false,
            http_server_port: 24368,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Server health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerHealth {
    pub server_id: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub last_checked: DateTime<Utc>,
}
