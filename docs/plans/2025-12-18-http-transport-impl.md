# HTTP/SSE Transport Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add HTTP/SSE transport support for MCP servers, enabling connection to remote servers via URL instead of local STDIO processes.

**Architecture:** Replace flat `command`/`args`/`env` fields with a `Transport` enum that supports STDIO, SSE, and Streamable HTTP variants. Update database schema, Rust models, TypeScript types, config sync, and UI to handle the new transport types.

**Tech Stack:** Rust/Tauri backend, React/TypeScript frontend, SQLite database, serde for serialization

---

## Phase 1: Backend Data Model

### Task 1: Add Transport Enum to Rust Models

**Files:**
- Modify: `src-tauri/src/models/mod.rs`

**Step 1: Add the Transport enum after the existing imports**

Add this after line 3 (after `use uuid::Uuid;`):

```rust
/// Transport configuration for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Transport {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: std::collections::HashMap<String, String>,
    },
    Sse {
        url: String,
        #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
        headers: std::collections::HashMap<String, String>,
    },
    Http {
        url: String,
        #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
        headers: std::collections::HashMap<String, String>,
    },
}

impl Transport {
    /// Create a new STDIO transport
    pub fn stdio(command: String, args: Vec<String>, env: std::collections::HashMap<String, String>) -> Self {
        Transport::Stdio { command, args, env }
    }

    /// Create a new SSE transport
    pub fn sse(url: String, headers: std::collections::HashMap<String, String>) -> Self {
        Transport::Sse { url, headers }
    }

    /// Create a new Streamable HTTP transport
    pub fn http(url: String, headers: std::collections::HashMap<String, String>) -> Self {
        Transport::Http { url, headers }
    }

    /// Get the transport type as a string
    pub fn transport_type(&self) -> &'static str {
        match self {
            Transport::Stdio { .. } => "stdio",
            Transport::Sse { .. } => "sse",
            Transport::Http { .. } => "http",
        }
    }
}
```

**Step 2: Update McpServer struct to use Transport**

Replace the `McpServer` struct (lines 8-26) with:

```rust
/// Represents an MCP server configuration in the central registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub transport: Transport,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ServerSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Step 3: Update McpServer::new() constructor**

Replace the `impl McpServer` block (lines 28-48) with:

```rust
impl McpServer {
    /// Create a new STDIO server (backward compatible)
    pub fn new(name: String, command: String, args: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            transport: Transport::stdio(command, args, std::collections::HashMap::new()),
            tags: Vec::new(),
            source: Some(ServerSource {
                source_type: SourceType::Manual,
                url: None,
            }),
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new server with explicit transport
    pub fn with_transport(name: String, transport: Transport) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            transport,
            tags: Vec::new(),
            source: Some(ServerSource {
                source_type: SourceType::Manual,
                url: None,
            }),
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Helper: get command for STDIO servers (for backward compat in some places)
    pub fn command(&self) -> Option<&str> {
        match &self.transport {
            Transport::Stdio { command, .. } => Some(command),
            _ => None,
        }
    }

    /// Helper: get URL for HTTP-based servers
    pub fn url(&self) -> Option<&str> {
        match &self.transport {
            Transport::Sse { url, .. } | Transport::Http { url, .. } => Some(url),
            _ => None,
        }
    }
}
```

**Step 4: Build to verify compilation**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | head -50`

Expected: Compilation errors (database code references old fields) - this is expected, we'll fix next.

**Step 5: Commit the model changes**

```bash
git add src-tauri/src/models/mod.rs
git commit -m "feat(models): add Transport enum for HTTP/SSE support

Add Transport enum with Stdio, Sse, and Http variants.
Update McpServer to use transport field instead of flat command/args/env.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: Update Database Schema and Migration

**Files:**
- Modify: `src-tauri/src/db/mod.rs`

**Step 1: Add migration for transport column**

In `init_schema()` function, after the existing migrations (around line 117), add:

```rust
        // Migration: Add transport column and migrate existing data
        let has_transport: bool = {
            let mut stmt = conn.prepare("PRAGMA table_info(servers)")?;
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))?
                .filter_map(|r| r.ok())
                .collect();
            columns.contains(&"transport".to_string())
        };

        if !has_transport {
            // Add transport column
            conn.execute("ALTER TABLE servers ADD COLUMN transport TEXT", [])?;

            // Migrate existing data: convert command/args/env to transport JSON
            let mut stmt = conn.prepare("SELECT id, command, args, env FROM servers")?;
            let rows: Vec<(String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);

            for (id, command, args_json, env_json) in rows {
                let transport_json = format!(
                    r#"{{"type":"stdio","command":{},"args":{},"env":{}}}"#,
                    serde_json::to_string(&command).unwrap_or_else(|_| "\"\"".to_string()),
                    args_json,
                    env_json
                );
                conn.execute(
                    "UPDATE servers SET transport = ?1 WHERE id = ?2",
                    params![transport_json, id],
                )?;
            }

            log::info!("Migrated servers table to use transport column");
        }
