import { useEffect, useState } from "react";
import { Moon, Sun, Monitor, FolderOpen, Save } from "lucide-react";
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
import type { AppSettings } from "@/types";
import { invoke } from "@tauri-apps/api/core";

export function Settings() {
  const { settings, loadSettings, saveSettings } = useStore();
  const [localSettings, setLocalSettings] = useState<AppSettings>(settings);
  const [appDataDir, setAppDataDir] = useState<string>("");
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadSettings();
    invoke<string>("get_app_data_dir").then(setAppDataDir).catch(console.error);
  }, [loadSettings]);

  useEffect(() => {
    setLocalSettings(settings);
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
    } catch (error) {
      console.error("Failed to save settings:", error);
    } finally {
      setSaving(false);
    }
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
