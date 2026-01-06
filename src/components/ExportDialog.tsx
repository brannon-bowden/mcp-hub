import { useState, useMemo, useEffect, useRef } from "react";
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
  const selectAllRef = useRef<HTMLButtonElement>(null);

  // Clear copied state after timeout with proper cleanup
  useEffect(() => {
    if (copied) {
      const timeoutId = setTimeout(() => setCopied(false), 2000);
      return () => clearTimeout(timeoutId);
    }
  }, [copied]);

  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setSelectedIds(new Set(preSelectedIds || servers.map((s) => s.id)));
      setMaskSensitive(true);
      setCopied(false);
    }
  }, [open, preSelectedIds, servers]);

  const selectedServers = useMemo(
    () => servers.filter((s) => selectedIds.has(s.id)),
    [servers, selectedIds]
  );

  const allSelected = selectedIds.size === servers.length && servers.length > 0;
  const someSelected = selectedIds.size > 0 && !allSelected;

  // Handle indeterminate state for select-all checkbox
  useEffect(() => {
    if (selectAllRef.current) {
      (selectAllRef.current as HTMLInputElement).indeterminate = someSelected;
    }
  }, [someSelected]);

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
      setCopied(true); // useEffect handles cleanup timeout
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
              ref={selectAllRef}
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