```

**Step 2: Update create_server() to use transport**

Replace `create_server` function (lines 124-161) with:

```rust
    pub fn create_server(&self, server: &McpServer) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        let transport_json = serde_json::to_string(&server.transport).unwrap_or_default();
        let tags_json = serde_json::to_string(&server.tags).unwrap_or_default();
        let source_type = server
            .source
            .as_ref()
            .map(|s| match s.source_type {
                SourceType::Manual => "manual",
                SourceType::Imported => "imported",
                SourceType::Registry => "registry",
            })
            .unwrap_or("manual");
        let source_url = server.source.as_ref().and_then(|s| s.url.clone());

        conn.execute(
            "INSERT INTO servers (id, name, description, command, args, env, transport, tags, source_type, source_url, parent_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                server.id,
                server.name,
                server.description,
                // Keep old columns for backward compat during transition
                match &server.transport {
                    crate::models::Transport::Stdio { command, .. } => command.clone(),
                    _ => String::new(),
                },
                match &server.transport {
                    crate::models::Transport::Stdio { args, .. } => serde_json::to_string(args).unwrap_or_default(),
                    _ => "[]".to_string(),
                },
                match &server.transport {
                    crate::models::Transport::Stdio { env, .. } => serde_json::to_string(env).unwrap_or_default(),
                    _ => "{}".to_string(),
                },
                transport_json,
                tags_json,
                source_type,
                source_url,
                server.parent_id,
                server.created_at.to_rfc3339(),
                server.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }
```

**Step 3: Update update_server() to use transport**

Replace `update_server` function (lines 200-236) with:

```rust
    pub fn update_server(&self, server: &McpServer) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        let transport_json = serde_json::to_string(&server.transport).unwrap_or_default();
        let tags_json = serde_json::to_string(&server.tags).unwrap_or_default();
        let source_type = server
            .source
            .as_ref()
            .map(|s| match s.source_type {
                SourceType::Manual => "manual",
                SourceType::Imported => "imported",
                SourceType::Registry => "registry",
            })
            .unwrap_or("manual");
        let source_url = server.source.as_ref().and_then(|s| s.url.clone());

        conn.execute(
            "UPDATE servers SET name = ?2, description = ?3, command = ?4, args = ?5, env = ?6,
             transport = ?7, tags = ?8, source_type = ?9, source_url = ?10, parent_id = ?11, updated_at = ?12 WHERE id = ?1",
            params![
                server.id,
                server.name,
                server.description,
                // Keep old columns for backward compat
                match &server.transport {
                    crate::models::Transport::Stdio { command, .. } => command.clone(),
                    _ => String::new(),
                },
                match &server.transport {
                    crate::models::Transport::Stdio { args, .. } => serde_json::to_string(args).unwrap_or_default(),
                    _ => "[]".to_string(),
                },
                match &server.transport {
                    crate::models::Transport::Stdio { env, .. } => serde_json::to_string(env).unwrap_or_default(),
                    _ => "{}".to_string(),
                },
                transport_json,
                tags_json,
                source_type,
                source_url,
                server.parent_id,
                server.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }
```

**Step 4: Update row_to_server() to read transport**

Replace `row_to_server` function (lines 244-280) with:

```rust
    fn row_to_server(row: &rusqlite::Row) -> SqlResult<McpServer> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;

        // Try to read transport column first (new format)
        let transport_str: Option<String> = row.get(12).ok();

        let transport = if let Some(ref t) = transport_str {
            if !t.is_empty() {
                serde_json::from_str(t).unwrap_or_else(|_| {
                    // Fallback to old format
                    let command: String = row.get(3).unwrap_or_default();
                    let args_str: String = row.get(4).unwrap_or_default();
                    let env_str: String = row.get(5).unwrap_or_default();
                    crate::models::Transport::Stdio {
                        command,
                        args: serde_json::from_str(&args_str).unwrap_or_default(),
                        env: serde_json::from_str(&env_str).unwrap_or_default(),
                    }
                })
            } else {
                // Empty transport, use old columns
                let command: String = row.get(3)?;
                let args_str: String = row.get(4)?;
                let env_str: String = row.get(5)?;
                crate::models::Transport::Stdio {
                    command,
                    args: serde_json::from_str(&args_str).unwrap_or_default(),
                    env: serde_json::from_str(&env_str).unwrap_or_default(),
                }
            }
        } else {
            // No transport column yet, use old columns
            let command: String = row.get(3)?;
            let args_str: String = row.get(4)?;
            let env_str: String = row.get(5)?;
            crate::models::Transport::Stdio {
                command,
                args: serde_json::from_str(&args_str).unwrap_or_default(),
                env: serde_json::from_str(&env_str).unwrap_or_default(),
            }
        };

        let tags_str: Option<String> = row.get(6)?;
        let source_type: Option<String> = row.get(7)?;
        let source_url: Option<String> = row.get(8)?;
        let parent_id: Option<String> = row.get(9)?;
        let created_at_str: String = row.get(10)?;
        let updated_at_str: String = row.get(11)?;

        Ok(McpServer {
            id,
            name,
            description,
            transport,
            tags: tags_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            source: source_type.map(|st| ServerSource {
                source_type: match st.as_str() {
                    "imported" => SourceType::Imported,
                    "registry" => SourceType::Registry,
                    _ => SourceType::Manual,
                },
                url: source_url,
            }),
            parent_id,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
```

**Step 5: Update SELECT queries to include transport column**

Update `get_server` (line 166-168):
```rust
        let mut stmt = conn.prepare(
            "SELECT id, name, description, command, args, env, tags, source_type, source_url, parent_id, created_at, updated_at, transport
             FROM servers WHERE id = ?1",
        )?;
```

Update `get_all_servers` (line 185-187):
```rust
        let mut stmt = conn.prepare(
            "SELECT id, name, description, command, args, env, tags, source_type, source_url, parent_id, created_at, updated_at, transport
             FROM servers ORDER BY name",
        )?;
```

**Step 6: Build to verify compilation**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | head -50`

Expected: More compilation errors in config.rs and commands.rs - we'll fix those next.

**Step 7: Commit database changes**

```bash
git add src-tauri/src/db/mod.rs
git commit -m "feat(db): add transport column with migration

Add transport column to servers table.
Migrate existing STDIO servers to new transport JSON format.
Update CRUD operations to use transport field.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: Update Config Service for Multi-Transport Output

**Files:**
- Modify: `src-tauri/src/services/config.rs`

**Step 1: Add client transport capability function**

Add after the imports (around line 6):

```rust
use crate::models::Transport;

/// Get supported transports for a client type
pub fn client_supported_transports(client_type: &ClientType) -> Vec<&'static str> {
    match client_type {
        // Full HTTP support (SSE + Streamable HTTP)
        ClientType::ClaudeDesktop
        | ClientType::ClaudeCode
        | ClientType::Cursor
        | ClientType::Cline
        | ClientType::RooCode
        | ClientType::KiloCode => vec!["stdio", "sse", "http"],

        // SSE only
        ClientType::Vscode
        | ClientType::VscodeInsiders
        | ClientType::Continue
        | ClientType::Cody => vec!["stdio", "sse"],

        // STDIO only (default for unknown/older clients)
        _ => vec!["stdio"],
    }
}
```

**Step 2: Add McpServerEntryHttp struct for HTTP config format**

Add after `McpServerEntry` in models/mod.rs (around line 322):

```rust
/// MCP server entry for HTTP/SSE transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntryHttp {
    pub transport: String,  // "sse" or "http"
    pub url: String,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub headers: std::collections::HashMap<String, String>,
}
```

**Step 3: Add unified entry enum for config output**

Add in models/mod.rs:

```rust
/// Unified server entry for config files (can be STDIO or HTTP)
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum McpServerEntryUnified {
    Stdio(McpServerEntry),
    Http(McpServerEntryHttp),
}
```

**Step 4: Update sync_servers_to_instance to handle transports**

In `config.rs`, replace `sync_servers_to_instance` function (lines 606-647) with:

```rust
/// Result of syncing servers to an instance
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub backup_path: Option<PathBuf>,
    pub skipped_servers: Vec<String>,  // Names of servers skipped due to transport incompatibility
}

