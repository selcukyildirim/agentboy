import { useMemo, useState } from "react";
import { Check, CircleDollarSign, Clock3, Play, X, Zap } from "lucide-react";
import { IconButton, StatusPill, cx } from "../../components/ui";
import { AgentInputForm } from "./AgentInputForm";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";
import { useAppData } from "../../app/AppDataProvider";
import { formatDuration, formatTokens, formatUsd } from "../../lib/format";
import { defaultInputFromSchema } from "../../lib/agentInput";

function summarize(value, max = 600) {
  if (value === undefined || value === null) return "";
  const text = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

export function AgentInspector({ agent, onClose, openRun }) {
  const { t } = useI18n();
  const toast = useToast();
  const { reloadExecutions, reloadStats } = useAppData();

  const [input, setInput] = useState(() => defaultInputFromSchema(agent.input_schema));
  const [mode, setMode] = useState("form");
  const [messages, setMessages] = useState([]);
  const [running, setRunning] = useState(false);

  const fields = useMemo(() => agent.input_schema || [], [agent]);

  const run = async () => {
    if (running) return;
    setRunning(true);
    setMessages((list) => [...list, { role: "user", text: "▶ " + agent.name }]);
    try {
      const offline = localStorage.getItem("agentboy.offline") === "true";
      const result = await api.executeAgent(agent.id, input, offline);
      setMessages((list) => [
        ...list,
        {
          role: "assistant",
          text: summarize(result.output),
          meta: {
            duration: result.duration_ms,
            tokens: result.usage?.total_tokens,
            cost: result.cost_usd,
            model: result.model,
          },
        },
      ]);
      toast.success(t("toast.executionDone"));
      await Promise.all([reloadExecutions(), reloadStats(7)]);
    } catch (err) {
      setMessages((list) => [
        ...list,
        {
          role: "assistant",
          error: true,
          text: `${t("common.error")}: ${err.message}`,
        },
      ]);
      toast.error(err.message);
    } finally {
      setRunning(false);
    }
  };

  return (
    <aside className="inspector">
      <div className="inspectorHead">
        <span>{t("inspector.title")}</span>
        <IconButton label={t("common.close")} onClick={onClose}>
          <X size={14} />
        </IconButton>
      </div>

      <div className="agentHero">
        <div
          className="heroIcon"
          style={{ background: "var(--accent-soft)", color: "var(--accent-2)" }}
        >
          <Zap size={22} />
        </div>
        <h2>{agent.name}</h2>
        <span>{agent.department}</span>
        <p>{agent.description}</p>
      </div>

      <div className="details">
        <div>
          <span>{t("inspector.status")}</span>
          <strong>
            <StatusPill status={t("agents.ready")} />
          </strong>
        </div>
        <div>
          <span>{t("inspector.runs")}</span>
          <strong>{agent.tier}</strong>
        </div>
        <div>
          <span>v</span>
          <strong>{agent.version}</strong>
        </div>
      </div>

      <div className="inspectorBody">
        {messages.length > 0 && (
          <div className="messages" style={{ marginBottom: 12 }}>
            {messages.map((m, i) => (
              <div key={i} className={cx("bubble", m.role, m.error && "error")}>
                {m.text || t("inspector.thinking")}
                {m.meta && (
                  <div className="bubbleMeta">
                    <span>
                      <Clock3 size={11} /> {formatDuration(m.meta.duration)}
                    </span>
                    <span>
                      <Zap size={11} /> {formatTokens(m.meta.tokens)}
                    </span>
                    {m.meta.cost != null && (
                      <span>
                        <CircleDollarSign size={11} /> {formatUsd(m.meta.cost)}
                      </span>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        <h4>{t("inspector.inputs")}</h4>
        <AgentInputForm
          fields={fields}
          value={input}
          onChange={setInput}
          mode={mode}
          setMode={setMode}
        />
      </div>

      <div className="composer">
        <button
          className="composerRun"
          onClick={run}
          disabled={running}
          aria-label={t("common.run")}
        >
          {running ? <Check size={14} /> : <Play size={14} fill="currentColor" />}
        </button>
        <span className="composerHint">
          {fields.length} {t("inspector.inputs")}
        </span>
      </div>
    </aside>
  );
}
