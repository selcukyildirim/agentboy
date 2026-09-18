import { useEffect, useState } from "react";
import { RefreshCw, Scale, Sparkles } from "lucide-react";
import { Badge, Button, EmptyState, Panel, PanelHead, cx } from "../../components/ui";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";

export function DecisionsPage() {
  const { t } = useI18n();
  const toast = useToast();
  const [types, setTypes] = useState([]);
  const [selected, setSelected] = useState(null);
  const [context, setContext] = useState(null);
  const [recommendation, setRecommendation] = useState(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api.decisionTypes().then(setTypes).catch(() => setTypes([]));
  }, []);

  const choose = async (type) => {
    setSelected(type);
    setRecommendation(null);
    setContext(null);
    try {
      setContext(await api.decisionContext(type.id));
    } catch (err) {
      toast.error(err.message);
    }
  };

  const generate = async () => {
    if (!selected || !context) return;
    setLoading(true);
    try {
      const rec = await api.recommendation(selected.id, context.context);
      setRecommendation(rec.recommendation);
    } catch (err) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  };

  const confidenceTone = (value) =>
    value === "High" ? "ok" : value === "Medium" ? "warn" : "error";

  return (
    <div className="settingsGrid" style={{ gridTemplateColumns: "1fr" }}>
      <Panel>
        <PanelHead
          title={t("decisions.title")}
          subtitle={t("decisions.subtitle")}
          actions={
            <Button
              variant="ghost"
              size="sm"
              icon={RefreshCw}
              onClick={() => api.decisionTypes().then(setTypes).catch(() => {})}
            >
              {t("common.refresh")}
            </Button>
          }
        />
        {types.length === 0 ? (
          <EmptyState icon={Scale} title={t("common.empty")} />
        ) : (
          <div className="decisionList" style={{ padding: 12 }}>
            {types.map((type) => (
              <article
                key={type.id}
                className={cx("decisionCard", selected?.id === type.id && "selected")}
                onClick={() => choose(type)}
              >
                <h4>{type.name}</h4>
                <p>
                  {t("decisions.category")}: {type.category}
                </p>
                <p>
                  {t("decisions.risk")}: {type.risk_level}
                </p>
                <p>
                  {t("decisions.requiredFacts")}: {type.required_facts.join(", ")}
                </p>
              </article>
            ))}
          </div>
        )}
      </Panel>

      {selected && context && (
        <Panel className="decisionDetail">
          <PanelHead title={selected.name} subtitle={t("decisions.context")} />
          <div style={{ padding: 16 }}>
            <div className="settingRow">
              <span>{t("decisions.confidence")}</span>
              <Badge tone={confidenceTone(context.context?.confidence)}>
                {context.context?.confidence}
              </Badge>
            </div>
            <div className="settingRow">
              <span>{t("decisions.missingFacts")}</span>
              <strong>
                {(context.context?.missing_facts || []).join(", ") || t("common.none")}
              </strong>
            </div>

            <div style={{ marginTop: 14 }}>
              <Button icon={Sparkles} onClick={generate} disabled={loading}>
                {loading ? t("common.loading") : t("decisions.generate")}
              </Button>
            </div>

            {recommendation && (
              <div style={{ marginTop: 16 }}>
                <h4>{t("decisions.recommendation")}</h4>
                <div className="resultItem">
                  <strong>
                    {recommendation.action}{" "}
                    <Badge tone={confidenceTone(recommendation.confidence)}>
                      {recommendation.confidence}
                    </Badge>
                  </strong>
                  <p>{recommendation.reasoning}</p>
                </div>
              </div>
            )}
          </div>
        </Panel>
      )}
    </div>
  );
}
