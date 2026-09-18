import {
  useCallback,
  useMemo,
  useState,
} from "react";
import {
  Activity,
  BarChart3,
  Bell,
  BookOpen,
  Bot,
  Check,
  ChevronDown,
  LayoutDashboard,
  Menu,
  Moon,
  PanelLeftClose,
  Plus,
  Scale,
  Search,
  Settings,
  Sun,
  User,
  Workflow,
} from "lucide-react";
import { useI18n } from "../app/I18nProvider";
import { useTheme } from "../app/ThemeProvider";
import { useAppData } from "../app/AppDataProvider";
import { IconButton, Spinner, cx } from "./ui";
import { CommandPalette } from "./CommandPalette";
import { RunModal } from "./RunModal";
import { ShellContext } from "./shellContext";
import { AgentInspector } from "../features/agents/Inspector";
import { WorkspacePage } from "../features/workspace/WorkspacePage";
import { AgentsPage } from "../features/agents/AgentsPage";
import { FlowsPage } from "../features/flows/FlowsPage";
import { RunsPage } from "../features/runs/RunsPage";
import { ReportsPage } from "../features/reports/ReportsPage";
import { KnowledgePage } from "../features/knowledge/KnowledgePage";
import { DecisionsPage } from "../features/decisions/DecisionsPage";
import { SettingsPage } from "../features/settings/SettingsPage";

const NAV = [
  { id: "home", labelKey: "nav.home", icon: LayoutDashboard },
  { id: "agents", labelKey: "nav.agents", icon: Bot },
  { id: "flows", labelKey: "nav.flows", icon: Workflow },
  { id: "runs", labelKey: "nav.runs", icon: Activity },
  { id: "reports", labelKey: "nav.reports", icon: BarChart3 },
  { id: "knowledge", labelKey: "nav.knowledge", icon: BookOpen },
  { id: "decisions", labelKey: "nav.decisions", icon: Scale },
];

const DEPT_COLORS = [
  "#f0b94d",
  "#39c6a3",
  "#f07c68",
  "#58a6ff",
  "#ef6bb4",
  "#a58cff",
  "#5ad1c8",
  "#e0a63c",
  "#7c9cf0",
  "#d98cff",
];

export function AppShell() {
  const { t } = useI18n();
  const { dark, toggleDark } = useTheme();
  const { agents, executions, stats, health, documents, providers, loading } =
    useAppData();

  const [activeNav, setActiveNav] = useState("home");
  const [sideOpen, setSideOpen] = useState(true);
  const [inspectorOpen, setInspectorOpen] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState(null);
  const [query, setQuery] = useState("");
  const [selectedCollection, setSelectedCollection] = useState("");
  const [commandOpen, setCommandOpen] = useState(false);
  const [runModal, setRunModal] = useState(null);
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);

  const goTo = useCallback((page) => {
    setActiveNav(page);
    setNotificationsOpen(false);
    setProfileOpen(false);
  }, []);

  const openAgent = useCallback((agent) => {
    setSelectedAgent(agent);
    setInspectorOpen(true);
  }, []);

  const openRun = useCallback(
    (agent) => {
      if (agent) setSelectedAgent(agent);
      setRunModal(agent || selectedAgent || agents[0] || null);
    },
    [selectedAgent, agents]
  );

  const departments = useMemo(() => {
    const map = new Map();
    agents.forEach((a) => map.set(a.department, (map.get(a.department) || 0) + 1));
    return Array.from(map.entries()).map(([name, count], i) => ({
      name,
      count,
      color: DEPT_COLORS[i % DEPT_COLORS.length],
    }));
  }, [agents]);

  const shell = useMemo(
    () => ({
      activeNav,
      goTo,
      sideOpen,
      setSideOpen,
      inspectorOpen,
      setInspectorOpen,
      selectedAgent,
      openAgent,
      query,
      setQuery,
      selectedCollection,
      setSelectedCollection,
      openCommand: () => setCommandOpen(true),
      openRun,
      t,
    }),
    [
      activeNav,
      goTo,
      sideOpen,
      inspectorOpen,
      selectedAgent,
      openAgent,
      query,
      selectedCollection,
      openRun,
      t,
    ]
  );

  const pageProps = { agents, executions, stats, health, documents };
  const providerCount = providers?.length || 0;

  return (
    <ShellContext.Provider value={shell}>
      <div className="app">
        <TitleBar
          dark={dark}
          toggleDark={toggleDark}
          version={health?.version}
          onOpenCommand={() => setCommandOpen(true)}
          notifications={executions.slice(0, 4)}
          notificationsOpen={notificationsOpen}
          setNotificationsOpen={setNotificationsOpen}
          profileOpen={profileOpen}
          setProfileOpen={setProfileOpen}
          goTo={goTo}
        />

        <div
          className={cx(
            "bodyGrid",
            !sideOpen && "sideClosed",
            !inspectorOpen && "inspectorClosed"
          )}
        >
          <Rail activeNav={activeNav} goTo={goTo} health={health} />

          <Sidebar
            agents={agents}
            departments={departments}
            selectedCollection={selectedCollection}
            setSelectedCollection={setSelectedCollection}
            query={query}
            setQuery={setQuery}
            selectedAgent={selectedAgent}
            openAgent={openAgent}
            goTo={goTo}
            setSideOpen={setSideOpen}
            openRun={openRun}
            stats={stats}
          />

          <main className="main">
            {!sideOpen && (
              <button className="iconBtn openSide" onClick={() => setSideOpen(true)}>
                <Menu size={17} />
              </button>
            )}
            <Welcome
              activeNav={activeNav}
              openRun={openRun}
              onCreateFlow={() => goTo("flows")}
            />
            {loading && agents.length === 0 ? (
              <div className="loadingOverlay">
                <Spinner label={t("common.loading")} />
              </div>
            ) : (
              <>
                {activeNav === "home" && <WorkspacePage {...pageProps} />}
                {activeNav === "agents" && (
                  <AgentsPage
                    agents={agents}
                    query={query}
                    selectedCollection={selectedCollection}
                    setSelectedCollection={setSelectedCollection}
                    openAgent={openAgent}
                    openRun={openRun}
                  />
                )}
                {activeNav === "flows" && <FlowsPage openRun={openRun} />}
                {activeNav === "runs" && (
                  <RunsPage executions={executions} openRun={openRun} />
                )}
                {activeNav === "reports" && <ReportsPage stats={stats} />}
                {activeNav === "knowledge" && <KnowledgePage documents={documents} />}
                {activeNav === "decisions" && <DecisionsPage />}
                {activeNav === "settings" && <SettingsPage />}
              </>
            )}
          </main>

          {inspectorOpen && selectedAgent && (
            <AgentInspector
              agent={selectedAgent}
              onClose={() => setInspectorOpen(false)}
              openRun={openRun}
            />
          )}
        </div>

        <StatusBar health={health} providers={providerCount} offline={false} />

        <CommandPalette
          open={commandOpen}
          onClose={() => setCommandOpen(false)}
          onSelectAgent={(agent) => {
            openAgent(agent);
            setCommandOpen(false);
          }}
          goTo={(page) => {
            goTo(page);
            setCommandOpen(false);
          }}
        />

        {runModal && (
          <RunModal
            agent={runModal}
            agents={agents}
            onSelectAgent={setSelectedAgent}
            onClose={() => setRunModal(null)}
            onDone={(execution) => {
              setRunModal(null);
              goTo("runs");
            }}
          />
        )}
      </div>
    </ShellContext.Provider>
  );
}

