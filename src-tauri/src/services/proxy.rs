//! MCP Proxy Service
//!
//! Provides an HTTP/SSE proxy that aggregates multiple MCP servers
//! and exposes them through a single endpoint per client instance.
//!
//! Architecture:
//! - Each client instance can have its own proxy endpoint
//! - The proxy spawns and manages stdio-based MCP servers as child processes
//! - Tools, resources, and prompts are namespaced by server name (e.g., "filesystem__read_file")
//! - JSON-RPC requests are routed to the appropriate backend server

use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::{sse::Event, IntoResponse, Json, Sse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::models::{ClientInstance, McpServer};

// ==================== Types ====================

/// A running MCP server process
struct McpProcess {
    /// The child process handle
    child: Child,
    /// Channel to send requests to this server
    request_tx: mpsc::Sender<JsonRpcRequest>,
    /// The server ID this process belongs to
    server_id: String,
    /// The server name (used for namespacing)
    server_name: String,
}

/// JSON-RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Aggregated tool info with namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespacedTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    /// Original server this tool belongs to
    #[serde(skip)]
    pub server_name: String,
    /// Original tool name before namespacing
    #[serde(skip)]
    pub original_name: String,
}

/// Aggregated resource info with namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespacedResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip)]
    pub server_name: String,
}

/// Aggregated prompt info with namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespacedPrompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<Value>>,
    #[serde(skip)]
    pub server_name: String,
    #[serde(skip)]
    pub original_name: String,
}

/// State for a single instance proxy
pub struct InstanceProxyState {
    /// Instance ID
    pub instance_id: String,
    /// Instance name
    pub instance_name: String,
    /// Running server processes (server_id -> process)
    pub processes: RwLock<HashMap<String, McpProcess>>,
    /// Aggregated tools from all servers
    pub tools: RwLock<Vec<NamespacedTool>>,
    /// Aggregated resources from all servers
    pub resources: RwLock<Vec<NamespacedResource>>,
    /// Aggregated prompts from all servers
    pub prompts: RwLock<Vec<NamespacedPrompt>>,
    /// Server configurations
    pub servers: RwLock<Vec<McpServer>>,
    /// Session ID counter
    pub session_counter: RwLock<u64>,
    /// Active sessions
    pub sessions: RwLock<HashMap<String, SessionState>>,
}

/// State for a client session
pub struct SessionState {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub initialized: bool,
}

/// Global proxy state managing all instance proxies
pub struct ProxyState {
    /// Instance proxies (instance_id -> proxy state)
    pub instances: RwLock<HashMap<String, Arc<InstanceProxyState>>>,
    /// Port assignments (instance_id -> port)
    pub ports: RwLock<HashMap<String, u16>>,
    /// Base port for proxy servers
    pub base_port: u16,
}

impl ProxyState {
    pub fn new(base_port: u16) -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            ports: RwLock::new(HashMap::new()),
            base_port,
        }
    }
}

// ==================== Namespacing Helpers ====================

/// Create a namespaced name from server name and item name
fn namespace_name(server_name: &str, item_name: &str) -> String {
    let sanitized = server_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    format!("{}__{}", sanitized, item_name)
}

/// Parse a namespaced name back to (server_name_prefix, original_name)
fn parse_namespaced_name(namespaced: &str) -> Option<(&str, &str)> {
    namespaced.split_once("__")
}

// ==================== Process Management ====================

