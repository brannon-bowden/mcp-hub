# Custom Registries Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow users to add their own MCP server registries via local files or remote URLs (including private GitHub repos with token auth).

**Architecture:** New `custom_registries` database table stores registry metadata and cached data. New Rust service handles fetching, parsing, and caching. Frontend adds registry management UI to ImportDialog. GitHub tokens stored in OS keychain via existing credentials service.

**Tech Stack:** Rust/Tauri backend, SQLite database, React/TypeScript frontend, reqwest for HTTP, keyring for credentials

**Related:** See `docs/designs/2026-01-01-custom-registries-design.md` for full design.

---

## Task 1: Database Schema

**Files:**
- Modify: `src-tauri/src/db/mod.rs`

**Step 1: Add custom_registries table to schema**

In `db/mod.rs`, add the table creation to the `initialize` function after the existing tables:

```rust
// Add after the existing CREATE TABLE statements in initialize()
conn.execute(
    "CREATE TABLE IF NOT EXISTS custom_registries (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        url TEXT NOT NULL,
        description TEXT,
        icon TEXT,
        requires_auth INTEGER NOT NULL DEFAULT 0,
        cached_data TEXT,
        cached_at TEXT,
        created_at TEXT NOT NULL
    )",
    [],
)?;
```

**Step 2: Verify by running cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src-tauri/src/db/mod.rs
git commit -m "feat(db): Add custom_registries table schema"
```

---

## Task 2: Database CRUD Operations

**Files:**
- Modify: `src-tauri/src/db/mod.rs`

**Step 1: Add CustomRegistry struct**

Add near the top of the file with other structs:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRegistry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub requires_auth: bool,
    pub cached_data: Option<String>,
    pub cached_at: Option<String>,
    pub created_at: String,
}
```

**Step 2: Add create_custom_registry method**

Add to the `impl Database` block:

```rust
pub fn create_custom_registry(&self, registry: &CustomRegistry) -> Result<()> {
    self.conn.execute(
        "INSERT INTO custom_registries (id, name, url, description, icon, requires_auth, cached_data, cached_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            registry.id,
            registry.name,
            registry.url,
            registry.description,
            registry.icon,
            registry.requires_auth as i32,
            registry.cached_data,
            registry.cached_at,
            registry.created_at,
        ],
    )?;
    Ok(())
}
```

**Step 3: Add get_custom_registries method**

```rust
pub fn get_custom_registries(&self) -> Result<Vec<CustomRegistry>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, name, url, description, icon, requires_auth, cached_data, cached_at, created_at
         FROM custom_registries ORDER BY name"
    )?;

    let registries = stmt.query_map([], |row| {
        Ok(CustomRegistry {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            icon: row.get(4)?,
            requires_auth: row.get::<_, i32>(5)? != 0,
            cached_data: row.get(6)?,
            cached_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    registries.collect()
}
```

**Step 4: Add get_custom_registry method**

```rust
pub fn get_custom_registry(&self, id: &str) -> Result<Option<CustomRegistry>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, name, url, description, icon, requires_auth, cached_data, cached_at, created_at
         FROM custom_registries WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(CustomRegistry {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            icon: row.get(4)?,
            requires_auth: row.get::<_, i32>(5)? != 0,
            cached_data: row.get(6)?,
            cached_at: row.get(7)?,
            created_at: row.get(8)?,
        }))
    } else {
        Ok(None)
    }
}
```

**Step 5: Add update_custom_registry method**

```rust
pub fn update_custom_registry(&self, registry: &CustomRegistry) -> Result<()> {
    self.conn.execute(
        "UPDATE custom_registries
         SET name = ?2, url = ?3, description = ?4, icon = ?5, requires_auth = ?6, cached_data = ?7, cached_at = ?8
         WHERE id = ?1",
        params![
            registry.id,
            registry.name,
            registry.url,
            registry.description,
            registry.icon,
            registry.requires_auth as i32,
            registry.cached_data,
            registry.cached_at,
        ],
    )?;
    Ok(())
}
```

**Step 6: Add delete_custom_registry method**

```rust
pub fn delete_custom_registry(&self, id: &str) -> Result<()> {
    self.conn.execute("DELETE FROM custom_registries WHERE id = ?1", params![id])?;
    Ok(())
}
```

