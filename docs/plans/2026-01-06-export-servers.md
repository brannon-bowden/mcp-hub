# Export MCP Servers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add export functionality to export MCP server configurations to clipboard or file, with smart masking of sensitive environment variables.

**Architecture:** Frontend-only feature. New export utility handles JSON generation and sensitive value detection. ExportDialog component provides server selection and export options. Per-server quick export added to server cards. Uses Tauri's dialog plugin for file save.

**Tech Stack:** React, TypeScript, Tailwind CSS, Tauri dialog plugin, Clipboard API

---

## Task 1: Create exportConfig Utility

**Files:**
- Create: `src/lib/exportConfig.ts`

**Step 1: Create the export utility with sensitive detection**

```typescript
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
```

**Step 2: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/lib/exportConfig.ts
git commit -m "feat(lib): add exportConfig utility with sensitive detection"
```

---

## Task 2: Create ExportDialog Component

**Files:**
- Create: `src/components/ExportDialog.tsx`

**Step 1: Create the export dialog component**

```typescript
import { useState, useMemo } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Copy,
  Download,
  Check,
  Loader2,
  ShieldAlert,
} from "lucide-react";
import type { McpServer } from "@/types";
import {
  exportToJson,
  copyToClipboard,
  getSensitiveKeys,
} from "@/lib/exportConfig";

interface ExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  servers: McpServer[];
  preSelectedIds?: string[];
}