function TitleBar({
  dark,
  toggleDark,
  version,
  onOpenCommand,
  notifications,
  notificationsOpen,
  setNotificationsOpen,
  profileOpen,
  setProfileOpen,
  goTo,
}) {
  const { t } = useI18n();
  return (
    <header className="titlebar">
      <div className="traffic">
        <i />
        <i />
        <i />
      </div>
      <button className="brand" onClick={() => goTo("home")}>
        <span>
          <Bot size={16} />
        </span>
        {t("app.name")}
      </button>
      <div className="menuAnchor">
        <button className="workspace" onClick={() => goTo("settings")}>
          {t("app.tagline")} <ChevronDown size={13} />
        </button>
      </div>
      <button className="commandTrigger" onClick={onOpenCommand}>
        <Search size={14} />
        <span>{t("titlebar.search")}</span>
        <kbd>⌘ K</kbd>
      </button>
      <div className="windowActions">
        <div className="menuAnchor">
          <IconButton
            label={t("titlebar.notifications")}
            active={notificationsOpen}
            onClick={() => setNotificationsOpen((v) => !v)}
          >
            <Bell size={16} />
          </IconButton>
          {notificationsOpen && (
            <div className="popover right" onClick={(e) => e.stopPropagation()}>
              <strong>{t("titlebar.notifications")}</strong>
              {notifications.length === 0 ? (
                <p>{t("common.empty")}</p>
              ) : (
                notifications.map((n) => (
                  <p key={n.id}>
                    <Check size={14} /> {n.agent_id} · {n.status}
                  </p>
                ))
              )}
            </div>
          )}
        </div>
        <IconButton
          label={dark ? t("titlebar.theme.toLight") : t("titlebar.theme.toDark")}
          onClick={toggleDark}
        >
          {dark ? <Sun size={16} /> : <Moon size={16} />}
        </IconButton>
        <div className="menuAnchor">
          <button className="avatar" onClick={() => setProfileOpen((v) => !v)}>
            AB
          </button>
          {profileOpen && (
            <div className="popover right" onClick={(e) => e.stopPropagation()}>
              <strong>AgentBoy</strong>
              <button onClick={() => goTo("settings")}>
                <Settings size={14} /> {t("nav.settings")}
              </button>
              <button onClick={() => goTo("agents")}>
                <User size={14} /> {t("nav.agents")}
              </button>
            </div>
          )}
        </div>
      </div>
      <span hidden>{version}</span>
    </header>
  );
}

