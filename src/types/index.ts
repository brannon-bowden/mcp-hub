export interface McpServer {
  id: string;
  name: string;
  description?: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  tags: string[];
  source?: ServerSource;
  createdAt: string;
  updatedAt: string;
}

export interface ServerSource {
  sourceType: "manual" | "imported" | "registry";
  url?: string;
}

export type ClientType =
  | "claude-desktop"
  | "claude-code"
  | "cursor"
  | "windsurf"
  | "vscode"
  | "vscode-insiders"
  | "zed"
  | "continue"
  | "cody"
  | "cline"
  | "roo-code"
  | "kilo-code"
  | "amp"
  | "augment"
  | "antigravity"
  | "jetbrains"
  | "gemini-cli"
  | "qwen-coder"
  | "opencode"
  | "openai-codex"
  | "kiro"
  | "trae"
  | "lm-studio"
  | "visual-studio"
  | "crush"
  | "boltai"
  | "rovo-dev"
  | "zencoder"
  | "qodo-gen"
  | "perplexity"
  | "factory"
  | "emdash"
  | "amazon-q"
  | "warp"
  | "copilot-agent"
  | "copilot-cli"
  | "smithery"
  | "custom";

export interface ClientInstance {
  id: string;
  name: string;
  clientType: ClientType;
  configPath: string;
  enabledServers: string[];
  isDefault: boolean;
  lastSynced?: string;
  lastModified?: string;
  createdAt: string;
}

export interface ConfigBackup {
  id: string;
  instanceId: string;
  backupPath: string;
  createdAt: string;
}

export interface AppSettings {
  theme: "light" | "dark" | "system";
  autoStart: boolean;
  createBackups: boolean;
  backupRetentionDays: number;
}

export type HealthStatus = "healthy" | "error" | "unknown";

export interface ServerHealth {
  serverId: string;
  status: HealthStatus;
  errorMessage?: string;
  lastChecked: string;
}

export interface DetectedClient {
  clientType: ClientType;
  configPath: string;
  hasConfig: boolean;
}

export interface McpConfigFile {
  mcpServers: Record<string, McpServerEntry>;
}

export interface McpServerEntry {
  command: string;
  args: string[];
  env?: Record<string, string>;
}

export const CLIENT_TYPE_LABELS: Record<ClientType, string> = {
  "claude-desktop": "Claude Desktop",
  "claude-code": "Claude Code",
  cursor: "Cursor",
  windsurf: "Windsurf",
  vscode: "VS Code",
  "vscode-insiders": "VS Code Insiders",
  zed: "Zed",
  continue: "Continue",
  cody: "Sourcegraph Cody",
  cline: "Cline",
  "roo-code": "Roo Code",
  "kilo-code": "Kilo Code",
  amp: "Amp",
  augment: "Augment Code",
  antigravity: "Google Antigravity",
  jetbrains: "JetBrains AI",
  "gemini-cli": "Gemini CLI",
  "qwen-coder": "Qwen Coder",
  opencode: "Opencode",
  "openai-codex": "OpenAI Codex",
  kiro: "Kiro",
  trae: "Trae",
  "lm-studio": "LM Studio",
  "visual-studio": "Visual Studio 2022",
  crush: "Crush",
  boltai: "BoltAI",
  "rovo-dev": "Rovo Dev CLI",
  zencoder: "Zencoder",
  "qodo-gen": "Qodo Gen",
  perplexity: "Perplexity Desktop",
  factory: "Factory",
  emdash: "Emdash",
  "amazon-q": "Amazon Q Developer",
  warp: "Warp",
  "copilot-agent": "Copilot Coding Agent",
  "copilot-cli": "Copilot CLI",
  smithery: "Smithery",
  custom: "Custom",
};

export const CLIENT_TYPE_ICONS: Record<ClientType, string> = {
  "claude-desktop": "MessageSquare",
  "claude-code": "Terminal",
  cursor: "Code",
  windsurf: "Wind",
  vscode: "FileCode",
  "vscode-insiders": "FileCode",
  zed: "Zap",
  continue: "FastForward",
  cody: "Bot",
  cline: "Sparkles",
  "roo-code": "Kangaroo",
  "kilo-code": "Gauge",
  amp: "Zap",
  augment: "Wand",
  antigravity: "Rocket",
  jetbrains: "Code2",
  "gemini-cli": "Terminal",
  "qwen-coder": "Brain",
  opencode: "Code",
  "openai-codex": "Bot",
  kiro: "Layers",
  trae: "Box",
  "lm-studio": "Server",
  "visual-studio": "FileCode",
  crush: "Hammer",
  boltai: "Zap",
  "rovo-dev": "Terminal",
  zencoder: "Code2",
  "qodo-gen": "Wand",
  perplexity: "Search",
  factory: "Factory",
  emdash: "Type",
  "amazon-q": "Cloud",
  warp: "Terminal",
  "copilot-agent": "Github",
  "copilot-cli": "Terminal",
  smithery: "Hammer",
  custom: "Settings",
};

// Registry types
export interface RegistrySource {
  id: string;
  name: string;
  description: string;
  url: string;
  icon?: string;
  serverCount?: number;
}

export interface RegistryServer {
  name: string;
  description?: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  tags: string[];
  repository?: string;
  homepage?: string;
}