/// Convert servers to MCP config format and write to instance config file.
pub fn sync_servers_to_instance(
    instance: &ClientInstance,
    servers: &[McpServer],
    backup_dir: Option<&PathBuf>,
) -> Result<SyncResult, String> {
    let config_path = PathBuf::from(&instance.config_path);
    let mut backup_path = None;
    let mut skipped_servers = Vec::new();

    // Get supported transports for this client
    let supported = client_supported_transports(&instance.client_type);

    // Create backup if requested and file exists
    if let Some(dir) = backup_dir {
        if config_exists(&config_path) {
            backup_path = Some(backup_config_file(&config_path, dir)?);
        }
    }

    // Build the MCP servers map
    let mut mcp_servers: HashMap<String, serde_json::Value> = HashMap::new();

    // Sync all enabled servers
    for server in servers {
        if instance.enabled_servers.contains(&server.id) {
            let transport_type = server.transport.transport_type();

            // Check if this transport is supported by the client
            if !supported.contains(&transport_type) {
                skipped_servers.push(server.name.clone());
                continue;
            }

            let key = sanitize_server_name(&server.name);

            let entry = match &server.transport {
                Transport::Stdio { command, args, env } => {
                    serde_json::json!({
                        "command": command,
                        "args": args,
                        "env": env
                    })
                }
                Transport::Sse { url, headers } => {
                    serde_json::json!({
                        "transport": "sse",
                        "url": url,
                        "headers": headers
                    })
                }
                Transport::Http { url, headers } => {
                    serde_json::json!({
                        "transport": "http",
                        "url": url,
                        "headers": headers
                    })
                }
            };

            mcp_servers.insert(key, entry);
        }
    }

    // Use merge-aware write for clients that have other settings in their config file
    if client_requires_merge_write(&instance.client_type) {
        write_mcp_servers_json_preserving_config(&config_path, &mcp_servers)?;
    } else {
        let config = serde_json::json!({ "mcpServers": mcp_servers });
        write_json_config_file(&config_path, &config)?;
    }

    Ok(SyncResult {
        backup_path,
        skipped_servers,
    })
}

/// Write MCP servers (as JSON values) to a config file, preserving other fields
fn write_mcp_servers_json_preserving_config(
    path: &PathBuf,
    mcp_servers: &HashMap<String, serde_json::Value>,
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
    obj.insert("mcpServers".to_string(), serde_json::json!(mcp_servers));

    // Write back the merged config
    let content = serde_json::to_string_pretty(&existing)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))
}

