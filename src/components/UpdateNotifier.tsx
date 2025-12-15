import { useEffect, useState, useCallback } from "react";
import { X, Download, RotateCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

type UpdateState = "idle" | "available" | "downloading" | "ready" | "dismissed";

export function UpdateNotifier() {
  const [updateState, setUpdateState] = useState<UpdateState>("idle");
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const checkForUpdates = useCallback(async () => {
    try {
      const update = await check();
      if (update) {
        setUpdateVersion(update.version);
        setUpdateState("available");
      }
    } catch (error) {
      // Silently fail on startup check - user can manually check in Settings
      console.error("Update check failed:", error);
    }
  }, []);

  const downloadUpdate = useCallback(async () => {
    setUpdateState("downloading");
    setDownloadProgress(0);
    try {
      const update = await check();
      if (!update) return;

      let contentLength = 0;
      let downloaded = 0;

      await update.downloadAndInstall((progress) => {
        if (progress.event === "Started" && progress.data.contentLength) {
          contentLength = progress.data.contentLength;
        } else if (progress.event === "Progress") {
          downloaded += progress.data.chunkLength;
          if (contentLength > 0) {
            setDownloadProgress((downloaded / contentLength) * 100);
          }
        } else if (progress.event === "Finished") {
          setDownloadProgress(100);
        }
      });

      setUpdateState("ready");
    } catch (error) {
      console.error("Download failed:", error);
      setUpdateState("available"); // Allow retry
    }
  }, []);

  const restartApp = useCallback(async () => {
    try {
      await relaunch();
    } catch (error) {
      console.error("Restart failed:", error);
    }
  }, []);

  const dismiss = useCallback(() => {
    setUpdateState("dismissed");
  }, []);

  // Check for updates on mount
  useEffect(() => {
    // Delay check by 3 seconds to let the app fully load
    const timer = setTimeout(checkForUpdates, 3000);
    return () => clearTimeout(timer);
  }, [checkForUpdates]);

  // Don't render if idle or dismissed
  if (updateState === "idle" || updateState === "dismissed") {
    return null;
  }

  return (
    <div className="fixed top-0 left-0 right-0 z-50 bg-primary text-primary-foreground px-4 py-2">
      <div className="max-w-screen-xl mx-auto flex items-center justify-between gap-4">
        <div className="flex items-center gap-2 text-sm">
          {updateState === "available" && (
            <>
              <Download className="w-4 h-4" />
              <span>MCP Hub {updateVersion} is available!</span>
            </>
          )}
          {updateState === "downloading" && (
            <>
              <Download className="w-4 h-4 animate-pulse" />
              <span>Downloading update... {Math.round(downloadProgress)}%</span>
            </>
          )}
          {updateState === "ready" && (
            <>
              <RotateCw className="w-4 h-4" />
              <span>Update ready! Restart to apply.</span>
            </>
          )}
        </div>
        <div className="flex items-center gap-2">
          {updateState === "available" && (
            <Button
              size="sm"
              variant="secondary"
              onClick={downloadUpdate}
            >
              Download Now
            </Button>
          )}
          {updateState === "ready" && (
            <Button
              size="sm"
              variant="secondary"
              onClick={restartApp}
            >
              Restart Now
            </Button>
          )}
          {updateState !== "downloading" && (
            <Button
              size="icon"
              variant="ghost"
              className="h-6 w-6 text-primary-foreground hover:bg-primary-foreground/20"
              onClick={dismiss}
            >
              <X className="w-4 h-4" />
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
