import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  McpServer,
  ClientInstance,
  AppSettings,
  DetectedClient,
  RegistrySource,
  RegistryServer,
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
}

export const useStore = create<AppState>((set, get) => ({
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
    set({ serversLoading: true, serversError: null });
    try {
      const servers = await invoke<McpServer[]>("get_servers");
      set({ servers, serversLoading: false });
    } catch (error) {
      set({
        serversError: error instanceof Error ? error.message : String(error),
        serversLoading: false,
      });
    }
  },

  createServer: async (server: McpServer) => {
    const created = await invoke<McpServer>("create_server", { server });
    set({ servers: [...get().servers, created] });
    return created;
  },

  updateServer: async (server: McpServer) => {
    const updated = await invoke<McpServer>("update_server", { server });
    set({
      servers: get().servers.map((s) => (s.id === updated.id ? updated : s)),
    });
    return updated;
  },

  deleteServer: async (id: string) => {
    await invoke("delete_server", { id });
    set({ servers: get().servers.filter((s) => s.id !== id) });
  },

  // Instance actions
  loadInstances: async () => {
    set({ instancesLoading: true, instancesError: null });
    try {
      const instances = await invoke<ClientInstance[]>("get_instances");
      set({ instances, instancesLoading: false });
    } catch (error) {
      set({
        instancesError: error instanceof Error ? error.message : String(error),
        instancesLoading: false,
      });
    }
  },

  createInstance: async (instance: ClientInstance) => {
    const created = await invoke<ClientInstance>("create_instance", {
      instance,
    });
    set({ instances: [...get().instances, created] });
    return created;
  },

  updateInstance: async (instance: ClientInstance) => {
    const updated = await invoke<ClientInstance>("update_instance", {
      instance,
    });
    set({
      instances: get().instances.map((i) =>
        i.id === updated.id ? updated : i
      ),
    });
    return updated;
  },

  deleteInstance: async (id: string) => {
    await invoke("delete_instance", { id });
    set({ instances: get().instances.filter((i) => i.id !== id) });
  },

  setServerEnabled: async (
    instanceId: string,
    serverId: string,
    enabled: boolean
  ) => {
    await invoke("set_server_enabled", { instanceId, serverId, enabled });
    // Update local state
    set({
      instances: get().instances.map((instance) => {
        if (instance.id !== instanceId) return instance;
        const enabledServers = enabled
          ? [...instance.enabledServers, serverId]
          : instance.enabledServers.filter((id) => id !== serverId);
        return { ...instance, enabledServers };
      }),
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
    set({ settingsLoading: true });
    try {
      const settings = await invoke<AppSettings>("get_settings");
      set({ settings, settingsLoading: false });
    } catch (error) {
      console.error("Failed to load settings:", error);
      set({ settingsLoading: false });
    }
  },

  saveSettings: async (settings: AppSettings) => {
    await invoke("save_settings", { settings });
    set({ settings });
  },

  // Detection actions
  detectClients: async () => {
    const detectedClients = await invoke<DetectedClient[]>("detect_clients");
    set({ detectedClients });
  },

  importFromFile: async (path: string) => {
    const servers = await invoke<McpServer[]>("import_from_file", { path });
    set({ servers: [...get().servers, ...servers] });
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
    set({ servers: [...get().servers, ...imported] });
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
}));
