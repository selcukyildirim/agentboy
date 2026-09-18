import { useMemo, useState } from "react";
import { FileText, Sparkles, Upload } from "lucide-react";
import { Field, cx } from "../../components/ui";
import { useI18n } from "../../app/I18nProvider";

const KIND_LABEL_KEY = {
  file: "input.file",
  csv: "input.csv",
  number: "input.number",
  text: "input.text",
  json: "input.json",
};

function parseMaybeJson(text) {
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

/**
 * Controlled form generated from an agent's `input_schema`.
 * `value` is the JSON payload to send to the backend.
 */
export function AgentInputForm({ fields, value, onChange, mode, setMode }) {
  const { t } = useI18n();
  const [jsonError, setJsonError] = useState(null);
  const [fileNames, setFileNames] = useState({});
  const [rawJson, setRawJson] = useState(() => JSON.stringify(value, null, 2));

  const advanced = mode === "advanced";

  const setField = (key, next) => {
    onChange({ ...value, [key]: next });
  };

  const handleFile = async (key, file) => {
    if (!file) return;
    const text = await file.text();
    setFileNames((prev) => ({ ...prev, [key]: file.name }));
    setField(key, text);
  };

  const loadExample = (field) => {
    if (field.example == null) return;
    if (field.kind === "number") {
      setField(field.key, Number(field.example));
    } else if (field.kind === "json") {
      const parsed = parseMaybeJson(field.example);
      setField(field.key, parsed.ok ? parsed.value : field.example);
    } else {
      setField(field.key, field.example);
    }
  };

  const handleAdvancedChange = (text) => {
    setRawJson(text);
    const parsed = parseMaybeJson(text);
    if (parsed.ok) {
      setJsonError(null);
      onChange(parsed.value);
    } else {
      setJsonError(parsed.error);
    }
  };

  const tabs = (
    <div className="tabRow">
      <button
        type="button"
        className={cx("tabBtn", !advanced && "active")}
        onClick={() => setMode?.("form")}
      >
        {t("inspector.inputs")}
      </button>
      <button
        type="button"
        className={cx("tabBtn", advanced && "active")}
        onClick={() => {
          setRawJson(JSON.stringify(value, null, 2));
          setMode?.("advanced");
        }}
      >
        {t("inspector.advanced")}
      </button>
    </div>
  );

  if (advanced) {
    return (
      <div>
        {tabs}
        <Field label={t("inspector.advanced")} hint={jsonError || undefined}>
          <textarea
            value={rawJson}
            onChange={(e) => handleAdvancedChange(e.target.value)}
            spellCheck={false}
            style={{ fontFamily: "ui-monospace, monospace", minHeight: 160 }}
          />
        </Field>
      </div>
    );
  }

  return (
    <div>
      {tabs}
      {fields.map((field) => {
        const label = `${field.label}${field.required ? " *" : ""}`;
        const kindLabel = t(KIND_LABEL_KEY[field.kind] || "input.text");
        const raw = value?.[field.key];

        return (
          <div key={field.key} className="field">
            <div className="fieldHeader">
              <span className="fieldLabel">
                {label} <em className="kindTag">{kindLabel}</em>
              </span>
              {field.example != null && (
                <button
                  type="button"
                  className="textButton"
                  onClick={() => loadExample(field)}
                >
                  <Sparkles size={12} /> {t("inspector.loadExample")}
                </button>
              )}
            </div>

            {field.kind === "file" && (
              <>
                <label className="uploadArea compactUpload">
                  <Upload size={18} />
                  <p>{t("inspector.dropFile")}</p>
                  <input
                    type="file"
                    accept=".csv,.txt,.md,.json,.xml,text/*"
                    onChange={(e) => handleFile(field.key, e.target.files?.[0])}
                    hidden
                  />
                </label>
                {fileNames[field.key] && (
                  <span className="fileChip">
                    <FileText size={12} /> {fileNames[field.key]}
                  </span>
                )}
              </>
            )}

            {field.kind === "csv" && (
              <textarea
                value={typeof raw === "string" ? raw : ""}
                onChange={(e) => setField(field.key, e.target.value)}
                placeholder={field.example || "a,b\n1,2"}
                style={{ minHeight: 96, fontFamily: "ui-monospace, monospace" }}
              />
            )}

            {field.kind === "number" && (
              <input
                type="number"
                value={raw ?? ""}
                onChange={(e) =>
                  setField(
                    field.key,
                    e.target.value === "" ? undefined : Number(e.target.value)
                  )
                }
                placeholder={field.example || ""}
              />
            )}

            {field.kind === "text" && (
              <input
                type="text"
                value={raw ?? ""}
                onChange={(e) => setField(field.key, e.target.value)}
                placeholder={field.example || ""}
              />
            )}

            {field.kind === "json" && (
              <textarea
                value={
                  typeof raw === "string"
                    ? raw
                    : raw !== undefined
                    ? JSON.stringify(raw, null, 2)
                    : ""
                }
                onChange={(e) => {
                  const parsed = parseMaybeJson(e.target.value);
                  setField(field.key, parsed.ok ? parsed.value : e.target.value);
                }}
                placeholder={field.example || "{}"}
                spellCheck={false}
                style={{ minHeight: 96, fontFamily: "ui-monospace, monospace" }}
              />
            )}

            {field.description && <span className="fieldHint">{field.description}</span>}
          </div>
        );
      })}
    </div>
  );
}
