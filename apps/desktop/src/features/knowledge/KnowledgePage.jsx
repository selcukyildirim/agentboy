import { useState } from "react";
import { BookOpen, FileText, Search, Upload } from "lucide-react";
import { Button, EmptyState, Panel, PanelHead } from "../../components/ui";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useToast } from "../../app/ToastProvider";
import { useAppData } from "../../app/AppDataProvider";
import { formatBytes } from "../../lib/format";

export function KnowledgePage({ documents }) {
  const { t } = useI18n();
  const toast = useToast();
  const { reloadDocuments } = useAppData();
  const [uploading, setUploading] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState(null);
  const [searching, setSearching] = useState(false);

  const upload = async (file) => {
    if (!file) return;
    setUploading(true);
    try {
      const buffer = new Uint8Array(await file.arrayBuffer());
      await api.uploadDocument(
        file.name,
        Array.from(buffer),
        file.type || "text/plain"
      );
      await reloadDocuments();
      toast.success(t("toast.uploaded"));
    } catch (err) {
      toast.error(err.message);
    } finally {
      setUploading(false);
    }
  };

  const search = async () => {
    if (!query.trim()) return;
    setSearching(true);
    try {
      const res = await api.queryRag(query.trim(), 5);
      setResults(res.results || []);
    } catch (err) {
      toast.error(err.message);
    } finally {
      setSearching(false);
    }
  };

  return (
    <Panel className="fullPanel">
      <PanelHead title={t("knowledge.title")} subtitle={t("knowledge.subtitle")} />

      <div style={{ padding: 16 }}>
        <label className="uploadArea">
          <Upload size={22} />
          <p>{uploading ? t("common.loading") : t("knowledge.dropHint")}</p>
          <p className="fieldHint">{t("knowledge.supported")}</p>
          <input
            type="file"
            accept=".csv,.txt,.md,.json,.xml,text/*"
            hidden
            onChange={(e) => upload(e.target.files?.[0])}
          />
        </label>

        <div className="statsRow">
          <div className="statBox">
            <strong>{documents.length}</strong>
            <span>{t("knowledge.documents")}</span>
          </div>
          <div className="statBox">
            <strong>{documents.reduce((a, d) => a + (d.chunk_count || 0), 0)}</strong>
            <span>{t("knowledge.chunks")}</span>
          </div>
          <div className="statBox">
            <strong>{formatBytes(documents.reduce((a, d) => a + (d.size_bytes || 0), 0))}</strong>
            <span>{t("knowledge.size")}</span>
          </div>
        </div>

        <div className="inputRow" style={{ marginBottom: 16 }}>
          <input
            type="text"
            value={query}
            placeholder={t("knowledge.askPlaceholder")}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && search()}
          />
          <Button icon={Search} onClick={search} disabled={searching}>
            {t("common.search")}
          </Button>
        </div>

        {results && (
          <>
            <h4>
              {results.length} {t("knowledge.results")}
            </h4>
            {results.length === 0 ? (
              <EmptyState title={t("knowledge.noResults")} />
            ) : (
              <div className="resultList">
                {results.map((r) => (
                  <div className="resultItem" key={r.chunk_id}>
                    <strong>
                      {r.document_id?.slice(0, 8)} · {t("runs.cost")}: {r.score}
                    </strong>
                    <p>{r.content?.slice(0, 320)}</p>
                  </div>
                ))}
              </div>
            )}
          </>
        )}

        {documents.length > 0 && (
          <>
            <h4 style={{ marginTop: 16 }}>{t("knowledge.documents")}</h4>
            <div className="documentList">
              {documents.map((doc) => (
                <div className="documentItem" key={doc.id}>
                  <span>
                    <FileText size={13} /> {doc.name}
                  </span>
                  <span>
                    {doc.chunk_count} {t("knowledge.chunks")} · {formatBytes(doc.size_bytes)}
                  </span>
                </div>
              ))}
            </div>
          </>
        )}

        {documents.length === 0 && !results && (
          <EmptyState icon={BookOpen} title={t("knowledge.dropHint")} />
        )}
      </div>
    </Panel>
  );
}