/// Write a JSON config file
fn write_json_config_file(path: &PathBuf, config: &serde_json::Value) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))
}
```

**Step 5: Update import_servers_from_config to handle both formats**

Replace `import_servers_from_config` function (lines 660-675) with:

```rust
/// Import servers from an existing config file (supports both STDIO and HTTP formats)
pub fn import_servers_from_config(path: &PathBuf) -> Result<Vec<McpServer>, String> {
    if !config_exists(path) {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    let mcp_servers = json.get("mcpServers")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "No mcpServers found in config".to_string())?;

    let mut servers = Vec::new();

    for (name, entry) in mcp_servers {
        let transport = if let Some(transport_type) = entry.get("transport").and_then(|v| v.as_str()) {
            // HTTP/SSE format
            let url = entry.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Missing url for server {}", name))?
                .to_string();
            let headers: std::collections::HashMap<String, String> = entry.get("headers")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            match transport_type {
                "sse" => Transport::Sse { url, headers },
                "http" => Transport::Http { url, headers },
                _ => return Err(format!("Unknown transport type: {}", transport_type)),
            }
        } else {
            // STDIO format
            let command = entry.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Missing command for server {}", name))?
                .to_string();
            let args: Vec<String> = entry.get("args")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let env: std::collections::HashMap<String, String> = entry.get("env")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            Transport::Stdio { command, args, env }
        };

        let mut server = McpServer::with_transport(name.clone(), transport);
        server.source = Some(crate::models::ServerSource {
            source_type: crate::models::SourceType::Imported,
            url: Some(path.to_string_lossy().to_string()),
        });
        servers.push(server);
    }

    Ok(servers)
}
```

**Step 6: Build to verify compilation**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | head -50`

Expected: Compilation errors in commands.rs - we'll fix those next.

**Step 7: Commit config service changes**

```bash
git add src-tauri/src/services/config.rs src-tauri/src/models/mod.rs
git commit -m "feat(config): support multi-transport config sync

Add client_supported_transports() capability matrix.
Update sync to output correct JSON format per transport type.
Skip incompatible servers and return list of skipped names.
Support importing both STDIO and HTTP config formats.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: Update Commands to Handle New Return Types

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`

**Step 1: Update sync_instance to return skipped servers**

Add new return type after imports:

```rust
/// Result of syncing an instance
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncInstanceResult {
    pub backup_path: Option<String>,
    pub skipped_servers: Vec<String>,
}
```

**Step 2: Update sync_instance command**

Replace the `sync_instance` function (lines 189-257) to return `SyncInstanceResult`:

```rust
#[tauri::command]
pub fn sync_instance(state: State<AppState>, instance_id: String) -> Result<SyncInstanceResult, String> {
    // Log the sync attempt
    if let Ok(mut buffer) = state.log_buffer.lock() {
        buffer.add("INFO", format!("Starting sync for instance: {}", instance_id));
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Get instance
    let mut instance = db
        .get_instance(&instance_id)
        .map_err(|e| e.to_string())?
        .ok_or("Instance not found")?;

    if let Ok(mut buffer) = state.log_buffer.lock() {
        buffer.add("DEBUG", format!("Instance '{}' config_path={}",
            instance.name, instance.config_path));
    }

    // Get enabled servers for this instance
    instance.enabled_servers = db
        .get_enabled_servers_for_instance(&instance_id)
        .map_err(|e| e.to_string())?;

    if let Ok(mut buffer) = state.log_buffer.lock() {
        buffer.add("DEBUG", format!("Enabled servers: {:?}", instance.enabled_servers));
    }

    // Get all servers
    let servers = db.get_all_servers().map_err(|e| e.to_string())?;

    // Get backup directory
    let backup_dir = config::get_backup_dir();

    // Sync configuration
    let result = config::sync_servers_to_instance(
        &instance,
        &servers,
        backup_dir.as_ref(),
    );

    match &result {
        Ok(sync_result) => {
            if let Ok(mut buffer) = state.log_buffer.lock() {
                buffer.add("INFO", format!("Sync successful for '{}', backup: {:?}, skipped: {:?}",
                    instance.name, sync_result.backup_path, sync_result.skipped_servers));
            }
        }
        Err(e) => {
            if let Ok(mut buffer) = state.log_buffer.lock() {
                buffer.add("ERROR", format!("Sync failed for '{}': {}", instance.name, e));
            }
        }
    }

    let sync_result = result?;

    // Record backup if created
    if let Some(ref path) = sync_result.backup_path {
        let backup = ConfigBackup::new(instance_id.clone(), path.to_string_lossy().to_string());
        db.create_backup(&backup).map_err(|e| e.to_string())?;
    }

    // Update last synced timestamp
    instance.last_synced = Some(Utc::now());
    db.update_instance(&instance).map_err(|e| e.to_string())?;

    Ok(SyncInstanceResult {
        backup_path: sync_result.backup_path.map(|p| p.to_string_lossy().to_string()),
        skipped_servers: sync_result.skipped_servers,
    })
}
```

**Step 3: Update check_server_health to handle HTTP servers**

Replace `check_server_health` function (lines 384-433) with:

```rust
#[tauri::command]
pub async fn check_server_health(server: McpServer) -> Result<ServerHealth, String> {
    use std::time::Duration;

    match &server.transport {
        crate::models::Transport::Stdio { command, .. } => {
            // For STDIO: try to run the command with --version
            let result = tokio::time::timeout(Duration::from_secs(5), async {
                let output = std::process::Command::new(command)
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
        crate::models::Transport::Sse { url, .. } | crate::models::Transport::Http { url, .. } => {
            // For HTTP: try to connect to the URL
            let result = tokio::time::timeout(Duration::from_secs(5), async {
                match reqwest::get(url).await {
                    Ok(response) => {
                        if response.status().is_success() || response.status().is_redirection() {
                            Ok(ServerHealth {
                                server_id: server.id.clone(),
                                status: HealthStatus::Healthy,
                                error_message: None,
                                last_checked: Utc::now(),
                            })
                        } else {
                            Ok(ServerHealth {
                                server_id: server.id.clone(),
                                status: HealthStatus::Error,
                                error_message: Some(format!("HTTP status: {}", response.status())),
                                last_checked: Utc::now(),
                            })
                        }
                    }
                    Err(e) => Ok(ServerHealth {
                        server_id: server.id.clone(),
                        status: HealthStatus::Error,
                        error_message: Some(format!("Connection failed: {}", e)),
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
    }
}
```

