//! Client detection utilities
//!
//! This module provides functions to detect installed MCP clients
//! on the system.

use std::path::PathBuf;

use crate::models::ClientType;

use super::client_paths::get_default_config_path;
use super::operations::config_exists;

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
