import { NavLink, Outlet } from "react-router-dom";
import {
  LayoutDashboard,
  Server,
  Layers,
  Settings,
  RefreshCw,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useStore } from "@/store";

const navigation = [
  { name: "Dashboard", href: "/", icon: LayoutDashboard },
  { name: "Servers", href: "/servers", icon: Server },
  { name: "Instances", href: "/instances", icon: Layers },
  { name: "Settings", href: "/settings", icon: Settings },
];

export function Layout() {
  const { syncAllInstances, instances } = useStore();

  const handleSyncAll = async () => {
    try {
      await syncAllInstances();
    } catch (error) {
      console.error("Failed to sync:", error);
    }
  };

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside className="w-64 border-r bg-sidebar-background flex flex-col">
        {/* Logo */}
        <div className="h-16 flex items-center px-6 border-b">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <Server className="w-5 h-5 text-primary-foreground" />
            </div>
            <span className="font-semibold text-lg">MCP Hub</span>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 px-3 py-4 space-y-1">
          {navigation.map((item) => (
            <NavLink
              key={item.name}
              to={item.href}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors",
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
                )
              }
            >
              <item.icon className="w-5 h-5" />
              {item.name}
            </NavLink>
          ))}
        </nav>

        {/* Sync All Button */}
        <div className="p-4 border-t">
          <Button
            variant="outline"
            className="w-full justify-start gap-2"
            onClick={handleSyncAll}
            disabled={instances.length === 0}
          >
            <RefreshCw className="w-4 h-4" />
            Sync All Instances
          </Button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
