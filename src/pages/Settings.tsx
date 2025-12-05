import { useEffect, useState, useCallback } from "react";
import {
  Moon,
  Sun,
  Monitor,
  FolderOpen,
  Save,
  Globe,
  FileText,
  RefreshCw,
  ExternalLink,
  CheckCircle2,
  XCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useStore } from "@/store";
import type { AppSettings, DiscoveryStatus, DiscoverySettings } from "@/types";
import { invoke } from "@tauri-apps/api/core";

const DEFAULT_DISCOVERY_SETTINGS: DiscoverySettings = {
  mcpDirectoryEnabled: false,
  httpServerEnabled: false,
  httpServerPort: 24368,
};

export function Settings() {
  const { settings, loadSettings, saveSettings } = useStore();
  const [localSettings, setLocalSettings] = useState<AppSettings>(settings);
  const [appDataDir, setAppDataDir] = useState<string>("");
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);
  const [discoveryStatus, setDiscoveryStatus] =
    useState<DiscoveryStatus | null>(null);
  const [refreshingDiscovery, setRefreshingDiscovery] = useState(false);

  const loadDiscoveryStatus = useCallback(async () => {
    try {
      const status = await invoke<DiscoveryStatus>("get_discovery_status");
      setDiscoveryStatus(status);
    } catch (error) {
      console.error("Failed to load discovery status:", error);
    }
  }, []);

  useEffect(() => {
    loadSettings();
    invoke<string>("get_app_data_dir").then(setAppDataDir).catch(console.error);
    loadDiscoveryStatus();
  }, [loadSettings, loadDiscoveryStatus]);

  useEffect(() => {
    // Ensure discovery settings have defaults
    const settingsWithDefaults = {
      ...settings,
      discovery: settings.discovery || DEFAULT_DISCOVERY_SETTINGS,
    };
    setLocalSettings(settingsWithDefaults);
  }, [settings]);

  useEffect(() => {
    setHasChanges(JSON.stringify(localSettings) !== JSON.stringify(settings));
  }, [localSettings, settings]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveSettings(localSettings);
      // Apply theme
      applyTheme(localSettings.theme);

      // Update discovery settings if changed
      const discoveryChanged =
        JSON.stringify(localSettings.discovery) !==
        JSON.stringify(settings.discovery);
      if (discoveryChanged && localSettings.discovery) {
        await invoke("update_discovery_settings", {
          settings: localSettings.discovery,
        });
        await loadDiscoveryStatus();
      }
    } catch (error) {
      console.error("Failed to save settings:", error);
    } finally {
      setSaving(false);
    }
  };

  const handleRefreshDiscovery = async () => {
    setRefreshingDiscovery(true);
    try {
      await invoke("refresh_discovery");
      await loadDiscoveryStatus();
    } catch (error) {
      console.error("Failed to refresh discovery:", error);
    } finally {
      setRefreshingDiscovery(false);
    }
  };

  const updateDiscoverySettings = (
    updates: Partial<DiscoverySettings>
  ) => {
    setLocalSettings({
      ...localSettings,
      discovery: {
        ...(localSettings.discovery || DEFAULT_DISCOVERY_SETTINGS),
        ...updates,
      },
    });
  };

  const applyTheme = (theme: "light" | "dark" | "system") => {
    const root = document.documentElement;
    if (theme === "system") {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)"
      ).matches;
      root.classList.toggle("dark", prefersDark);
    } else {
      root.classList.toggle("dark", theme === "dark");
    }
  };

  const handleThemeChange = (theme: "light" | "dark" | "system") => {
    setLocalSettings({ ...localSettings, theme });
    applyTheme(theme);
  };

  return (
    <div className="p-8 max-w-2xl">
      <div className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground mt-2">
          Configure MCP Hub preferences
        </p>
      </div>

      <div className="space-y-6">
        {/* Appearance */}
        <Card>
          <CardHeader>
            <CardTitle>Appearance</CardTitle>
            <CardDescription>Customize how MCP Hub looks</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <Label>Theme</Label>
              <div className="grid grid-cols-3 gap-2">
                <button
                  type="button"
                  onClick={() => handleThemeChange("light")}
                  className={`flex flex-col items-center gap-2 p-4 rounded-lg border transition-colors ${
                    localSettings.theme === "light"
                      ? "border-primary bg-primary/5"
                      : "border-input hover:bg-muted"
                  }`}
                >
                  <Sun className="w-5 h-5" />
                  <span className="text-sm font-medium">Light</span>
                </button>
                <button
                  type="button"
                  onClick={() => handleThemeChange("dark")}
                  className={`flex flex-col items-center gap-2 p-4 rounded-lg border transition-colors ${
                    localSettings.theme === "dark"
                      ? "border-primary bg-primary/5"
                      : "border-input hover:bg-muted"
                  }`}
                >
                  <Moon className="w-5 h-5" />
                  <span className="text-sm font-medium">Dark</span>
                </button>
                <button
                  type="button"
                  onClick={() => handleThemeChange("system")}
                  className={`flex flex-col items-center gap-2 p-4 rounded-lg border transition-colors ${
                    localSettings.theme === "system"
                      ? "border-primary bg-primary/5"
                      : "border-input hover:bg-muted"
                  }`}
                >
                  <Monitor className="w-5 h-5" />
                  <span className="text-sm font-medium">System</span>
                </button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Backups */}
        <Card>
          <CardHeader>
            <CardTitle>Backups</CardTitle>
            <CardDescription>
              Configure automatic backup settings
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="createBackups">Create Backups</Label>
                <p className="text-xs text-muted-foreground">
                  Create backups before modifying config files
                </p>
              </div>
              <Switch
                id="createBackups"
                checked={localSettings.createBackups}
                onCheckedChange={(checked) =>
                  setLocalSettings({ ...localSettings, createBackups: checked })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="retentionDays">Backup Retention (days)</Label>
              <Input
                id="retentionDays"
                type="number"
                min="1"
                max="365"
                value={localSettings.backupRetentionDays}
                onChange={(e) =>
                  setLocalSettings({
                    ...localSettings,
                    backupRetentionDays: parseInt(e.target.value) || 30,
                  })
                }
                className="max-w-[120px]"
                disabled={!localSettings.createBackups}
              />
              <p className="text-xs text-muted-foreground">
                Number of days to keep backup files
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Startup */}
        <Card>
          <CardHeader>
            <CardTitle>Startup</CardTitle>
            <CardDescription>Configure startup behavior</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="autoStart">Start on Login</Label>
                <p className="text-xs text-muted-foreground">
                  Automatically start MCP Hub when you log in
                </p>
              </div>
              <Switch
                id="autoStart"
                checked={localSettings.autoStart}
                onCheckedChange={(checked) =>
                  setLocalSettings({ ...localSettings, autoStart: checked })
                }
              />
            </div>
          </CardContent>
        </Card>

        {/* MCP Discovery */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>MCP Discovery</CardTitle>
                <CardDescription>
                  Allow other apps to discover your MCP servers
                </CardDescription>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={handleRefreshDiscovery}
                disabled={refreshingDiscovery}
              >
                <RefreshCw
                  className={`w-4 h-4 mr-2 ${
                    refreshingDiscovery ? "animate-spin" : ""
                  }`}
                />
                Refresh
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* ~/.mcp Directory Discovery */}
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-muted rounded-lg">
                    <FileText className="w-4 h-4" />
                  </div>
                  <div>
                    <Label htmlFor="mcpDirectory">~/.mcp Directory</Label>
                    <p className="text-xs text-muted-foreground">
                      Write server configs to ~/.mcp/ for local discovery
                    </p>
                  </div>
                </div>
                <Switch
                  id="mcpDirectory"
                  checked={localSettings.discovery?.mcpDirectoryEnabled ?? false}
                  onCheckedChange={(checked) =>
                    updateDiscoverySettings({ mcpDirectoryEnabled: checked })
                  }
                />
              </div>
              {discoveryStatus?.mcpDirectoryEnabled && (
                <div className="ml-11 p-3 bg-muted/50 rounded-lg space-y-1">
                  <div className="flex items-center gap-2 text-sm">
                    <CheckCircle2 className="w-4 h-4 text-green-500" />
                    <span>
                      {discoveryStatus.mcpDirectoryFileCount} server
                      {discoveryStatus.mcpDirectoryFileCount !== 1 ? "s" : ""}{" "}
                      published
                    </span>
                  </div>
                  {discoveryStatus.mcpDirectoryPath && (
                    <code className="text-xs text-muted-foreground block">
                      {discoveryStatus.mcpDirectoryPath}
                    </code>
                  )}
                </div>
              )}
            </div>

            <div className="border-t" />

            {/* HTTP Server Discovery */}
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-muted rounded-lg">
                    <Globe className="w-4 h-4" />
                  </div>
                  <div>
                    <Label htmlFor="httpServer">HTTP Discovery Server</Label>
                    <p className="text-xs text-muted-foreground">
                      Run a local server exposing /.well-known/mcp.json
                    </p>
                  </div>
                </div>
                <Switch
                  id="httpServer"
                  checked={localSettings.discovery?.httpServerEnabled ?? false}
                  onCheckedChange={(checked) =>
                    updateDiscoverySettings({ httpServerEnabled: checked })
                  }
                />
              </div>

              {(localSettings.discovery?.httpServerEnabled ||
                discoveryStatus?.httpServerEnabled) && (
                <div className="ml-11 space-y-3">
                  <div className="flex items-center gap-2">
                    <Label htmlFor="httpPort" className="text-sm">
                      Port:
                    </Label>
                    <Input
                      id="httpPort"
                      type="number"
                      min="1024"
                      max="65535"
                      value={localSettings.discovery?.httpServerPort ?? 24368}
                      onChange={(e) =>
                        updateDiscoverySettings({
                          httpServerPort: parseInt(e.target.value) || 24368,
                        })
                      }
                      className="w-24"
                    />
                  </div>

                  {discoveryStatus?.httpServerRunning && (
                    <div className="p-3 bg-muted/50 rounded-lg space-y-2">
                      <div className="flex items-center gap-2 text-sm">
                        <CheckCircle2 className="w-4 h-4 text-green-500" />
                        <span>Server running</span>
                      </div>
                      {discoveryStatus.httpServerUrl && (
                        <a
                          href={`${discoveryStatus.httpServerUrl}/.well-known/mcp.json`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center gap-1 text-xs text-primary hover:underline"
                        >
                          {discoveryStatus.httpServerUrl}/.well-known/mcp.json
                          <ExternalLink className="w-3 h-3" />
                        </a>
                      )}
                    </div>
                  )}

                  {discoveryStatus?.httpServerEnabled &&
                    !discoveryStatus?.httpServerRunning && (
                      <div className="p-3 bg-destructive/10 rounded-lg">
                        <div className="flex items-center gap-2 text-sm text-destructive">
                          <XCircle className="w-4 h-4" />
                          <span>Server not running</span>
                        </div>
                      </div>
                    )}
                </div>
              )}
            </div>

            <div className="border-t pt-4">
              <p className="text-xs text-muted-foreground">
                MCP Discovery allows other AI assistants and tools to find and
                use the MCP servers you've configured. The ~/.mcp directory
                follows the{" "}
                <a
                  href="https://github.com/jonnyzzz/mcp-local-spec"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  mcp-local-spec
                </a>
                , while the HTTP server implements{" "}
                <a
                  href="https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1649"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  SEP-1649
                </a>
                .
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Data Location */}
        <Card>
          <CardHeader>
            <CardTitle>Data Location</CardTitle>
            <CardDescription>
              Where MCP Hub stores its data
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2 p-3 bg-muted rounded-lg">
              <FolderOpen className="w-4 h-4 text-muted-foreground" />
              <code className="text-sm">{appDataDir || "Loading..."}</code>
            </div>
          </CardContent>
        </Card>

        {/* About */}
        <Card>
          <CardHeader>
            <CardTitle>About</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Version</span>
                <span>0.1.0</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Built with</span>
                <span>Tauri + React</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Save Button */}
      {hasChanges && (
        <div className="fixed bottom-8 right-8">
          <Button onClick={handleSave} disabled={saving} size="lg">
            <Save className="w-4 h-4 mr-2" />
            {saving ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      )}
    </div>
  );
}
