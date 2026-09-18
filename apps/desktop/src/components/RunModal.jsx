import { useMemo, useState } from "react";
import { Play } from "lucide-react";
import { Button, Field, Modal, cx } from "./ui";
import { AgentInputForm } from "../features/agents/AgentInputForm";
import { api } from "../lib/api";
import { defaultInputFromSchema } from "../lib/agentInput";
import { useI18n } from "../app/I18nProvider";
import { useToast } from "../app/ToastProvider";
import { useAppData } from "../app/AppDataProvider";

export function RunModal({ agent, agents, onSelectAgent, onClose, onDone }) {
  const { t } = useI18n();
  const toast = useToast();
  const { reloadExecutions, reloadStats } = useAppData();

  const [current, setCurrent] = useState(agent || agents[0] || null);
  const [mode, setMode] = useState("form");
  const [input, setInput] = useState(() =>
    defaultInputFromSchema(agent?.input_schema)
  );
  const [offline, setOffline] = useState(
    () => localStorage.getItem("agentboy.offline") === "true"
  );
  const [running, setRunning] = useState(false);

  const fields = useMemo(() => current?.input_schema || [], [current]);

  const chooseAgent = (next) => {
    setCurrent(next);
    onSelectAgent?.(next);
    setInput(defaultInputFromSchema(next.input_schema));
    setMode("form");
  };

  const run = async () => {
    if (!current || running) return;
    setRunning(true);
    try {
      const result = await api.executeAgent(current.id, input, offline);
      toast.success(t("toast.executionDone"));
      await Promise.all([reloadExecutions(), reloadStats(7)]);
      onDone?.(result);
    } catch (err) {
      toast.error(`${t("common.error")}: ${err.message}`);
    } finally {
      setRunning(false);
    }
  };

  if (!current) return null;

  return (
    <Modal
      size="lg"
      eyebrow={t("app.name")}
      title={t("modal.newRun")}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button icon={Play} onClick={run} disabled={running}>
            {running ? t("common.loading") : t("common.run")}
          </Button>
        </>
      }
    >
      <Field label={t("flows.stepAgent")}>
        <div className="agentPicker">
          {agents.slice(0, 8).map((a) => (
            <button
              key={a.id}
              type="button"
              className={cx(current.id === a.id && "selected")}
              onClick={() => chooseAgent(a)}
            >
              {a.name}
            </button>
          ))}
        </div>
      </Field>

      <div className="settingRow">
        <span>{t("settings.offline")}</span>
        <button
          type="button"
          role="switch"
          aria-checked={offline}
          className={cx("switch", offline && "on")}
          onClick={() => setOffline((v) => !v)}
        >
          <i />
        </button>
      </div>

      <AgentInputForm
        fields={fields}
        value={input}
        onChange={setInput}
        mode={mode}
        setMode={setMode}
      />
    </Modal>
  );
}
