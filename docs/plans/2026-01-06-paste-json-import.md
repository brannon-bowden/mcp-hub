# Paste JSON Import Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a "Paste" tab to the Import dialog allowing users to paste MCP server configuration JSON snippets directly.

**Architecture:** Frontend-only feature. New parsing utility detects JSON format (full config, named entries, single server), extracts server definitions, and presents editable preview. Uses existing `createServer` store action for persistence.

**Tech Stack:** React, TypeScript, Zustand, Tailwind CSS, existing UI components

---

## Task 1: Add ParsedServer Type

**Files:**
- Modify: `src/types/index.ts` (add to end of file)

**Step 1: Add the ParsedServer interface**

Add after line 264 (after `LogEntry` interface):

```typescript
// Parsed server for paste import preview
export interface ParsedServer {
  id: string;                      // Temporary ID for React keys
  originalName: string;            // Name from JSON or auto-generated
  editedName: string;              // User-modified name
  command: string;
  args: string[];
  env: Record<string, string>;
  warnings: string[];              // Placeholder warnings
  isValid: boolean;                // Name not empty, no duplicates
}
```

**Step 2: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/types/index.ts
git commit -m "feat(types): add ParsedServer interface for paste import"
```

---

## Task 2: Create parseConfig Utility - Core Parsing

**Files:**
- Create: `src/lib/parseConfig.ts`

**Step 1: Create the parsing utility with format detection**

```typescript
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
  return typeof entry.command === "string";
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
```

**Step 2: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/lib/parseConfig.ts
git commit -m "feat(lib): add parseConfig utility for JSON format detection"
```

---

## Task 3: Create ParsedServerCard Component

**Files:**
- Create: `src/components/ParsedServerCard.tsx`

**Step 1: Create the editable server preview card**

```typescript
import { X, AlertTriangle } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import type { ParsedServer } from "@/types";

interface ParsedServerCardProps {
  server: ParsedServer;
  onNameChange: (id: string, name: string) => void;
  onEnvChange: (id: string, key: string, value: string) => void;
  onRemove: (id: string) => void;
}

export function ParsedServerCard({
  server,
  onNameChange,
  onEnvChange,
  onRemove,
}: ParsedServerCardProps) {
  const envEntries = Object.entries(server.env);
  const hasEnvWarnings = server.warnings.some(w => w.includes("placeholder"));

  return (
    <div
      className={`p-4 rounded-lg border ${
        !server.isValid
          ? "border-destructive/50 bg-destructive/5"
          : server.warnings.length > 0
          ? "border-yellow-500/50 bg-yellow-500/5"
          : "border-border"
      }`}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 space-y-3">
          {/* Name field */}
          <div className="space-y-1">
            <Label htmlFor={`name-${server.id}`} className="text-xs text-muted-foreground">
              Server Name
            </Label>
            <Input
              id={`name-${server.id}`}
              value={server.editedName}
              onChange={(e) => onNameChange(server.id, e.target.value)}
              className={`h-8 ${!server.editedName.trim() ? "border-destructive" : ""}`}
              placeholder="Enter server name"
            />
          </div>

          {/* Command display */}
          <div className="space-y-1">
            <span className="text-xs text-muted-foreground">Command</span>
            <code className="block text-sm px-2 py-1 bg-muted rounded">
              {server.command}
            </code>
          </div>

          {/* Args display */}
          {server.args.length > 0 && (
            <div className="space-y-1">
              <span className="text-xs text-muted-foreground">Arguments</span>
              <div className="text-sm text-muted-foreground truncate">
                {server.args.join(" ")}
              </div>
            </div>
          )}

          {/* Environment variables */}
          {envEntries.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">Environment Variables</span>
                {hasEnvWarnings && (
                  <AlertTriangle className="h-3 w-3 text-yellow-500" />
                )}
              </div>
              <div className="space-y-2">
                {envEntries.map(([key, value]) => {
                  const hasWarning = server.warnings.some(
                    (w) => w.includes(key) && w.includes("placeholder")
                  );
                  return (
                    <div key={key} className="flex items-center gap-2">
                      <code className="text-xs px-1.5 py-0.5 bg-muted rounded min-w-[100px]">
                        {key}
                      </code>
                      <Input
                        value={value}
                        onChange={(e) => onEnvChange(server.id, key, e.target.value)}
                        className={`h-7 text-sm flex-1 ${
                          hasWarning ? "border-yellow-500" : ""
                        }`}
                      />
                      {hasWarning && (
                        <AlertTriangle className="h-4 w-4 text-yellow-500 flex-shrink-0" />
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Warnings */}
          {server.warnings.length > 0 && (
            <div className="flex flex-wrap gap-1 pt-1">
              {server.warnings.map((warning, i) => (
                <Badge
                  key={i}
                  variant="outline"
                  className="text-xs text-yellow-600 border-yellow-500/50"
                >
                  {warning}
                </Badge>
              ))}
            </div>
          )}
        </div>

        {/* Remove button */}
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground hover:text-destructive flex-shrink-0"
          onClick={() => onRemove(server.id)}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
```