/// Spawn an MCP server process and set up communication
async fn spawn_mcp_server(server: &McpServer) -> Result<McpProcess, String> {
    let mut cmd = Command::new(&server.command);
    cmd.args(&server.args)
        .envs(&server.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", server.name, e))?;

    let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

    let (request_tx, mut request_rx) = mpsc::channel::<JsonRpcRequest>(32);
    let (response_tx, _response_rx) = mpsc::channel::<JsonRpcResponse>(32);

    let server_name = server.name.clone();
    let server_id = server.id.clone();

    // Spawn writer task
    let mut stdin = stdin;
    tokio::spawn(async move {
        while let Some(request) = request_rx.recv().await {
            let json = serde_json::to_string(&request).unwrap();
            let msg = format!("{}\n", json);
            if stdin.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
            let _ = stdin.flush().await;
        }
    });

    // Spawn reader task
    let stdout = BufReader::new(stdout);
    tokio::spawn(async move {
        let mut lines = stdout.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&line) {
                if response_tx.send(response).await.is_err() {
                    break;
                }
            }
        }
    });

    Ok(McpProcess {
        child,
        request_tx,
        server_id,
        server_name,
    })
}

// ==================== Protocol Handlers ====================

/// Initialize a server and get its capabilities
async fn initialize_server(process: &McpProcess) -> Result<Value, String> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "MCP Hub Proxy",
                "version": "0.1.0"
            }
        })),
    };

    process
        .request_tx
        .send(request)
        .await
        .map_err(|e| format!("Failed to send initialize: {}", e))?;

    // For now, return a placeholder - we'll implement proper response handling
    Ok(json!({}))
}

/// Get tools from a server
async fn get_server_tools(process: &McpProcess) -> Result<Vec<Value>, String> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };

    process
        .request_tx
        .send(request)
        .await
        .map_err(|e| format!("Failed to send tools/list: {}", e))?;

    // Placeholder - proper implementation needs response channel
    Ok(vec![])
}

// ==================== HTTP Handlers ====================

/// Handle POST requests (JSON-RPC messages from client)
async fn handle_post(
    Path(instance_id): Path<String>,
    State(state): State<Arc<ProxyState>>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let instances = state.instances.read().await;
    let instance = match instances.get(&instance_id) {
        Some(i) => i.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: format!("Instance not found: {}", instance_id),
                        data: None,
                    }),
                }),
            )
                .into_response();
        }
    };

    // Handle different methods
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&instance, &request).await,
        "tools/list" => handle_tools_list(&instance, &request).await,
        "tools/call" => handle_tools_call(&instance, &request).await,
        "resources/list" => handle_resources_list(&instance, &request).await,
        "resources/read" => handle_resources_read(&instance, &request).await,
        "prompts/list" => handle_prompts_list(&instance, &request).await,
        "prompts/get" => handle_prompts_get(&instance, &request).await,
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    };

    Json(response).into_response()
}

async fn handle_initialize(
    instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // Generate session ID
    let mut counter = instance.session_counter.write().await;
    *counter += 1;
    let session_id = format!("mcp-hub-{}-{}", instance.instance_id, *counter);

    // Create session
    let mut sessions = instance.sessions.write().await;
    sessions.insert(
        session_id.clone(),
        SessionState {
            session_id: session_id.clone(),
            created_at: chrono::Utc::now(),
            initialized: true,
        },
    );

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": format!("MCP Hub - {}", instance.instance_name),
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

async fn handle_tools_list(
    instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let tools = instance.tools.read().await;
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "tools": tools.iter().map(|t| json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema
            })).collect::<Vec<_>>()
        })),
        error: None,
    }
}

async fn handle_tools_call(
    instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params = match &request.params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            }
        }
    };

    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let tool_args = params.get("arguments").cloned().unwrap_or(json!({}));

    // Find the tool and its server
    let tools = instance.tools.read().await;
    let tool = tools.iter().find(|t| t.name == tool_name);

    let tool = match tool {
        Some(t) => t.clone(),
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Tool not found: {}", tool_name),
                    data: None,
                }),
            }
        }
    };
    drop(tools);

    // Route to the appropriate server
    let processes = instance.processes.read().await;
    let process = processes
        .values()
        .find(|p| p.server_name == tool.server_name);

    let process = match process {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: format!("Server not running: {}", tool.server_name),
                    data: None,
                }),
            }
        }
    };

    // Forward the request with the original tool name
    let forward_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": tool.original_name,
            "arguments": tool_args
        })),
    };

    if let Err(e) = process.request_tx.send(forward_request).await {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Failed to forward request: {}", e),
                data: None,
            }),
        };
    }

    // For now, return a placeholder - proper implementation needs response channel
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "content": [{"type": "text", "text": "Request forwarded"}]
        })),
        error: None,
    }
}

