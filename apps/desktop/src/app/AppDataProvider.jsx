import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { api } from "../lib/api";

const AppDataContext = createContext(null);

const EMPTY = {
  health: null,
  metrics: null,
  agents: [],
  workflows: [],
  executions: [],
  stats: null,
  cache: null,
  providers: [],
  activeProvider: null,
  documents: [],
};

export function AppDataProvider({ children }) {
  const [data, setData] = useState(EMPTY);
  const [loading, setLoading] = useState(true);
  const mounted = useRef(true);

  const patch = useCallback((partial) => {
    if (mounted.current) setData((prev) => ({ ...prev, ...partial }));
  }, []);

  const reloadAgents = useCallback(async () => {
    try {
      patch({ agents: await api.agents() });
    } catch {
      /* keep previous */
    }
  }, [patch]);

  const reloadWorkflows = useCallback(async () => {
    try {
      patch({ workflows: await api.workflows() });
    } catch {
      /* ignore */
    }
  }, [patch]);

  const reloadExecutions = useCallback(async () => {
    try {
      patch({ executions: await api.executions() });
    } catch {
      /* ignore */
    }
  }, [patch]);

  const reloadStats = useCallback(async (days = 7) => {
    try {
      patch({ stats: await api.executionStats(days) });
    } catch {
      /* ignore */
    }
  }, [patch]);

  const reloadProviders = useCallback(async () => {
    try {
      const [providers, activeProvider] = await Promise.all([
        api.providerStatus(),
        api.activeProvider(),
      ]);
      patch({ providers, activeProvider });
    } catch {
      /* ignore */
    }
  }, [patch]);

  const reloadDocuments = useCallback(async () => {
    try {
      patch({ documents: await api.documents() });
    } catch {
      /* ignore */
    }
  }, [patch]);

  const reloadHealth = useCallback(async () => {
    try {
      const [health, metrics, cache] = await Promise.all([
        api.health(),
        api.metrics(),
        api.cacheStats(),
      ]);
      patch({ health, metrics, cache });
    } catch {
      /* ignore */
    }
  }, [patch]);

  const reloadAll = useCallback(async () => {
    await Promise.allSettled([
      reloadAgents(),
      reloadWorkflows(),
      reloadExecutions(),
      reloadStats(7),
      reloadProviders(),
      reloadDocuments(),
      reloadHealth(),
    ]);
    if (mounted.current) setLoading(false);
  }, [
    reloadAgents,
    reloadWorkflows,
    reloadExecutions,
    reloadStats,
    reloadProviders,
    reloadDocuments,
    reloadHealth,
  ]);

  useEffect(() => {
    mounted.current = true;
    reloadAll();
    return () => {
      mounted.current = false;
    };
  }, [reloadAll]);

  const value = useMemo(
    () => ({
      ...data,
      loading,
      reloadAll,
      reloadAgents,
      reloadWorkflows,
      reloadExecutions,
      reloadStats,
      reloadProviders,
      reloadDocuments,
      reloadHealth,
    }),
    [
      data,
      loading,
      reloadAll,
      reloadAgents,
      reloadWorkflows,
      reloadExecutions,
      reloadStats,
      reloadProviders,
      reloadDocuments,
      reloadHealth,
    ]
  );

  return <AppDataContext.Provider value={value}>{children}</AppDataContext.Provider>;
}

export function useAppData() {
  const ctx = useContext(AppDataContext);
  if (!ctx) throw new Error("useAppData must be used within AppDataProvider");
  return ctx;
}