**Step 7: Add update_custom_registry_cache method**

```rust
pub fn update_custom_registry_cache(&self, id: &str, cached_data: &str, cached_at: &str) -> Result<()> {
    self.conn.execute(
        "UPDATE custom_registries SET cached_data = ?2, cached_at = ?3 WHERE id = ?1",
        params![id, cached_data, cached_at],
    )?;
    Ok(())
}
```

**Step 8: Verify by running cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 9: Commit**

```bash
git add src-tauri/src/db/mod.rs
git commit -m "feat(db): Add CRUD operations for custom_registries"
```

---

## Task 3: Custom Registry Service

**Files:**
- Create: `src-tauri/src/services/custom_registry.rs`
- Modify: `src-tauri/src/services/mod.rs`

**Step 1: Create custom_registry.rs service file**

```rust
use crate::db::{CustomRegistry, Database};
use crate::services::credentials;
use crate::services::registry::RegistryServer;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

/// JSON structure for custom registry files
#[derive(Debug, Deserialize)]
pub struct CustomRegistryFile {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub servers: Vec<RegistryServer>,
}

/// Result of fetching a custom registry
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub servers: Vec<RegistryServer>,
    pub from_cache: bool,
    pub cached_at: Option<String>,
}

/// Add a new custom registry
pub fn add_custom_registry(
    db: &Database,
    url: &str,
    name_override: Option<&str>,
    token: Option<&str>,
) -> Result<CustomRegistry, String> {
    // First, try to fetch and validate the registry
    let registry_file = fetch_registry_file(url, token)?;

    let id = Uuid::new_v4().to_string();
    let name = name_override.unwrap_or(&registry_file.name).to_string();
    let requires_auth = token.is_some();

    // Store token in keychain if provided
    if let Some(token) = token {
        let credential_key = format!("custom-registry:{}", id);
        credentials::store_credential(&credential_key, token)
            .map_err(|e| format!("Failed to store token: {}", e))?;
    }

    // Cache the fetched data
    let cached_data = serde_json::to_string(&registry_file.servers)
        .map_err(|e| format!("Failed to serialize servers: {}", e))?;
    let cached_at = Utc::now().to_rfc3339();

    let registry = CustomRegistry {
        id: id.clone(),
        name,
        url: url.to_string(),
        description: registry_file.description,
        icon: registry_file.icon,
        requires_auth,
        cached_data: Some(cached_data),
        cached_at: Some(cached_at),
        created_at: Utc::now().to_rfc3339(),
    };

    db.create_custom_registry(&registry)
        .map_err(|e| format!("Failed to save registry: {}", e))?;

    Ok(registry)
}

/// Update an existing custom registry
pub fn update_custom_registry(
    db: &Database,
    id: &str,
    url: Option<&str>,
    name_override: Option<&str>,
    token: Option<&str>,
) -> Result<CustomRegistry, String> {
    let mut registry = db
        .get_custom_registry(id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Registry not found".to_string())?;

    // Update URL if provided
    if let Some(new_url) = url {
        registry.url = new_url.to_string();
    }

    // Update name if provided
    if let Some(new_name) = name_override {
        registry.name = new_name.to_string();
    }

    // Update token if provided
    if let Some(new_token) = token {
        let credential_key = format!("custom-registry:{}", id);
        credentials::store_credential(&credential_key, new_token)
            .map_err(|e| format!("Failed to store token: {}", e))?;
        registry.requires_auth = true;
    }

    // Re-fetch to update cache
    let token_for_fetch = if registry.requires_auth {
        get_registry_token(id).ok()
    } else {
        None
    };

    if let Ok(registry_file) = fetch_registry_file(&registry.url, token_for_fetch.as_deref()) {
        let cached_data = serde_json::to_string(&registry_file.servers)
            .map_err(|e| format!("Failed to serialize servers: {}", e))?;
        registry.cached_data = Some(cached_data);
        registry.cached_at = Some(Utc::now().to_rfc3339());
    }

    db.update_custom_registry(&registry)
        .map_err(|e| format!("Failed to update registry: {}", e))?;

    Ok(registry)
}

/// Delete a custom registry
pub fn delete_custom_registry(db: &Database, id: &str) -> Result<(), String> {
    // Remove token from keychain if it exists
    let credential_key = format!("custom-registry:{}", id);
    let _ = credentials::delete_credential(&credential_key);

    db.delete_custom_registry(id)
        .map_err(|e| format!("Failed to delete registry: {}", e))
}

/// Fetch servers from a custom registry
pub fn fetch_custom_registry_servers(
    db: &Database,
    id: &str,
    force_refresh: bool,
) -> Result<FetchResult, String> {
    let registry = db
        .get_custom_registry(id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Registry not found".to_string())?;

    // Return cached data if available and not forcing refresh
    if !force_refresh {
        if let Some(cached_data) = &registry.cached_data {
            let servers: Vec<RegistryServer> = serde_json::from_str(cached_data)
                .map_err(|e| format!("Failed to parse cached data: {}", e))?;
            return Ok(FetchResult {
                servers,
                from_cache: true,
                cached_at: registry.cached_at.clone(),
            });
        }
    }

    // Fetch fresh data
    let token = if registry.requires_auth {
        get_registry_token(id).ok()
    } else {
        None
    };

    let registry_file = fetch_registry_file(&registry.url, token.as_deref())?;

    // Update cache
    let cached_data = serde_json::to_string(&registry_file.servers)
        .map_err(|e| format!("Failed to serialize servers: {}", e))?;
    let cached_at = Utc::now().to_rfc3339();

    db.update_custom_registry_cache(id, &cached_data, &cached_at)
        .map_err(|e| format!("Failed to update cache: {}", e))?;

    Ok(FetchResult {
        servers: registry_file.servers,
        from_cache: false,
        cached_at: Some(cached_at),
    })
}

/// Get all custom registries
pub fn get_all_custom_registries(db: &Database) -> Result<Vec<CustomRegistry>, String> {
    db.get_custom_registries()
        .map_err(|e| format!("Database error: {}", e))
}

/// Fetch and parse a registry file from URL or local path
fn fetch_registry_file(url: &str, token: Option<&str>) -> Result<CustomRegistryFile, String> {
    let content = if url.starts_with("http://") || url.starts_with("https://") {
        fetch_remote_registry(url, token)?
    } else {
        fetch_local_registry(url)?
    };

    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid registry JSON: {}", e))
}

/// Fetch registry from remote URL
fn fetch_remote_registry(url: &str, token: Option<&str>) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut request = client.get(url);

    // Add auth header for GitHub URLs if token provided
    if let Some(token) = token {
        if url.contains("github.com") || url.contains("raw.githubusercontent.com") {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
    }

    request = request.header("User-Agent", "MCP-Hub");

    let response = request
        .send()
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 || response.status() == 403 {
        return Err("Authentication failed. Check your GitHub token.".to_string());
    }

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))
}

/// Fetch registry from local file
fn fetch_local_registry(path: &str) -> Result<String, String> {
    // Handle file:// prefix
    let path = path.strip_prefix("file://").unwrap_or(path);

    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Get token for a registry from keychain
fn get_registry_token(id: &str) -> Result<String, String> {
    let credential_key = format!("custom-registry:{}", id);
    credentials::get_credential(&credential_key)
}

/// Test a registry URL without saving it
pub fn test_registry_url(url: &str, token: Option<&str>) -> Result<CustomRegistryFile, String> {
    fetch_registry_file(url, token)
}
```

