import { useEffect, useState } from "react";
import { RefreshCw, ScrollText } from "lucide-react";
import { Button, EmptyState, Panel, PanelHead, Badge } from "../../components/ui";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";
import { formatDateTime } from "../../lib/format";

export function AuditPage() {
  const { t } = useI18n();
  const toast = useToast();
  const [events, setEvents] = useState([]);
  const [loading, setLoading] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      setEvents(await api.audit(100));
    } catch (err) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const tone = (result) =>
    result === "Success" ? "ok" : result === "Denied" ? "warn" : "error";

  return (
    <Panel className="fullPanel">
      <PanelHead
        title={t("audit.title")}
        subtitle={t("audit.subtitle")}
        actions={
          <Button variant="ghost" size="sm" icon={RefreshCw} onClick={load} disabled={loading}>
            {t("common.refresh")}
          </Button>
        }
      />
      {events.length === 0 ? (
        <EmptyState icon={ScrollText} title={t("audit.empty")} />
      ) : (
        <div className="runsTable">
          <div className="tableRow tableHead" style={{ gridTemplateColumns: "1.4fr 0.8fr 1fr 0.8fr 1.4fr" }}>
            <span>{t("audit.agent")}</span>
            <span>{t("audit.action")}</span>
            <span>{t("audit.result")}</span>
            <span>{t("audit.time")}</span>
            <span>{t("audit.resource")}</span>
          </div>
          {events.map((e) => (
            <div
              className="tableRow"
              key={e.event_id}
              style={{ gridTemplateColumns: "1.4fr 0.8fr 1fr 0.8fr 1.4fr" }}
            >
              <span>{e.agent_id}</span>
              <span>{e.action}</span>
              <span>
                <Badge tone={tone(e.result)}>{e.result}</Badge>
              </span>
              <span>{formatDateTime(e.timestamp)}</span>
              <span>{e.resource}</span>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}
