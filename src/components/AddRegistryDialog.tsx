import { useState } from "react";
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
import { Label } from "@/components/ui/label";
import { useStore } from "@/store";
import { Loader2, CheckCircle, AlertCircle, Eye, EyeOff } from "lucide-react";
import type { CustomRegistry, CustomRegistryFile } from "@/types";

interface AddRegistryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editRegistry?: CustomRegistry;
  onSuccess?: () => void;
}

export function AddRegistryDialog({
  open,
  onOpenChange,
  editRegistry,
  onSuccess,
}: AddRegistryDialogProps) {
  const { addCustomRegistry, updateCustomRegistry, testCustomRegistryUrl } = useStore();

  const [url, setUrl] = useState(editRegistry?.url || "");
  const [name, setName] = useState(editRegistry?.name || "");
  const [token, setToken] = useState("");
  const [showToken, setShowToken] = useState(false);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testResult, setTestResult] = useState<CustomRegistryFile | null>(null);
  const [error, setError] = useState<string | null>(null);

  const isGitHubUrl = url.includes("github.com") || url.includes("githubusercontent.com");
  const isEditing = !!editRegistry;

  const handleTest = async () => {
    setTesting(true);
    setError(null);
    setTestResult(null);

    try {
      const result = await testCustomRegistryUrl(url, token || undefined);
      setTestResult(result);
      if (!name && result.name) {
        setName(result.name);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);

    try {
      if (isEditing) {
        await updateCustomRegistry(
          editRegistry.id,
          url !== editRegistry.url ? url : undefined,
          name !== editRegistry.name ? name : undefined,
          token || undefined
        );
      } else {
        await addCustomRegistry(url, name || undefined, token || undefined);
      }
      onSuccess?.();
      onOpenChange(false);
      resetForm();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const resetForm = () => {
    setUrl("");
    setName("");
    setToken("");
    setTestResult(null);
    setError(null);
  };

  const handleClose = (open: boolean) => {
    if (!open) {
      resetForm();
    }
    onOpenChange(open);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit" : "Add"} Custom Registry</DialogTitle>
          <DialogDescription>
            {isEditing
              ? "Update your custom registry settings"
              : "Add a registry from a local file or remote URL"}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="url">Registry URL</Label>
            <Input
              id="url"
              placeholder="https://raw.githubusercontent.com/... or /path/to/registry.json"
              value={url}
              onChange={(e) => {
                setUrl(e.target.value);
                setTestResult(null);
              }}
            />
          </div>

          {isGitHubUrl && (
            <div className="space-y-2">
              <Label htmlFor="token">GitHub Token (optional)</Label>
              <div className="relative">
                <Input
                  id="token"
                  type={showToken ? "text" : "password"}
                  placeholder="ghp_xxxx (for private repos)"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowToken(!showToken)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  {showToken ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              <p className="text-xs text-muted-foreground">
                Required for private repositories. Token stored securely in your system keychain.
              </p>
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="name">Name Override (optional)</Label>
            <Input
              id="name"
              placeholder="Will use name from registry file if not set"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          {error && (
            <div className="flex items-center gap-2 p-3 bg-destructive/10 border border-destructive/20 rounded-md text-destructive text-sm">
              <AlertCircle className="h-4 w-4 flex-shrink-0" />
              {error}
            </div>
          )}

          {testResult && (
            <div className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-md text-green-600 dark:text-green-400 text-sm">
              <CheckCircle className="h-4 w-4 flex-shrink-0" />
              Found {testResult.servers.length} server(s) in "{testResult.name}"
            </div>
          )}
        </div>

        <DialogFooter className="flex-col sm:flex-row gap-2">
          <Button
            variant="outline"
            onClick={handleTest}
            disabled={!url || testing || saving}
          >
            {testing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Test Connection
          </Button>
          <Button
            onClick={handleSave}
            disabled={!url || !testResult || saving}
          >
            {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {isEditing ? "Save Changes" : "Add Registry"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
