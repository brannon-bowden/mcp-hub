//! Stdio Bridge for MCP Hub Proxy
//!
//! This module provides a stdio-to-HTTP bridge that allows Claude Desktop
//! and other MCP clients that only support stdio transport to connect to
//! the MCP Hub proxy server.
//!
//! Usage:
//!   mcp-hub-stdio --url http://localhost:24369/mcp/instance-id
//!
//! The bridge:
//! 1. Reads JSON-RPC messages from stdin (line-delimited)
//! 2. Forwards them to the MCP Hub proxy via HTTP POST
//! 3. Writes responses to stdout
//!
//! Cross-platform compatible: Works on Windows, macOS, and Linux.
//! Uses a dedicated thread for stdin to avoid tokio async stdin issues on Windows.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// Write a response to stdout with proper flushing (blocking, for use in sync context)
fn write_response_sync(response: &str) -> Result<(), std::io::Error> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", response)?;
    stdout.flush()?;
    Ok(())
}

/// Run the stdio bridge, connecting stdin/stdout to an HTTP proxy endpoint.
/// Uses a dedicated thread for stdin reading to ensure cross-platform compatibility,
/// especially on Windows where tokio's async stdin can have issues.
pub async fn run_bridge(proxy_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for long-running tools
        .build()?;

    // Channel for stdin lines from the dedicated thread
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Spawn a dedicated thread for reading stdin (avoids tokio async stdin issues on Windows)
    let stdin_thread = std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let reader = stdin.lock();

        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if tx.blocking_send(l).is_err() {
                        // Receiver dropped, exit
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    break;
                }
            }
        }
    });

    eprintln!("MCP Hub stdio bridge started, connecting to: {}", proxy_url);

    // Process messages from stdin thread
    while let Some(line) = rx.recv().await {
        // Skip empty lines
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse the incoming JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse JSON-RPC request: {}", e);
                let error_response = JsonRpcResponse::error(
                    None,
                    -32700,
                    format!("Parse error: {}", e),
                );
                let response_json = serde_json::to_string(&error_response)?;
                write_response_sync(&response_json)?;
                continue;
            }
        };

        eprintln!("Received request: {} (id: {:?})", request.method, request.id);

        // Forward to HTTP proxy
        let response = match client
            .post(&proxy_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("HTTP request failed: {}", e);
                let error_response = JsonRpcResponse::error(
                    request.id,
                    -32603,
                    format!("Internal error: Failed to connect to proxy: {}", e),
                );
                let response_json = serde_json::to_string(&error_response)?;
                write_response_sync(&response_json)?;
                continue;
            }
        };

        // Get response body
        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Failed to read response body: {}", e);
                let error_response = JsonRpcResponse::error(
                    request.id,
                    -32603,
                    format!("Internal error: Failed to read response: {}", e),
                );
                let response_json = serde_json::to_string(&error_response)?;
                write_response_sync(&response_json)?;
                continue;
            }
        };

        eprintln!("Proxy response: {}", response_text);

        // Write response to stdout (already JSON)
        write_response_sync(&response_text)?;
    }

    // Wait for stdin thread to finish (it will when stdin closes)
    let _ = stdin_thread.join();

    eprintln!("MCP Hub stdio bridge shutting down");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_json_rpc_error_response() {
        let response = JsonRpcResponse::error(
            Some(serde_json::json!(1)),
            -32600,
            "Invalid Request".to_string(),
        );
        assert_eq!(response.error.as_ref().unwrap().code, -32600);
    }
}