async fn handle_resources_list(
    instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let resources = instance.resources.read().await;
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "resources": resources.iter().map(|r| json!({
                "uri": r.uri,
                "name": r.name,
                "description": r.description,
                "mimeType": r.mime_type
            })).collect::<Vec<_>>()
        })),
        error: None,
    }
}

async fn handle_resources_read(
    _instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // TODO: Implement resource reading with routing
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "resources/read not yet implemented".to_string(),
            data: None,
        }),
    }
}

async fn handle_prompts_list(
    instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let prompts = instance.prompts.read().await;
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(json!({
            "prompts": prompts.iter().map(|p| json!({
                "name": p.name,
                "description": p.description,
                "arguments": p.arguments
            })).collect::<Vec<_>>()
        })),
        error: None,
    }
}

async fn handle_prompts_get(
    _instance: &InstanceProxyState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // TODO: Implement prompt getting with routing
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "prompts/get not yet implemented".to_string(),
            data: None,
        }),
    }
}

/// Handle GET requests (SSE stream for server-initiated messages)
async fn handle_get(
    Path(_instance_id): Path<String>,
    State(_state): State<Arc<ProxyState>>,
) -> impl IntoResponse {
    // Return an SSE stream for server-initiated messages
    // For now, return a simple keepalive stream
    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            yield Ok::<_, std::convert::Infallible>(Event::default().comment("keepalive"));
        }
    };

    Sse::new(stream)
}

/// Info endpoint showing proxy status
async fn handle_info(
    Path(instance_id): Path<String>,
    State(state): State<Arc<ProxyState>>,
) -> impl IntoResponse {
    let instances = state.instances.read().await;

    if let Some(instance) = instances.get(&instance_id) {
        let tools = instance.tools.read().await;
        let resources = instance.resources.read().await;
        let processes = instance.processes.read().await;

        Json(json!({
            "instance_id": instance.instance_id,
            "instance_name": instance.instance_name,
            "status": "running",
            "servers": processes.len(),
            "tools": tools.len(),
            "resources": resources.len()
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Instance not found"})),
        )
            .into_response()
    }
}

/// Root endpoint with usage info
async fn handle_root() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>MCP Hub Proxy</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        code { background: #f4f4f4; padding: 2px 6px; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>MCP Hub Proxy Server</h1>
    <p>This server provides MCP proxy endpoints for client instances.</p>
    <h2>Usage</h2>
    <p>Configure your MCP client to connect to:</p>
    <pre><code>http://localhost:PORT/mcp/{instance-id}</code></pre>
    <p>Where <code>{instance-id}</code> is your client instance ID.</p>
    <p><small>Powered by <a href="https://github.com/mcp-hub">MCP Hub</a></small></p>
</body>
</html>"#;

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], html)
}

// ==================== Server Setup ====================

/// Create the proxy router
pub fn create_proxy_router(state: Arc<ProxyState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION]);

    Router::new()
        .route("/", get(handle_root))
        .route("/mcp/{instance_id}", post(handle_post).get(handle_get))
        .route("/mcp/{instance_id}/info", get(handle_info))
        .layer(cors)
        .with_state(state)
}

