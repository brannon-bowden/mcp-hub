import { useEffect, useState, useCallback } from "react";
import {
  Plus,
  RefreshCw,
  Pencil,
  Trash2,
  Layers,
  CheckCircle,
  Clock,
  FolderOpen,
  Download,
  AlertCircle,
  Search,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useStore } from "@/store";
import { CLIENT_TYPE_LABELS, type ClientInstance, type ClientType, type McpServerEntry } from "@/types";

interface InstanceFormData {
  name: string;
  clientType: ClientType;
  configPath: string;
  isDefault: boolean;
}

const emptyFormData: InstanceFormData = {
  name: "",
  clientType: "claude-desktop",
  configPath: "",
  isDefault: false,
};

export function Instances() {
  const {
    servers,
    instances,
    loadServers,
    loadInstances,
    createInstance,
    updateInstance,
    deleteInstance,
    setServerEnabled,
    syncInstance,
    detectClients,
    detectedClients,
    readConfigFile,
    importFromFile,
  } = useStore();

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isServersDialogOpen, setIsServersDialogOpen] = useState(false);
  const [editingInstance, setEditingInstance] = useState<ClientInstance | null>(
    null
  );
  const [selectedInstance, setSelectedInstance] =
    useState<ClientInstance | null>(null);
  const [formData, setFormData] = useState<InstanceFormData>(emptyFormData);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [instanceToDelete, setInstanceToDelete] =
    useState<ClientInstance | null>(null);
  const [syncing, setSyncing] = useState<string | null>(null);
  const [existingConfigServers, setExistingConfigServers] = useState<Record<string, McpServerEntry> | null>(null);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [importing, setImporting] = useState(false);
  const [clientTypeSearch, setClientTypeSearch] = useState("");

  useEffect(() => {
    loadServers();
    loadInstances();
    detectClients();
  }, [loadServers, loadInstances, detectClients]);

  // Helper to determine if an instance needs to be synced
  const needsSync = (instance: ClientInstance): boolean => {
    // Never synced = needs sync
    if (!instance.lastSynced) return true;
    // Modified after last sync = needs sync
    if (instance.lastModified) {
      return new Date(instance.lastModified) > new Date(instance.lastSynced);
    }
    return false;
  };

  // Check for existing config when config path changes
  const checkForExistingConfig = useCallback(async (configPath: string) => {
    if (!configPath) {
      setExistingConfigServers(null);
      return;
    }

    const config = await readConfigFile(configPath);
    if (config && config.mcpServers && Object.keys(config.mcpServers).length > 0) {
      setExistingConfigServers(config.mcpServers);
    } else {
      setExistingConfigServers(null);
    }
  }, [readConfigFile]);

  const handleImportServers = async () => {
    if (!formData.configPath) return;

    setImporting(true);
    try {
      await importFromFile(formData.configPath);
      setIsImportDialogOpen(false);
      setExistingConfigServers(null);
    } catch (error) {
      console.error("Failed to import servers:", error);
    } finally {
      setImporting(false);
    }
  };

  const handleOpenDialog = async (instance?: ClientInstance) => {
    setExistingConfigServers(null);
    if (instance) {
      setEditingInstance(instance);
      setFormData({
        name: instance.name,
        clientType: instance.clientType,
        configPath: instance.configPath,
        isDefault: instance.isDefault,
      });
    } else {
      setEditingInstance(null);
      // Pre-fill with detected client if available
      if (detectedClients.length > 0) {
        const first = detectedClients[0];
        const newFormData = {
          name: CLIENT_TYPE_LABELS[first.clientType],
          clientType: first.clientType,
          configPath: first.configPath,
          isDefault: false,
        };
        setFormData(newFormData);
        // Check for existing config
        if (first.hasConfig) {
          await checkForExistingConfig(first.configPath);
        }
      } else {
        setFormData(emptyFormData);
      }
    }
    setIsDialogOpen(true);
  };

  const handleCloseDialog = () => {
    setIsDialogOpen(false);
    setEditingInstance(null);
    setFormData(emptyFormData);
    setExistingConfigServers(null);
    setClientTypeSearch("");
  };

  const handleSubmit = async () => {
    try {
      const now = new Date().toISOString();

      if (editingInstance) {
        await updateInstance({
          ...editingInstance,
          name: formData.name,
          clientType: formData.clientType,
          configPath: formData.configPath,
          isDefault: formData.isDefault,
        });
      } else {
        await createInstance({
          id: crypto.randomUUID(),
          name: formData.name,
          clientType: formData.clientType,
          configPath: formData.configPath,
          enabledServers: [],
          isDefault: formData.isDefault,
          createdAt: now,
        });
      }

      handleCloseDialog();
    } catch (error) {
      console.error("Failed to save instance:", error);
    }
  };

  const handleDelete = async () => {
    if (instanceToDelete) {
      try {
        await deleteInstance(instanceToDelete.id);
        setIsDeleteDialogOpen(false);
        setInstanceToDelete(null);
      } catch (error) {
        console.error("Failed to delete instance:", error);
      }
    }
  };

  const confirmDelete = (instance: ClientInstance) => {
    setInstanceToDelete(instance);
    setIsDeleteDialogOpen(true);
  };

  const handleOpenServersDialog = (instance: ClientInstance) => {
    setSelectedInstance(instance);
    setIsServersDialogOpen(true);
  };

  const handleServerToggle = async (serverId: string, enabled: boolean) => {
    if (selectedInstance) {
      try {
        await setServerEnabled(selectedInstance.id, serverId, enabled);
        // Update local state
        setSelectedInstance((prev) => {
          if (!prev) return prev;
          const enabledServers = enabled
            ? [...prev.enabledServers, serverId]
            : prev.enabledServers.filter((id) => id !== serverId);
          return { ...prev, enabledServers };
        });
      } catch (error) {
        console.error("Failed to toggle server:", error);
      }
    }
  };

  const handleSync = async (instanceId: string) => {
    setSyncing(instanceId);
    try {
      await syncInstance(instanceId);
    } catch (error) {
      console.error("Failed to sync:", error);
    } finally {
      setSyncing(null);
    }
  };

  const handleClientTypeChange = async (clientType: ClientType) => {
    setExistingConfigServers(null);
    // Auto-fill config path from detected clients
    const detected = detectedClients.find((c) => c.clientType === clientType);

    // Update name if it's empty or matches any client type label (i.e., not customized)
    const isNameAClientLabel = Object.values(CLIENT_TYPE_LABELS).includes(formData.name);
    const newName = (!formData.name || isNameAClientLabel) ? CLIENT_TYPE_LABELS[clientType] : formData.name;

    if (detected) {
      setFormData({
        ...formData,
        clientType,
        configPath: detected.configPath,
        name: newName,
      });
      // Check for existing config
      if (detected.hasConfig) {
        await checkForExistingConfig(detected.configPath);
      }
    } else {
      setFormData({ ...formData, clientType, name: newName });
    }
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Client Instances</h1>
          <p className="text-muted-foreground mt-2">
            Manage MCP client configurations and server assignments
          </p>
        </div>
        <Button onClick={() => handleOpenDialog()}>
          <Plus className="w-4 h-4 mr-2" />
          Add Instance
        </Button>
      </div>

      {/* Detected Clients Info */}
      {detectedClients.length > 0 && instances.length === 0 && (
        <Card className="mb-6 border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950">
          <CardContent className="py-4">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center">
                <Layers className="w-4 h-4 text-white" />
              </div>
              <div>
                <p className="font-medium">
                  Detected {detectedClients.length} MCP client
                  {detectedClients.length === 1 ? "" : "s"}
                </p>
                <p className="text-sm text-muted-foreground">
                  {detectedClients.map((c) => CLIENT_TYPE_LABELS[c.clientType]).join(", ")}
                </p>
              </div>
              <Button
                variant="secondary"
                size="sm"
                className="ml-auto"
                onClick={() => handleOpenDialog()}
              >
                Set Up Instance
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Instances List */}
      {instances.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Layers className="w-12 h-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No instances configured</h3>
            <p className="text-muted-foreground text-center mb-4">
              Set up client instances to deploy MCP servers to your AI
              applications
            </p>
            <Button onClick={() => handleOpenDialog()}>
              <Plus className="w-4 h-4 mr-2" />
              Add Instance
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {instances.map((instance) => {
            const instanceNeedsSync = needsSync(instance);
            return (
            <Card key={instance.id}>
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-3 h-3 rounded-full ${
                        instanceNeedsSync ? "bg-yellow-500" : "bg-green-500"
                      }`}
                      title={instanceNeedsSync ? "Changes pending sync" : "Up to date"}
                    />
                    <div>
                      <CardTitle className="text-lg flex items-center gap-2">
                        {instance.name}
                        {instance.isDefault && (
                          <Badge variant="secondary" className="text-xs">
                            Default
                          </Badge>
                        )}
                        {instanceNeedsSync && (
                          <Badge variant="outline" className="text-xs text-yellow-600 border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:text-yellow-400">
                            Needs sync
                          </Badge>
                        )}
                      </CardTitle>
                      <CardDescription>
                        {CLIENT_TYPE_LABELS[instance.clientType]}
                      </CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      variant={instanceNeedsSync ? "default" : "outline"}
                      size="sm"
                      onClick={() => handleSync(instance.id)}
                      disabled={syncing === instance.id}
                    >
                      <RefreshCw
                        className={`w-4 h-4 mr-2 ${syncing === instance.id ? "animate-spin" : ""}`}
                      />
                      Sync
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleOpenDialog(instance)}
                    >
                      <Pencil className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="text-destructive"
                      onClick={() => confirmDelete(instance)}
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex items-center justify-between">
                  <div className="space-y-1">
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <FolderOpen className="w-4 h-4" />
                      <code className="text-xs">{instance.configPath}</code>
                    </div>
                    <div className="flex items-center gap-4 text-sm">
                      <span className="flex items-center gap-1">
                        <CheckCircle className="w-4 h-4 text-green-500" />
                        {instance.enabledServers.length} server
                        {instance.enabledServers.length === 1 ? "" : "s"} enabled
                      </span>
                      {instance.lastSynced && (
                        <span className="flex items-center gap-1 text-muted-foreground">
                          <Clock className="w-4 h-4" />
                          Last synced{" "}
                          {new Date(instance.lastSynced).toLocaleString()}
                        </span>
                      )}
                    </div>
                  </div>
                  <Button
                    variant="secondary"
                    onClick={() => handleOpenServersDialog(instance)}
                  >
                    Configure Servers
                  </Button>
                </div>
              </CardContent>
            </Card>
          );
          })}
        </div>
      )}

      {/* Add/Edit Instance Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {editingInstance ? "Edit Instance" : "Add Instance"}
            </DialogTitle>
            <DialogDescription>
              Configure an MCP client instance
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="My Claude Desktop"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label>Client Type</Label>
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <Input
                  placeholder="Search clients..."
                  value={clientTypeSearch}
                  onChange={(e) => setClientTypeSearch(e.target.value)}
                  className="pl-9"
                />
              </div>
              <div className="grid grid-cols-2 gap-2 max-h-72 overflow-y-auto pr-1">
                {(
                  [
                    "amazon-q",
                    "amp",
                    "antigravity",
                    "augment",
                    "boltai",
                    "claude-code",
                    "claude-desktop",
                    "cline",
                    "cody",
                    "continue",
                    "copilot-agent",
                    "copilot-cli",
                    "crush",
                    "cursor",
                    "emdash",
                    "factory",
                    "gemini-cli",
                    "jetbrains",
                    "kilo-code",
                    "kiro",
                    "lm-studio",
                    "openai-codex",
                    "opencode",
                    "perplexity",
                    "qodo-gen",
                    "qwen-coder",
                    "roo-code",
                    "rovo-dev",
                    "smithery",
                    "trae",
                    "visual-studio",
                    "vscode",
                    "vscode-insiders",
                    "warp",
                    "windsurf",
                    "zed",
                    "zencoder",
                    "custom",
                  ] as ClientType[]
                )
                  .filter((type) =>
                    CLIENT_TYPE_LABELS[type]
                      .toLowerCase()
                      .includes(clientTypeSearch.toLowerCase())
                  )
                  .map((type) => (
                    <button
                      key={type}
                      type="button"
                      onClick={() => handleClientTypeChange(type)}
                      className={`p-3 rounded-lg border text-left transition-colors ${
                        formData.clientType === type
                          ? "border-primary bg-primary/5"
                          : "border-input hover:bg-muted"
                      }`}
                    >
                      <div className="font-medium text-sm">
                        {CLIENT_TYPE_LABELS[type]}
                      </div>
                    </button>
                  ))}
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="configPath">Config Path</Label>
              <Input
                id="configPath"
                placeholder="/path/to/config.json"
                value={formData.configPath}
                onChange={(e) =>
                  setFormData({ ...formData, configPath: e.target.value })
                }
              />
              <p className="text-xs text-muted-foreground">
                The path to the MCP configuration file for this client
              </p>
            </div>
            {/* Existing Config Detected Banner */}
            {existingConfigServers && !editingInstance && (
              <div className="rounded-lg border border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950 p-4">
                <div className="flex items-start gap-3">
                  <AlertCircle className="w-5 h-5 text-blue-500 mt-0.5 flex-shrink-0" />
                  <div className="flex-1 space-y-2">
                    <p className="font-medium text-sm">Existing configuration found</p>
                    <p className="text-xs text-muted-foreground">
                      Found {Object.keys(existingConfigServers).length} server{Object.keys(existingConfigServers).length === 1 ? "" : "s"} in this config file:
                    </p>
                    <ul className="text-xs text-muted-foreground list-disc list-inside">
                      {Object.keys(existingConfigServers).slice(0, 5).map((name) => (
                        <li key={name}>{name}</li>
                      ))}
                      {Object.keys(existingConfigServers).length > 5 && (
                        <li>...and {Object.keys(existingConfigServers).length - 5} more</li>
                      )}
                    </ul>
                    <Button
                      type="button"
                      size="sm"
                      variant="secondary"
                      onClick={() => setIsImportDialogOpen(true)}
                      className="mt-2"
                    >
                      <Download className="w-4 h-4 mr-2" />
                      Import to Server Registry
                    </Button>
                  </div>
                </div>
              </div>
            )}
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="isDefault">Default Instance</Label>
                <p className="text-xs text-muted-foreground">
                  Mark as the default instance for this client type
                </p>
              </div>
              <Switch
                id="isDefault"
                checked={formData.isDefault}
                onCheckedChange={(checked) =>
                  setFormData({ ...formData, isDefault: checked })
                }
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={handleCloseDialog}>
              Cancel
            </Button>
            <Button
              onClick={handleSubmit}
              disabled={!formData.name || !formData.configPath}
            >
              {editingInstance ? "Save Changes" : "Add Instance"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Configure Servers Dialog */}
      <Dialog open={isServersDialogOpen} onOpenChange={setIsServersDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Configure Servers</DialogTitle>
            <DialogDescription>
              Select which servers to enable for {selectedInstance?.name}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4 max-h-96 overflow-auto">
            {servers.length === 0 ? (
              <p className="text-center text-muted-foreground py-8">
                No servers configured. Add some servers first.
              </p>
            ) : (
              <div className="space-y-3">
                {servers.map((server) => {
                  const isEnabled =
                    selectedInstance?.enabledServers.includes(server.id) ??
                    false;
                  return (
                    <div
                      key={server.id}
                      className="flex items-center justify-between p-3 rounded-lg border"
                    >
                      <div className="flex items-center gap-3">
                        <Checkbox
                          id={server.id}
                          checked={isEnabled}
                          onCheckedChange={(checked) =>
                            handleServerToggle(server.id, checked === true)
                          }
                        />
                        <div>
                          <label
                            htmlFor={server.id}
                            className="font-medium cursor-pointer"
                          >
                            {server.name}
                          </label>
                          <p className="text-xs text-muted-foreground">
                            {server.command} {server.args.join(" ")}
                          </p>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setIsServersDialogOpen(false)}>Done</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Instance</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{instanceToDelete?.name}"? This
              will not delete the actual config file.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setIsDeleteDialogOpen(false)}
            >
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDelete}>
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Import Servers Confirmation Dialog */}
      <Dialog open={isImportDialogOpen} onOpenChange={setIsImportDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Import Servers</DialogTitle>
            <DialogDescription>
              Import {existingConfigServers ? Object.keys(existingConfigServers).length : 0} server{existingConfigServers && Object.keys(existingConfigServers).length === 1 ? "" : "s"} from this config file into your server registry?
            </DialogDescription>
          </DialogHeader>
          {existingConfigServers && (
            <div className="py-4 max-h-64 overflow-auto">
              <div className="space-y-2">
                {Object.entries(existingConfigServers).map(([name, entry]) => (
                  <div key={name} className="p-3 rounded-lg border">
                    <p className="font-medium text-sm">{name}</p>
                    <p className="text-xs text-muted-foreground font-mono">
                      {entry.command} {entry.args.join(" ")}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setIsImportDialogOpen(false)}
              disabled={importing}
            >
              Cancel
            </Button>
            <Button onClick={handleImportServers} disabled={importing}>
              {importing ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  <Download className="w-4 h-4 mr-2" />
                  Import Servers
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
