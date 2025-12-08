#!/usr/bin/env node

/**
 * MCP Hub Stdio Bridge
 *
 * This script provides a stdio-to-HTTP bridge for the MCP Hub proxy,
 * allowing Claude Desktop and other stdio-only MCP clients to connect
 * to the MCP Hub proxy server.
 *
 * Usage:
 *   node mcp-hub-stdio.mjs --url http://localhost:24369/mcp/your-instance-id
 *
 * Or if installed globally / via npx:
 *   mcp-hub-stdio --url http://localhost:24369/mcp/your-instance-id
 *
 * Claude Desktop Configuration:
 *   Add to your claude_desktop_config.json:
 *   {
 *     "mcpServers": {
 *       "mcp-hub": {
 *         "command": "node",
 *         "args": ["/path/to/mcp-hub-stdio.mjs", "--url", "http://localhost:24369/mcp/your-instance-id"]
 *       }
 *     }
 *   }
 */

import { createInterface } from "readline";

// Parse command line arguments
function parseArgs() {
  const args = process.argv.slice(2);
  let url = null;
  let verbose = false;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--url" || args[i] === "-u") {
      url = args[i + 1];
      i++;
    } else if (args[i] === "--verbose" || args[i] === "-v") {
      verbose = true;
    } else if (args[i] === "--help" || args[i] === "-h") {
      console.error(`
MCP Hub Stdio Bridge

This tool bridges stdio-based MCP clients (like Claude Desktop) to the MCP Hub
HTTP proxy server. It reads JSON-RPC messages from stdin, forwards them to the
proxy via HTTP, and writes responses to stdout.

USAGE:
    node mcp-hub-stdio.mjs --url <proxy-url> [--verbose]

OPTIONS:
    -u, --url <url>     The MCP Hub proxy URL (required)
                        Example: http://localhost:24369/mcp/instance-id
    -v, --verbose       Enable verbose logging to stderr
    -h, --help          Show this help message

EXAMPLE:
    node mcp-hub-stdio.mjs --url http://localhost:24369/mcp/instance-abc123

CLAUDE DESKTOP CONFIGURATION:
    Add to your claude_desktop_config.json:
    {
      "mcpServers": {
        "mcp-hub": {
          "command": "node",
          "args": ["/path/to/mcp-hub-stdio.mjs", "--url", "http://localhost:24369/mcp/your-instance-id"]
        }
      }
    }
`);
      process.exit(0);
    }
  }

  return { url, verbose };
}

// Create a JSON-RPC error response
function createErrorResponse(id, code, message) {
  return {
    jsonrpc: "2.0",
    id: id ?? null,
    error: {
      code,
      message,
    },
  };
}

// Write response to stdout
function writeResponse(response) {
  const json = typeof response === "string" ? response : JSON.stringify(response);
  process.stdout.write(json + "\n");
}

// Log to stderr (doesn't interfere with MCP protocol on stdout)
function log(message, verbose, forceLog = false) {
  if (verbose || forceLog) {
    console.error(`[mcp-hub-stdio] ${message}`);
  }
}

async function main() {
  const { url, verbose } = parseArgs();

  if (!url) {
    console.error("Error: --url is required");
    console.error("Usage: node mcp-hub-stdio.mjs --url http://localhost:24369/mcp/instance-id");
    process.exit(1);
  }

  if (!url.startsWith("http://") && !url.startsWith("https://")) {
    console.error("Error: URL must start with http:// or https://");
    process.exit(1);
  }

  log(`MCP Hub stdio bridge started, connecting to: ${url}`, verbose, true);

  // Create readline interface for stdin
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  });

  // Process each line from stdin
  rl.on("line", async (line) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    let request;
    try {
      request = JSON.parse(trimmed);
    } catch (e) {
      log(`Failed to parse JSON-RPC request: ${e.message}`, verbose);
      writeResponse(createErrorResponse(null, -32700, `Parse error: ${e.message}`));
      return;
    }

    log(`Received request: ${request.method} (id: ${request.id})`, verbose);

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(request),
      });

      const responseText = await response.text();
      log(`Proxy response: ${responseText}`, verbose);

      // Write response directly (it's already JSON from the proxy)
      writeResponse(responseText);
    } catch (e) {
      log(`HTTP request failed: ${e.message}`, verbose);
      writeResponse(
        createErrorResponse(
          request.id,
          -32603,
          `Internal error: Failed to connect to proxy: ${e.message}`
        )
      );
    }
  });

  rl.on("close", () => {
    log("MCP Hub stdio bridge shutting down", verbose, true);
    process.exit(0);
  });

  // Handle errors
  rl.on("error", (err) => {
    log(`Readline error: ${err.message}`, verbose, true);
    process.exit(1);
  });
}

main().catch((err) => {
  console.error(`Fatal error: ${err.message}`);
  process.exit(1);
});
