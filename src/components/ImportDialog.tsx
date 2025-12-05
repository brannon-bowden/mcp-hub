import { useState, useEffect, useMemo } from "react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useStore } from "@/store";
import type { RegistryServer, RegistrySource, DetectedClient, ClientType } from "@/types";
import { CLIENT_TYPE_LABELS } from "@/types";
import {
  FileJson,
  Download,
  Package,
  Loader2,
  ExternalLink,
  Check,
  FolderOpen,
  Search,
  X,
  ShieldCheck,
  Star,
  Hammer,
  LayoutGrid,
} from "lucide-react";

interface ImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

// Category definitions with display names and included tags
const CATEGORIES = [
  { id: "all", label: "All", tags: [] },
  { id: "official", label: "Official", tags: ["official"] },
  { id: "database", label: "Databases", tags: ["database", "sql", "nosql"] },
  { id: "productivity", label: "Productivity", tags: ["productivity", "notes", "calendar", "email"] },
  { id: "dev-tools", label: "Dev Tools", tags: ["development", "devops", "docker", "kubernetes", "shell", "vscode"] },
  { id: "ai-search", label: "AI & Search", tags: ["ai", "search", "llm"] },
  { id: "cloud", label: "Cloud", tags: ["cloud", "aws", "vercel", "cloudflare"] },
  { id: "messaging", label: "Messaging", tags: ["messaging", "chat", "communication"] },
  { id: "media", label: "Media", tags: ["media", "youtube", "spotify", "music", "video"] },
  { id: "project-mgmt", label: "Project Mgmt", tags: ["project-management", "jira", "asana", "trello"] },
  { id: "other", label: "Other", tags: [] },
] as const;

// Icon mapping for registry sources
const getRegistryIcon = (icon?: string) => {
  switch (icon) {
    case "package":
      return <Package className="h-4 w-4" />;
    case "shield-check":
      return <ShieldCheck className="h-4 w-4" />;
    case "star":
      return <Star className="h-4 w-4" />;
    case "hammer":
      return <Hammer className="h-4 w-4" />;
    case "layout-grid":
      return <LayoutGrid className="h-4 w-4" />;
    case "download":
      return <Download className="h-4 w-4" />;
    default:
      return <Package className="h-4 w-4" />;
  }
};

