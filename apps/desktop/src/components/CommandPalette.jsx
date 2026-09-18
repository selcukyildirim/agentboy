import { useEffect, useMemo, useRef, useState } from "react";
import { Bot, Command, CornerDownLeft, Search } from "lucide-react";
import { useI18n } from "../app/I18nProvider";
import { useAppData } from "../app/AppDataProvider";

export function CommandPalette({ open, onClose, onSelectAgent, goTo }) {
  const { t } = useI18n();
  const { agents } = useAppData();
  const [value, setValue] = useState("");
  const inputRef = useRef(null);

  useEffect(() => {
    if (open) {
      setValue("");
      setTimeout(() => inputRef.current?.focus(), 30);
    }
  }, [open]);

  useEffect(() => {
    const onKey = (e) => {
      if (e.key === "Escape") onClose();
    };
    if (open) window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  const results = useMemo(() => {
    const q = value.trim().toLocaleLowerCase("tr-TR");
    const list = agents.filter(
      (a) =>
        !q ||
        `${a.name} ${a.department} ${a.id}`.toLocaleLowerCase("tr-TR").includes(q)
    );
    return list.slice(0, 8);
  }, [agents, value]);

  if (!open) return null;

  return (
    <div className="commandOverlay" onMouseDown={onClose} role="presentation">
      <div className="commandBox" onMouseDown={(e) => e.stopPropagation()}>
        <div className="commandInput">
          <Command size={17} />
          <input
            ref={inputRef}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder={t("titlebar.search")}
          />
          <kbd>ESC</kbd>
        </div>
        <div className="commandResults">
          {results.length > 0 && (
            <>
              <span className="commandLabel">{t("nav.agents")}</span>
              {results.map((agent) => (
                <button key={agent.id} onClick={() => onSelectAgent(agent)}>
                  <Bot size={16} style={{ color: "var(--accent-2)" }} />
                  <span>
                    {agent.name}
                    <small>{agent.department}</small>
                  </span>
                  <CornerDownLeft size={13} />
                </button>
              ))}
            </>
          )}
          {value.trim() === "" && (
            <>
              <span className="commandLabel">{t("nav.settings")}</span>
              <button onClick={() => goTo("providers")}>
                <Search size={16} />
                <span>
                  {t("providers.title")}
                  <small>{t("providers.subtitle")}</small>
                </span>
              </button>
              <button onClick={() => goTo("runs")}>
                <Search size={16} />
                <span>
                  {t("runs.title")}
                  <small>{t("runs.subtitle")}</small>
                </span>
              </button>
            </>
          )}
          {results.length === 0 && value.trim() !== "" && (
            <div className="emptyState">
              <strong>{t("common.empty")}</strong>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
