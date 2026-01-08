import { useState, useEffect } from "react";
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
import { useStore } from "@/store";
import { CLIENT_TYPE_LABELS } from "@/types";
import type { McpServer } from "@/types";

interface AddToClientsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  server: McpServer | null;
}

export function AddToClientsDialog({
  open,
  onOpenChange,
  server,
}: AddToClientsDialogProps) {
  const { instances, setServerEnabled, syncInstance } = useStore();
  const [selectedInstances, setSelectedInstances] = useState<Set<string>>(new Set());
  const [isSaving, setIsSaving] = useState(false);

  // Reset selection when dialog opens with a new server
  useEffect(() => {
    if (open && server) {
      setSelectedInstances(new Set());
    }
  }, [open, server]);

  const toggleInstance = (instanceId: string) => {
    setSelectedInstances((prev) => {
      const next = new Set(prev);
      if (next.has(instanceId)) {
        next.delete(instanceId);
      } else {
        next.add(instanceId);
      }
      return next;
    });
  };

  const selectAll = () => {
    setSelectedInstances(new Set(instances.map((i) => i.id)));
  };

  const selectNone = () => {
    setSelectedInstances(new Set());
  };

  const handleSubmit = async () => {
    if (!server || selectedInstances.size === 0) {
      onOpenChange(false);
      return;
    }

    setIsSaving(true);
    try {
      // Enable the server on each selected instance
      for (const instanceId of selectedInstances) {
        await setServerEnabled(instanceId, server.id, true);
      }

      // Optionally sync all affected instances
      for (const instanceId of selectedInstances) {
        try {
          await syncInstance(instanceId);
        } catch (err) {
          console.error(`Failed to sync instance ${instanceId}:`, err);
          // Continue with other instances even if one fails
        }
      }

      onOpenChange(false);
    } catch (error) {
      console.error("Failed to add server to clients:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSkip = () => {
    onOpenChange(false);
  };

  if (!server) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Add to Clients</DialogTitle>
          <DialogDescription>
            Would you like to enable "{server.name}" on any of your configured
            clients? The changes will be synced automatically.
          </DialogDescription>
        </DialogHeader>

        {instances.length === 0 ? (
          <div className="py-6 text-center text-muted-foreground">
            <p>No client instances configured yet.</p>
            <p className="text-sm mt-1">
              Add client instances in the Instances page first.
            </p>
          </div>
        ) : (
          <div className="py-4">
            <div className="flex justify-between items-center mb-3">
              <span className="text-sm text-muted-foreground">
                {selectedInstances.size} of {instances.length} selected
              </span>
              <div className="flex gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={selectAll}
                  className="h-7 px-2 text-xs"
                >
                  Select All
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={selectNone}
                  className="h-7 px-2 text-xs"
                >
                  Clear
                </Button>
              </div>
            </div>

            <div className="space-y-2 max-h-[300px] overflow-y-auto">
              {instances.map((instance) => {
                const alreadyEnabled = instance.enabledServers.includes(server.id);
                return (
                  <div
                    key={instance.id}
                    className={`flex items-center space-x-3 p-3 rounded-lg border ${
                      alreadyEnabled
                        ? "bg-muted/50 border-muted"
                        : "hover:bg-accent cursor-pointer"
                    }`}
                    onClick={() => !alreadyEnabled && toggleInstance(instance.id)}
                  >
                    <Checkbox
                      id={instance.id}
                      checked={alreadyEnabled || selectedInstances.has(instance.id)}
                      disabled={alreadyEnabled}
                      onCheckedChange={() => toggleInstance(instance.id)}
                    />
                    <Label
                      htmlFor={instance.id}
                      className={`flex-1 cursor-pointer ${
                        alreadyEnabled ? "text-muted-foreground" : ""
                      }`}
                    >
                      <div className="font-medium">{instance.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {CLIENT_TYPE_LABELS[instance.clientType]}
                        {alreadyEnabled && " (already enabled)"}
                      </div>
                    </Label>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleSkip}>
            Skip
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={selectedInstances.size === 0 || isSaving}
          >
            {isSaving
              ? "Adding..."
              : selectedInstances.size > 0
              ? `Add to ${selectedInstances.size} Client${selectedInstances.size > 1 ? "s" : ""}`
              : "Add to Clients"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
