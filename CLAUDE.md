# MCP Hub - Claude Code Instructions

## Project Overview

MCP Hub is a cross-platform desktop application for centralized management of Model Context Protocol (MCP) servers. Built with:

- **Frontend**: React 19 / TypeScript / Vite in `/src`
- **Backend**: Rust / Tauri 2.0 in `/src-tauri`

## Development Workflow

### For Each New Task

1. **Create a GitHub Issue**

   - Create an issue describing the task/feature/bug
   - Include acceptance criteria
   - Add appropriate labels (feature, bug, enhancement, etc.)

2. **Create a Feature Branch**

   - Branch from `main` (not `development`)
   - Branch naming: `feature/<issue-number>-<short-description>` or `fix/<issue-number>-<short-description>`
   - Example: `feature/42-server-groups` or `fix/43-config-sync-error`

3. **Development**

   - Make commits with clear, descriptive messages
   - Reference the issue number in commits when relevant
   - Run builds before creating PR

4. **Create a Pull Request**

   - Target branch: `main`
   - Title should be descriptive and reference the issue
   - PR body must include:
     - **Summary**: What changes were made and why
     - **Testing Completed**: List of manual tests performed
     - **Screenshots** (if UI changes)
     - Closes #<issue-number>
   - Request review when ready

5. **Merge**
   - Ensure CI passes (builds for macOS, Windows, Linux)
   - Get approval
   - Squash and merge preferred

## Build Commands

### Frontend (`/` root)

```bash
pnpm install           # Install dependencies
pnpm dev               # Start Vite dev server (port 1420)
pnpm build             # TypeScript check + Vite production build
pnpm preview           # Preview built frontend
```

### Full Application

```bash
pnpm tauri dev         # Run full app in development with hot reload
pnpm tauri build       # Build production binaries (.dmg, .msi, etc.)
```

### Rust Backend Only (`/src-tauri`)

```bash
cargo check            # Type check Rust code
cargo build            # Build Rust backend
cargo build --release  # Build optimized release
```

Note: Ensure Rust is in PATH. You may need:
```bash
PATH="$HOME/.cargo/bin:$PATH" pnpm tauri dev
```

## Code Style

- TypeScript for frontend, Rust for backend
- Tailwind CSS for styling
- Prefer editing existing files over creating new ones
- Keep changes focused and minimal

## Key Directories

### Frontend (`/src`)

- `pages/` - Main UI pages (Dashboard, Servers, Instances, Settings)
- `components/` - React components
- `components/ui/` - Radix UI wrapper components
- `store/` - Zustand state management
- `types/` - TypeScript type definitions
- `hooks/` - Custom React hooks
- `lib/` - Utility functions

### Backend (`/src-tauri/src`)

- `commands/` - Tauri IPC command handlers (CRUD, sync, discovery)
- `services/` - Core business logic:
  - `config.rs` - Client config file management
  - `registry.rs` - Server registry integration
  - `proxy.rs` - MCP proxy server
  - `discovery.rs` - MCP directory and HTTP discovery
  - `credentials.rs` - OS keyring integration
  - `stdio_bridge.rs` - Stdio bridge for Claude Desktop
- `db/` - SQLite database layer
- `models/` - Data structures
- `bin/` - Executable entry points

## Architecture

```
React Frontend (Vite)
        ↓
  Zustand Store
        ↓
  Tauri Commands (IPC)
        ↓
  Rust Backend
        ↓
  Services Layer
        ↓
  SQLite Database
```

### Database Tables

- `servers` - MCP server definitions
- `client_instances` - Configured client applications
- `instance_servers` - Server-to-instance mapping (many-to-many)
- `backups` - Config file backups
- `settings` - Application settings

## Data Storage

- **macOS**: `~/Library/Application Support/com.mcp-hub.app/`
- **Windows**: `%APPDATA%\com.mcp-hub.app\`
- **Linux**: `~/.local/share/com.mcp-hub.app/`

## Testing Guidelines

### Before Creating a PR

1. **Frontend**: Run `pnpm build` - must compile without TypeScript errors
2. **Backend**: Run `cargo check` in `/src-tauri` - must compile without errors
3. **Full Build**: Run `pnpm tauri build` to verify release build works
4. **Manual Testing**: Test affected features in the running app

### Testing Checklist for PRs

- [ ] Frontend builds successfully (`pnpm build`)
- [ ] Rust compiles successfully (`cargo check`)
- [ ] Application runs (`pnpm tauri dev`)
- [ ] Tested affected features manually
- [ ] No regressions in existing functionality

## IPC Commands

Commands are defined in `src-tauri/src/commands/mod.rs` and invoked from the frontend via `@tauri-apps/api`:

```typescript
import { invoke } from "@tauri-apps/api/core";

// Example usage
const servers = await invoke<McpServer[]>("get_servers");
await invoke("create_server", { server: newServer });
```

Key command categories:
- Server CRUD: `get_servers`, `create_server`, `update_server`, `delete_server`
- Instance CRUD: `get_instances`, `create_instance`, `update_instance`
- Mapping: `add_server_to_instance`, `remove_server_from_instance`
- Config: `sync_instance_config`, `get_instance_config`
- Discovery: `start_discovery_server`, `stop_discovery_server`
- Proxy: `start_proxy`, `stop_proxy`, `get_proxy_status`

## Supported Clients

MCP Hub supports 34+ client applications including:
- Claude Desktop, Claude Code
- VS Code, Cursor, Windsurf
- JetBrains IDEs (IntelliJ, WebStorm, etc.)
- Zed, Continue, Cline
- And many more

See README.md for the full support matrix.

## Repository

- Owner: `brannon-bowden`
- Repo: `mcp-hub`
- Main branch: `main`
