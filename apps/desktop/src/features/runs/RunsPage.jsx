import { useState } from "react";
import { Activity, RefreshCw, Trash2 } from "lucide-react";
import {
  Button,
  ConfirmDialog,
  EmptyState,
  Modal,
  Panel,
  PanelHead,
  StatusPill,
} from "../../components/ui";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";
import { useAppData } from "../../app/AppDataProvider";
import {
  formatDateTime,
  formatDuration,
  formatTokens,
  formatUsd,
  relativeTime,
  shortId,
} from "../../lib/format";

export function RunsPage({ executions }) {
  const { t } = useI18n();
  const toast = useToast();
  const { reloadExecutions } = useAppData();
  const [detail, setDetail] = useState(null);
  const [deleteTarget, setDeleteTarget] = useState(null);

  const openDetail = async (run) => {
    setDetail({ ...run, loading: true });
    try {
      const steps = await api.executionSteps(run.id);
      setDetail({ ...run, steps, loading: false });
    } catch {
      setDetail({ ...run, steps: run.steps || [], loading: false });
    }
  };

  const remove = async () => {
    try {
      await api.deleteExecution(deleteTarget.id);
      await reloadExecutions();
      toast.success(t("toast.deleted"));
    } catch (err) {
      toast.error(err.message);
    }
  };

  return (
    <>
      <Panel className="fullPanel">
        <PanelHead
          title={t("runs.title")}
          subtitle={t("runs.subtitle")}
          actions={
            <Button
              variant="ghost"
              size="sm"
              icon={RefreshCw}
              onClick={() => reloadExecutions()}
            >
              {t("common.refresh")}
            </Button>
          }
        />
        {executions.length === 0 ? (
          <EmptyState icon={Activity} title={t("runs.empty")} />
        ) : (
          <div className="runsTable">
            <div className="tableRow tableHead">
              <span>{t("runs.agent")}</span>
              <span>{t("runs.status")}</span>
              <span>{t("runs.time")}</span>
              <span>{t("runs.duration")}</span>
              <span>{t("runs.tokens")}</span>
              <span>{t("runs.cost")}</span>
              <span />
            </div>
            {executions.map((run) => (
              <button className="tableRow" key={run.id} onClick={() => openDetail(run)}>
                <span>{run.agent_id}</span>
                <span>
                  <StatusPill status={t(`status.${run.status}`)} />
                </span>
                <span>{relativeTime(run.started_at)}</span>
                <span>{formatDuration(run.duration_ms)}</span>
                <span>{formatTokens((run.input_tokens || 0) + (run.output_tokens || 0))}</span>
                <span>{run.cost_usd != null ? formatUsd(run.cost_usd) : "—"}</span>
                <span className="viewCell">{t("common.view")}</span>
              </button>
            ))}
          </div>
        )}
      </Panel>

      {detail && (
        <Modal
          size="lg"
          title={detail.agent_id}
          eyebrow={shortId(detail.id)}
          onClose={() => setDetail(null)}
          footer={
            <>
              <Button
                variant="ghost"
                icon={Trash2}
                onClick={() => {
                  setDeleteTarget(detail);
                  setDetail(null);
                }}
              >
                {t("common.delete")}
              </Button>
              <Button onClick={() => setDetail(null)}>{t("common.close")}</Button>
            </>
          }
        >
          <div className="settingRow">
            <span>{t("common.status")}</span>
            <StatusPill status={t(`status.${detail.status}`)} />
          </div>
          <div className="settingRow">
            <span>{t("runs.time")}</span>
            <strong>{formatDateTime(detail.started_at)}</strong>
          </div>
          <div className="settingRow">
            <span>{t("runs.duration")}</span>
            <strong>{formatDuration(detail.duration_ms)}</strong>
          </div>
          <div className="settingRow">
            <span>{t("workspace.metric.tokens")}</span>
            <strong>
              {formatTokens((detail.input_tokens || 0) + (detail.output_tokens || 0))} (
              {detail.input_tokens || 0}/{detail.output_tokens || 0})
            </strong>
          </div>
          <div className="settingRow">
            <span>{t("runs.cost")}</span>
            <strong>{detail.cost_usd != null ? formatUsd(detail.cost_usd) : "—"}</strong>
          </div>
          {detail.model && (
            <div className="settingRow">
              <span>{t("providers.model")}</span>
              <strong>{detail.model}</strong>
            </div>
          )}

          <h4 style={{ marginTop: 16 }}>{t("runs.steps")}</h4>
          <div className="runList" style={{ padding: 0 }}>
            {(detail.steps || []).map((step, i) => (
              <div className="run" key={i}>
                <i className={step.status === "completed" ? "" : "error"} />
                <div>
                  <strong>
                    {step.step_number}. {step.step_type}
                  </strong>
                  <span>{step.description}</span>
                </div>
                <div className="runMeta">
                  <span>{formatDuration(step.duration_ms)}</span>
                </div>
              </div>
            ))}
          </div>

          {detail.error && (
            <>
              <h4 style={{ marginTop: 16 }}>{t("common.error")}</h4>
              <p className="dialogText" style={{ color: "var(--error)" }}>
                {detail.error}
              </p>
            </>
          )}

          {detail.output && (
            <>
              <h4 style={{ marginTop: 16 }}>{t("runs.result")}</h4>
              <pre className="jsonBlock">{JSON.stringify(detail.output, null, 2)}</pre>
            </>
          )}
        </Modal>
      )}

      {deleteTarget && (
        <ConfirmDialog
          danger
          title={t("common.delete")}
          message={t("runs.deleteConfirm")}
          confirmLabel={t("common.delete")}
          onConfirm={remove}
          onClose={() => setDeleteTarget(null)}
        />
      )}
    </>
  );
}
