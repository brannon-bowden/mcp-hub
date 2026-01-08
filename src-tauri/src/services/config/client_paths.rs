//! Client-specific configuration path functions
//!
//! This module contains functions that return the default config file path
//! for each supported MCP client application.

use std::path::PathBuf;

use crate::models::ClientType;

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
    dirs::home_dir().map(|home| home.join(".claude.json"))
}

fn get_cursor_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".cursor/mcp.json"))
}

fn get_windsurf_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codeium/windsurf/mcp_config.json"))
}

fn get_vscode_config_path() -> Option<PathBuf> {
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
        dirs::home_dir().map(|home| home.join(".config/zed/settings.json"))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("Zed/settings.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|home| home.join(".config/zed/settings.json"))
    }
}

fn get_continue_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".continue/config.json"))
}

fn get_cody_config_path() -> Option<PathBuf> {
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
    dirs::home_dir().map(|home| home.join(".amp/mcp.json"))
}

fn get_augment_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Code/User/settings.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("Code/User/settings.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("Code/User/settings.json"))
    }
}

fn get_antigravity_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".gemini/antigravity/mcp_config.json"))
}

fn get_jetbrains_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".junie/mcp/mcp.json"))
}

fn get_gemini_cli_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".gemini/settings.json"))
}

fn get_qwen_coder_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".qwen-coder/mcp.json"))
}

fn get_opencode_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".opencode/mcp.json"))
}

fn get_openai_codex_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex/mcp.json"))
}

fn get_kiro_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".kiro/settings/mcp.json"))
}

fn get_trae_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".trae/mcp.json"))
}

fn get_lm_studio_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/LM Studio/mcp.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("LM Studio/mcp.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("LM Studio/mcp.json"))
    }
}

fn get_visual_studio_config_path() -> Option<PathBuf> {
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
    dirs::home_dir().map(|home| home.join(".crush/mcp.json"))
}

fn get_boltai_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/BoltAI/mcp.json")
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        dirs::config_dir().map(|config| config.join("BoltAI/mcp.json"))
    }
}

fn get_rovo_dev_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".rovo/mcp.json"))
}

fn get_zencoder_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".zencoder/mcp.json"))
}

fn get_qodo_gen_config_path() -> Option<PathBuf> {
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
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| {
            home.join("Library/Application Support/Perplexity/mcp.json")
        })
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|config| config.join("Perplexity/mcp.json"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|config| config.join("Perplexity/mcp.json"))
    }
}

fn get_factory_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".factory/mcp.json"))
}

fn get_emdash_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".emdash/mcp.json"))
}

fn get_amazon_q_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".aws/amazonq/mcp.json"))
}

fn get_warp_config_path() -> Option<PathBuf> {
    // Warp terminal configures MCP via Warp Drive sync, not a local config file
    None
}

fn get_copilot_agent_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".github/copilot/mcp.json"))
}

fn get_copilot_cli_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".github/copilot-cli/mcp.json"))
}

fn get_smithery_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".smithery/mcp.json"))
}

/// Check if a client type uses a config file that contains other settings
/// beyond just MCP servers (requiring merge-aware writes)
pub fn client_requires_merge_write(client_type: &ClientType) -> bool {
    matches!(
        client_type,
        ClientType::ClaudeCode
            | ClientType::Zed           // settings.json with other Zed settings
            | ClientType::Augment       // VS Code settings.json
            | ClientType::GeminiCli     // settings.json with other Gemini settings
    )
}
