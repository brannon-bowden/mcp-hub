# Custom Registries Feature Design

## Overview

Add support for user-defined MCP server registries, allowing users to point to their own registry files (local or remote) for private/company servers. Custom registries appear alongside built-in registries in the Import dialog.

## Requirements

- Support multiple custom registries per user
- Accept both local file paths and remote URLs
- Support GitHub authentication for private repositories
- Cache registry data locally with manual refresh
- Manage registries directly from the Import dialog

## Registry JSON Format

Custom registry files use this structure:

```json
{
  "name": "Acme Corp MCP Servers",
  "description": "Internal company MCP servers",
  "icon": "building",
  "servers": [
    {
      "name": "acme-database",
      "description": "PostgreSQL connector for internal DBs",
      "command": "npx",
      "args": ["@acme-corp/mcp-database"],
      "env": {
        "ACME_REGION": "us-west-2"
      },
      "tags": ["database", "internal", "postgres"],
      "repository": "https://github.com/acme-corp/mcp-database",
      "homepage": "https://docs.acme.com/mcp/database"
    }
  ]
}
```

### Field Definitions

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name for the registry |
| `description` | No | Brief description shown in dropdown |
| `icon` | No | Icon identifier (building, star, etc.) |
| `servers` | Yes | Array of server definitions |

Server fields match the existing `RegistryServer` structure.

## Data Model

### Database Table: `custom_registries`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT (UUID) | Primary key |
| `name` | TEXT | Display name (from JSON or user override) |
| `url` | TEXT | Local path or remote URL |
| `icon` | TEXT | Optional icon identifier |
| `requires_auth` | BOOLEAN | Whether GitHub token is needed |
| `cached_data` | TEXT | Cached JSON response |
| `cached_at` | TIMESTAMP | When cache was last updated |
| `created_at` | TIMESTAMP | When registry was added |

### Credential Storage

GitHub tokens stored in OS keychain via existing `credentials.rs` service, keyed by registry ID: `mcp-hub:custom-registry:{id}`.

## UI Design

### Import Dialog Changes

The registry dropdown gets additions:

1. Custom registries appear below built-in registries, separated by a divider
2. "+" button next to dropdown opens "Add Custom Registry" dialog
3. Refresh button appears when custom registry is selected
4. Gear icon for edit/delete when custom registry is selected

```
┌─────────────────────────────────────────────────┐
│ Registry: [MCP Hub Built-in        ▼] [+] [⟳]  │
│                                                 │
│  ├─ MCP Hub Built-in (40+ servers)             │
│  ├─ Anthropic Official (19 servers)            │
│  ├─ Awesome MCP Servers                        │
│  ├─ ──────────────────                         │
│  ├─ ⭐ Acme Corp Servers (12 servers)          │
│  └─ ⭐ Personal Tools (3 servers)              │
└─────────────────────────────────────────────────┘
```

### Add Custom Registry Dialog

Form fields:

- **URL** (required) - Local path or remote URL
- **GitHub Token** (optional) - Password field, shown when URL contains `github.com`
- **Name Override** (optional) - Override the name from JSON file

"Test Connection" button validates URL and shows preview before saving.

### Edit/Delete

When custom registry is selected, gear icon dropdown provides:
- Edit registry
- Delete registry

## Backend Implementation

### Fetching Flow

```
1. Check cache
   ├─ Cache exists & not force refresh?
   │   └─ Return cached servers immediately
   └─ No cache or refresh requested?
       └─ Fetch from URL

2. Fetch from URL
   ├─ Local file (starts with "/" or "file://")
   │   └─ Read directly from filesystem
   └─ Remote URL (https://)
       ├─ GitHub URL + has token?
       │   └─ Add Authorization: Bearer header
       └─ Fetch with reqwest

3. Parse & Validate
   ├─ Parse JSON
   ├─ Validate structure
   └─ Convert to Vec<RegistryServer>

4. Update cache
   └─ Store in database with timestamp

5. Return servers
```

### Error Handling

| Error | User Message |
|-------|--------------|
| Network timeout | "Could not reach registry. Using cached data." |
| Invalid JSON | "Registry file is not valid JSON." |
| Missing fields | "Registry missing required 'servers' field." |
| Auth failed (401/403) | "Authentication failed. Check your GitHub token." |
| File not found | "Registry file not found at path." |

Cached data serves as fallback when remote fetch fails.

### Rust Services

New file `src-tauri/src/services/custom_registry.rs`:

```rust
pub struct CustomRegistry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub requires_auth: bool,
    pub cached_at: Option<DateTime<Utc>>,
    pub server_count: Option<usize>,
}

pub fn add_custom_registry(url: &str, name: Option<&str>, token: Option<&str>) -> Result<CustomRegistry, String>;
pub fn update_custom_registry(id: &str, url: Option<&str>, name: Option<&str>, token: Option<&str>) -> Result<CustomRegistry, String>;
pub fn delete_custom_registry(id: &str) -> Result<(), String>;
pub fn fetch_custom_registry_servers(id: &str, force_refresh: bool) -> Result<Vec<RegistryServer>, String>;
pub fn get_all_custom_registries() -> Result<Vec<CustomRegistry>, String>;
```

### Tauri Commands

New commands:

```rust
#[tauri::command]
pub fn add_custom_registry(url: String, name: Option<String>, token: Option<String>) -> Result<CustomRegistry, String>;

#[tauri::command]
pub fn update_custom_registry(id: String, url: Option<String>, name: Option<String>, token: Option<String>) -> Result<CustomRegistry, String>;

#[tauri::command]
pub fn delete_custom_registry(id: String) -> Result<(), String>;

#[tauri::command]
pub fn refresh_custom_registry(id: String) -> Result<Vec<RegistryServer>, String>;
```

Modified commands:
- `get_registries()` - Returns both built-in and custom registries
- `get_registry_servers(id)` - Routes to custom registry handler when ID matches

## File Changes

| File | Changes |
|------|---------|
| `src-tauri/src/services/custom_registry.rs` | New - fetch, cache, CRUD logic |
| `src-tauri/src/services/registry.rs` | Integrate custom registries in getters |
| `src-tauri/src/db/mod.rs` | Add `custom_registries` table & queries |
| `src-tauri/src/commands/mod.rs` | New Tauri commands |
| `src/components/ImportDialog.tsx` | Add "+", refresh, gear UI elements |
| `src/components/AddRegistryDialog.tsx` | New - form for adding/editing registries |
| `src/types/index.ts` | Add `CustomRegistry` type |
| `src/store/index.ts` | Add custom registry actions |

## URL Support

### Local Files

- Absolute paths: `/Users/brannon/registries/acme.json`
- File URLs: `file:///Users/brannon/registries/acme.json`

### Remote URLs

- Public: `https://raw.githubusercontent.com/acme/mcp-registry/main/registry.json`
- Private (with token): Same URL, token added via Authorization header

## Security Considerations

- GitHub tokens stored in OS keychain, never in database
- Tokens only sent to github.com and raw.githubusercontent.com domains
- Local file access restricted to readable paths (OS-enforced)
- Registry JSON validated before parsing to prevent injection
