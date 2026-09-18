import { Activity, Check, CircleDollarSign, Clock3, Play, Zap } from "lucide-react";
import { Panel, PanelHead, EmptyState } from "../../components/ui";
import { useI18n } from "../../app/I18nProvider";
import { useShell } from "../../components/shellContext";
import {
  formatDuration,
  formatPercent,
  formatUsd,
  relativeTime,
} from "../../lib/format";

function Metric({ icon: Icon, accent, label, value, sub }) {
  return (
    <article className="metric">
      <div className="metricIcon" style={{ color: accent, background: `${accent}18` }}>
        <Icon size={17} />
      </div>
      <div className="metricBody">
        <span>{label}</span>
        <strong>{value}</strong>
        <small>{sub}</small>
      </div>
    </article>
  );
}

export function WorkspacePage({ agents, executions, stats }) {
  const { t } = useI18n();
  const { openAgent, openRun, goTo } = useShell();

  const daily = stats?.daily?.slice(-7) || [];
  const maxCount = Math.max(1, ...daily.map((d) => d.count));
  const recent = executions.slice(0, 5);
  const readyAgents = agents.slice(0, 6);

  return (
    <>
      <section className="metrics">
        <Metric
          icon={Zap}
          accent="#a58cff"
          label={t("workspace.metric.runs")}
          value={stats?.total ?? 0}
          sub={t("runs.subtitle")}
        />
        <Metric
          icon={Check}
          accent="#39c6a3"
          label={t("workspace.metric.success")}
          value={formatPercent(stats?.success_rate ?? 0, 1)}
          sub={`${stats?.success ?? 0}/${stats?.total ?? 0}`}
        />
        <Metric
          icon={Clock3}
          accent="#58a6ff"
          label={t("workspace.metric.avg")}
          value={formatDuration(stats?.avg_duration_ms ?? 0)}
          sub={`p95 ${formatDuration(stats?.p95_duration_ms ?? 0)}`}
        />
        <Metric
          icon={CircleDollarSign}
          accent="#f0b94d"
          label={t("workspace.metric.cost")}
          value={formatUsd(stats?.total_cost_usd ?? 0)}
          sub={`${stats?.total_input_tokens ?? 0} in / ${stats?.total_output_tokens ?? 0} out`}
        />
      </section>

      <section className="workspaceGrid">
        <Panel className="agentsPanel">
          <PanelHead
            title={t("workspace.readyAgents")}
            subtitle={`${agents.length} ${t("agents.results")}`}
            actions={
              <button className="textButton" onClick={() => goTo("agents")}>
                {t("workspace.seeAll")}
              </button>
            }
          />
          {readyAgents.length === 0 ? (
            <EmptyState title={t("common.empty")} />
          ) : (
            <div className="agentList">
              {readyAgents.map((agent) => (
                <button
                  key={agent.id}
                  className="agentRow"
                  onClick={() => openAgent(agent)}
                  onDoubleClick={() => openRun(agent)}
                >
                  <div className="agentGlyph" style={{ background: "var(--accent-soft)", color: "var(--accent-2)" }}>
                    <Activity size={17} />
                  </div>
                  <div className="agentName">
                    <strong>{agent.name}</strong>
                    <span>{agent.department}</span>
                  </div>
                  <span className="ready">
                    <i />
                    {t("agents.ready")}
                  </span>
                  <span className="runCount">{agent.tier}</span>
                  <Play size={13} className="rowPlay" fill="currentColor" />
                </button>
              ))}
            </div>
          )}
        </Panel>

        <Panel className="recentPanel">
          <PanelHead
            title={t("workspace.recentRuns")}
            subtitle={t("runs.subtitle")}
            actions={
              <button className="textButton" onClick={() => goTo("runs")}>
                {t("workspace.seeAll")}
              </button>
            }
          />
          {recent.length === 0 ? (
            <EmptyState title={t("runs.empty")} />
          ) : (
            <div className="runList">
              {recent.map((run) => (
                <div className="run" key={run.id}>
                  <i className={run.status === "completed" ? "" : run.status === "failed" ? "error" : "warn"} />
                  <div>
                    <strong>{run.agent_id}</strong>
                    <span>{t(`status.${run.status}`)}</span>
                  </div>
                  <div className="runMeta">
                    <strong>{relativeTime(run.started_at)}</strong>
                    <span>{formatDuration(run.duration_ms)}</span>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="activityChart">
            <div className="chartTitle">
              <span>{t("workspace.usage")}</span>
              <strong>{stats?.total ?? 0}</strong>
            </div>
            <div className="bars">
              {daily.length === 0 ? (
                <i style={{ height: "10%" }} />
              ) : (
                daily.map((d) => (
                  <i
                    key={d.date}
                    style={{ height: `${Math.max(10, (d.count / maxCount) * 100)}%` }}
                    title={`${d.date}: ${d.count}`}
                  />
                ))
              )}
            </div>
          </div>
        </Panel>
      </section>
    </>
  );
}
