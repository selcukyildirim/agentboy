import { Activity, Check, CircleDollarSign, Clock3, Download } from "lucide-react";
import { Button, Panel, PanelHead } from "../../components/ui";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";
import { useAppData } from "../../app/AppDataProvider";
import {
  formatDuration,
  formatPercent,
  formatTokens,
  formatUsd,
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

export function ReportsPage({ stats }) {
  const { t } = useI18n();
  const toast = useToast();
  const { reloadStats } = useAppData();

  const daily = stats?.daily || [];
  const maxCount = Math.max(1, ...daily.map((d) => d.count));

  const exportCsv = () => {
    const header = "date,count,success,failed,tokens,cost_usd\n";
    const rows = daily
      .map((d) => `${d.date},${d.count},${d.success},${d.failed},${d.tokens},${d.cost_usd.toFixed(6)}`)
      .join("\n");
    const blob = new Blob([header + rows], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "agentboy-report.csv";
    a.click();
    URL.revokeObjectURL(url);
    toast.success(t("reports.exported"));
  };

  return (
    <>
      <section className="metrics">
        <Metric
          icon={Activity}
          accent="#a58cff"
          label={t("reports.totalRuns")}
          value={stats?.total ?? 0}
          sub={`${stats?.success ?? 0} ✓ / ${stats?.failed ?? 0} ✗`}
        />
        <Metric
          icon={Check}
          accent="#39c6a3"
          label={t("reports.success")}
          value={formatPercent(stats?.success_rate ?? 0, 1)}
          sub={`p50 ${formatDuration(stats?.p50_duration_ms ?? 0)}`}
        />
        <Metric
          icon={Clock3}
          accent="#58a6ff"
          label={t("reports.totalDuration")}
          value={formatDuration(
            (stats?.avg_duration_ms ?? 0) * (stats?.total ?? 0)
          )}
          sub={`p95 ${formatDuration(stats?.p95_duration_ms ?? 0)}`}
        />
        <Metric
          icon={CircleDollarSign}
          accent="#f0b94d"
          label={t("reports.totalCost")}
          value={formatUsd(stats?.total_cost_usd ?? 0)}
          sub={`${formatTokens(
            (stats?.total_input_tokens ?? 0) + (stats?.total_output_tokens ?? 0)
          )} tokens`}
        />
      </section>

      <Panel className="reportPanel">
        <PanelHead
          title={t("reports.trend")}
          subtitle={t("reports.trendDays")}
          actions={
            <>
              <Button
                variant="ghost"
                size="sm"
                icon={Activity}
                onClick={() => reloadStats(14)}
              >
                {t("common.refresh")}
              </Button>
              <Button variant="ghost" size="sm" icon={Download} onClick={exportCsv}>
                {t("common.export")}
              </Button>
            </>
          }
        />
        <div className="bigChart">
          {daily.length === 0 ? (
            <div className="emptyState" style={{ width: "100%" }}>
              <strong>{t("common.empty")}</strong>
            </div>
          ) : (
            daily.map((d) => (
              <i
                key={d.date}
                style={{ height: `${Math.max(6, (d.count / maxCount) * 100)}%` }}
                title={`${d.date}: ${d.count} (${d.success}✓)`}
              />
            ))
          )}
        </div>
      </Panel>
    </>
  );
}