**Step 4: Add reqwest dependency for HTTP health checks**

In `src-tauri/Cargo.toml`, add under `[dependencies]`:

```toml
reqwest = { version = "0.11", features = ["json"] }
```

**Step 5: Build to verify compilation**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | head -50`

Expected: Should compile (or show registry.rs errors - we'll fix those next).

**Step 6: Commit commands changes**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/Cargo.toml
git commit -m "feat(commands): update sync and health check for transports

Add SyncInstanceResult with skipped_servers list.
Update check_server_health to handle HTTP servers.
Add reqwest dependency for HTTP connectivity checks.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: Update Registry Service

**Files:**
- Modify: `src-tauri/src/services/registry.rs`

**Step 1: Update RegistryServer to use Transport**

Replace the `RegistryServer` struct (lines 10-27) with:

```rust
/// A registry server entry from external sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryServer {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub transport: crate::models::Transport,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
}

impl RegistryServer {
    /// Create a STDIO registry server (backward compatible helper)
    pub fn stdio(
        name: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            name,
            description: None,
            transport: crate::models::Transport::Stdio { command, args, env },
            tags,
            repository: None,
            homepage: None,
        }
    }
}
```

**Step 2: Update registry_server_to_mcp_server**

Find and update the `registry_server_to_mcp_server` function:

```rust
/// Convert a registry server to an MCP server
pub fn registry_server_to_mcp_server(registry_server: &RegistryServer, registry_id: &str) -> McpServer {
    let mut server = McpServer::with_transport(
        registry_server.name.clone(),
        registry_server.transport.clone(),
    );
    server.description = registry_server.description.clone();
    server.tags = registry_server.tags.clone();
    server.source = Some(crate::models::ServerSource {
        source_type: crate::models::SourceType::Registry,
        url: Some(registry_id.to_string()),
    });
    server
}
```

**Step 3: Update builtin servers to use new format**

All the builtin server definitions need to use the new Transport format. Update the helper function and server definitions:

```rust
/// Helper to create a STDIO registry server
fn stdio_server(name: &str, desc: &str, command: &str, args: &[&str], tags: &[&str]) -> RegistryServer {
    RegistryServer {
        name: name.to_string(),
        description: Some(desc.to_string()),
        transport: crate::models::Transport::Stdio {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
        },
        tags: tags.iter().map(|s| s.to_string()).collect(),
        repository: None,
        homepage: None,
    }
}
```

Then update all the server definitions to use this helper. For example:
```rust
stdio_server(
    "filesystem",
    "Secure file operations with configurable access controls",
    "npx",
    &["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/dir"],
    &["filesystem", "official"],
),
```

**Step 4: Build to verify compilation**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml`

Expected: Should compile successfully.

**Step 5: Commit registry changes**

```bash
git add src-tauri/src/services/registry.rs
git commit -m "feat(registry): update to use Transport enum

Update RegistryServer to use Transport instead of flat fields.
Update registry_server_to_mcp_server conversion.
Update builtin server definitions.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 2: Frontend Types and Store

### Task 6: Update TypeScript Types

**Files:**
- Modify: `src/types/index.ts`

**Step 1: Add Transport type**

Add after line 1:

```typescript
export type Transport =
  | { type: "stdio"; command: string; args: string[]; env: Record<string, string> }
  | { type: "sse"; url: string; headers?: Record<string, string> }
  | { type: "http"; url: string; headers?: Record<string, string> };
```

**Step 2: Update McpServer interface**

Replace the `McpServer` interface (lines 1-14) with:

```typescript
export interface McpServer {
  id: string;
  name: string;
  description?: string;
  transport: Transport;
  tags: string[];
  source?: ServerSource;
  parentId?: string;
  createdAt: string;
  updatedAt: string;
}
```

**Step 3: Add helper functions**

Add after the McpServer interface:

```typescript
// Helper functions for working with Transport
export function isStdioTransport(transport: Transport): transport is { type: "stdio"; command: string; args: string[]; env: Record<string, string> } {
  return transport.type === "stdio";
}

export function isHttpTransport(transport: Transport): transport is { type: "sse" | "http"; url: string; headers?: Record<string, string> } {
  return transport.type === "sse" || transport.type === "http";
}

export function getTransportDisplayText(transport: Transport): string {
  if (transport.type === "stdio") {
    return `${transport.command} ${transport.args.join(" ")}`.trim();
  }
  return transport.url;
}

export function createStdioTransport(command: string, args: string[], env: Record<string, string>): Transport {
  return { type: "stdio", command, args, env };
}

export function createSseTransport(url: string, headers?: Record<string, string>): Transport {
  return { type: "sse", url, headers: headers || {} };
}

export function createHttpTransport(url: string, headers?: Record<string, string>): Transport {
  return { type: "http", url, headers: headers || {} };
}
```

**Step 4: Update RegistryServer interface**

Replace the `RegistryServer` interface (lines 221-230) with:

```typescript
export interface RegistryServer {
  name: string;
  description?: string;
  transport: Transport;
  tags: string[];
  repository?: string;
  homepage?: string;
}
```

**Step 5: Add SyncInstanceResult type**

Add after AppSettings:

```typescript
export interface SyncInstanceResult {
  backupPath: string | null;
  skippedServers: string[];
}
```

**Step 6: Build frontend to check types**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && pnpm build 2>&1 | head -50`