export function ExportDialog({
  open,
  onOpenChange,
  servers,
  preSelectedIds,
}: ExportDialogProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(
    new Set(preSelectedIds || servers.map((s) => s.id))
  );
  const [maskSensitive, setMaskSensitive] = useState(true);
  const [copying, setCopying] = useState(false);
  const [saving, setSaving] = useState(false);
  const [copied, setCopied] = useState(false);

  // Reset state when dialog opens
  useState(() => {
    if (open) {
      setSelectedIds(new Set(preSelectedIds || servers.map((s) => s.id)));
      setMaskSensitive(true);
      setCopied(false);
    }
  });

  const selectedServers = useMemo(
    () => servers.filter((s) => selectedIds.has(s.id)),
    [servers, selectedIds]
  );

  const allSelected = selectedIds.size === servers.length;
  const someSelected = selectedIds.size > 0 && !allSelected;

  const toggleServer = (id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  };

  const toggleAll = () => {
    if (allSelected) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(servers.map((s) => s.id)));
    }
  };

  const handleCopy = async () => {
    if (selectedServers.length === 0) return;

    setCopying(true);
    try {
      const json = exportToJson(selectedServers, {
        maskSensitiveValues: maskSensitive,
      });
      await copyToClipboard(json);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy:", error);
    } finally {
      setCopying(false);
    }
  };

  const handleSave = async () => {
    if (selectedServers.length === 0) return;

    setSaving(true);
    try {
      const json = exportToJson(selectedServers, {
        maskSensitiveValues: maskSensitive,
      });

      const filePath = await save({
        filters: [{ name: "JSON", extensions: ["json"] }],
        defaultPath: "mcp-servers.json",
      });

      if (filePath) {
        await writeTextFile(filePath, json);
        onOpenChange(false);
      }
    } catch (error) {
      console.error("Failed to save:", error);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Export MCP Servers</DialogTitle>
          <DialogDescription>
            Select servers to export and choose output format
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-hidden flex flex-col gap-4 py-4">
          {/* Select All */}
          <div className="flex items-center gap-2">
            <Checkbox
              id="select-all"
              checked={allSelected}
              ref={(el) => {
                if (el) {
                  (el as HTMLButtonElement & { indeterminate: boolean }).indeterminate = someSelected;
                }
              }}
              onCheckedChange={toggleAll}
            />
            <Label htmlFor="select-all" className="font-medium">
              Select All ({servers.length} servers)
            </Label>
          </div>

          {/* Server List */}
          <div className="flex-1 overflow-y-auto border rounded-lg divide-y max-h-[250px]">
            {servers.map((server) => {
              const sensitiveKeys = getSensitiveKeys(server);
              const hasSensitive = sensitiveKeys.length > 0;

              return (
                <div
                  key={server.id}
                  className="flex items-start gap-3 p-3 hover:bg-muted/50"
                >
                  <Checkbox
                    checked={selectedIds.has(server.id)}
                    onCheckedChange={() => toggleServer(server.id)}
                    className="mt-0.5"
                  />
                  <div className="flex-1 min-w-0">
                    <div className="font-medium">{server.name}</div>
                    <div className="text-sm text-muted-foreground">
                      Command: {server.command}
                    </div>
                    {hasSensitive && maskSensitive && (
                      <div className="flex items-center gap-1 mt-1 text-xs text-yellow-600">
                        <ShieldAlert className="h-3 w-3" />
                        <span>
                          {sensitiveKeys.join(", ")} → will be masked
                        </span>
                      </div>
                    )}
                    {Object.keys(server.env).length > 0 && !hasSensitive && (
                      <div className="text-xs text-muted-foreground mt-1">
                        Env: {Object.keys(server.env).join(", ")}
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Mask Toggle */}
          <div className="flex items-start gap-3 p-3 border rounded-lg bg-muted/30">
            <Checkbox
              id="mask-sensitive"
              checked={maskSensitive}
              onCheckedChange={(checked) => setMaskSensitive(checked === true)}
              className="mt-0.5"
            />
            <div>
              <Label htmlFor="mask-sensitive" className="font-medium">
                Mask sensitive values (recommended)
              </Label>
              <p className="text-sm text-muted-foreground">
                API keys, tokens, and secrets will be replaced with placeholders
              </p>
            </div>
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={handleCopy}
            disabled={selectedServers.length === 0 || copying}
          >
            {copying ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : copied ? (
              <Check className="mr-2 h-4 w-4 text-green-500" />
            ) : (
              <Copy className="mr-2 h-4 w-4" />
            )}
            {copied ? "Copied!" : "Copy to Clipboard"}
          </Button>
          <Button
            onClick={handleSave}
            disabled={selectedServers.length === 0 || saving}
          >
            {saving ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Download className="mr-2 h-4 w-4" />
            )}
            Save to File
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

**Step 2: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/components/ExportDialog.tsx
git commit -m "feat(components): add ExportDialog for server export"
```

---

## Task 3: Add Export Button and Dialog to Servers Page

**Files:**
- Modify: `src/pages/Servers.tsx`

**Step 1: Add imports**

Add after existing imports (around line 10):

```typescript
import { ExportDialog } from "@/components/ExportDialog";
import { Upload } from "lucide-react";
```

Note: `Upload` icon represents export (arrow going out). Alternatively use `Share` or `FileUp`.

**Step 2: Add export dialog state**

Add after other useState declarations (around line 62):

```typescript
const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);
```

**Step 3: Add Export button next to Import button**

Find the header buttons section (around line 254-264) and add Export button:

```typescript
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setIsExportDialogOpen(true)}>
            <Upload className="w-4 h-4 mr-2" />
            Export
          </Button>
          <Button variant="outline" onClick={() => setIsImportDialogOpen(true)}>
            <Download className="w-4 h-4 mr-2" />
            Import
          </Button>
          <Button onClick={() => handleOpenDialog()}>
            <Plus className="w-4 h-4 mr-2" />
            Add Server
          </Button>
        </div>
```

**Step 4: Add ExportDialog at the end of the component**

Add after the ImportDialog (around line 520):

```typescript
      {/* Export Dialog */}
      <ExportDialog
        open={isExportDialogOpen}
        onOpenChange={setIsExportDialogOpen}
        servers={servers}
      />
```

**Step 5: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add src/pages/Servers.tsx
git commit -m "feat(Servers): add Export button and dialog"
```

---

## Task 4: Add Per-Server Quick Export Menu

**Files:**
- Modify: `src/pages/Servers.tsx`

**Step 1: Add DropdownMenu imports**

Add to existing imports:

```typescript
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { exportToJson, copyToClipboard } from "@/lib/exportConfig";
```

**Step 2: Add state for export feedback**

Add after other state:

```typescript
const [exportedServerId, setExportedServerId] = useState<string | null>(null);
```

**Step 3: Add quick export handler functions**

Add before the return statement:

```typescript
  const handleQuickExport = async (
    server: McpServer,
    maskSensitive: boolean
  ) => {
    try {
      const json = exportToJson([server], { maskSensitiveValues: maskSensitive });
      await copyToClipboard(json);
      setExportedServerId(server.id);
      setTimeout(() => setExportedServerId(null), 2000);
    } catch (error) {
      console.error("Failed to export:", error);
    }
  };

  const handleQuickExportToFile = async (server: McpServer) => {
    // Open export dialog with just this server pre-selected
    setIsExportDialogOpen(true);
    // We'll need to pass preSelectedIds - handled in next step
  };
```

**Step 4: Replace the Copy button with export dropdown**

Find the server card buttons section (around line 329-340). Replace the Copy button with a dropdown:

```typescript
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          title="Export"
                        >
                          {exportedServerId === server.id ? (
                            <Check className="w-4 h-4 text-green-500" />
                          ) : (
                            <Upload className="w-4 h-4" />
                          )}
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem
                          onClick={() => handleQuickExport(server, true)}
                        >
                          <Copy className="w-4 h-4 mr-2" />
                          Copy (masked)
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => handleQuickExport(server, false)}
                        >
                          <Copy className="w-4 h-4 mr-2" />
                          Copy (with values)
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          onClick={() => {
                            setPreSelectedExportIds([server.id]);
                            setIsExportDialogOpen(true);
                          }}
                        >
                          <Download className="w-4 h-4 mr-2" />
                          Save to File...
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
```

**Step 5: Add preSelectedExportIds state and pass to dialog**

Add state:

```typescript
const [preSelectedExportIds, setPreSelectedExportIds] = useState<string[] | undefined>();
```

Update ExportDialog to pass preSelectedIds and reset on close:

```typescript
      <ExportDialog
        open={isExportDialogOpen}
        onOpenChange={(open) => {
          setIsExportDialogOpen(open);
          if (!open) setPreSelectedExportIds(undefined);
        }}
        servers={servers}
        preSelectedIds={preSelectedExportIds}
      />
```

**Step 6: Add Check icon import**

Make sure `Check` is imported from lucide-react.

**Step 7: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 8: Commit**

```bash
git add src/pages/Servers.tsx
git commit -m "feat(Servers): add per-server quick export menu"
```

---

## Task 5: Fix ExportDialog State Reset

**Files:**
- Modify: `src/components/ExportDialog.tsx`

**Step 1: Fix the useEffect for state reset**

The current useState pattern is wrong. Replace with proper useEffect:

Find and replace the incorrect useState call (around line 35-41):

```typescript
  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setSelectedIds(new Set(preSelectedIds || servers.map((s) => s.id)));
      setMaskSensitive(true);
      setCopied(false);
    }
  }, [open, preSelectedIds, servers]);
```

Add `useEffect` to imports if not present.

**Step 2: Verify TypeScript compiles**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/components/ExportDialog.tsx
git commit -m "fix(ExportDialog): fix state reset on dialog open"
```

---

## Task 6: Manual Testing and PR

**Step 1: Verify full build**

Run: `pnpm build`
Expected: Build succeeds

**Step 2: Verify Rust check**

Run: `cd src-tauri && /Users/brannonbowden/.cargo/bin/cargo check`
Expected: Check succeeds

**Step 3: Push branch**

```bash
git push -u origin feature/36-export-servers
```

**Step 4: Create PR**

Title: `[#36] Add export functionality for MCP servers`

Body:
```markdown
## Summary

Adds export functionality to the Servers page, allowing users to export MCP server configurations to clipboard or file with smart sensitive value masking.

## Changes

- Created `exportConfig.ts` utility with sensitive env var detection
- Created `ExportDialog` component for bulk export
- Added Export button to Servers page header
- Added per-server quick export dropdown menu

## Features

- **Export selected or all servers** to standard MCP config format
- **Smart sensitive value detection:**
  - Key patterns: KEY, TOKEN, SECRET, PASSWORD, CREDENTIAL, AUTH, API
  - Value patterns: sk-*, ghp_*, xox*, eyJ* (JWT), AWS keys, connection strings
- **Copy to clipboard** for quick sharing
- **Save to file** for backups
- **Per-server quick export** with masked/unmasked options

## Testing Completed

- [ ] Frontend builds successfully (`pnpm build`)
- [ ] Rust compiles successfully (`cargo check`)
- [ ] Export button appears on Servers page
- [ ] Export dialog shows server list with checkboxes
- [ ] Sensitive env vars are detected and shown
- [ ] Copy to clipboard works
- [ ] Save to file works
- [ ] Per-server export menu works
- [ ] Exported JSON is valid

Closes #36
```
