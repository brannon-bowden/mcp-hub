# MCP Hub

A cross-platform desktop application for centralized management of Model Context Protocol (MCP) servers. Configure MCP servers once and deploy configurations to multiple AI client applications.

## Features

### Core Features
- **Central Server Registry**: Manage all your MCP server configurations in one place
- **Multi-Client Support**: Deploy to 40+ AI client applications simultaneously
- **Client Instances**: Create multiple profiles for each client (e.g., "Work" and "Personal")
- **Server Instances**: Create multiple instances of the same server with different configurations
- **Server-to-Instance Mapping**: Selectively enable servers for each client instance
- **Configuration Sync**: One-click sync to update client configuration files
- **Automatic Backups**: Creates backups before modifying any config files

### Server Management
- **Import from File**: Import existing MCP configurations from any client's config file
- **Server Registries**: Browse and install from multiple server registries:
  - MCP Hub Built-in (55+ curated servers)
  - Anthropic Official (19 servers)
  - Awesome MCP Servers (100+ community servers)
  - Smithery Registry (200+ servers)
  - Glama MCP Directory (150+ servers)
  - mcp-get Registry (80+ servers)
- **Environment Variables**: Configure server-specific environment variables
- **Tags**: Organize servers with custom tags for easy filtering

### Security
- **Secure Credential Storage**: Uses OS-native secure storage (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Config File Preservation**: Safely merges MCP configs with existing settings (e.g., Claude Code's `.claude.json`)

### Discovery (Experimental)
- **MCP Directory**: Expose servers via `~/.mcp/` directory for compatible clients
- **HTTP Discovery Server**: Local HTTP server for programmatic server discovery

## Supported Clients

MCP Hub supports 40+ AI clients across all major platforms:

### AI Assistants & Chat
| Client | macOS | Windows | Linux |
|--------|:-----:|:-------:|:-----:|
| Claude Desktop | ✓ | ✓ | ✓ |
| Claude Code | ✓ | ✓ | ✓ |
| Perplexity Desktop | ✓ | ✓ | ✓ |
| BoltAI | ✓ | - | - |

### Code Editors & IDEs
| Client | macOS | Windows | Linux |
|--------|:-----:|:-------:|:-----:|
| VS Code | ✓ | ✓ | ✓ |
| VS Code Insiders | ✓ | ✓ | ✓ |
| Cursor | ✓ | ✓ | ✓ |
| Windsurf | ✓ | ✓ | ✓ |
| Zed | ✓ | ✓ | ✓ |
| Visual Studio 2022 | - | ✓ | - |
| JetBrains IDEs | ✓ | ✓ | ✓ |

### AI Coding Extensions
| Client | macOS | Windows | Linux |
|--------|:-----:|:-------:|:-----:|
| Cline | ✓ | ✓ | ✓ |
| Continue | ✓ | ✓ | ✓ |
| Sourcegraph Cody | ✓ | ✓ | ✓ |
| Roo Code | ✓ | ✓ | ✓ |
| Kilo Code | ✓ | ✓ | ✓ |
| Augment Code | ✓ | ✓ | ✓ |
| Qodo Gen | ✓ | ✓ | ✓ |
| GitHub Copilot | ✓ | ✓ | ✓ |

### CLI Tools
| Client | macOS | Windows | Linux |
|--------|:-----:|:-------:|:-----:|
| Amp | ✓ | ✓ | ✓ |
| Gemini CLI | ✓ | ✓ | ✓ |
| Amazon Q Developer | ✓ | ✓ | ✓ |
| Rovo Dev CLI | ✓ | ✓ | ✓ |
| OpenAI Codex | ✓ | ✓ | ✓ |
| Copilot CLI | ✓ | ✓ | ✓ |

### Other Clients
| Client | macOS | Windows | Linux |
|--------|:-----:|:-------:|:-----:|
| LM Studio | ✓ | ✓ | ✓ |
| Google Antigravity | ✓ | ✓ | ✓ |
| Kiro | ✓ | ✓ | ✓ |
| Smithery | ✓ | ✓ | ✓ |
| Custom | ✓ | ✓ | ✓ |

*And many more...*

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
2. Click **Add Server** or **Import** to add from registries/files
3. Fill in the server details:
   - **Name**: Human-readable name
   - **Command**: The executable (e.g., `npx`, `uvx`, `node`, `python`)
   - **Arguments**: Command arguments (one per line)
   - **Environment Variables**: KEY=value pairs (one per line)
   - **Tags**: Comma-separated tags for organization
4. Click **Add Server**

### Creating Server Instances

To run multiple instances of the same server with different configurations:

1. Find the server in the **Servers** view
2. Click the **Copy** icon to create an instance
3. Modify the name, arguments, or environment variables as needed
4. Click **Add Server** to create the instance

Instances are visually grouped with their parent server and display an "instance" badge.

### Importing from Registries

1. Click **Import** in the Servers view
2. Select a registry from the available sources
3. Browse or search for servers
4. Select servers to import
5. Click **Import Selected**

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

### Database

MCP Hub uses SQLite to store:
- Server configurations
- Client instances
- Server-to-instance mappings
- Backup records
- Application settings

## Development

### Project Structure

```
mcp-hub/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Page components
│   │   ├── Dashboard.tsx   # Overview dashboard
│   │   ├── Servers.tsx     # Server management
│   │   ├── Instances.tsx   # Client instance management
│   │   └── Settings.tsx    # Application settings
│   ├── store/              # Zustand state management
│   └── types/              # TypeScript types
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── db/             # SQLite database
│       ├── models/         # Data models
│       └── services/
│           ├── config.rs       # Config file management
│           ├── credentials.rs  # Secure credential storage
│           ├── discovery.rs    # MCP discovery features
│           └── registry.rs     # Server registry integration
└── ...
```

### Technology Stack

- **Frontend**: React 19, TypeScript, Tailwind CSS v4, Radix UI
- **Backend**: Rust, Tauri 2.0
- **Database**: SQLite (via rusqlite)
- **State Management**: Zustand
- **Secure Storage**: OS keyring (via keyring crate)

### Key Commands

```bash
# Development
pnpm tauri dev

# Build
pnpm tauri build

# Type checking
pnpm build       # Frontend only
cargo check      # Backend only
```

## Screenshots

*Coming soon*

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please read the contributing guidelines before submitting a PR.

## Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io) by Anthropic
- [Tauri](https://tauri.app) for the desktop framework
- All the MCP server authors whose work is cataloged in the registries