function Rail({ activeNav, goTo, health }) {
  const { t } = useI18n();
  return (
    <aside className="rail">
      <div className="railTop">
        {NAV.map((item) => (
          <IconButton
            key={item.id}
            label={t(item.labelKey)}
            active={activeNav === item.id}
            onClick={() => goTo(item.id)}
          >
            <item.icon size={19} />
          </IconButton>
        ))}
      </div>
      <div className="railBottom">
        <IconButton
          label={t("nav.settings")}
          active={activeNav === "settings"}
          onClick={() => goTo("settings")}
        >
          <Settings size={19} />
        </IconButton>
        <span
          className="onlineDot"
          title={health?.status || "unknown"}
          style={health?.status === "healthy" ? undefined : { background: "var(--warn)" }}
        />
      </div>
    </aside>
  );
}

function Sidebar({
  agents,
  departments,
  selectedCollection,
  setSelectedCollection,
  query,
  setQuery,
  selectedAgent,
  openAgent,
  goTo,
  setSideOpen,
  openRun,
  stats,
}) {
  const { t } = useI18n();
  const pinned = agents.slice(0, 4);
  const usedPct = stats?.total_cost_usd
    ? Math.min(100, (stats.total_cost_usd / 25) * 100)
    : 0;

  return (
    <aside className="sidebar">
      <div className="sideHead">
        <span>{t("agents.collections")}</span>
        <IconButton label={t("common.close")} onClick={() => setSideOpen(false)}>
          <PanelLeftClose size={15} />
        </IconButton>
      </div>
      <button className="newTask" onClick={() => openRun(selectedAgent)}>
        <Plus size={15} /> {t("workspace.newRun")} <kbd>⌘ N</kbd>
      </button>
      <label className="sideSearch">
        <Search size={14} />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("agents.searchPlaceholder")}
        />
      </label>

      <div className="sideSection">
        <div className="sectionLabel">{t("agents.collections")}</div>
        {departments.map((dept) => (
          <button
            className={cx("collection", selectedCollection === dept.name && "selected")}
            key={dept.name}
            onClick={() => {
              setSelectedCollection(selectedCollection === dept.name ? "" : dept.name);
              goTo("agents");
            }}
          >
            <i style={{ background: dept.color }} />
            {dept.name}
            <span>{dept.count}</span>
          </button>
        ))}
      </div>

      <div className="sideSection grow">
        <div className="sectionLabel">{t("agents.pinned")}</div>
        {pinned.map((agent) => (
          <button
            className={cx("pinned", selectedAgent?.id === agent.id && "selected")}
            key={agent.id}
            onClick={() => openAgent(agent)}
          >
            <Bot size={15} style={{ color: "var(--accent-2)" }} />
            <span>{agent.name}</span>
          </button>
        ))}
      </div>

      <div className="budget">
        <div>
          <span>{t("workspace.metric.cost")}</span>
          <strong>${(stats?.total_cost_usd || 0).toFixed(2)}</strong>
        </div>
        <div className="progress">
          <i style={{ width: `${Math.max(4, usedPct)}%` }} />
        </div>
      </div>
    </aside>
  );
}

function Welcome({ activeNav, openRun, onCreateFlow }) {
  const { t } = useI18n();
  const meta = {
    home: ["workspace.title", "workspace.subtitle"],
    agents: ["agents.title", "agents.subtitle"],
    flows: ["flows.title", "flows.subtitle"],
    runs: ["runs.title", "runs.subtitle"],
    reports: ["reports.title", "reports.subtitle"],
    knowledge: ["knowledge.title", "knowledge.subtitle"],
    decisions: ["decisions.title", "decisions.subtitle"],
    settings: ["settings.title", "settings.appearanceDesc"],
  }[activeNav] || ["workspace.title", "workspace.subtitle"];

  return (
    <section className="welcome">
      <div>
        <span className="eyebrow">{t("app.name")}</span>
        <h1>{t(meta[0])}</h1>
        <p>{t(meta[1])}</p>
      </div>
      <div className="welcomeActions">
        <button className="btn ghost" onClick={onCreateFlow}>
          <Workflow size={15} /> {t("workspace.createFlow")}
        </button>
        <button className="btn primary" onClick={() => openRun(null)}>
          <Activity size={14} /> {t("workspace.newRun")}
        </button>
      </div>
    </section>
  );
}

function StatusBar({ health, providers, offline }) {
  const { t } = useI18n();
  const healthy = health?.status === "healthy";
  return (
    <footer className="statusbar">
      <div>
        <span className={cx("statusOnline", !healthy && "off")}>
          <i /> {healthy ? t("statusbar.connected") : t("statusbar.disconnected")}
        </span>
        <span>
          {t("statusbar.providers")}: {providers}
        </span>
      </div>
      <div>
        <span>{offline ? t("statusbar.offline") : t("statusbar.pii")}</span>
        <span>AgentBoy {health?.version || "4.0"}</span>
      </div>
    </footer>
  );
}
