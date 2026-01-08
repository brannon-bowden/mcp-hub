//! Command validation for MCP server spawning
//!
//! Provides security controls to prevent command injection and execution of
//! dangerous executables when spawning MCP servers.

use std::collections::HashSet;

/// Allowed commands for MCP servers
/// These are the standard package managers and runtimes used by MCP servers
const ALLOWED_COMMANDS: &[&str] = &[
    // Node.js package managers
    "npx",
    "npm",
    "node",
    "pnpm",
    "yarn",
    "bunx",
    "bun",
    // Python package managers
    "uvx",
    "uv",
    "python",
    "python3",
    "pip",
    "pipx",
    // Docker (for containerized MCP servers)
    "docker",
    // Direct executables (platform specific)
    "mcp-server",
];

/// Blocked argument patterns that could indicate command injection
const BLOCKED_ARG_PATTERNS: &[&str] = &[
    // Shell metacharacters
    ";",
    "&&",
    "||",
    "|",
    "`",
    "$(",
    "${",
    // Redirections
    ">",
    "<",
    ">>",
    // Background execution
    "&",
    // Common shell commands that shouldn't appear in MCP args
    "/bin/sh",
    "/bin/bash",
    "/bin/zsh",
    "cmd.exe",
    "powershell",
    "pwsh",
];

/// Blocked environment variable names
/// These could be used to hijack command execution
const BLOCKED_ENV_VARS: &[&str] = &[
    // Path hijacking
    "PATH",
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    // Shell execution
    "SHELL",
    "COMSPEC",
    // Python path hijacking
    "PYTHONPATH",
    "PYTHONHOME",
    // Node.js path hijacking
    "NODE_PATH",
    "NODE_OPTIONS",
    // Credential/key variables (shouldn't be blocked for MCP servers - they need these)
    // But we should warn if they contain shell metacharacters
];

/// Result of command validation
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, msg: String) {
        self.is_valid = false;
        self.errors.push(msg);
    }

    fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }
}

/// Validate an MCP server command for security
///
/// Returns Ok(()) if the command is safe, Err with details if not
pub fn validate_mcp_command(
    command: &str,
    args: &[String],
    env: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let result = validate_mcp_command_detailed(command, args, env);

    if result.is_valid {
        // Log warnings if any
        for warning in &result.warnings {
            log::warn!("MCP server command warning: {}", warning);
        }
        Ok(())
    } else {
        Err(result.errors.join("; "))
    }
}

/// Validate an MCP server command with detailed results
pub fn validate_mcp_command_detailed(
    command: &str,
    args: &[String],
    env: &std::collections::HashMap<String, String>,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Validate command
    validate_command(&mut result, command);

    // Validate arguments
    validate_args(&mut result, args);

    // Validate environment variables
    validate_env(&mut result, env);

    result
}

fn validate_command(result: &mut ValidationResult, command: &str) {
    // Normalize command - get just the executable name
    let cmd_name = std::path::Path::new(command)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(command)
        .to_lowercase();

    // Remove .exe suffix for Windows compatibility
    let cmd_name = cmd_name.strip_suffix(".exe").unwrap_or(&cmd_name);

    // Check against allowlist
    let allowed_set: HashSet<&str> = ALLOWED_COMMANDS.iter().cloned().collect();
    if !allowed_set.contains(cmd_name) {
        result.add_error(format!(
            "Command '{}' is not in the allowed list. Allowed commands: {}",
            command,
            ALLOWED_COMMANDS.join(", ")
        ));
    }

    // Check for path traversal attempts
    if command.contains("..") {
        result.add_error(format!(
            "Command '{}' contains path traversal (..) which is not allowed",
            command
        ));
    }

    // Check for shell metacharacters in command itself
    for pattern in BLOCKED_ARG_PATTERNS {
        if command.contains(pattern) {
            result.add_error(format!(
                "Command '{}' contains blocked pattern '{}' which could indicate command injection",
                command, pattern
            ));
        }
    }
}

fn validate_args(result: &mut ValidationResult, args: &[String]) {
    for (i, arg) in args.iter().enumerate() {
        // Check for blocked patterns
        for pattern in BLOCKED_ARG_PATTERNS {
            if arg.contains(pattern) {
                result.add_error(format!(
                    "Argument {} ('{}') contains blocked pattern '{}' which could indicate command injection",
                    i, arg, pattern
                ));
            }
        }

        // Check for shell commands as arguments
        let arg_lower = arg.to_lowercase();
        if arg_lower.contains("/bin/") || arg_lower.contains("\\system32\\") {
            result.add_warning(format!(
                "Argument {} ('{}') appears to reference a shell or system binary",
                i, arg
            ));
        }
    }
}

fn validate_env(result: &mut ValidationResult, env: &std::collections::HashMap<String, String>) {
    for (key, value) in env {
        // Check for blocked environment variables
        let key_upper = key.to_uppercase();
        if BLOCKED_ENV_VARS.contains(&key_upper.as_str()) {
            result.add_error(format!(
                "Environment variable '{}' is blocked for security reasons",
                key
            ));
        }

        // Check for shell metacharacters in values
        for pattern in &[";", "&&", "||", "`", "$(", "${"] {
            if value.contains(pattern) {
                result.add_warning(format!(
                    "Environment variable '{}' contains shell metacharacter '{}' which could be dangerous",
                    key, pattern
                ));
            }
        }
    }
}

/// Check if a command is in the allowlist (for quick checks)
pub fn is_command_allowed(command: &str) -> bool {
    let cmd_name = std::path::Path::new(command)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(command)
        .to_lowercase();

    let cmd_name = cmd_name.strip_suffix(".exe").unwrap_or(&cmd_name);
    let allowed_set: HashSet<&str> = ALLOWED_COMMANDS.iter().cloned().collect();
    allowed_set.contains(cmd_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_allowed_commands() {
        assert!(is_command_allowed("npx"));
        assert!(is_command_allowed("uvx"));
        assert!(is_command_allowed("python3"));
        assert!(is_command_allowed("node"));
        assert!(is_command_allowed("docker"));
    }

    #[test]
    fn test_blocked_commands() {
        assert!(!is_command_allowed("sh"));
        assert!(!is_command_allowed("bash"));
        assert!(!is_command_allowed("curl"));
        assert!(!is_command_allowed("wget"));
        assert!(!is_command_allowed("rm"));
    }

    #[test]
    fn test_command_with_unix_path() {
        assert!(is_command_allowed("/usr/local/bin/npx"));
        assert!(!is_command_allowed("/bin/sh"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_command_with_windows_path() {
        assert!(is_command_allowed("C:\\Users\\node.exe"));
    }

    #[test]
    fn test_valid_mcp_server() {
        let result = validate_mcp_command(
            "npx",
            &["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
            &HashMap::new(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_blocked_command_injection() {
        let result = validate_mcp_command(
            "npx",
            &["; rm -rf /".to_string()],
            &HashMap::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_env_var() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/malicious/path".to_string());
        let result = validate_mcp_command("npx", &[], &env);
        assert!(result.is_err());
    }
}
