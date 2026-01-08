import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import type {
  McpServer,
  ClientInstance,
  AppSettings,
  DetectedClient,
  RegistrySource,
  RegistryServer,
  CustomRegistry,
  CustomRegistryFile,
  FetchResult,
} from "@/types";

interface AppState {
  // Servers
  servers: McpServer[];
  serversLoading: boolean;
  serversError: string | null;

  // Instances
  instances: ClientInstance[];
  instancesLoading: boolean;
  instancesError: string | null;

  // Settings
  settings: AppSettings;
  settingsLoading: boolean;

  // Detected clients
  detectedClients: DetectedClient[];

  // Actions
  loadServers: () => Promise<void>;
  createServer: (server: McpServer) => Promise<McpServer>;
  updateServer: (server: McpServer) => Promise<McpServer>;
  deleteServer: (id: string) => Promise<void>;

  loadInstances: () => Promise<void>;
  createInstance: (instance: ClientInstance) => Promise<ClientInstance>;
  updateInstance: (instance: ClientInstance) => Promise<ClientInstance>;
  deleteInstance: (id: string) => Promise<void>;

  setServerEnabled: (
    instanceId: string,
    serverId: string,
    enabled: boolean
  ) => Promise<void>;
  syncInstance: (instanceId: string) => Promise<string | null>;
  syncAllInstances: () => Promise<string[]>;

  loadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;

  detectClients: () => Promise<void>;
  importFromFile: (path: string) => Promise<McpServer[]>;

  // Registry
  getRegistries: () => Promise<RegistrySource[]>;
  getRegistryServers: (registryId: string) => Promise<RegistryServer[]>;
  importFromRegistry: (registryId: string, servers: RegistryServer[]) => Promise<McpServer[]>;

  // Config reading
  readConfigFile: (path: string) => Promise<{ mcpServers: Record<string, { command: string; args: string[]; env?: Record<string, string> }> } | null>;

  // Custom registries
  addCustomRegistry: (url: string, name?: string, token?: string) => Promise<CustomRegistry>;
  updateCustomRegistry: (id: string, url?: string, name?: string, token?: string) => Promise<CustomRegistry>;
  deleteCustomRegistry: (id: string) => Promise<void>;
  testCustomRegistryUrl: (url: string, token?: string) => Promise<CustomRegistryFile>;
  refreshCustomRegistry: (id: string) => Promise<FetchResult>;
}

