//! MCP Discovery Service
//!
//! Implements two discovery mechanisms:
//! 1. ~/.mcp/ directory - Markdown files for each server (mcp-local-spec)
//! 2. Local HTTP server - /.well-known/mcp.json endpoint (SEP-1649)

use crate::models::McpServer;
use axum::{
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

/// MCP Server Card format (SEP-1649 compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerCard {
    /// Schema version
    pub schema_version: String,
    /// Server name
    pub name: String,
    /// Server description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Server homepage/documentation URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Server icon URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Transport configuration for this server
    pub transport: TransportConfig,
    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportConfig {
    /// Transport type (stdio for local servers)
    #[serde(rename = "type")]
    pub transport_type: String,
    /// Command to run the server
    pub command: String,
    /// Arguments for the command
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// Discovery index format for /.well-known/mcp.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDiscoveryIndex {
    /// Schema version
    pub schema_version: String,
    /// Provider name
    pub provider: String,
    /// Provider description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// List of available servers
    pub servers: Vec<McpServerCard>,
    /// Timestamp of last update
    pub updated_at: String,
}

/// State shared with the HTTP server
pub struct DiscoveryState {
    pub servers: RwLock<Vec<McpServer>>,
}

// ==================== ~/.mcp/ Directory Discovery ====================

/// Get the ~/.mcp directory path
pub fn get_mcp_directory() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".mcp"))
}

