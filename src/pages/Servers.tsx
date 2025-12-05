import { useEffect, useState } from "react";
import {
  Plus,
  Search,
  Pencil,
  Trash2,
  Download,
  Server as ServerIcon,
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useStore } from "@/store";
import { ImportDialog } from "@/components/ImportDialog";
import type { McpServer } from "@/types";

interface ServerFormData {
  name: string;
  description: string;
  command: string;
  args: string;
  env: string;
  tags: string;
}

const emptyFormData: ServerFormData = {
  name: "",
  description: "",
  command: "",
  args: "",
  env: "",
  tags: "",
};

export function Servers() {
  const { servers, loadServers, createServer, updateServer, deleteServer } =
    useStore();
  const [searchQuery, setSearchQuery] = useState("");
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [editingServer, setEditingServer] = useState<McpServer | null>(null);
  const [formData, setFormData] = useState<ServerFormData>(emptyFormData);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [serverToDelete, setServerToDelete] = useState<McpServer | null>(null);

  useEffect(() => {
    loadServers();
  }, [loadServers]);

  const filteredServers = servers.filter(
    (server) =>
      server.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      server.command.toLowerCase().includes(searchQuery.toLowerCase()) ||
      server.tags.some((tag) =>
        tag.toLowerCase().includes(searchQuery.toLowerCase())
      )
  );

  const handleOpenDialog = (server?: McpServer) => {
    if (server) {
      setEditingServer(server);
      setFormData({
        name: server.name,
        description: server.description || "",
        command: server.command,
        args: server.args.join("\n"),
        env: Object.entries(server.env)
          .map(([k, v]) => `${k}=${v}`)
          .join("\n"),
        tags: server.tags.join(", "),
      });
    } else {
      setEditingServer(null);
      setFormData(emptyFormData);
    }
    setIsDialogOpen(true);
  };

  const handleCloseDialog = () => {
    setIsDialogOpen(false);
    setEditingServer(null);
    setFormData(emptyFormData);
  };

  const handleSubmit = async () => {
    try {
      const args = formData.args
        .split("\n")
        .map((a) => a.trim())
        .filter(Boolean);
      const env: Record<string, string> = {};
      formData.env
        .split("\n")
        .map((e) => e.trim())
        .filter(Boolean)
        .forEach((line) => {
          const [key, ...valueParts] = line.split("=");
          if (key) {
            env[key.trim()] = valueParts.join("=").trim();
          }
        });
      const tags = formData.tags
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);

      const now = new Date().toISOString();

      if (editingServer) {
        await updateServer({
          ...editingServer,
          name: formData.name,
          description: formData.description || undefined,
          command: formData.command,
          args,
          env,
          tags,
          updatedAt: now,
        });
      } else {
        await createServer({
          id: crypto.randomUUID(),
          name: formData.name,
          description: formData.description || undefined,
          command: formData.command,
          args,
          env,
          tags,
          source: { sourceType: "manual" },
          createdAt: now,
          updatedAt: now,
        });
      }

      handleCloseDialog();
    } catch (error) {
      console.error("Failed to save server:", error);
    }
  };

  const handleDelete = async () => {
    if (serverToDelete) {
      try {
        await deleteServer(serverToDelete.id);
        setIsDeleteDialogOpen(false);
        setServerToDelete(null);
      } catch (error) {
        console.error("Failed to delete server:", error);
      }
    }
  };

  const confirmDelete = (server: McpServer) => {
    setServerToDelete(server);
    setIsDeleteDialogOpen(true);
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">MCP Servers</h1>
          <p className="text-muted-foreground mt-2">
            Manage your MCP server configurations
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setIsImportDialogOpen(true)}>
            <Download className="w-4 h-4 mr-2" />
            Import
          </Button>
          <Button onClick={() => handleOpenDialog()}>
            <Plus className="w-4 h-4 mr-2" />
            Add Server
          </Button>
        </div>
      </div>

      {/* Search */}
      <div className="mb-6">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <Input
            placeholder="Search servers..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 max-w-sm"
          />
        </div>
      </div>

      {/* Server List */}
      {filteredServers.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <ServerIcon className="w-12 h-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No servers found</h3>
            <p className="text-muted-foreground text-center mb-4">
              {searchQuery
                ? "Try a different search term"
                : "Get started by adding your first MCP server"}
            </p>
            {!searchQuery && (
              <Button onClick={() => handleOpenDialog()}>
                <Plus className="w-4 h-4 mr-2" />
                Add Server
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredServers.map((server) => (
            <Card key={server.id} className="relative group">
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle className="text-lg">{server.name}</CardTitle>
                    {server.description && (
                      <CardDescription className="mt-1">
                        {server.description}
                      </CardDescription>
                    )}
                  </div>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => handleOpenDialog(server)}
                    >
                      <Pencil className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 text-destructive"
                      onClick={() => confirmDelete(server)}
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm">
                    <code className="px-2 py-1 bg-muted rounded text-xs">
                      {server.command}
                    </code>
                  </div>
                  {server.args.length > 0 && (
                    <div className="text-xs text-muted-foreground truncate">
                      Args: {server.args.join(" ")}
                    </div>
                  )}
                  {server.tags.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-2">
                      {server.tags.map((tag) => (
                        <Badge key={tag} variant="secondary" className="text-xs">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Add/Edit Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {editingServer ? "Edit Server" : "Add Server"}
            </DialogTitle>
            <DialogDescription>
              {editingServer
                ? "Update your MCP server configuration"
                : "Configure a new MCP server"}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="My MCP Server"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description (optional)</Label>
              <Input
                id="description"
                placeholder="A brief description"
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="command">Command</Label>
              <Input
                id="command"
                placeholder="npx"
                value={formData.command}
                onChange={(e) =>
                  setFormData({ ...formData, command: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="args">Arguments (one per line)</Label>
              <Textarea
                id="args"
                placeholder={"-y\n@modelcontextprotocol/server-filesystem\n/path/to/dir"}
                value={formData.args}
                onChange={(e) =>
                  setFormData({ ...formData, args: e.target.value })
                }
                rows={3}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="env">
                Environment Variables (KEY=value, one per line)
              </Label>
              <Textarea
                id="env"
                placeholder="API_KEY=your-key-here"
                value={formData.env}
                onChange={(e) =>
                  setFormData({ ...formData, env: e.target.value })
                }
                rows={2}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="tags">Tags (comma separated)</Label>
              <Input
                id="tags"
                placeholder="filesystem, tools"
                value={formData.tags}
                onChange={(e) =>
                  setFormData({ ...formData, tags: e.target.value })
                }
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={handleCloseDialog}>
              Cancel
            </Button>
            <Button onClick={handleSubmit} disabled={!formData.name || !formData.command}>
              {editingServer ? "Save Changes" : "Add Server"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Server</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{serverToDelete?.name}"? This
              action cannot be undone.
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

      {/* Import Dialog */}
      <ImportDialog
        open={isImportDialogOpen}
        onOpenChange={setIsImportDialogOpen}
      />
    </div>
  );
}