export const useStore = create<AppState>()(
  immer((set, get) => ({
  // Initial state
  servers: [],
  serversLoading: false,
  serversError: null,

  instances: [],
  instancesLoading: false,
  instancesError: null,

  settings: {
    theme: "system",
    autoStart: false,
    createBackups: true,
    backupRetentionDays: 30,
    discovery: {
      mcpDirectoryEnabled: false,
      httpServerEnabled: false,
      httpServerPort: 24368,
    },
  },
  settingsLoading: false,

  detectedClients: [],

  // Server actions
  loadServers: async () => {
    set((state) => {
      state.serversLoading = true;
      state.serversError = null;
    });
    try {
      const servers = await invoke<McpServer[]>("get_servers");
      set((state) => {
        state.servers = servers;
        state.serversLoading = false;
      });
    } catch (error) {
      set((state) => {
        state.serversError = error instanceof Error ? error.message : String(error);
        state.serversLoading = false;
      });
    }
  },

  createServer: async (server: McpServer) => {
    const created = await invoke<McpServer>("create_server", { server });
    set((state) => {
      state.servers.push(created);
    });
    return created;
  },

  updateServer: async (server: McpServer) => {
    const updated = await invoke<McpServer>("update_server", { server });
    const now = new Date().toISOString();
    set((state) => {
      // Update server in-place
      const serverIndex = state.servers.findIndex((s) => s.id === updated.id);
      if (serverIndex !== -1) {
        state.servers[serverIndex] = updated;
      }
      // Mark instances using this server as needing sync
      for (const instance of state.instances) {
        if (instance.enabledServers.includes(updated.id)) {
          instance.lastModified = now;
        }
      }
    });
    return updated;
  },

  deleteServer: async (id: string) => {
    await invoke("delete_server", { id });
    set((state) => {
      state.servers = state.servers.filter((s) => s.id !== id);
    });
  },

  // Instance actions
  loadInstances: async () => {
    set((state) => {
      state.instancesLoading = true;
      state.instancesError = null;
    });
    try {
      const instances = await invoke<ClientInstance[]>("get_instances");
      set((state) => {
        state.instances = instances;
        state.instancesLoading = false;
      });
    } catch (error) {
      set((state) => {
        state.instancesError = error instanceof Error ? error.message : String(error);
        state.instancesLoading = false;
      });
    }
  },

  createInstance: async (instance: ClientInstance) => {
    const created = await invoke<ClientInstance>("create_instance", {
      instance,
    });
    set((state) => {
      state.instances.push(created);
    });
    return created;
  },

  updateInstance: async (instance: ClientInstance) => {
    const updated = await invoke<ClientInstance>("update_instance", {
      instance,
    });
    set((state) => {
      const index = state.instances.findIndex((i) => i.id === updated.id);
      if (index !== -1) {
        state.instances[index] = updated;
      }
    });
    return updated;
  },

  deleteInstance: async (id: string) => {
    await invoke("delete_instance", { id });
    set((state) => {
      state.instances = state.instances.filter((i) => i.id !== id);
    });
  },

  setServerEnabled: async (
    instanceId: string,
    serverId: string,
    enabled: boolean
  ) => {
    await invoke("set_server_enabled", { instanceId, serverId, enabled });
    // Update local state including lastModified to trigger "needs sync" status
    const now = new Date().toISOString();
    set((state) => {
      const instance = state.instances.find((i) => i.id === instanceId);
      if (instance) {
        if (enabled) {
          instance.enabledServers.push(serverId);
        } else {
          instance.enabledServers = instance.enabledServers.filter((id) => id !== serverId);
        }
        instance.lastModified = now;
      }
    });
  },

  syncInstance: async (instanceId: string) => {
    const backupPath = await invoke<string | null>("sync_instance", {
      instanceId,
    });
    // Reload instances to get updated lastSynced
    await get().loadInstances();
    return backupPath;
  },

  syncAllInstances: async () => {
    const synced = await invoke<string[]>("sync_all_instances");
    await get().loadInstances();
    return synced;
  },

  // Settings actions
  loadSettings: async () => {
    set((state) => { state.settingsLoading = true; });
    try {
      const settings = await invoke<AppSettings>("get_settings");
      set((state) => {
        state.settings = settings;
        state.settingsLoading = false;
      });
    } catch (error) {
      console.error("Failed to load settings:", error);
      set((state) => { state.settingsLoading = false; });
    }
  },

  saveSettings: async (settings: AppSettings) => {
    await invoke("save_settings", { settings });
    set((state) => { state.settings = settings; });
  },

  // Detection actions
  detectClients: async () => {
    const detectedClients = await invoke<DetectedClient[]>("detect_clients");
    set((state) => { state.detectedClients = detectedClients; });
  },

  importFromFile: async (path: string) => {
    const servers = await invoke<McpServer[]>("import_from_file", { path });
    set((state) => {
      state.servers.push(...servers);
    });
    return servers;
  },

  // Registry actions
  getRegistries: async () => {
    return await invoke<RegistrySource[]>("get_registries");
  },

  getRegistryServers: async (registryId: string) => {
    return await invoke<RegistryServer[]>("get_registry_servers", { registryId });
  },

  importFromRegistry: async (registryId: string, servers: RegistryServer[]) => {
    const imported = await invoke<McpServer[]>("import_from_registry", { registryId, servers });
    set((state) => {
      state.servers.push(...imported);
    });
    return imported;
  },

  // Config reading
  readConfigFile: async (path: string) => {
    try {
      const config = await invoke<{ mcpServers: Record<string, { command: string; args: string[]; env?: Record<string, string> }> }>("read_config_file", { path });
      return config;
    } catch {
      return null;
    }
  },

  // Custom registry actions
  addCustomRegistry: async (url: string, name?: string, token?: string) => {
    return invoke<CustomRegistry>("add_custom_registry", { url, name, token });
  },

  updateCustomRegistry: async (id: string, url?: string, name?: string, token?: string) => {
    return invoke<CustomRegistry>("update_custom_registry", { id, url, name, token });
  },

  deleteCustomRegistry: async (id: string) => {
    await invoke("delete_custom_registry", { id });
  },

  testCustomRegistryUrl: async (url: string, token?: string) => {
    return invoke<CustomRegistryFile>("test_custom_registry_url", { url, token });
  },

  refreshCustomRegistry: async (id: string) => {
    return invoke<FetchResult>("fetch_custom_registry_servers", { id, forceRefresh: true });
  },
})));