Expected: TypeScript errors in components that reference old fields - we'll fix next.

**Step 7: Commit type changes**

```bash
git add src/types/index.ts
git commit -m "feat(types): add Transport type and helpers

Add Transport union type for STDIO/SSE/HTTP.
Update McpServer and RegistryServer to use Transport.
Add helper functions for type guards and display.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 7: Update Zustand Store

**Files:**
- Modify: `src/store/index.ts`

**Step 1: Update syncInstance return type**

Update the `syncInstance` action type in the interface (around line 46):

```typescript
  syncInstance: (instanceId: string) => Promise<SyncInstanceResult>;
```

**Step 2: Update the implementation**

Update the `syncInstance` implementation (around line 180):

```typescript
  syncInstance: async (instanceId: string) => {
    const result = await invoke<SyncInstanceResult>("sync_instance", {
      instanceId,
    });
    // Reload instances to get updated lastSynced
    await get().loadInstances();
    return result;
  },
```

**Step 3: Update readConfigFile return type**

Update the `readConfigFile` type and implementation to handle the new format:

```typescript
  readConfigFile: async (path: string) => {
    try {
      const config = await invoke<{ mcpServers: Record<string, unknown> }>("read_config_file", { path });
      return config;
    } catch {
      return null;
    }
  },
```

**Step 4: Add import for SyncInstanceResult**

Update the import at the top:

```typescript
import type {
  McpServer,
  ClientInstance,
  AppSettings,
  DetectedClient,
  RegistrySource,
  RegistryServer,
  SyncInstanceResult,
} from "@/types";
```

**Step 5: Build to check compilation**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && pnpm build 2>&1 | head -50`

Expected: More TypeScript errors in UI components - we'll fix next.

**Step 6: Commit store changes**

```bash
git add src/store/index.ts
git commit -m "feat(store): update for new transport types

Update syncInstance to return SyncInstanceResult.
Update readConfigFile for new config format.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 3: UI Components

### Task 8: Update Servers Page - Form Data and Dialog

**Files:**
- Modify: `src/pages/Servers.tsx`

**Step 1: Update imports**

Add at top:

```typescript
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Globe, Terminal } from "lucide-react";
import { isStdioTransport, getTransportDisplayText, createStdioTransport, createSseTransport, createHttpTransport } from "@/types";
```

**Step 2: Update ServerFormData interface**

Replace the `ServerFormData` interface (lines 35-42) with:

```typescript
type TransportType = "stdio" | "sse" | "http";

interface ServerFormData {
  name: string;
  description: string;
  transportType: TransportType;
  // STDIO fields
  command: string;
  args: string;
  env: string;
  // HTTP fields
  url: string;
  headers: string;
  // Common
  tags: string;
}

const emptyFormData: ServerFormData = {
  name: "",
  description: "",
  transportType: "stdio",
  command: "",
  args: "",
  env: "",
  url: "",
  headers: "",
  tags: "",
};
```

**Step 3: Update handleOpenDialog to handle transport**

Replace the `handleOpenDialog` function (lines 121-139) with:

```typescript
  const handleOpenDialog = (server?: McpServer) => {
    if (server) {
      setEditingServer(server);
      if (isStdioTransport(server.transport)) {
        setFormData({
          name: server.name,
          description: server.description || "",
          transportType: "stdio",
          command: server.transport.command,
          args: server.transport.args.join("\n"),
          env: Object.entries(server.transport.env)
            .map(([k, v]) => `${k}=${v}`)
            .join("\n"),
          url: "",
          headers: "",
          tags: server.tags.join(", "),
        });
      } else {
        setFormData({
          name: server.name,
          description: server.description || "",
          transportType: server.transport.type as TransportType,
          command: "",
          args: "",
          env: "",
          url: server.transport.url,
          headers: Object.entries(server.transport.headers || {})
            .map(([k, v]) => `${k}=${v}`)
            .join("\n"),
          tags: server.tags.join(", "),
        });
      }
    } else {
      setEditingServer(null);
      setFormData(emptyFormData);
    }
    setIsDialogOpen(true);
  };
```

**Step 4: Update handleSubmit to create correct transport**

Replace the `handleSubmit` function (lines 148-203) with:

```typescript
  const handleSubmit = async () => {
    try {
      const tags = formData.tags
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);

      const now = new Date().toISOString();

      let transport;
      if (formData.transportType === "stdio") {
        const args = formData.args
          .split("\n")
          .map((a) => a.trim())
          .filter(Boolean);
        const env: Record<string, string> = {};
        formData.env
          .split("\n")
          .map((e) => e.trim())
          .filter(Boolean)
          .forEach((line) => {
            const [key, ...valueParts] = line.split("=");
            if (key) {
              env[key.trim()] = valueParts.join("=").trim();
            }
          });
        transport = createStdioTransport(formData.command, args, env);
      } else {
        const headers: Record<string, string> = {};
        formData.headers
          .split("\n")
          .map((h) => h.trim())
          .filter(Boolean)
          .forEach((line) => {
            const [key, ...valueParts] = line.split("=");
            if (key) {
              headers[key.trim()] = valueParts.join("=").trim();
            }
          });
        transport = formData.transportType === "sse"
          ? createSseTransport(formData.url, headers)
          : createHttpTransport(formData.url, headers);
      }

      if (editingServer) {
        await updateServer({
          ...editingServer,
          name: formData.name,
          description: formData.description || undefined,
          transport,
          tags,
          updatedAt: now,
        });
      } else {
        await createServer({
          id: crypto.randomUUID(),
          name: formData.name,
          description: formData.description || undefined,
          transport,
          tags,
          source: { sourceType: "manual" },
          parentId: duplicatingFromId || undefined,
          createdAt: now,
          updatedAt: now,
        });
      }

      handleCloseDialog();
    } catch (error) {
      console.error("Failed to save server:", error);
    }
  };
