import { useState } from "react";
import { Play, Plus, Trash2, Workflow } from "lucide-react";
import {
  Badge,
  Button,
  ConfirmDialog,
  EmptyState,
  Field,
  Modal,
  Panel,
  PanelHead,
  cx,
} from "../../components/ui";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";
import { useAppData } from "../../app/AppDataProvider";
import { relativeTime } from "../../lib/format";

export function FlowsPage() {
  const { t } = useI18n();
  const toast = useToast();
  const { workflows, agents, reloadWorkflows } = useAppData();

  const [createOpen, setCreateOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [selected, setSelected] = useState(null);
  const [stepAgent, setStepAgent] = useState("");
  const [deleteTarget, setDeleteTarget] = useState(null);

  const create = async () => {
    if (!name.trim()) return;
    try {
      const wf = await api.createWorkflow(name.trim(), description.trim());
      setName("");
      setDescription("");
      setCreateOpen(false);
      await reloadWorkflows();
      setSelected(wf);
      toast.success(t("toast.workflowCreated"));
    } catch (err) {
      toast.error(err.message);
    }
  };

  const addStep = async () => {
    if (!selected || !stepAgent) return;
    const agent = agents.find((a) => a.id === stepAgent);
    try {
      const updated = await api.addWorkflowStep(selected.id, stepAgent, agent?.name || stepAgent);
      setSelected(updated);
      setStepAgent("");
      await reloadWorkflows();
      toast.success(t("toast.stepAdded"));
    } catch (err) {
      toast.error(err.message);
    }
  };

  const run = async (wf) => {
    try {
      await api.executeWorkflow(wf.id, {});
      toast.success(t("toast.workflowStarted"));
    } catch (err) {
      toast.error(err.message);
    }
  };

  const remove = async () => {
    try {
      await api.deleteWorkflow(deleteTarget.id);
      await reloadWorkflows();
      if (selected?.id === deleteTarget.id) setSelected(null);
      toast.success(t("toast.deleted"));
    } catch (err) {
      toast.error(err.message);
    }
  };

  return (
    <>
      <Panel className="fullPanel">
        <PanelHead
          title={t("flows.title")}
          subtitle={`${workflows.length} ${t("flows.steps")}`}
          actions={
            <Button icon={Plus} onClick={() => setCreateOpen(true)}>
              {t("flows.new")}
            </Button>
          }
        />
        {workflows.length === 0 ? (
          <EmptyState icon={Workflow} title={t("flows.empty")} />
        ) : (
          <div className="flowGrid" style={{ padding: 12 }}>
            {workflows.map((wf) => (
              <article
                key={wf.id}
                className={cx("panel flowCard", selected?.id === wf.id && "selected")}
                onClick={() => setSelected(wf)}
              >
                <i style={{ background: "var(--accent)" }} />
                <Workflow size={21} style={{ color: "var(--accent-2)" }} />
                <h2>{wf.name}</h2>
                <p>{wf.description || "—"}</p>
                <span>
                  {wf.step_count} {t("flows.steps")} · {relativeTime(wf.created_at)}
                </span>
                <div className="flowCardActions">
                  <button
                    className="btn ghost sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setDeleteTarget(wf);
                    }}
                  >
                    <Trash2 size={12} />
                  </button>
                  <button
                    className="btn primary sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      run(wf);
                    }}
                  >
                    <Play size={12} fill="currentColor" /> {t("flows.start")}
                  </button>
                </div>
                {selected?.id === wf.id && (
                  <div className="flowSteps">
                    {wf.steps?.map((s, i) => (
                      <div className="flowStep" key={`${s.agent_id}-${i}`}>
                        <Badge tone="primary">{i + 1}</Badge> {s.name}
                      </div>
                    ))}
                  </div>
                )}
              </article>
            ))}
          </div>
        )}

        {selected && (
          <div style={{ padding: 14, borderTop: "1px solid var(--border)" }}>
            <h4 style={{ marginTop: 0 }}>{t("flows.addStep")}</h4>
            <div className="inputRow" style={{ maxWidth: 520 }}>
              <select value={stepAgent} onChange={(e) => setStepAgent(e.target.value)}>
                <option value="">{t("flows.stepAgent")}…</option>
                {agents.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name} — {a.department}
                  </option>
                ))}
              </select>
              <Button icon={Plus} onClick={addStep} disabled={!stepAgent}>
                {t("common.add")}
              </Button>
            </div>
          </div>
        )}
      </Panel>

      {createOpen && (
        <Modal
          title={t("modal.newFlow")}
          onClose={() => setCreateOpen(false)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setCreateOpen(false)}>
                {t("common.cancel")}
              </Button>
              <Button onClick={create}>{t("common.create")}</Button>
            </>
          }
        >
          <Field label={t("common.name")}>
            <input type="text" value={name} onChange={(e) => setName(e.target.value)} autoFocus />
          </Field>
          <Field label={t("common.description")}>
            <textarea value={description} onChange={(e) => setDescription(e.target.value)} />
          </Field>
        </Modal>
      )}

      {deleteTarget && (
        <ConfirmDialog
          danger
          title={t("common.delete")}
          message={t("flows.deleteConfirm")}
          confirmLabel={t("common.delete")}
          onConfirm={remove}
          onClose={() => setDeleteTarget(null)}
        />
      )}
    </>
  );
}