**Step 2: Export the module in services/mod.rs**

Add to `src-tauri/src/services/mod.rs`:

```rust
pub mod custom_registry;
```

**Step 3: Verify by running cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src-tauri/src/services/custom_registry.rs src-tauri/src/services/mod.rs
git commit -m "feat: Add custom registry service with fetch, cache, and CRUD"
```

---

## Task 4: Tauri Commands

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`

**Step 1: Add imports**

At the top of the file, add:

```rust
use crate::services::custom_registry;
```

**Step 2: Add add_custom_registry command**

```rust
#[tauri::command]
pub fn add_custom_registry(
    state: State<AppState>,
    url: String,
    name: Option<String>,
    token: Option<String>,
) -> Result<crate::db::CustomRegistry, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    custom_registry::add_custom_registry(
        &db,
        &url,
        name.as_deref(),
        token.as_deref(),
    )
}
```

**Step 3: Add update_custom_registry command**

```rust
#[tauri::command]
pub fn update_custom_registry(
    state: State<AppState>,
    id: String,
    url: Option<String>,
    name: Option<String>,
    token: Option<String>,
) -> Result<crate::db::CustomRegistry, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    custom_registry::update_custom_registry(
        &db,
        &id,
        url.as_deref(),
        name.as_deref(),
        token.as_deref(),
    )
}
```

