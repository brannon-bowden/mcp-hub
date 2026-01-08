//! MCP Discovery Service
//!
//! Implements two discovery mechanisms:
//! 1. ~/.mcp/ directory - Markdown files for each server (mcp-local-spec)
//! 2. Local HTTP server - /.well-known/mcp.json endpoint (SEP-1649)

use crate::models::McpServer;
use axum::{
    extract::ConnectInfo,
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::{Any, CorsLayer};

// ==================== Rate Limiting ====================

/// Rate limiter configuration
const RATE_LIMIT_MAX_REQUESTS: usize = 100;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

/// Request timestamps for rate limiting
struct RequestLog {
    timestamps: Vec<Instant>,
}

impl RequestLog {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
        }
    }

    /// Clean up old timestamps and return current count
    fn count_within_window(&mut self, window: Duration) -> usize {
        let now = Instant::now();
        self.timestamps.retain(|&t| now.duration_since(t) < window);
        self.timestamps.len()
    }

    /// Add a new request timestamp
    fn add_request(&mut self) {
        self.timestamps.push(Instant::now());
    }
}

/// Rate limiter state shared across requests
pub struct RateLimiter {
    requests: Mutex<HashMap<IpAddr, RequestLog>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if a request from the given IP should be allowed
    /// Returns Ok(()) if allowed, Err(remaining_time) if rate limited
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), Duration> {
        let mut requests = self.requests.lock().await;
        let log = requests.entry(ip).or_insert_with(RequestLog::new);

        let count = log.count_within_window(self.window);

        if count >= self.max_requests {
            // Calculate retry-after time (approximate)
            let oldest = log.timestamps.first().copied().unwrap_or_else(Instant::now);
            let retry_after = self.window.saturating_sub(Instant::now().duration_since(oldest));
            return Err(retry_after);
        }

        log.add_request();
        Ok(())
    }

    /// Periodically clean up stale entries (call every few minutes)
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        let mut requests = self.requests.lock().await;
        let window = self.window;
        requests.retain(|_, log| {
            log.count_within_window(window) > 0
        });
    }
}

/// Axum middleware for rate limiting
async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::State(limiter): axum::extract::State<Arc<RateLimiter>>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    match limiter.check_rate_limit(ip).await {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let retry_secs = retry_after.as_secs().max(1);
            log::warn!(
                "Rate limit exceeded for {} - retry after {} seconds",
                ip,
                retry_secs
            );
            (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    (header::RETRY_AFTER, retry_secs.to_string()),
                    (header::CONTENT_TYPE, "text/plain".to_string()),
                ],
                format!(
                    "Rate limit exceeded. Please retry after {} seconds.",
                    retry_secs
                ),
            )
                .into_response()
        }
    }
}

// ==================== Authentication ====================

/// Authentication configuration - None means no auth required (public access)
#[derive(Clone)]
pub struct AuthConfig {
    /// Bearer token required for access (None = no authentication)
    token: Option<String>,
}

impl AuthConfig {
    /// Create config with no authentication (public access)
    pub fn none() -> Self {
        Self { token: None }
    }

    /// Create config with Bearer token authentication
    pub fn with_token(token: String) -> Self {
        Self { token: Some(token) }
    }

    /// Check if a request is authenticated
    fn is_authenticated(&self, auth_header: Option<&str>) -> bool {
        match &self.token {
            None => true, // No auth required
            Some(expected_token) => {
                auth_header
                    .and_then(|h| h.strip_prefix("Bearer "))
                    .map(|t| t == expected_token)
                    .unwrap_or(false)
            }
        }
    }
}

/// Axum middleware for Bearer token authentication
async fn auth_middleware(
    axum::extract::State(auth_config): axum::extract::State<AuthConfig>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    if auth_config.is_authenticated(auth_header) {
        next.run(request).await
    } else {
        log::warn!("Unauthorized access attempt to discovery server");
        (
            StatusCode::UNAUTHORIZED,
            [
                (header::WWW_AUTHENTICATE, "Bearer".to_string()),
                (header::CONTENT_TYPE, "text/plain".to_string()),
            ],
            "Unauthorized. Please provide a valid Bearer token.",
        )
            .into_response()
    }
}

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

/// Create the HTTP server router with rate limiting and optional authentication
fn create_router(
    state: Arc<DiscoveryState>,
    rate_limiter: Arc<RateLimiter>,
    auth_config: AuthConfig,
) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION]);

    // Routes that need rate limiting and authentication
    // Authentication is applied first (innermost), then rate limiting
    let protected_routes = Router::new()
        .route("/", get(root_handler))
        .route("/.well-known/mcp.json", get(well_known_mcp_handler))
        .route_layer(middleware::from_fn_with_state(
            auth_config.clone(),
            auth_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .with_state(state);

    // Health check is exempt from rate limiting and authentication (for monitoring)
    let health_route = Router::new().route("/health", get(health_handler));

    protected_routes.merge(health_route).layer(cors)
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
///
/// # Arguments
/// * `port` - Port number to bind to (localhost only)
/// * `initial_servers` - Initial list of MCP servers to expose
/// * `auth_token` - Optional Bearer token for authentication. If None, no auth is required.
pub async fn start_discovery_server(
    port: u16,
    initial_servers: Vec<McpServer>,
    auth_token: Option<String>,
) -> Result<DiscoveryServerHandle, String> {
    let state = Arc::new(DiscoveryState {
        servers: RwLock::new(initial_servers),
    });

    // Create rate limiter (100 requests per 60 seconds per IP)
    let rate_limiter = Arc::new(RateLimiter::new(
        RATE_LIMIT_MAX_REQUESTS,
        RATE_LIMIT_WINDOW_SECS,
    ));

    // Create auth config (optional authentication)
    let auth_config = match auth_token {
        Some(token) => {
            log::info!("Discovery server authentication enabled");
            AuthConfig::with_token(token)
        }
        None => {
            log::info!("Discovery server running without authentication");
            AuthConfig::none()
        }
    };

    let router = create_router(state.clone(), rate_limiter, auth_config);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_state = state.clone();
    tokio::spawn(async move {
        // Use into_make_service_with_connect_info to provide client IP to middleware
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
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