/// Sanitize server name for use as filename
fn sanitize_filename(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Generate markdown content for a server (mcp-local-spec format)
fn generate_server_markdown(server: &McpServer) -> String {
    let mut content = String::new();

    // YAML frontmatter
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", server.id));
    content.push_str(&format!("name: {}\n", server.name));
    if let Some(ref desc) = server.description {
        content.push_str(&format!("description: {}\n", desc));
    }
    content.push_str(&format!("command: {}\n", server.command));
    if !server.args.is_empty() {
        content.push_str("args:\n");
        for arg in &server.args {
            content.push_str(&format!("  - \"{}\"\n", arg));
        }
    }
    if !server.env.is_empty() {
        content.push_str("env:\n");
        for (key, value) in &server.env {
            // Mask sensitive values
            let masked = if key.to_lowercase().contains("key")
                || key.to_lowercase().contains("secret")
                || key.to_lowercase().contains("token")
                || key.to_lowercase().contains("password")
            {
                "***REDACTED***".to_string()
            } else {
                value.clone()
            };
            content.push_str(&format!("  {}: \"{}\"\n", key, masked));
        }
    }
    if !server.tags.is_empty() {
        content.push_str(&format!("tags: [{}]\n", server.tags.join(", ")));
    }
    content.push_str(&format!("provider: MCP Hub\n"));
    content.push_str(&format!("updated_at: {}\n", server.updated_at.to_rfc3339()));
    content.push_str("---\n\n");

    // Human-readable content
    content.push_str(&format!("# {}\n\n", server.name));

    if let Some(ref desc) = server.description {
        content.push_str(&format!("{}\n\n", desc));
    }

    content.push_str("## Configuration\n\n");
    content.push_str(&format!("**Command:** `{}`\n\n", server.command));

    if !server.args.is_empty() {
        content.push_str("**Arguments:**\n");
        for arg in &server.args {
            content.push_str(&format!("- `{}`\n", arg));
        }
        content.push('\n');
    }

    if !server.env.is_empty() {
        content.push_str("**Environment Variables:**\n");
        for key in server.env.keys() {
            content.push_str(&format!("- `{}`\n", key));
        }
        content.push('\n');
    }

    if !server.tags.is_empty() {
        content.push_str(&format!("**Tags:** {}\n\n", server.tags.join(", ")));
    }

    content.push_str("---\n");
    content.push_str("*Managed by [MCP Hub](https://github.com/mcp-hub)*\n");

    content
}

/// Write all servers to ~/.mcp/ directory
pub fn write_mcp_directory(servers: &[McpServer]) -> Result<(), String> {
    let mcp_dir = get_mcp_directory().ok_or("Could not determine home directory")?;

    // Create directory if it doesn't exist
    fs::create_dir_all(&mcp_dir).map_err(|e| format!("Failed to create ~/.mcp directory: {}", e))?;

    // Track existing MCP Hub managed files
    let mut managed_files: Vec<PathBuf> = Vec::new();

    // Write each server
    for server in servers {
        let filename = format!("mcp-hub-{}.md", sanitize_filename(&server.name));
        let filepath = mcp_dir.join(&filename);
        managed_files.push(filepath.clone());

        let content = generate_server_markdown(server);
        fs::write(&filepath, content)
            .map_err(|e| format!("Failed to write {}: {}", filename, e))?;
    }

    // Clean up old MCP Hub managed files that are no longer needed
    if let Ok(entries) = fs::read_dir(&mcp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Only delete files we manage (prefixed with mcp-hub-)
                if name.starts_with("mcp-hub-") && name.ends_with(".md") {
                    if !managed_files.contains(&path) {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Remove all MCP Hub managed files from ~/.mcp/ directory
pub fn clear_mcp_directory() -> Result<(), String> {
    let mcp_dir = get_mcp_directory().ok_or("Could not determine home directory")?;

    if !mcp_dir.exists() {
        return Ok(());
    }

    if let Ok(entries) = fs::read_dir(&mcp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("mcp-hub-") && name.ends_with(".md") {
                    fs::remove_file(&path)
                        .map_err(|e| format!("Failed to remove {}: {}", name, e))?;
                }
            }
        }
    }

    Ok(())
}

// ==================== Local HTTP Server Discovery ====================

/// Convert McpServer to McpServerCard format
fn server_to_card(server: &McpServer) -> McpServerCard {
    McpServerCard {
        schema_version: "1.0".to_string(),
        name: server.name.clone(),
        description: server.description.clone(),
        homepage: None,
        icon: None,
        transport: TransportConfig {
            transport_type: "stdio".to_string(),
            command: server.command.clone(),
            args: server.args.clone(),
            // Don't expose environment variables in HTTP response for security
            env: HashMap::new(),
        },
        tags: server.tags.clone(),
    }
}

/// Create discovery index from servers
fn create_discovery_index(servers: &[McpServer]) -> McpDiscoveryIndex {
    McpDiscoveryIndex {
        schema_version: "1.0".to_string(),
        provider: "MCP Hub".to_string(),
        description: Some("MCP servers managed by MCP Hub".to_string()),
        servers: servers.iter().map(server_to_card).collect(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Handler for /.well-known/mcp.json
async fn well_known_mcp_handler(
    axum::extract::State(state): axum::extract::State<Arc<DiscoveryState>>,
) -> impl IntoResponse {
    let servers = state.servers.read().await;
    let index = create_discovery_index(&servers);
    Json(index)
}

/// Handler for /health
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Handler for / (root)
async fn root_handler() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>MCP Hub Discovery</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        code { background: #f4f4f4; padding: 2px 6px; border-radius: 4px; }
        a { color: #0066cc; }
    </style>
</head>
<body>
    <h1>MCP Hub Discovery Server</h1>
    <p>This server provides MCP server discovery for other applications.</p>
    <h2>Endpoints</h2>
    <ul>
        <li><a href="/.well-known/mcp.json"><code>/.well-known/mcp.json</code></a> - MCP server discovery index</li>
        <li><a href="/health"><code>/health</code></a> - Health check</li>
    </ul>
    <p><small>Powered by <a href="https://github.com/mcp-hub">MCP Hub</a></small></p>
</body>
</html>"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        html,
    )
}

/// Create the HTTP server router
fn create_router(state: Arc<DiscoveryState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT]);

    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/.well-known/mcp.json", get(well_known_mcp_handler))
        .layer(cors)
        .with_state(state)
}

/// Discovery server handle for controlling the server
pub struct DiscoveryServerHandle {
    state: Arc<DiscoveryState>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl DiscoveryServerHandle {
    /// Update the servers in the discovery index
    pub async fn update_servers(&self, servers: Vec<McpServer>) {
        let mut guard = self.state.servers.write().await;
        *guard = servers;
    }

    /// Shutdown the server
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Start the discovery HTTP server
pub async fn start_discovery_server(
    port: u16,
    initial_servers: Vec<McpServer>,
) -> Result<DiscoveryServerHandle, String> {
    let state = Arc::new(DiscoveryState {
        servers: RwLock::new(initial_servers),
    });

    let router = create_router(state.clone());
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_state = state.clone();
    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    log::info!("Discovery server started on http://127.0.0.1:{}", port);

    Ok(DiscoveryServerHandle {
        state: server_state,
        shutdown_tx: Some(shutdown_tx),
    })
}

/// Check if the discovery server port is available
pub async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
        .await
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Server"), "my-server");
        assert_eq!(sanitize_filename("server_123"), "server_123");
        assert_eq!(sanitize_filename("hello@world!"), "hello-world");
    }

    #[test]
    fn test_generate_server_markdown() {
        let server = McpServer::new(
            "Test Server".to_string(),
            "npx".to_string(),
            vec!["@test/server".to_string()],
        );
        let markdown = generate_server_markdown(&server);
        assert!(markdown.contains("# Test Server"));
        assert!(markdown.contains("command: npx"));
    }
}