```

**Step 5: Update handleDuplicate**

Replace the `handleDuplicate` function (lines 222-241) with:

```typescript
  const handleDuplicate = (server: McpServer) => {
    const instanceNumber = servers.filter(
      (s) => s.parentId === server.id || s.id === server.id
    ).length;
    setEditingServer(null);

    if (isStdioTransport(server.transport)) {
      setFormData({
        name: `${server.name} (${instanceNumber + 1})`,
        description: server.description || "",
        transportType: "stdio",
        command: server.transport.command,
        args: server.transport.args.join("\n"),
        env: Object.entries(server.transport.env)
          .map(([k, v]) => `${k}=${v}`)
          .join("\n"),
        url: "",
        headers: "",
        tags: server.tags.join(", "),
      });
    } else {
      setFormData({
        name: `${server.name} (${instanceNumber + 1})`,
        description: server.description || "",
        transportType: server.transport.type as TransportType,
        command: "",
        args: "",
        env: "",
        url: server.transport.url,
        headers: Object.entries(server.transport.headers || {})
          .map(([k, v]) => `${k}=${v}`)
          .join("\n"),
        tags: server.tags.join(", "),
      });
    }
    setDuplicatingFromId(server.id);
    setIsDialogOpen(true);
  };
```

**Step 6: Update matchesSearch**

Replace the `matchesSearch` function (lines 69-74) with:

```typescript
  const matchesSearch = (server: McpServer) => {
    const searchLower = searchQuery.toLowerCase();
    const transportText = getTransportDisplayText(server.transport).toLowerCase();
    return (
      server.name.toLowerCase().includes(searchLower) ||
      transportText.includes(searchLower) ||
      server.tags.some((tag) => tag.toLowerCase().includes(searchLower))
    );
  };
