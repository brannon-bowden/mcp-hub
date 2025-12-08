//! MCP Hub Stdio Bridge CLI
//!
//! This binary provides a stdio-to-HTTP bridge for the MCP Hub proxy,
//! allowing Claude Desktop and other stdio-only MCP clients to connect
//! to the MCP Hub proxy server.
//!
//! # Usage
//!
//! ```bash
//! mcp-hub-stdio --url http://localhost:24369/mcp/your-instance-id
//! ```
//!
//! # Claude Desktop Configuration
//!
//! Add to your `claude_desktop_config.json`:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "mcp-hub": {
//!       "command": "/path/to/mcp-hub-stdio",
//!       "args": ["--url", "http://localhost:24369/mcp/your-instance-id"]
//!     }
//!   }
//! }
//! ```

use clap::Parser;
use mcp_hub_lib::services::stdio_bridge;

#[derive(Parser, Debug)]
#[command(name = "mcp-hub-stdio")]
#[command(author = "MCP Hub Team")]
#[command(version = "0.1.0")]
#[command(about = "Stdio bridge for MCP Hub proxy - connects stdio MCP clients to the HTTP proxy")]
#[command(long_about = r#"
MCP Hub Stdio Bridge

This tool bridges stdio-based MCP clients (like Claude Desktop) to the MCP Hub
HTTP proxy server. It reads JSON-RPC messages from stdin, forwards them to the
proxy via HTTP, and writes responses to stdout.

USAGE:
    1. Start MCP Hub and enable the proxy server
    2. Register an instance to get its proxy URL
    3. Run this bridge with the proxy URL
    4. Configure Claude Desktop to use this bridge

EXAMPLE:
    mcp-hub-stdio --url http://localhost:24369/mcp/instance-abc123
"#)]
struct Args {
    /// The MCP Hub proxy URL (e.g., http://localhost:24369/mcp/instance-id)
    #[arg(short, long)]
    url: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logger
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Validate URL
    if !args.url.starts_with("http://") && !args.url.starts_with("https://") {
        eprintln!("Error: URL must start with http:// or https://");
        std::process::exit(1);
    }

    // Run the bridge
    if let Err(e) = stdio_bridge::run_bridge(args.url).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
