import type { ParsedServer } from "@/types";

/**
 * Patterns that indicate placeholder values needing user attention
 */
const PLACEHOLDER_PATTERNS = [
  /your[-_]?\w*[-_]?here/i,
  /<[^>]+>/,
  /TODO/i,
  /CHANGEME/i,
  /xxx+/i,
  /placeholder/i,
  /^sk-\.{3,}/,
  /^ghp_\.{3,}/,
  /example\.com/i,
];

/**
 * Extract a reasonable server name from command arguments
 */
function extractNameFromArgs(command: string, args: string[]): string {
  // Look for npm package patterns in args
  for (const arg of args) {
    // Match @scope/package-name or package-name
    const npmMatch = arg.match(/@[\w-]+\/([\w-]+)/) || arg.match(/^([\w-]+)$/);
    if (npmMatch && !arg.startsWith("-")) {
      return npmMatch[1];
    }
  }
  // Fallback to command name
  return command;
}

/**
 * Check environment variables for placeholder values
 */
function detectPlaceholders(env: Record<string, string>): string[] {
  const warnings: string[] = [];
  for (const [key, value] of Object.entries(env)) {
    for (const pattern of PLACEHOLDER_PATTERNS) {
      if (pattern.test(value)) {
        warnings.push(`${key} appears to contain a placeholder value`);
        break;
      }
    }
  }
  return warnings;
}

/**
 * Validate a server entry has required fields
 */
function isValidServerEntry(obj: unknown): obj is { command: string; args?: string[]; env?: Record<string, string> } {
  if (typeof obj !== "object" || obj === null) return false;
  const entry = obj as Record<string, unknown>;
  return typeof entry.command === "string" && entry.command.trim().length > 0;
}

/**
 * Parse a single server entry into ParsedServer
 */
function parseServerEntry(
  name: string,
  entry: { command: string; args?: string[]; env?: Record<string, string> },
  existingNames: Set<string>
): ParsedServer {
  const args = Array.isArray(entry.args) ? entry.args : [];
  const env = typeof entry.env === "object" && entry.env !== null ? entry.env : {};

  // Auto-generate name if empty
  const finalName = name || extractNameFromArgs(entry.command, args);

  // Check for duplicates
  let uniqueName = finalName;
  let counter = 1;
  while (existingNames.has(uniqueName.toLowerCase())) {
    uniqueName = `${finalName}-${counter}`;
    counter++;
  }
  existingNames.add(uniqueName.toLowerCase());

  const warnings = detectPlaceholders(env);

  return {
    id: crypto.randomUUID(),
    originalName: finalName,
    editedName: uniqueName,
    command: entry.command,
    args,
    env,
    warnings,
    isValid: true,
  };
}

export interface ParseResult {
  servers: ParsedServer[];
  error?: string;
}

/**
 * Parse JSON string into array of ParsedServer
 * Supports three formats:
 * 1. Full config: { "mcpServers": { "name": {...} } }
 * 2. Named entries: { "name": { "command": "..." } }
 * 3. Single entry: { "command": "...", "args": [...] }
 */
export function parseConfig(jsonString: string): ParseResult {
  // Trim and validate non-empty
  const trimmed = jsonString.trim();
  if (!trimmed) {
    return { servers: [], error: "No JSON content provided" };
  }

  // Parse JSON
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch (e) {
    const error = e instanceof SyntaxError ? e.message : "Invalid JSON";
    return { servers: [], error: `JSON parse error: ${error}` };
  }

  if (typeof parsed !== "object" || parsed === null) {
    return { servers: [], error: "JSON must be an object" };
  }

  const obj = parsed as Record<string, unknown>;
  const servers: ParsedServer[] = [];
  const existingNames = new Set<string>();

  // Format 1: Full config with mcpServers
  if ("mcpServers" in obj && typeof obj.mcpServers === "object" && obj.mcpServers !== null) {
    const mcpServers = obj.mcpServers as Record<string, unknown>;
    for (const [name, entry] of Object.entries(mcpServers)) {
      if (isValidServerEntry(entry)) {
        servers.push(parseServerEntry(name, entry, existingNames));
      }
    }
    if (servers.length === 0) {
      return { servers: [], error: "No valid server entries found in mcpServers" };
    }
    return { servers };
  }

  // Format 3: Single server entry (check this before Format 2)
  if (isValidServerEntry(obj)) {
    const args = Array.isArray(obj.args) ? obj.args : [];
    const name = extractNameFromArgs(obj.command, args);
    servers.push(parseServerEntry(name, obj, existingNames));
    return { servers };
  }

  // Format 2: Named server objects
  let foundValidEntries = false;
  for (const [name, entry] of Object.entries(obj)) {
    if (isValidServerEntry(entry)) {
      servers.push(parseServerEntry(name, entry, existingNames));
      foundValidEntries = true;
    }
  }

  if (!foundValidEntries) {
    return { servers: [], error: "No valid server entries found. Expected { command: string, args?: string[] }" };
  }

  return { servers };
}

/**
 * Validate parsed servers for import readiness
 * Returns updated servers with isValid flags
 */
export function validateParsedServers(
  servers: ParsedServer[],
  existingServerNames: string[]
): ParsedServer[] {
  const existingLower = new Set(existingServerNames.map(n => n.toLowerCase()));
  const seenNames = new Set<string>();

  return servers.map(server => {
    const nameLower = server.editedName.toLowerCase().trim();
    const isEmpty = !server.editedName.trim();
    const isDuplicate = seenNames.has(nameLower);
    const alreadyExists = existingLower.has(nameLower);

    seenNames.add(nameLower);

    const warnings = [...server.warnings];
    if (alreadyExists) {
      warnings.push(`Server "${server.editedName}" already exists`);
    }
    if (isDuplicate) {
      warnings.push(`Duplicate name in paste`);
    }

    return {
      ...server,
      warnings,
      isValid: !isEmpty && !isDuplicate && !alreadyExists,
    };
  });
}
