import { useEffect } from "react";
import { Link } from "react-router-dom";
import {
  Server,
  Layers,
  Plus,
  RefreshCw,
  CheckCircle,
  AlertCircle,
  Clock,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useStore } from "@/store";
import { CLIENT_TYPE_LABELS } from "@/types";

export function Dashboard() {
  const {
    servers,
    instances,
    loadServers,
    loadInstances,
    serversLoading,
    instancesLoading,
    syncAllInstances,
  } = useStore();

  useEffect(() => {
    loadServers();
    loadInstances();
  }, [loadServers, loadInstances]);

  const isLoading = serversLoading || instancesLoading;

  const recentlySynced = instances.filter((i) => i.lastSynced);
  const needsSync = instances.filter((i) => !i.lastSynced);

  const handleSyncAll = async () => {
    try {
      await syncAllInstances();
    } catch (error) {
      console.error("Failed to sync:", error);
    }
  };

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground mt-2">
          Overview of your MCP server configurations
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-8">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Servers</CardTitle>
            <Server className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{servers.length}</div>
            <p className="text-xs text-muted-foreground">
              {servers.length === 0
                ? "No servers configured"
                : `${servers.length} server${servers.length === 1 ? "" : "s"} in registry`}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Client Instances
            </CardTitle>
            <Layers className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{instances.length}</div>
            <p className="text-xs text-muted-foreground">
              {instances.length === 0
                ? "No instances configured"
                : `${instances.length} instance${instances.length === 1 ? "" : "s"} configured`}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Synced</CardTitle>
            <CheckCircle className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{recentlySynced.length}</div>
            <p className="text-xs text-muted-foreground">
              Instances up to date
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Needs Sync</CardTitle>
            <AlertCircle className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{needsSync.length}</div>
            <p className="text-xs text-muted-foreground">Instances pending</p>
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions */}
      <div className="grid gap-4 md:grid-cols-2 mb-8">
        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
            <CardDescription>Common tasks at a glance</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2">
            <Button asChild>
              <Link to="/servers">
                <Plus className="w-4 h-4 mr-2" />
                Add Server
              </Link>
            </Button>
            <Button variant="outline" asChild>
              <Link to="/instances">
                <Layers className="w-4 h-4 mr-2" />
                Manage Instances
              </Link>
            </Button>
            <Button
              variant="secondary"
              onClick={handleSyncAll}
              disabled={instances.length === 0}
            >
              <RefreshCw className="w-4 h-4 mr-2" />
              Sync All
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Recent Servers</CardTitle>
            <CardDescription>Recently added MCP servers</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="text-sm text-muted-foreground">Loading...</div>
            ) : servers.length === 0 ? (
              <div className="text-sm text-muted-foreground">
                No servers configured yet.{" "}
                <Link to="/servers" className="text-primary hover:underline">
                  Add your first server
                </Link>
              </div>
            ) : (
              <div className="space-y-2">
                {servers.slice(0, 5).map((server) => (
                  <div
                    key={server.id}
                    className="flex items-center justify-between py-2 border-b last:border-0"
                  >
                    <div>
                      <div className="font-medium">{server.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {server.command}
                      </div>
                    </div>
                    {server.tags.length > 0 && (
                      <Badge variant="secondary">{server.tags[0]}</Badge>
                    )}
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Instances Overview */}
      <Card>
        <CardHeader>
          <CardTitle>Client Instances</CardTitle>
          <CardDescription>
            Status of your MCP client configurations
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : instances.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              No instances configured yet.{" "}
              <Link to="/instances" className="text-primary hover:underline">
                Set up your first instance
              </Link>
            </div>
          ) : (
            <div className="space-y-3">
              {instances.map((instance) => (
                <div
                  key={instance.id}
                  className="flex items-center justify-between p-3 rounded-lg border"
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-2 h-2 rounded-full ${instance.lastSynced ? "bg-green-500" : "bg-yellow-500"}`}
                    />
                    <div>
                      <div className="font-medium">{instance.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {CLIENT_TYPE_LABELS[instance.clientType]} &bull;{" "}
                        {instance.enabledServers.length} server
                        {instance.enabledServers.length === 1 ? "" : "s"}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <Clock className="w-3 h-3" />
                    {instance.lastSynced
                      ? new Date(instance.lastSynced).toLocaleDateString()
                      : "Never synced"}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
