# Setting Up Auto-Updates

This guide explains how to enable automatic updates for MCP Hub.

## Prerequisites

- Tauri CLI installed (`cargo install tauri-cli`)
- Access to GitHub repository settings

## Step 1: Generate Signing Keypair

The updater requires a signing keypair to verify update authenticity. Generate one using:

```bash
cargo tauri signer generate -w ~/.tauri/mcp-hub.key
```

This creates:
- **Private key**: `~/.tauri/mcp-hub.key` (keep this secret!)
- **Public key**: Displayed in terminal output

Save both the private key file and the password you set.

## Step 2: Add Public Key to tauri.conf.json

Copy the public key (starts with `dW5...` or similar) and add it to `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/brannon-bowden/mcp-hub/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

## Step 3: Add Secrets to GitHub

Go to your repository **Settings > Secrets and variables > Actions** and add:

| Secret Name | Value |
|-------------|-------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `~/.tauri/mcp-hub.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password you set when generating the key |

## Step 4: Test the Setup

1. Create a PR and merge it to `main`
2. The workflow will:
   - Bump the version automatically
   - Build the app for all platforms
   - Create a signed release with update manifest

## How Updates Work

1. **On app startup**: MCP Hub checks GitHub releases for newer versions
2. **If update available**: A banner appears at the top of the app
3. **User clicks "Download"**: Update downloads with progress indicator
4. **User clicks "Restart"**: App restarts with new version

Users can also manually check for updates in **Settings > About**.

## Troubleshooting

### "Update check failed" error
- The public key in `tauri.conf.json` must match the private key in GitHub secrets
- Ensure the release has a valid `latest.json` file

### Updates not appearing
- Check that the GitHub release is published (not draft)
- Verify the version in the release is higher than the installed version

### Signing errors during build
- Verify `TAURI_SIGNING_PRIVATE_KEY` secret contains the full key file contents
- Ensure `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` matches what you set during generation