/// Proxy server handle
pub struct ProxyServerHandle {
    state: Arc<ProxyState>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ProxyServerHandle {
    /// Get the proxy state
    pub fn state(&self) -> Arc<ProxyState> {
        self.state.clone()
    }

    /// Shutdown the proxy server
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Start the proxy server
pub async fn start_proxy_server(port: u16) -> Result<ProxyServerHandle, String> {
    let state = Arc::new(ProxyState::new(port));
    let router = create_proxy_router(state.clone());
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    log::info!("MCP Proxy server started on http://127.0.0.1:{}", port);

    Ok(ProxyServerHandle {
        state,
        shutdown_tx: Some(shutdown_tx),
    })
}

/// Register an instance with the proxy
pub async fn register_instance(
    state: &ProxyState,
    instance: &ClientInstance,
    servers: Vec<McpServer>,
) -> Result<(), String> {
    let instance_state = Arc::new(InstanceProxyState {
        instance_id: instance.id.clone(),
        instance_name: instance.name.clone(),
        processes: RwLock::new(HashMap::new()),
        tools: RwLock::new(Vec::new()),
        resources: RwLock::new(Vec::new()),
        prompts: RwLock::new(Vec::new()),
        servers: RwLock::new(servers),
        session_counter: RwLock::new(0),
        sessions: RwLock::new(HashMap::new()),
    });

    let mut instances = state.instances.write().await;
    instances.insert(instance.id.clone(), instance_state);

    Ok(())
}

/// Start servers for an instance and aggregate their capabilities
pub async fn start_instance_servers(state: &ProxyState, instance_id: &str) -> Result<(), String> {
    let instances = state.instances.read().await;
    let instance = instances
        .get(instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?
        .clone();
    drop(instances);

    let servers = instance.servers.read().await.clone();
    let mut all_tools = Vec::new();
    let all_resources = Vec::new();
    let all_prompts = Vec::new();
    let mut processes = HashMap::new();

    for server in servers {
        match spawn_mcp_server(&server).await {
            Ok(process) => {
                // Initialize the server
                if let Err(e) = initialize_server(&process).await {
                    log::warn!("Failed to initialize {}: {}", server.name, e);
                    continue;
                }

                // Get tools and namespace them
                if let Ok(tools) = get_server_tools(&process).await {
                    for tool in tools {
                        if let Some(name) = tool.get("name").and_then(|v| v.as_str()) {
                            all_tools.push(NamespacedTool {
                                name: namespace_name(&server.name, name),
                                description: tool
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                input_schema: tool
                                    .get("inputSchema")
                                    .cloned()
                                    .unwrap_or(json!({})),
                                server_name: server.name.clone(),
                                original_name: name.to_string(),
                            });
                        }
                    }
                }

                processes.insert(server.id.clone(), process);
            }
            Err(e) => {
                log::error!("Failed to spawn {}: {}", server.name, e);
            }
        }
    }

    // Update instance state
    *instance.tools.write().await = all_tools;
    *instance.resources.write().await = all_resources;
    *instance.prompts.write().await = all_prompts;
    *instance.processes.write().await = processes;

    Ok(())
}

/// Stop all servers for an instance
pub async fn stop_instance_servers(state: &ProxyState, instance_id: &str) -> Result<(), String> {
    let instances = state.instances.read().await;
    let instance = instances
        .get(instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?
        .clone();
    drop(instances);

    let mut processes = instance.processes.write().await;
    for (_, mut process) in processes.drain() {
        let _ = process.child.kill().await;
    }

    instance.tools.write().await.clear();
    instance.resources.write().await.clear();
    instance.prompts.write().await.clear();

    Ok(())
}

/// Unregister an instance from the proxy
pub async fn unregister_instance(state: &ProxyState, instance_id: &str) -> Result<(), String> {
    // Stop servers first
    let _ = stop_instance_servers(state, instance_id).await;

    let mut instances = state.instances.write().await;
    instances.remove(instance_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_name() {
        assert_eq!(namespace_name("filesystem", "read_file"), "filesystem__read_file");
        assert_eq!(namespace_name("My Server", "tool"), "my_server__tool");
        assert_eq!(namespace_name("server-1", "list"), "server_1__list");
    }

    #[test]
    fn test_parse_namespaced_name() {
        assert_eq!(
            parse_namespaced_name("filesystem__read_file"),
            Some(("filesystem", "read_file"))
        );
        assert_eq!(parse_namespaced_name("no_namespace"), None);
    }
}