**Step 4: Add delete_custom_registry command**

```rust
#[tauri::command]
pub fn delete_custom_registry(
    state: State<AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    custom_registry::delete_custom_registry(&db, &id)
}
```

**Step 5: Add get_custom_registries command**

```rust
#[tauri::command]
pub fn get_custom_registries(
    state: State<AppState>,
) -> Result<Vec<crate::db::CustomRegistry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    custom_registry::get_all_custom_registries(&db)
}
```

**Step 6: Add fetch_custom_registry_servers command**

```rust
#[tauri::command]
pub fn fetch_custom_registry_servers(
    state: State<AppState>,
    id: String,
    force_refresh: bool,
) -> Result<custom_registry::FetchResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    custom_registry::fetch_custom_registry_servers(&db, &id, force_refresh)
}
```

**Step 7: Add test_custom_registry_url command**

```rust
#[tauri::command]
pub fn test_custom_registry_url(
    url: String,
    token: Option<String>,
) -> Result<custom_registry::CustomRegistryFile, String> {
    custom_registry::test_registry_url(&url, token.as_deref())
}
```

**Step 8: Register commands in main.rs**

In `src-tauri/src/main.rs`, add the new commands to the `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::add_custom_registry,
    commands::update_custom_registry,
    commands::delete_custom_registry,
    commands::get_custom_registries,
    commands::fetch_custom_registry_servers,
    commands::test_custom_registry_url,
])
```

**Step 9: Verify by running cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 10: Commit**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/main.rs
git commit -m "feat: Add Tauri commands for custom registry management"
```

---

## Task 5: Integrate Custom Registries with Existing Registry Commands

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/services/registry.rs`

**Step 1: Add isCustom field to RegistrySource**

In `src-tauri/src/services/registry.rs`, modify the `RegistrySource` struct:

```rust
pub struct RegistrySource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: Option<String>,
    pub server_count: Option<usize>,
    pub is_custom: bool,  // Add this field
}
```

**Step 2: Update get_available_registries to set is_custom: false**

In the `get_available_registries` function, add `is_custom: false` to each registry entry.

**Step 3: Modify get_registries command to include custom registries**

In `commands/mod.rs`, update the `get_registries` command:

```rust
#[tauri::command]
pub fn get_registries(state: State<AppState>) -> Result<Vec<services::registry::RegistrySource>, String> {
    let mut registries = services::registry::get_available_registries();

    // Add custom registries
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let custom = custom_registry::get_all_custom_registries(&db)?;

    for reg in custom {
        // Parse cached data to get server count
        let server_count = reg.cached_data.as_ref()
            .and_then(|data| serde_json::from_str::<Vec<services::registry::RegistryServer>>(data).ok())
            .map(|servers| servers.len());

        registries.push(services::registry::RegistrySource {
            id: reg.id,
            name: reg.name,
            description: reg.description.unwrap_or_default(),
            url: reg.url,
            icon: reg.icon,
            server_count,
            is_custom: true,
        });
    }

    Ok(registries)
}
```

**Step 4: Modify get_registry_servers to handle custom registries**

Update the `get_registry_servers` command to route custom registry IDs:

```rust
#[tauri::command]
pub async fn get_registry_servers(
    state: State<'_, AppState>,
    registry_id: String,
) -> Result<Vec<services::registry::RegistryServer>, String> {
    // Check if this is a custom registry (UUID format)
    if registry_id.contains('-') && registry_id.len() == 36 {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let result = custom_registry::fetch_custom_registry_servers(&db, &registry_id, false)?;
        return Ok(result.servers);
    }

    // Existing built-in registry handling
    services::registry::fetch_registry_servers(&registry_id).await
}
```

**Step 5: Verify by running cargo check**

