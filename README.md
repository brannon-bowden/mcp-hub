# MCP Hub

A cross-platform desktop application for centralized management of Model Context Protocol (MCP) servers. Configure MCP servers once and deploy configurations to multiple AI client applications.

## Features

- **Central Server Registry**: Manage all your MCP server configurations in one place
- **Multi-Client Support**: Deploy to Claude Desktop, Claude Code, Cursor, and more
- **Client Instances**: Create multiple profiles for each client (e.g., "Work" and "Personal")
- **Server-to-Instance Mapping**: Selectively enable servers for each client instance
- **Configuration Sync**: One-click sync to update client configuration files
- **Automatic Backups**: Creates backups before modifying any config files
- **Secure Credential Storage**: Uses OS-native secure storage (Keychain, Credential Manager)
- **Auto-Detection**: Automatically detects installed MCP clients

## Supported Clients

| Client | macOS | Windows | Linux |
|--------|-------|---------|-------|
| Claude Desktop | Yes | Yes | Yes |
| Claude Code | Yes | Yes | Yes |
| Cursor (Cline) | Yes | Yes | Yes |
| Custom | Yes | Yes | Yes |

## Installation

### Pre-built Binaries

Download the latest release for your platform:

- **macOS**: `MCP Hub_x.x.x_aarch64.dmg` (Apple Silicon) or `MCP Hub_x.x.x_x64.dmg` (Intel)
- **Windows**: `MCP Hub_x.x.x_x64-setup.exe`
- **Linux**: `MCP Hub_x.x.x_amd64.AppImage` or `.deb`

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- [pnpm](https://pnpm.io/)

#### Steps

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-hub.git
cd mcp-hub

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Usage

### Adding a Server

1. Navigate to the **Servers** view
2. Click **Add Server**
3. Fill in the server details:
   - **Name**: Human-readable name
   - **Command**: The executable (e.g., `npx`, `node`, `python`)
   - **Arguments**: Command arguments (one per line)
   - **Environment Variables**: KEY=value pairs (one per line)
   - **Tags**: Comma-separated tags for organization
4. Click **Add Server**

### Setting Up Client Instances

1. Navigate to the **Instances** view
2. Click **Add Instance**
3. Select the client type or use auto-detection
4. Configure the instance name and config path
5. Click **Configure Servers** to select which servers to enable
6. Click **Sync** to write the configuration

### Syncing Configurations

- Click **Sync** on any instance to update its config file
- Use **Sync All** in the sidebar to sync all instances at once
- Backups are automatically created before each sync

## Data Storage

Application data is stored in:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/MCP Hub/` |
| Windows | `%APPDATA%\MCP Hub\` |
| Linux | `~/.config/mcp-hub/` |

## Development

### Project Structure

```
mcp-hub/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Page components
│   ├── store/              # Zustand state management
│   └── types/              # TypeScript types
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── db/             # SQLite database
│       ├── models/         # Data models
│       └── services/       # Business logic
└── ...
```

### Technology Stack

- **Frontend**: React 19, TypeScript, Tailwind CSS v4, Radix UI
- **Backend**: Rust, Tauri 2.0
- **Database**: SQLite (via rusqlite)
- **State Management**: Zustand
- **Secure Storage**: OS keyring (via keyring crate)

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please read the contributing guidelines before submitting a PR.