```

**Step 7: Update server card display**

Replace the server card content section (around lines 361-380) with:

```typescript
                <CardContent>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2 text-sm">
                      {isStdioTransport(server.transport) ? (
                        <Terminal className="w-4 h-4 text-muted-foreground" />
                      ) : (
                        <Globe className="w-4 h-4 text-muted-foreground" />
                      )}
                      <code className="px-2 py-1 bg-muted rounded text-xs truncate max-w-[200px]">
                        {isStdioTransport(server.transport)
                          ? server.transport.command
                          : server.transport.url}
                      </code>
                      {!isStdioTransport(server.transport) && (
                        <Badge variant="outline" className="text-xs">
                          {server.transport.type.toUpperCase()}
                        </Badge>
                      )}
                    </div>
                    {isStdioTransport(server.transport) && server.transport.args.length > 0 && (
                      <div className="text-xs text-muted-foreground truncate">
                        Args: {server.transport.args.join(" ")}
                      </div>
                    )}
                    {server.tags.length > 0 && (
                      <div className="flex flex-wrap gap-1 mt-2">
                        {server.tags.map((tag) => (
                          <Badge key={tag} variant="secondary" className="text-xs">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    )}
                  </div>
                </CardContent>
```

**Step 8: Update dialog form fields**

Replace the dialog form section (around lines 407-477) with:

```typescript
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="My MCP Server"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description (optional)</Label>
              <Input
                id="description"
                placeholder="A brief description"
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="transportType">Transport Type</Label>
              <Select
                value={formData.transportType}
                onValueChange={(value: TransportType) =>
                  setFormData({ ...formData, transportType: value })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="stdio">
                    <div className="flex items-center gap-2">
                      <Terminal className="w-4 h-4" />
                      STDIO (Local Process)
                    </div>
                  </SelectItem>
                  <SelectItem value="sse">
                    <div className="flex items-center gap-2">
                      <Globe className="w-4 h-4" />
                      SSE (Server-Sent Events)
                    </div>
                  </SelectItem>
                  <SelectItem value="http">
                    <div className="flex items-center gap-2">
                      <Globe className="w-4 h-4" />
                      Streamable HTTP
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            {formData.transportType === "stdio" ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="command">Command</Label>
                  <Input
                    id="command"
                    placeholder="npx"
                    value={formData.command}
                    onChange={(e) =>
                      setFormData({ ...formData, command: e.target.value })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="args">Arguments (one per line)</Label>
                  <Textarea
                    id="args"
                    placeholder={"-y\n@modelcontextprotocol/server-filesystem\n/path/to/dir"}
                    value={formData.args}
                    onChange={(e) =>
                      setFormData({ ...formData, args: e.target.value })
                    }
                    rows={3}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="env">
                    Environment Variables (KEY=value, one per line)
                  </Label>
                  <Textarea
                    id="env"
                    placeholder="API_KEY=your-key-here"
                    value={formData.env}
                    onChange={(e) =>
                      setFormData({ ...formData, env: e.target.value })
                    }
                    rows={2}
                  />
                </div>
              </>
            ) : (
              <>
                <div className="space-y-2">
                  <Label htmlFor="url">URL</Label>
                  <Input
                    id="url"
                    placeholder="http://localhost:3000/mcp"
                    value={formData.url}
                    onChange={(e) =>
                      setFormData({ ...formData, url: e.target.value })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="headers">
                    Headers (KEY=value, one per line)
                  </Label>
                  <Textarea
                    id="headers"
                    placeholder="Authorization=Bearer your-token"
                    value={formData.headers}
                    onChange={(e) =>
                      setFormData({ ...formData, headers: e.target.value })
                    }
                    rows={2}
                  />
                </div>
              </>
            )}

            <div className="space-y-2">
              <Label htmlFor="tags">Tags (comma separated)</Label>
              <Input
                id="tags"
                placeholder="filesystem, tools"
                value={formData.tags}
                onChange={(e) =>
                  setFormData({ ...formData, tags: e.target.value })
                }
              />
            </div>
          </div>
```

**Step 9: Update submit button disabled condition**

Update the submit button (around line 483):

```typescript
            <Button
              onClick={handleSubmit}
              disabled={
                !formData.name ||
                (formData.transportType === "stdio" ? !formData.command : !formData.url)
              }
            >
              {editingServer ? "Save Changes" : "Add Server"}
            </Button>
```

**Step 10: Build to verify**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && pnpm build 2>&1 | head -50`

Expected: Should build (or show ImportDialog errors - we'll fix next).

**Step 11: Commit UI changes**

```bash
git add src/pages/Servers.tsx
git commit -m "feat(ui): update Servers page for transport selection

Add transport type selector to add/edit dialog.
Show conditional fields based on transport type.
Update server cards to show transport icon and type badge.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 9: Update Import Dialog

**Files:**
- Modify: `src/components/ImportDialog.tsx`

**Step 1: Update registry server display**

Find where registry servers are displayed and update to show transport type badges and use the new transport field.

Add imports:
```typescript
import { isStdioTransport, getTransportDisplayText } from "@/types";
import { Globe, Terminal } from "lucide-react";
```

Update server list item display to show transport type.

**Step 2: Build and verify**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && pnpm build`

Expected: Should compile successfully.

**Step 3: Commit**

```bash
git add src/components/ImportDialog.tsx
git commit -m "feat(ui): update ImportDialog for transport display

Show transport type badges on registry servers.
Use transport helper functions for display.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 10: Add Sync Warning Toast

**Files:**
- Modify: `src/pages/Instances.tsx` (or wherever sync is triggered)

**Step 1: Update sync handler to show skipped servers warning**

Find the sync button handler and update it to check for skipped servers:

```typescript
const handleSync = async (instanceId: string) => {
  try {
    const result = await syncInstance(instanceId);
    if (result.skippedServers.length > 0) {
      toast({
        title: "Sync completed with warnings",
        description: `${result.skippedServers.length} server(s) skipped due to transport incompatibility: ${result.skippedServers.join(", ")}`,
        variant: "warning",
      });
    } else {
      toast({
        title: "Sync successful",
        description: result.backupPath ? `Backup created at ${result.backupPath}` : "Configuration synced",
      });
    }
  } catch (error) {
    toast({
      title: "Sync failed",
      description: error instanceof Error ? error.message : String(error),
      variant: "destructive",
    });
  }
};
```

**Step 2: Build and verify**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && pnpm build`

**Step 3: Commit**

```bash
git add src/pages/Instances.tsx
git commit -m "feat(ui): add warning toast for skipped servers

Show toast when HTTP servers are skipped during sync.
Display list of incompatible server names.
Part of #26 - HTTP transport support.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 4: Final Integration

### Task 11: Full Build and Test

**Step 1: Build entire application**

Run: `cd /Volumes/Secondary/DevelopmentWork/Personal/mcp-hub && PATH="$HOME/.cargo/bin:$PATH" pnpm tauri build`

Expected: Should build successfully.

**Step 2: Manual testing checklist**

- [ ] Create a new STDIO server - verify it saves and displays correctly
- [ ] Create a new SSE server - verify it saves with URL and headers
- [ ] Create a new HTTP server - verify it saves correctly
- [ ] Edit an existing server - verify transport fields populate correctly
- [ ] Change transport type during edit - verify fields switch
- [ ] Sync to Claude Desktop - verify HTTP servers output correct format
- [ ] Sync to a STDIO-only client - verify HTTP servers are skipped with warning
- [ ] Import from registry - verify servers show transport badges
- [ ] Duplicate a server - verify transport is copied correctly
- [ ] Database migration - verify existing servers still work after upgrade

**Step 3: Commit final integration**

```bash
git add -A
git commit -m "feat: complete HTTP/SSE transport support

Closes #26

Summary:
- Added Transport enum (STDIO, SSE, HTTP) to data model
- Updated database schema with migration for existing servers
- Config sync outputs correct format per transport type
- Client capability matrix skips incompatible transports
- UI shows transport selector with conditional fields
- Import/export supports both formats
- Health check works for both STDIO and HTTP servers

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Testing Checklist

- [ ] Create STDIO server (existing flow works)
- [ ] Create SSE server with URL and headers
- [ ] Create Streamable HTTP server
- [ ] Edit existing server, change transport type
- [ ] Sync STDIO server to all clients
- [ ] Sync HTTP server to compatible client (Claude Desktop)
- [ ] Sync HTTP server to incompatible client - verify warning shown
- [ ] Import legacy config file (command/args format)
- [ ] Import new config file (transport format)
- [ ] Registry import shows transport badges
- [ ] Database migration preserves existing servers
- [ ] Health check works for STDIO servers
- [ ] Health check works for HTTP servers