export function ImportDialog({ open, onOpenChange }: ImportDialogProps) {
  const {
    importFromFile,
    getRegistries,
    getRegistryServers,
    importFromRegistry,
    detectClients,
    detectedClients,
    servers,
  } = useStore();

  const [activeTab, setActiveTab] = useState("registry");
  const [registries, setRegistries] = useState<RegistrySource[]>([]);
  const [selectedRegistry, setSelectedRegistry] = useState<string>("builtin");
  const [registryServers, setRegistryServers] = useState<RegistryServer[]>([]);
  const [selectedServers, setSelectedServers] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [loadingRegistries, setLoadingRegistries] = useState(false);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("all");

  // Load registries on open
  useEffect(() => {
    if (open) {
      loadRegistries();
      detectClients();
      setSearchQuery("");
      setSelectedCategory("all");
      setSelectedServers(new Set());
    }
  }, [open]);

  // Load servers when registry changes
  useEffect(() => {
    if (open && selectedRegistry) {
      loadRegistryServers(selectedRegistry);
    }
  }, [selectedRegistry, open]);

  const loadRegistries = async () => {
    setLoadingRegistries(true);
    try {
      const regs = await getRegistries();
      setRegistries(regs);
      if (regs.length > 0 && !selectedRegistry) {
        setSelectedRegistry(regs[0].id);
      }
    } catch (err) {
      console.error("Failed to load registries:", err);
    } finally {
      setLoadingRegistries(false);
    }
  };

  const loadRegistryServers = async (registryId: string) => {
    setLoading(true);
    setError(null);
    setSelectedServers(new Set());
    try {
      const servers = await getRegistryServers(registryId);
      setRegistryServers(servers);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load registry");
      setRegistryServers([]);
    } finally {
      setLoading(false);
    }
  };

  // Filter servers based on search query and category
  const filteredServers = useMemo(() => {
    let filtered = registryServers;

    // Filter by category
    if (selectedCategory !== "all") {
      const category = CATEGORIES.find((c) => c.id === selectedCategory);
      if (category && category.tags.length > 0) {
        filtered = filtered.filter((server) =>
          server.tags.some((tag) =>
            category.tags.some((catTag) => tag.toLowerCase().includes(catTag.toLowerCase()))
          )
        );
      } else if (selectedCategory === "other") {
        // "Other" category: servers that don't match any specific category
        const allCategoryTags = CATEGORIES.flatMap((c) => c.tags).filter(Boolean);
        filtered = filtered.filter(
          (server) =>
            !server.tags.some((tag) =>
              allCategoryTags.some((catTag) => tag.toLowerCase().includes(catTag.toLowerCase()))
            )
        );
      }
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (server) =>
          server.name.toLowerCase().includes(query) ||
          server.description?.toLowerCase().includes(query) ||
          server.tags.some((tag) => tag.toLowerCase().includes(query))
      );
    }

    return filtered;
  }, [registryServers, searchQuery, selectedCategory]);

  // Count servers per category for badges
  const categoryCounts = useMemo(() => {
    const counts: Record<string, number> = { all: registryServers.length };

    for (const category of CATEGORIES) {
      if (category.id === "all") continue;

      if (category.id === "other") {
        const allCategoryTags = CATEGORIES.flatMap((c) => c.tags).filter(Boolean);
        counts[category.id] = registryServers.filter(
          (server) =>
            !server.tags.some((tag) =>
              allCategoryTags.some((catTag) => tag.toLowerCase().includes(catTag.toLowerCase()))
            )
        ).length;
      } else if (category.tags.length > 0) {
        counts[category.id] = registryServers.filter((server) =>
          server.tags.some((tag) =>
            category.tags.some((catTag) => tag.toLowerCase().includes(catTag.toLowerCase()))
          )
        ).length;
      }
    }

    return counts;
  }, [registryServers]);

  const toggleServer = (name: string) => {
    const newSelected = new Set(selectedServers);
    if (newSelected.has(name)) {
      newSelected.delete(name);
    } else {
      newSelected.add(name);
    }
    setSelectedServers(newSelected);
  };

  const selectAllVisible = () => {
    const existingNames = new Set(servers.map((s) => s.name.toLowerCase()));
    const newServers = filteredServers.filter(
      (s) => !existingNames.has(s.name.toLowerCase())
    );
    const newSelected = new Set(selectedServers);
    newServers.forEach((s) => newSelected.add(s.name));
    setSelectedServers(newSelected);
  };

  const selectNone = () => {
    setSelectedServers(new Set());
  };

  const handleImportFromRegistry = async () => {
    if (selectedServers.size === 0) return;

    setImporting(true);
    setError(null);
    try {
      const serversToImport = registryServers.filter((s) =>
        selectedServers.has(s.name)
      );
      const imported = await importFromRegistry(selectedRegistry, serversToImport);
      setSuccessMessage(`Successfully imported ${imported.length} server(s)`);
      setSelectedServers(new Set());
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

  const handleImportFromFile = async () => {
    try {
      const selected = await openFileDialog({
        multiple: false,
        filters: [
          {
            name: "JSON",
            extensions: ["json"],
          },
        ],
      });

      if (selected) {
        setImporting(true);
        setError(null);
        const imported = await importFromFile(selected);
        setSuccessMessage(`Successfully imported ${imported.length} server(s)`);
        setTimeout(() => {
          onOpenChange(false);
          setSuccessMessage(null);
        }, 1500);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to import file");
    } finally {
      setImporting(false);
    }
  };

  const handleImportFromClient = async (client: DetectedClient) => {
    if (!client.hasConfig) return;

    setImporting(true);
    setError(null);
    try {
      const imported = await importFromFile(client.configPath);
      setSuccessMessage(`Successfully imported ${imported.length} server(s)`);
      setTimeout(() => {
        onOpenChange(false);
        setSuccessMessage(null);
      }, 1500);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to import from client");
    } finally {
      setImporting(false);
    }
  };

  const isServerAlreadyAdded = (name: string) => {
    return servers.some((s) => s.name.toLowerCase() === name.toLowerCase());
  };

  const getClientLabel = (clientType: string) => {
    return CLIENT_TYPE_LABELS[clientType as ClientType] || clientType;
  };

  const currentRegistry = registries.find((r) => r.id === selectedRegistry);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Import MCP Servers</DialogTitle>
          <DialogDescription>
            Import servers from registries, a config file, or an existing client
          </DialogDescription>
        </DialogHeader>

        {successMessage && (
          <div className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-md text-green-600 dark:text-green-400">
            <Check className="h-4 w-4" />
            {successMessage}
          </div>
        )}

        {error && (
          <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
            {error}
          </div>
        )}

        <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col min-h-0">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="registry" className="flex items-center gap-2">
              <Package className="h-4 w-4" />
              Registry
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

          <TabsContent value="registry" className="flex-1 flex flex-col min-h-0 mt-4">
            {/* Registry Selector */}
            <div className="mb-4">
              <Select
                value={selectedRegistry}
                onValueChange={setSelectedRegistry}
                disabled={loadingRegistries}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select a registry" />
                </SelectTrigger>
                <SelectContent>
                  {registries.map((registry) => (
                    <SelectItem key={registry.id} value={registry.id}>
                      <div className="flex items-center gap-2">
                        {getRegistryIcon(registry.icon)}
                        <span>{registry.name}</span>
                        {registry.serverCount && (
                          <span className="text-muted-foreground text-xs">
                            ({registry.serverCount}+ servers)
                          </span>
                        )}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {currentRegistry && (
                <p className="text-xs text-muted-foreground mt-1.5 px-1">
                  {currentRegistry.description}
                </p>
              )}
            </div>

            {/* Search and Filter Bar */}
            <div className="space-y-3 mb-4">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search servers by name, description, or tag..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9 pr-9"
                />
                {searchQuery && (
                  <button
                    onClick={() => setSearchQuery("")}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  >
                    <X className="h-4 w-4" />
                  </button>
                )}
              </div>

              {/* Category Pills */}
              <div className="flex flex-wrap gap-2">
                {CATEGORIES.map((category) => (
                  <button
                    key={category.id}
                    onClick={() => setSelectedCategory(category.id)}
                    className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
                      selectedCategory === category.id
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted hover:bg-muted/80 text-muted-foreground"
                    }`}
                  >
                    {category.label}
                    {categoryCounts[category.id] > 0 && (
                      <span className="ml-1.5 opacity-70">
                        ({categoryCounts[category.id]})
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex items-center justify-between mb-3">
              <p className="text-sm text-muted-foreground">
                {filteredServers.length} server{filteredServers.length === 1 ? "" : "s"}
                {searchQuery || selectedCategory !== "all" ? " found" : " available"}
              </p>
              <div className="flex gap-2">
                <Button variant="outline" size="sm" onClick={selectAllVisible}>
                  Select Visible
                </Button>
                <Button variant="outline" size="sm" onClick={selectNone}>
                  Clear
                </Button>
              </div>
            </div>

            {loading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : filteredServers.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <Search className="h-8 w-8 mb-2" />
                <p>No servers found matching your criteria</p>
                <Button
                  variant="link"
                  size="sm"
                  onClick={() => {
                    setSearchQuery("");
                    setSelectedCategory("all");
                  }}
                >
                  Clear filters
                </Button>
              </div>
            ) : (
              <div className="flex-1 overflow-y-auto space-y-2 pr-2 min-h-0 max-h-[300px]">
                {filteredServers.map((server) => {
                  const alreadyAdded = isServerAlreadyAdded(server.name);
                  return (
                    <div
                      key={server.name}
                      className={`flex items-start gap-3 p-3 rounded-lg border ${
                        alreadyAdded
                          ? "bg-muted/50 opacity-60"
                          : selectedServers.has(server.name)
                          ? "border-primary bg-primary/5"
                          : "hover:bg-muted/50"
                      }`}
                    >
                      <Checkbox
                        checked={selectedServers.has(server.name)}
                        onCheckedChange={() => toggleServer(server.name)}
                        disabled={alreadyAdded}
                        className="mt-1"
                      />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="font-medium">{server.name}</span>
                          {alreadyAdded && (
                            <Badge variant="secondary" className="text-xs">
                              Already added
                            </Badge>
                          )}
                          {server.tags.includes("official") && (
                            <Badge variant="outline" className="text-xs">
                              Official
                            </Badge>
                          )}
                        </div>
                        {server.description && (
                          <p className="text-sm text-muted-foreground mt-1 line-clamp-2">
                            {server.description}
                          </p>
                        )}
                        <div className="flex items-center gap-2 mt-2 flex-wrap">
                          {server.tags
                            .filter((t) => t !== "official")
                            .slice(0, 4)
                            .map((tag) => (
                              <Badge key={tag} variant="secondary" className="text-xs">
                                {tag}
                              </Badge>
                            ))}
                          {server.repository && (
                            <a
                              href={server.repository}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                              onClick={(e) => e.stopPropagation()}
                            >
                              <ExternalLink className="h-3 w-3" />
                              Repo
                            </a>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}

            <DialogFooter className="mt-4">
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleImportFromRegistry}
                disabled={selectedServers.size === 0 || importing}
              >
                {importing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Import {selectedServers.size > 0 && `(${selectedServers.size})`}
              </Button>
            </DialogFooter>
          </TabsContent>

          <TabsContent value="file" className="flex-1 mt-4">
            <div className="text-center py-8">
              <FolderOpen className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="font-medium mb-2">Import from Config File</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Select a JSON configuration file containing MCP server definitions
              </p>
              <Button onClick={handleImportFromFile} disabled={importing}>
                {importing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Choose File
              </Button>
            </div>
          </TabsContent>

          <TabsContent value="client" className="flex-1 flex flex-col min-h-0 mt-4">
            <p className="text-sm text-muted-foreground mb-4">
              Import servers from an existing MCP client configuration
            </p>

            {detectedClients.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <p>No MCP clients detected on this system</p>
              </div>
            ) : (
              <div className="flex-1 overflow-y-auto space-y-2 pr-2 min-h-0 max-h-[400px]">
                {detectedClients.map((client) => (
                  <div
                    key={client.clientType}
                    className="flex items-center justify-between p-3 rounded-lg border hover:bg-muted/50"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="font-medium">
                        {getClientLabel(client.clientType)}
                      </div>
                      <div className="text-xs text-muted-foreground truncate">
                        {client.configPath}
                      </div>
                    </div>
                    <Button
                      size="sm"
                      variant={client.hasConfig ? "default" : "outline"}
                      disabled={!client.hasConfig || importing}
                      onClick={() => handleImportFromClient(client)}
                      className="ml-3 flex-shrink-0"
                    >
                      {!client.hasConfig ? "No Config" : "Import"}
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
