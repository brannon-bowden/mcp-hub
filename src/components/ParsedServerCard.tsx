import { X, AlertTriangle } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import type { ParsedServer } from "@/types";

interface ParsedServerCardProps {
  server: ParsedServer;
  onNameChange: (id: string, name: string) => void;
  onEnvChange: (id: string, key: string, value: string) => void;
  onRemove: (id: string) => void;
}

export function ParsedServerCard({
  server,
  onNameChange,
  onEnvChange,
  onRemove,
}: ParsedServerCardProps) {
  const envEntries = Object.entries(server.env);
  const hasEnvWarnings = server.warnings.some(w => w.includes("placeholder"));

  return (
    <div
      className={`p-4 rounded-lg border ${
        !server.isValid
          ? "border-destructive/50 bg-destructive/5"
          : server.warnings.length > 0
          ? "border-yellow-500/50 bg-yellow-500/5"
          : "border-border"
      }`}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 space-y-3">
          {/* Name field */}
          <div className="space-y-1">
            <Label htmlFor={`name-${server.id}`} className="text-xs text-muted-foreground">
              Server Name
            </Label>
            <Input
              id={`name-${server.id}`}
              value={server.editedName}
              onChange={(e) => onNameChange(server.id, e.target.value)}
              className={`h-8 ${!server.editedName.trim() ? "border-destructive" : ""}`}
              placeholder="Enter server name"
            />
          </div>

          {/* Command display */}
          <div className="space-y-1">
            <span className="text-xs text-muted-foreground">Command</span>
            <code className="block text-sm px-2 py-1 bg-muted rounded">
              {server.command}
            </code>
          </div>

          {/* Args display */}
          {server.args.length > 0 && (
            <div className="space-y-1">
              <span className="text-xs text-muted-foreground">Arguments</span>
              <div className="text-sm text-muted-foreground truncate">
                {server.args.join(" ")}
              </div>
            </div>
          )}

          {/* Environment variables */}
          {envEntries.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">Environment Variables</span>
                {hasEnvWarnings && (
                  <AlertTriangle className="h-3 w-3 text-yellow-500" />
                )}
              </div>
              <div className="space-y-2">
                {envEntries.map(([key, value]) => {
                  const hasWarning = server.warnings.some(
                    (w) => w.includes(key) && w.includes("placeholder")
                  );
                  return (
                    <div key={key} className="flex items-center gap-2">
                      <code className="text-xs px-1.5 py-0.5 bg-muted rounded min-w-[100px]">
                        {key}
                      </code>
                      <Input
                        value={value}
                        onChange={(e) => onEnvChange(server.id, key, e.target.value)}
                        className={`h-7 text-sm flex-1 ${
                          hasWarning ? "border-yellow-500" : ""
                        }`}
                      />
                      {hasWarning && (
                        <AlertTriangle className="h-4 w-4 text-yellow-500 flex-shrink-0" />
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Warnings */}
          {server.warnings.length > 0 && (
            <div className="flex flex-wrap gap-1 pt-1">
              {server.warnings.map((warning, i) => (
                <Badge
                  key={i}
                  variant="outline"
                  className="text-xs text-yellow-600 border-yellow-500/50"
                >
                  {warning}
                </Badge>
              ))}
            </div>
          )}
        </div>

        {/* Remove button */}
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground hover:text-destructive flex-shrink-0"
          onClick={() => onRemove(server.id)}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