**Step 2: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/components/ParsedServerCard.tsx
git commit -m "feat(components): add ParsedServerCard for paste import preview"
```

---

## Task 4: Add Paste Tab to ImportDialog

**Files:**
- Modify: `src/components/ImportDialog.tsx`

**Step 1: Add imports at top of file**

Add after line 45 (after `AddRegistryDialog` import):

```typescript
import { ParsedServerCard } from "./ParsedServerCard";
import { parseConfig, validateParsedServers, type ParseResult } from "@/lib/parseConfig";
import type { ParsedServer } from "@/types";
import { Clipboard } from "lucide-react";
```

**Step 2: Add state variables**

Add after line 120 (after `const [refreshing, setRefreshing] = useState(false);`):

```typescript
  // Paste tab state
  const [pasteContent, setPasteContent] = useState("");
  const [parsedServers, setParsedServers] = useState<ParsedServer[]>([]);
  const [parseError, setParseError] = useState<string | null>(null);
  const [isParsing, setIsParsing] = useState(false);
```

**Step 3: Add paste handling functions**

Add after the `handleRefreshRegistry` function (around line 347):

```typescript
  const handleParse = () => {
    setIsParsing(true);
    setParseError(null);

    const result = parseConfig(pasteContent);

    if (result.error) {
      setParseError(result.error);
      setParsedServers([]);
    } else {
      // Validate against existing servers
      const validated = validateParsedServers(
        result.servers,
        servers.map((s) => s.name)
      );
      setParsedServers(validated);
    }

    setIsParsing(false);
  };

  const handleParsedNameChange = (id: string, name: string) => {
    setParsedServers((prev) => {
      const updated = prev.map((s) =>
        s.id === id ? { ...s, editedName: name } : s
      );
      // Re-validate all servers
      return validateParsedServers(
        updated,
        servers.map((s) => s.name)
      );
    });
  };

  const handleParsedEnvChange = (id: string, key: string, value: string) => {
    setParsedServers((prev) =>
      prev.map((s) =>
        s.id === id
          ? { ...s, env: { ...s.env, [key]: value } }
          : s
      )
    );
  };

  const handleRemoveParsed = (id: string) => {
    setParsedServers((prev) => {
      const filtered = prev.filter((s) => s.id !== id);
      return validateParsedServers(
        filtered,
        servers.map((s) => s.name)
      );
    });
  };

  const handleImportFromPaste = async () => {
    const validServers = parsedServers.filter((s) => s.isValid);
    if (validServers.length === 0) return;

    setImporting(true);
    setError(null);

    try {
      const now = new Date().toISOString();

      for (const server of validServers) {
        await createServer({
          id: crypto.randomUUID(),
          name: server.editedName,
          command: server.command,
          args: server.args,
          env: server.env,
          tags: [],
          source: { sourceType: "imported" },
          createdAt: now,
          updatedAt: now,
        });
      }

      setSuccessMessage(`Successfully imported ${validServers.length} server(s)`);
      setPasteContent("");
      setParsedServers([]);

      setTimeout(() => {
        onOpenChange(false);
        setSuccessMessage(null);
      }, 1500);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to import servers");
    } finally {
      setImporting(false);
    }
  };

  const handleClearPaste = () => {
    setPasteContent("");
    setParsedServers([]);
    setParseError(null);
  };
