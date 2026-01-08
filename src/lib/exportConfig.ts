import type { McpServer } from "@/types";

/**
 * Patterns for sensitive environment variable keys
 */
const SENSITIVE_KEY_PATTERNS = [
  /key/i,
  /token/i,
  /secret/i,
  /password/i,
  /credential/i,
  /auth/i,
  /api/i,
];

/**
 * Patterns for sensitive environment variable values
 */
const SENSITIVE_VALUE_PATTERNS = [
  /^sk-/,        // OpenAI, Anthropic
  /^ghp_/,       // GitHub PAT
  /^gho_/,       // GitHub OAuth
  /^github_pat_/, // GitHub fine-grained PAT
  /^xox[baprs]-/, // Slack tokens
  /^eyJ/,        // JWT tokens (base64 encoded JSON)
  /^AKIA/,       // AWS access keys
  /^postgres:\/\//i, // Connection strings
  /^mongodb\+srv:\/\//i,
  /^redis:\/\//i,
];

/**
 * Check if an environment variable should be masked
 */
export function isSensitiveEnvVar(key: string, value: string): boolean {
  const keyIsSensitive = SENSITIVE_KEY_PATTERNS.some((pattern) =>
    pattern.test(key)
  );
  const valueIsSensitive = SENSITIVE_VALUE_PATTERNS.some((pattern) =>
    pattern.test(value)
  );
  return keyIsSensitive || valueIsSensitive;
}

/**
 * Get list of sensitive env var keys for a server
 */
export function getSensitiveKeys(server: McpServer): string[] {
  return Object.entries(server.env)
    .filter(([key, value]) => isSensitiveEnvVar(key, value))
    .map(([key]) => key);
}

/**
 * Mask sensitive values in env object
 */
function maskEnvValues(
  env: Record<string, string>,
  mask: boolean
): Record<string, string> {
  if (!mask) return env;

  const masked: Record<string, string> = {};
  for (const [key, value] of Object.entries(env)) {
    if (isSensitiveEnvVar(key, value)) {
      masked[key] = `<${key}>`;
    } else {
      masked[key] = value;
    }
  }
  return masked;
}

export interface ExportOptions {
  maskSensitiveValues: boolean;
}

export interface McpConfigExport {
  mcpServers: Record<
    string,
    {
      command: string;
      args: string[];
      env?: Record<string, string>;
    }
  >;
}

/**
 * Export servers to MCP config format
 */
export function exportServers(
  servers: McpServer[],
  options: ExportOptions
): McpConfigExport {
  const mcpServers: McpConfigExport["mcpServers"] = {};

  for (const server of servers) {
    const env = maskEnvValues(server.env, options.maskSensitiveValues);

    mcpServers[server.name] = {
      command: server.command,
      args: server.args,
      ...(Object.keys(env).length > 0 ? { env } : {}),
    };
  }

  return { mcpServers };
}

/**
 * Convert export to formatted JSON string
 */
export function exportToJson(
  servers: McpServer[],
  options: ExportOptions
): string {
  const config = exportServers(servers, options);
  return JSON.stringify(config, null, 2);
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}