Run: `cargo check`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/services/registry.rs
git commit -m "feat: Integrate custom registries with existing registry commands"
```

---

## Task 6: Frontend Types

**Files:**
- Modify: `src/types/index.ts`

**Step 1: Add CustomRegistry type**

```typescript
export interface CustomRegistry {
  id: string;
  name: string;
  url: string;
  description?: string;
  icon?: string;
  requiresAuth: boolean;
  cachedAt?: string;
  createdAt: string;
}
```

**Step 2: Add isCustom to RegistrySource**

Update the existing `RegistrySource` interface:

```typescript
export interface RegistrySource {
  id: string;
  name: string;
  description: string;
  url: string;
  icon?: string;
  serverCount?: number;
  isCustom?: boolean;  // Add this field
}
```

**Step 3: Add CustomRegistryFile type for test results**

```typescript
export interface CustomRegistryFile {
  name: string;
  description?: string;
  icon?: string;
  servers: RegistryServer[];
}

export interface FetchResult {
  servers: RegistryServer[];
  fromCache: boolean;
  cachedAt?: string;
}
```

**Step 4: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: Add TypeScript types for custom registries"
```

---

## Task 7: Zustand Store Actions

**Files:**
- Modify: `src/store/index.ts`

**Step 1: Add custom registry actions to store**

Add these actions to the store interface and implementation:

```typescript
// In the store interface
addCustomRegistry: (url: string, name?: string, token?: string) => Promise<CustomRegistry>;
updateCustomRegistry: (id: string, url?: string, name?: string, token?: string) => Promise<CustomRegistry>;
deleteCustomRegistry: (id: string) => Promise<void>;
testCustomRegistryUrl: (url: string, token?: string) => Promise<CustomRegistryFile>;
refreshCustomRegistry: (id: string) => Promise<FetchResult>;
```

```typescript
// Implementation
addCustomRegistry: async (url, name, token) => {
  return invoke<CustomRegistry>("add_custom_registry", { url, name, token });
},

updateCustomRegistry: async (id, url, name, token) => {
  return invoke<CustomRegistry>("update_custom_registry", { id, url, name, token });
},

deleteCustomRegistry: async (id) => {
  return invoke("delete_custom_registry", { id });
},

testCustomRegistryUrl: async (url, token) => {
  return invoke<CustomRegistryFile>("test_custom_registry_url", { url, token });
},

refreshCustomRegistry: async (id) => {
  return invoke<FetchResult>("fetch_custom_registry_servers", { id, forceRefresh: true });
},
```

**Step 2: Commit**

```bash
git add src/store/index.ts
git commit -m "feat: Add Zustand store actions for custom registries"
```

---

## Task 8: Add Registry Dialog Component

**Files:**
- Create: `src/components/AddRegistryDialog.tsx`

**Step 1: Create the dialog component**