```

**Step 4: Update store destructuring to include createServer**

Find line 94-104 and update to include `createServer`:

```typescript
  const {
    importFromFile,
    getRegistries,
    getRegistryServers,
    importFromRegistry,
    detectClients,
    detectedClients,
    servers,
    refreshCustomRegistry,
    deleteCustomRegistry,
    createServer,
  } = useStore();
```

**Step 5: Add reset for paste tab in the open effect**

Find the useEffect that runs on open (around line 123-130) and add paste tab reset:

```typescript
  // Load registries on open
  useEffect(() => {
    if (open) {
      loadRegistries();
      detectClients();
      setSearchQuery("");
      setSelectedCategory("all");
      setSelectedServers(new Set());
      // Reset paste tab
      setPasteContent("");
      setParsedServers([]);
      setParseError(null);
    }
  }, [open]);
```

**Step 6: Add 4th tab to TabsList**

Find the TabsList (around line 391-404) and add the Paste tab:

```typescript
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="registry" className="flex items-center gap-2">
              <Package className="h-4 w-4" />
              Registry
            </TabsTrigger>
            <TabsTrigger value="paste" className="flex items-center gap-2">
              <Clipboard className="h-4 w-4" />
              Paste
            </TabsTrigger>
            <TabsTrigger value="file" className="flex items-center gap-2">
              <FileJson className="h-4 w-4" />
              File
            </TabsTrigger>
            <TabsTrigger value="client" className="flex items-center gap-2">
              <Download className="h-4 w-4" />
              Client
            </TabsTrigger>
          </TabsList>
