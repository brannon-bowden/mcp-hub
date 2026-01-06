# Design: Import Server from Pasted JSON

**Issue:** [#35](https://github.com/brannon-bowden/mcp-hub/issues/35) - Ability to import server from a Claude Code / similar MCP server example

**Date:** 2026-01-06

## Summary

Add a "Paste" tab to the Import dialog allowing users to paste MCP server configuration JSON snippets directly, rather than selecting a file. This supports the common workflow of copying config examples from documentation, READMEs, or existing config files.

## User Flow

### Entry Point

- User clicks "Import" on the Servers page → ImportDialog opens
- New 4th tab: "Paste" with a clipboard icon, alongside Registry/File/Client

### Paste Flow

1. User sees a large textarea with placeholder text showing example formats
2. User pastes JSON config snippet
3. Click "Parse" button (or auto-parse on paste after brief debounce)
4. Parsed servers appear in an editable preview below the textarea
5. Each server shows: name (editable), command, args, env vars
6. Placeholder values highlighted with warning styling
7. User can remove servers from the list, edit names, or modify env values
8. Click "Import" to create servers

### Error Handling

- Invalid JSON: Show inline error message below textarea with line/position info
- Duplicate names: Highlight in preview, suggest appending number
- Empty result: "No servers found in pasted content"

## JSON Parsing Logic

### Format Detection (in order of precedence)

1. **Full config file**: `{ "mcpServers": { ... } }`
   - Extract all entries from `mcpServers` object
   - Each key becomes the server name

2. **Named server object**: `{ "my-server": { "command": "...", "args": [...] } }`
   - Detect by checking if values have `command` property
   - Key becomes server name

3. **Single server entry**: `{ "command": "...", "args": [...] }`
   - Detect by presence of `command` at root level
   - Auto-generate name from args (see below)

### Name Auto-Generation

- Scan args for npm package patterns: `@scope/package-name` or `package-name`
- Extract last segment: `@modelcontextprotocol/server-filesystem` → `server-filesystem`
- Fallback: use command name + counter (e.g., `npx-server-1`)

### Placeholder Detection

Regex patterns to flag as warnings:
- `your-*-here`, `<...>`, `TODO`, `CHANGEME`, `xxx`, `placeholder`
- Common dummy values: `sk-...`, `ghp_...` with obvious placeholders

## UI Components

### Paste Tab Layout

```
┌─────────────────────────────────────────────────┐
│ [Textarea - 6 rows]                             │
│ Paste MCP server configuration JSON...          │
│                                                 │
│ Supports: full config, named entries, or single │
│ server definitions                              │
└─────────────────────────────────────────────────┘
         [Clear]                    [Parse JSON]

┌─ Parsed Servers (2) ────────────────────────────┐
│ ┌─────────────────────────────────────────────┐ │
│ │ Name: [filesystem________] [x]              │ │
│ │ Command: npx                                │ │
│ │ Args: -y @modelcontextprotocol/server-fs... │ │
│ │ Env: (none)                                 │ │
│ └─────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────┐ │
│ │ Name: [github____________] [x]              │ │
│ │ Command: npx                                │ │
│ │ Args: -y @modelcontextprotocol/server-gi... │ │
│ │ Env: GITHUB_TOKEN=<your-token> ⚠️           │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘

                              [Cancel] [Import (2)]
```

### Component Breakdown

- Reuse existing `Textarea`, `Input`, `Button`, `Badge` from ui components
- New `ParsedServerCard` component for each parsed server
- Warning badge next to env vars with placeholders
- Expandable env section if many variables

## Implementation Details

### Files to Modify

| File | Changes |
|------|---------|
| `src/components/ImportDialog.tsx` | Add 4th "Paste" tab with textarea and preview |
| `src/components/ParsedServerCard.tsx` | New component for editable server preview |
| `src/lib/parseConfig.ts` | New utility for format detection and parsing |
| `src/types/index.ts` | Add `ParsedServer` interface for preview state |

### New Types

```typescript
interface ParsedServer {
  originalName: string;      // Name from JSON (or auto-generated)
  editedName: string;        // User-modified name
  command: string;
  args: string[];
  env: Record<string, string>;
  warnings: string[];        // Placeholder warnings
  isValid: boolean;          // Name not empty, no duplicates
}
```

### Data Flow

1. Paste JSON → `parseConfig()` → `ParsedServer[]`
2. User edits in preview → update local state
3. Import button → transform to `McpServer[]` → call existing `createServer()` for each
4. Source set to `{ sourceType: "imported" }`

### No Backend Changes Required

- All parsing happens in frontend
- Uses existing `createServer` Tauri command

## Acceptance Criteria

- [ ] Paste tab appears in Import dialog
- [ ] Full config format (`mcpServers`) is parsed correctly
- [ ] Named server objects are parsed correctly
- [ ] Single server entries auto-generate names from args
- [ ] Placeholder values are highlighted with warnings
- [ ] Server names are editable in preview
- [ ] Servers can be removed from preview before import
- [ ] Import creates servers with `sourceType: "imported"`
- [ ] Invalid JSON shows clear error message
- [ ] Duplicate server names are detected and flagged