```typescript
import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useStore } from "@/store";
import { Loader2, CheckCircle, AlertCircle, Eye, EyeOff } from "lucide-react";
import type { CustomRegistry, CustomRegistryFile } from "@/types";

interface AddRegistryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editRegistry?: CustomRegistry;
  onSuccess?: () => void;
}

export function AddRegistryDialog({
  open,
  onOpenChange,
  editRegistry,
  onSuccess,
}: AddRegistryDialogProps) {
  const { addCustomRegistry, updateCustomRegistry, testCustomRegistryUrl } = useStore();

  const [url, setUrl] = useState(editRegistry?.url || "");
  const [name, setName] = useState(editRegistry?.name || "");
  const [token, setToken] = useState("");
  const [showToken, setShowToken] = useState(false);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testResult, setTestResult] = useState<CustomRegistryFile | null>(null);
  const [error, setError] = useState<string | null>(null);

  const isGitHubUrl = url.includes("github.com") || url.includes("githubusercontent.com");
  const isEditing = !!editRegistry;

  const handleTest = async () => {
    setTesting(true);
    setError(null);
    setTestResult(null);

    try {
      const result = await testCustomRegistryUrl(url, token || undefined);
      setTestResult(result);
      if (!name && result.name) {
        setName(result.name);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);

    try {
      if (isEditing) {
        await updateCustomRegistry(
          editRegistry.id,
          url !== editRegistry.url ? url : undefined,
          name !== editRegistry.name ? name : undefined,
          token || undefined
        );
      } else {
        await addCustomRegistry(url, name || undefined, token || undefined);
      }
      onSuccess?.();
      onOpenChange(false);
      resetForm();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const resetForm = () => {
    setUrl("");
    setName("");
    setToken("");
    setTestResult(null);
    setError(null);
  };

  const handleClose = (open: boolean) => {
    if (!open) {
      resetForm();
    }
    onOpenChange(open);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit" : "Add"} Custom Registry</DialogTitle>
          <DialogDescription>
            {isEditing
              ? "Update your custom registry settings"
              : "Add a registry from a local file or remote URL"}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="url">Registry URL</Label>
            <Input
              id="url"
              placeholder="https://raw.githubusercontent.com/... or /path/to/registry.json"
              value={url}
              onChange={(e) => {
                setUrl(e.target.value);
                setTestResult(null);
              }}
            />
          </div>

          {isGitHubUrl && (
            <div className="space-y-2">
              <Label htmlFor="token">GitHub Token (optional)</Label>
              <div className="relative">
                <Input
                  id="token"
                  type={showToken ? "text" : "password"}
                  placeholder="ghp_xxxx (for private repos)"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowToken(!showToken)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  {showToken ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              <p className="text-xs text-muted-foreground">
                Required for private repositories. Token stored securely in your system keychain.
              </p>
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="name">Name Override (optional)</Label>
            <Input
              id="name"
              placeholder="Will use name from registry file if not set"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          {error && (
            <div className="flex items-center gap-2 p-3 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
              <AlertCircle className="h-4 w-4 flex-shrink-0" />
              {error}
            </div>
          )}

          {testResult && (
            <div className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-md text-green-600 dark:text-green-400 text-sm">
              <CheckCircle className="h-4 w-4 flex-shrink-0" />
              Found {testResult.servers.length} server(s) in "{testResult.name}"
            </div>
          )}
        </div>

        <DialogFooter className="flex-col sm:flex-row gap-2">
          <Button
            variant="outline"
            onClick={handleTest}
            disabled={!url || testing || saving}
          >
            {testing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Test Connection
          </Button>
          <Button
            onClick={handleSave}
            disabled={!url || !testResult || saving}
          >
            {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {isEditing ? "Save Changes" : "Add Registry"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

**Step 2: Commit**

```bash
git add src/components/AddRegistryDialog.tsx
git commit -m "feat: Add AddRegistryDialog component for managing custom registries"
```

---

## Task 9: Update ImportDialog with Custom Registry UI

**Files:**
- Modify: `src/components/ImportDialog.tsx`

**Step 1: Add imports and state**

Add to imports:

```typescript
import { AddRegistryDialog } from "./AddRegistryDialog";
import { Plus, RefreshCw, Settings, Trash2 } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
```

Add state inside the component:

```typescript
const [showAddRegistry, setShowAddRegistry] = useState(false);
const [editingRegistry, setEditingRegistry] = useState<CustomRegistry | undefined>();
const [refreshing, setRefreshing] = useState(false);

