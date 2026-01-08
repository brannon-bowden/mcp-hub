# Design: Export MCP Servers

**Issue:** [#36](https://github.com/brannon-bowden/mcp-hub/issues/36) - Ability to export MCP servers

**Date:** 2026-01-06

## Summary

Add export functionality allowing users to export MCP server configurations to clipboard or file, with smart detection to mask sensitive environment variables.

## User Flows

### Page-Level Export (Servers page)

1. User clicks "Export" button (next to Import)
2. Export dialog opens showing:
   - Server selection: checkboxes for each server, "Select All" option
   - ENV handling: toggle "Mask sensitive values" (on by default)
   - Preview of which env vars will be masked
3. Two action buttons: "Copy to Clipboard" / "Save to File"
4. Success feedback shown

### Per-Server Quick Export

1. User clicks export icon on a server card
2. Small dropdown with:
   - "Copy to Clipboard" (with masking)
   - "Copy to Clipboard (with values)"
   - "Save to File..."
3. Instant action, no dialog needed for clipboard options

### Smart ENV Detection

**Sensitive key patterns to mask:**
- Keys containing: `KEY`, `TOKEN`, `SECRET`, `PASSWORD`, `CREDENTIAL`, `AUTH`, `API`

**Sensitive value patterns to mask:**
- API key prefixes: `sk-*`, `ghp_*`, `xox[baprs]-*` (Slack)
- JWT tokens: `eyJ*`

**Masked format:** `"API_KEY": "<API_KEY>"` (placeholder uses the key name)

## UI Components

### Export Dialog Layout

```
┌─ Export MCP Servers ─────────────────────────────┐
│                                                  │
│ ☑ Select All (3 servers)                        │
│ ┌──────────────────────────────────────────────┐ │
│ │ ☑ filesystem                                 │ │
│ │   Command: npx                               │ │
│ │   Env: (none)                                │ │
│ ├──────────────────────────────────────────────┤ │
│ │ ☑ github                                     │ │
│ │   Command: npx                               │ │
│ │   Env: GITHUB_TOKEN → will be masked         │ │
│ ├──────────────────────────────────────────────┤ │
│ │ ☐ slack                                      │ │
│ │   Command: npx                               │ │
│ │   Env: SLACK_TOKEN → will be masked          │ │
│ └──────────────────────────────────────────────┘ │
│                                                  │
│ ☑ Mask sensitive values (recommended)           │
│   API keys, tokens, and secrets will be         │
│   replaced with placeholders                    │
│                                                  │
│              [Copy to Clipboard] [Save to File] │
└──────────────────────────────────────────────────┘
```

### Per-Server Export Menu

- Small dropdown on hover/click of export icon on server cards
- 3 options: Copy (masked), Copy (with values), Save to File

## Implementation Details

### Files to Create/Modify

| File | Changes |
|------|---------|
| `src/lib/exportConfig.ts` | New - export logic, sensitive detection |
| `src/components/ExportDialog.tsx` | New - main export dialog |
| `src/pages/Servers.tsx` | Add Export button, per-server export menu |

### Sensitive Value Detection

```typescript
const SENSITIVE_KEY_PATTERNS = [
  /key/i, /token/i, /secret/i, /password/i,
  /credential/i, /auth/i, /api/i
];

const SENSITIVE_VALUE_PATTERNS = [
  /^sk-/, /^ghp_/, /^xox[baprs]-/, // Slack tokens
  /^eyJ/, // JWT tokens
];

function isSensitive(key: string, value: string): boolean {
  return SENSITIVE_KEY_PATTERNS.some(p => p.test(key)) ||
         SENSITIVE_VALUE_PATTERNS.some(p => p.test(value));
}
```

### Export Format

Standard MCP config format compatible with Claude Desktop/Code:

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "<GITHUB_TOKEN>"
      }
    }
  }
}
```

### APIs Used

- **Clipboard:** `navigator.clipboard.writeText()`
- **File Save:** Tauri's `@tauri-apps/plugin-dialog` save dialog with `{ filters: [{ name: 'JSON', extensions: ['json'] }] }`

## Acceptance Criteria

- [ ] Export button appears on Servers page next to Import
- [ ] Export dialog shows server list with checkboxes
- [ ] "Select All" toggles all servers
- [ ] Sensitive env vars are detected and shown as "will be masked"
- [ ] "Mask sensitive values" toggle controls masking behavior
- [ ] "Copy to Clipboard" copies JSON and shows success toast
- [ ] "Save to File" opens file picker and saves JSON
- [ ] Per-server export menu appears on server cards
- [ ] Quick export options work (masked, with values, save)
- [ ] Exported JSON is valid and importable