```

**Step 7: Add Paste TabsContent**

Add after the Registry TabsContent closing tag (after line ~668) and before the File TabsContent:

```typescript
          <TabsContent value="paste" className="flex-1 flex flex-col min-h-0 mt-4">
            <div className="space-y-4">
              {/* Paste textarea */}
              <div className="space-y-2">
                <Textarea
                  placeholder={`Paste MCP server configuration JSON...

Examples:
• Full config: {"mcpServers": {"name": {"command": "npx", ...}}}
• Named entry: {"my-server": {"command": "npx", "args": [...]}}
• Single server: {"command": "npx", "args": ["-y", "@scope/package"]}`}
                  value={pasteContent}
                  onChange={(e) => setPasteContent(e.target.value)}
                  className="min-h-[150px] font-mono text-sm"
                />
                <div className="flex justify-between">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleClearPaste}
                    disabled={!pasteContent}
                  >
                    Clear
                  </Button>
                  <Button
                    size="sm"
                    onClick={handleParse}
                    disabled={!pasteContent.trim() || isParsing}
                  >
                    {isParsing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                    Parse JSON
                  </Button>
                </div>
              </div>

              {/* Parse error */}
              {parseError && (
                <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
                  {parseError}
                </div>
              )}

              {/* Parsed servers preview */}
              {parsedServers.length > 0 && (
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <h4 className="font-medium">
                      Parsed Servers ({parsedServers.length})
                    </h4>
                    <span className="text-sm text-muted-foreground">
                      {parsedServers.filter((s) => s.isValid).length} ready to import
                    </span>
                  </div>
                  <div className="space-y-3 max-h-[250px] overflow-y-auto pr-2">
                    {parsedServers.map((server) => (
                      <ParsedServerCard
                        key={server.id}
                        server={server}
                        onNameChange={handleParsedNameChange}
                        onEnvChange={handleParsedEnvChange}
                        onRemove={handleRemoveParsed}
                      />
                    ))}
                  </div>
                </div>
              )}
            </div>

            <DialogFooter className="mt-4">
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleImportFromPaste}
                disabled={
                  parsedServers.filter((s) => s.isValid).length === 0 || importing
                }
              >
                {importing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Import{" "}
                {parsedServers.filter((s) => s.isValid).length > 0 &&
                  `(${parsedServers.filter((s) => s.isValid).length})`}
              </Button>
            </DialogFooter>
          </TabsContent>
```

**Step 8: Add Textarea import**

Check that Textarea is imported. Find the UI imports at the top and ensure Textarea is included:

```typescript
import { Textarea } from "@/components/ui/textarea";
```

(Note: Textarea is likely already imported since it may be used elsewhere. Verify first.)

**Step 9: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 10: Commit**

```bash
git add src/components/ImportDialog.tsx
git commit -m "feat(ImportDialog): add Paste tab for JSON config import"
```

---

## Task 5: Manual Testing

**Step 1: Start the dev server**

Run: `PATH="$HOME/.cargo/bin:$PATH" pnpm tauri dev`

**Step 2: Test Format 1 - Full config**

1. Navigate to Servers page
2. Click "Import" button
3. Click "Paste" tab
4. Paste:
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {}
    }
  }
}
```
5. Click "Parse JSON"
6. Verify: Server "filesystem" appears in preview
7. Click "Import"
8. Verify: Server appears in servers list

**Step 3: Test Format 2 - Named entry**

1. Open Import → Paste
2. Paste:
```json
{
  "my-github-server": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "env": {
      "GITHUB_TOKEN": "<your-token-here>"
    }
  }
}
```
3. Verify: Warning badge appears for placeholder
4. Edit the token value
5. Import and verify

**Step 4: Test Format 3 - Single entry**

1. Open Import → Paste
2. Paste:
```json
{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-slack"]
}
```
3. Verify: Auto-generated name "server-slack"
4. Edit name if desired
5. Import and verify

**Step 5: Test error handling**

1. Paste invalid JSON: `{ broken json`
2. Verify: Error message appears
3. Paste empty object: `{}`
4. Verify: "No valid server entries" error

**Step 6: Commit if all tests pass**

```bash
git add -A
git commit -m "test: verify paste import functionality"
```

---

## Task 6: Final Verification and PR Prep

**Step 1: Run full build**

Run: `pnpm build`
Expected: Build succeeds

**Step 2: Run Rust check**

Run: `cargo check` (in src-tauri)
Expected: Check succeeds

**Step 3: Push branch**

```bash
git push -u origin feature/35-paste-json-import
```

**Step 4: Create PR**

Title: `[#35] Add paste JSON import for MCP server configs`

Body:
```markdown
## Summary

Adds a "Paste" tab to the Import dialog, allowing users to paste MCP server configuration JSON snippets directly from documentation, READMEs, or existing config files.

## Changes

- Added `ParsedServer` type for preview state
- Created `parseConfig` utility with auto-detection for 3 JSON formats
- Created `ParsedServerCard` component for editable preview
- Added "Paste" tab to ImportDialog with parse and import flow

## Features

- Auto-detects JSON format (full config, named entries, single server)
- Auto-generates server names from package names in args
- Highlights placeholder values (e.g., `<your-token>`) as warnings
- Allows editing server names and env vars before import
- Validates against existing servers to prevent duplicates

## Testing Completed

- [x] Full config format parsing
- [x] Named entry format parsing
- [x] Single server format parsing
- [x] Name auto-generation from args
- [x] Placeholder detection and warnings
- [x] Inline editing of names and env vars
- [x] Duplicate detection
- [x] Error handling for invalid JSON
- [x] Import creates servers correctly

## Screenshots

[Add screenshots of paste tab UI]

Closes #35
```