const { refreshCustomRegistry, deleteCustomRegistry } = useStore();
```

**Step 2: Add refresh handler**

```typescript
const handleRefreshRegistry = async () => {
  if (!currentRegistry?.isCustom) return;

  setRefreshing(true);
  try {
    await refreshCustomRegistry(selectedRegistry);
    await loadRegistryServers(selectedRegistry);
  } catch (err) {
    setError(err instanceof Error ? err.message : "Failed to refresh");
  } finally {
    setRefreshing(false);
  }
};
```

**Step 3: Add delete handler**

```typescript
const handleDeleteRegistry = async () => {
  if (!currentRegistry?.isCustom) return;

  try {
    await deleteCustomRegistry(selectedRegistry);
    await loadRegistries();
    setSelectedRegistry("builtin");
  } catch (err) {
    setError(err instanceof Error ? err.message : "Failed to delete");
  }
};
```

**Step 4: Update Registry Selector section**

Replace the registry selector div with:

```typescript
{/* Registry Selector */}
<div className="mb-4">
  <div className="flex items-center gap-2">
    <Select
      value={selectedRegistry}
      onValueChange={setSelectedRegistry}
      disabled={loadingRegistries}
    >
      <SelectTrigger className="flex-1">
        <SelectValue placeholder="Select a registry" />
      </SelectTrigger>
      <SelectContent>
        {registries.filter(r => !r.isCustom).map((registry) => (
          <SelectItem key={registry.id} value={registry.id}>
            <div className="flex items-center gap-2">
              {getRegistryIcon(registry.icon)}
              <span>{registry.name}</span>
              {registry.serverCount && (
                <span className="text-muted-foreground text-xs">
                  ({registry.serverCount}+ servers)
                </span>
              )}
            </div>
          </SelectItem>
        ))}
        {registries.some(r => r.isCustom) && (
          <>
            <div className="px-2 py-1.5 text-xs text-muted-foreground border-t mt-1 pt-2">
              Custom Registries
            </div>
            {registries.filter(r => r.isCustom).map((registry) => (
              <SelectItem key={registry.id} value={registry.id}>
                <div className="flex items-center gap-2">
                  <Star className="h-4 w-4 text-yellow-500" />
                  <span>{registry.name}</span>
                  {registry.serverCount !== undefined && (
                    <span className="text-muted-foreground text-xs">
                      ({registry.serverCount} servers)
                    </span>
                  )}
                </div>
              </SelectItem>
            ))}
          </>
        )}
      </SelectContent>
    </Select>

    <Button
      variant="outline"
      size="icon"
      onClick={() => setShowAddRegistry(true)}
      title="Add custom registry"
    >
      <Plus className="h-4 w-4" />
    </Button>

    {currentRegistry?.isCustom && (
      <>
        <Button
          variant="outline"
          size="icon"
          onClick={handleRefreshRegistry}
          disabled={refreshing}
          title="Refresh registry"
        >
          <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
        </Button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="icon" title="Registry settings">
              <Settings className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => {
              setEditingRegistry(currentRegistry as unknown as CustomRegistry);
              setShowAddRegistry(true);
            }}>
              Edit registry
            </DropdownMenuItem>
            <DropdownMenuItem
              className="text-destructive"
              onClick={handleDeleteRegistry}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Delete registry
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </>
    )}
  </div>

  {currentRegistry && (
    <p className="text-xs text-muted-foreground mt-1.5 px-1">
      {currentRegistry.description}
    </p>
  )}
</div>
```

**Step 5: Add dialog at the end**

Before the closing `</Dialog>`, add:

```typescript
<AddRegistryDialog
  open={showAddRegistry}
  onOpenChange={(open) => {
    setShowAddRegistry(open);
    if (!open) setEditingRegistry(undefined);
  }}
  editRegistry={editingRegistry}
  onSuccess={loadRegistries}
/>
```

**Step 6: Verify by running build**

Run: `pnpm build`
Expected: Build succeeds

**Step 7: Commit**

```bash
git add src/components/ImportDialog.tsx
git commit -m "feat: Add custom registry management UI to ImportDialog"
```

---

## Task 10: Final Integration Test

**Step 1: Start the application**

Run: `pnpm tauri dev`

**Step 2: Manual test checklist**

- [ ] Open Import dialog
- [ ] Click "+" button - Add Registry dialog opens
- [ ] Enter a test URL (create a test JSON file locally)
- [ ] Click "Test Connection" - should show server count
- [ ] Click "Add Registry" - dialog closes
- [ ] New registry appears in dropdown under "Custom Registries"
- [ ] Select custom registry - servers load
- [ ] Click refresh button - servers reload
- [ ] Click gear > Edit - edit dialog opens
- [ ] Click gear > Delete - registry removed

**Step 3: Create example registry file for testing**

Create `~/test-registry.json`:

```json
{
  "name": "Test Registry",
  "description": "A test custom registry",
  "servers": [
    {
      "name": "test-server",
      "description": "A test MCP server",
      "command": "npx",
      "args": ["test-mcp-server"],
      "tags": ["test"]
    }
  ]
}
```

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: Complete custom registries feature implementation"
```

---

## Summary

This plan implements custom registries in 10 tasks:

1. Database schema
2. Database CRUD operations
3. Custom registry service (fetch, cache, auth)
4. Tauri commands
5. Integration with existing registry commands
6. Frontend types
7. Zustand store actions
8. AddRegistryDialog component
9. ImportDialog UI updates
10. Integration testing

Each task is independently committable and builds on the previous one.
