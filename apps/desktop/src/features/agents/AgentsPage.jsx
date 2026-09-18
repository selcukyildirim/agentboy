import { useMemo } from "react";
import { Bot, Play, X } from "lucide-react";
import { Panel, PanelHead, EmptyState, Badge } from "../../components/ui";
import { useI18n } from "../../app/I18nProvider";

export function AgentsPage({
  agents,
  query,
  selectedCollection,
  setSelectedCollection,
  openAgent,
  openRun,
}) {
  const { t } = useI18n();

  const filtered = useMemo(() => {
    const q = query.trim().toLocaleLowerCase("tr-TR");
    return agents.filter((a) => {
      const text = `${a.name} ${a.department} ${a.id}`.toLocaleLowerCase("tr-TR");
      const matchesText = !q || text.includes(q);
      const matchesCollection =
        !selectedCollection || a.department === selectedCollection;
      return matchesText && matchesCollection;
    });
  }, [agents, query, selectedCollection]);

  return (
    <Panel className="fullPanel">
      <PanelHead
        title={t("agents.allTitle")}
        subtitle={`${filtered.length} ${t("agents.results")}`}
        actions={
          selectedCollection ? (
            <button className="filterChip" onClick={() => setSelectedCollection("")}>
              {selectedCollection} <X size={12} />
            </button>
          ) : null
        }
      />
      {filtered.length === 0 ? (
        <EmptyState icon={Bot} title={t("common.empty")} />
      ) : (
        <div className="agentGrid">
          {filtered.map((agent) => (
            <article
              className="agentCard"
              key={agent.id}
              onClick={() => openAgent(agent)}
            >
              <div
                className="agentGlyph"
                style={{ background: "var(--accent-soft)", color: "var(--accent-2)" }}
              >
                <Bot size={18} />
              </div>
              <div>
                <h3>{agent.name}</h3>
                <span>{agent.department}</span>
              </div>
              <p>{agent.description}</p>
              <div className="agentCardFoot">
                <Badge tone="primary">{agent.tier}</Badge>
                <button
                  className="btn primary sm"
                  onClick={(e) => {
                    e.stopPropagation();
                    openRun(agent);
                  }}
                >
                  <Play size={12} fill="currentColor" /> {t("common.run")}
                </button>
              </div>
            </article>
          ))}
        </div>
      )}
    </Panel>
  );
}
