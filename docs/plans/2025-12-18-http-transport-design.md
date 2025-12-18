# HTTP/SSE Transport Support Design

**Issue**: #26
**Date**: 2025-12-18
**Status**: Approved

## Summary

Add support for HTTP-based MCP servers (SSE and Streamable HTTP transports) in addition to the existing STDIO transport. This enables connecting to remote servers, cloud-hosted services, and local HTTP development servers.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Use cases | All (remote, cloud, local) | Maximum flexibility |
| Transport protocols | Both SSE and Streamable HTTP | Manual selection by user |
| Authentication | Plain headers | User manages secrets |
| Client compatibility | Skip with warning | Balance UX and reliability |
| UI pattern | Transport dropdown first | Progressive disclosure |
| Registry support | Include HTTP servers | Future-proof design |

## Data Model

### Rust (`src-tauri/src/models/mod.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Transport {
    Stdio {
        command: String,
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    Sse {
        url: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,
    },
    Http {
        url: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,
    },
}

pub struct McpServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub transport: Transport,  // replaces command/args/env
    pub tags: Vec<String>,
    pub source: Option<ServerSource>,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### TypeScript (`src/types/index.ts`)

```typescript
export type Transport =
  | { type: "stdio"; command: string; args: string[]; env: Record<string, string> }
  | { type: "sse"; url: string; headers?: Record<string, string> }
  | { type: "http"; url: string; headers?: Record<string, string> };

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

### Helper Functions

```typescript
export function isStdioServer(server: McpServer): boolean {
  return server.transport.type === "stdio";
}

export function getServerDisplayCommand(server: McpServer): string {
  return server.transport.type === "stdio"
    ? `${server.transport.command} ${server.transport.args.join(" ")}`
    : server.transport.url;
}
```

## Database Migration

```sql
-- Migration: add_transport_field

-- Add transport column (JSON)
ALTER TABLE servers ADD COLUMN transport TEXT;

-- Migrate existing STDIO data into transport JSON
UPDATE servers SET transport = json_object(
    'type', 'stdio',
    'command', command,
    'args', json(args),
    'env', json(env)
);

-- Note: SQLite doesn't support DROP COLUMN in older versions
-- We'll handle old columns in the Rust code by ignoring them
```

## Config Sync

### Client Capability Matrix

```rust
pub fn client_supported_transports(client_type: &ClientType) -> Vec<&'static str> {
    match client_type {
        // Full HTTP support
        ClientType::ClaudeDesktop
        | ClientType::ClaudeCode
        | ClientType::Cursor
        | ClientType::Cline
        | ClientType::RooCode => vec!["stdio", "sse", "http"],

        // SSE only
        ClientType::Vscode
        | ClientType::VscodeInsiders
        | ClientType::Continue => vec!["stdio", "sse"],

        // STDIO only
        _ => vec!["stdio"],
    }
}
```

### Config Output Formats

**STDIO servers:**
```json
{ "command": "npx", "args": ["@mcp/server"], "env": {} }
```

**SSE servers:**
```json
{ "transport": "sse", "url": "http://localhost:3000/mcp", "headers": {} }
```

**Streamable HTTP servers:**
```json
{ "transport": "http", "url": "http://localhost:3000/mcp", "headers": {} }
```

### Backward-Compatible Import

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum McpServerEntryCompat {
    WithTransport { transport: Transport },
    LegacyStdio {
        command: String,
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

impl McpServerEntryCompat {
    pub fn into_transport(self) -> Transport {
        match self {
            Self::WithTransport { transport } => transport,
            Self::LegacyStdio { command, args, env } =>
                Transport::Stdio { command, args, env },
        }
    }
}
```

## UI Changes

### Add Server Dialog

1. Transport type selector at top (STDIO / SSE / Streamable HTTP)
2. Conditional fields based on selection:
   - STDIO: Command, Arguments, Environment Variables
   - SSE/HTTP: URL, Headers

### Server List Cards

- STDIO: Terminal icon + command preview
- SSE/HTTP: Globe icon + URL preview
- Badge showing transport type

### Sync Warning

When HTTP servers are skipped for incompatible clients, show a toast notification listing which servers were not synced.

## Registry Updates

### RegistryServer Type

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

### Import Dialog

- Transport type badge on each server
- Optional filter by transport type

## Implementation Phases

### Phase 1: Data Model & Migration
1. Update Rust `Transport` enum and `McpServer` struct
2. Write SQLite migration for existing servers
3. Update TypeScript types to mirror Rust
4. Add helper functions

### Phase 2: Config Sync
5. Add `client_supported_transports()` capability matrix
6. Update `sync_servers_to_instance()` for multi-transport output
7. Return skipped servers list for UI warning
8. Add backward-compatible import deserialization

### Phase 3: UI
9. Update Add Server dialog with transport selector
10. Conditional field rendering
11. Update server list cards
12. Add warning toast for skipped servers

### Phase 4: Registry
13. Update `RegistryServer` type
14. Update Import Dialog with badges and filter

## Files Changed

| File | Changes |
|------|---------|
| `src-tauri/src/models/mod.rs` | Add `Transport` enum, update `McpServer` |
| `src-tauri/src/db/mod.rs` | Add migration |
| `src-tauri/src/services/config.rs` | Config sync, client capability, compat layer |
| `src/types/index.ts` | TypeScript types |
| `src/pages/Servers.tsx` | Add/edit dialog, server cards |
| `src/components/ImportDialog.tsx` | Registry import UI |
| `src-tauri/src/services/registry.rs` | Registry format |

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
