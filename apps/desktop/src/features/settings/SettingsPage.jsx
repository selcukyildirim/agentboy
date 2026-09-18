import { useEffect, useState } from "react";
import { Check, Plus, RefreshCw, Trash2, X } from "lucide-react";
import {
  Badge,
  Button,
  ConfirmDialog,
  Field,
  Modal,
  Panel,
  PanelHead,
  Switch,
  cx,
} from "../../components/ui";
import { api } from "../../lib/api";
import { useI18n } from "../../app/I18nProvider";
import { useTheme } from "../../app/ThemeProvider";
import { useToast } from "../../app/ToastProvider";
import { useAppData } from "../../app/AppDataProvider";
import { formatPercent } from "../../lib/format";

const PROVIDERS = [
  { id: "openai", name: "OpenAI", models: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"], needsKey: true },
  { id: "anthropic", name: "Anthropic", models: ["claude-sonnet-4-20250514", "claude-3-5-haiku-20241022"], needsKey: true },
  { id: "gemini", name: "Google Gemini", models: ["gemini-2.0-flash", "gemini-1.5-pro"], needsKey: true },
  { id: "openai-compatible", name: "OpenAI Compatible", models: [], needsKey: false, needsBase: true },
  { id: "ollama", name: "Ollama (Local)", models: ["llama3.1", "mistral", "qwen2.5"], needsKey: false },
];

export function SettingsPage() {
  const { t, lang, setLanguage } = useI18n();
  const { dark, toggleDark, compact, toggleCompact } = useTheme();
  const toast = useToast();
  const { cache, reloadHealth, providers, activeProvider, reloadProviders } = useAppData();

  const [offline, setOffline] = useState(
    () => localStorage.getItem("agentboy.offline") === "true"
  );
  const [clearOpen, setClearOpen] = useState(false);
  const [configProvider, setConfigProvider] = useState(null);
  const [removeTarget, setRemoveTarget] = useState(null);

  useEffect(() => {
    localStorage.setItem("agentboy.offline", String(offline));
  }, [offline]);

  const clearCache = async () => {
    try {
      await api.clearCache();
      await reloadHealth();
      toast.success(t("toast.cacheCleared"));
    } catch (err) {
      toast.error(err.message);
    }
  };

  const configuredIds = new Set(providers.map((p) => p.provider));

  return (
    <div className="settingsGrid">
      <Panel className="settingCard">
        <h2>{t("settings.appearance")}</h2>
        <p>{t("settings.appearanceDesc")}</p>
        <label className="settingRow">
          <span>{t("settings.darkTheme")}</span>
          <Switch checked={dark} onChange={toggleDark} label={t("settings.darkTheme")} />
        </label>
        <label className="settingRow">
          <span>{t("settings.compact")}</span>
          <Switch checked={compact} onChange={toggleCompact} label={t("settings.compact")} />
        </label>
        <label className="settingRow">
          <span>{t("settings.language")}</span>
          <select value={lang} onChange={(e) => setLanguage(e.target.value)}>
            <option value="tr">Türkçe</option>
            <option value="en">English</option>
          </select>
        </label>
      </Panel>

      <Panel className="settingCard">
        <h2>{t("settings.privacy")}</h2>
        <p>{t("settings.privacyDesc")}</p>
        <label className="settingRow">
          <span>{t("settings.offline")}</span>
          <Switch checked={offline} onChange={setOffline} label={t("settings.offline")} />
        </label>
        <div className="settingRow">
          <span>{t("settings.pii")}</span>
          <Badge tone="ok">{t("providers.active")}</Badge>
        </div>
        <span className="fieldHint">{t("settings.offlineDesc")}</span>
      </Panel>

      <Panel className="settingCard">
        <h2>{t("settings.cache")}</h2>
        <p>{t("settings.cacheDesc")}</p>
        <div className="settingRow">
          <span>{t("settings.hitRate")}</span>
          <strong>{formatPercent(cache?.hit_rate ?? 0, 0)}</strong>
        </div>
        <div className="settingRow">
          <span>L1 / L2</span>
          <strong>
            {cache?.l1_entries ?? 0} / {cache?.l2_entries ?? 0}
          </strong>
        </div>
        <div style={{ marginTop: 14 }}>
          <Button variant="danger" icon={Trash2} onClick={() => setClearOpen(true)}>
            {t("settings.clearCache")}
          </Button>
        </div>
      </Panel>

      <Panel className="settingCard" style={{ gridColumn: "1 / -1" }}>
        <h2>{t("providers.title")}</h2>
        <p>{t("providers.subtitle")}</p>
        <div className="providerList">
          {PROVIDERS.map((provider) => {
            const configured = configuredIds.has(provider.id);
            const isActive = activeProvider === provider.id;
            return (
              <div className="providerItem" key={provider.id}>
                <div className="providerInfo">
                  <strong>
                    {provider.name}{" "}
                    {isActive && <Badge tone="primary">{t("providers.active")}</Badge>}
                  </strong>
                  <span>
                    {provider.models.slice(0, 2).join(", ") || t("providers.baseUrl")}
                  </span>
                </div>
                <Badge tone={configured ? "ok" : "default"}>
                  {configured ? t("providers.configured") : t("providers.notConfigured")}
                </Badge>
                <div className="providerActions">
                  {configured && !isActive && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={async () => {
                        await api.setActiveProvider(provider.id);
                        await reloadProviders();
                      }}
                    >
                      {t("providers.setActive")}
                    </Button>
                  )}
                  {configured && (
                    <Button
                      variant="ghost"
                      size="sm"
                      icon={Trash2}
                      onClick={() => setRemoveTarget(provider.id)}
                    />
                  )}
                  <Button
                    variant={configured ? "ghost" : "primary"}
                    size="sm"
                    icon={configured ? RefreshCw : Plus}
                    onClick={() => setConfigProvider(provider)}
                  >
                    {configured ? t("providers.refreshModels") : t("providers.configure")}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      </Panel>

      {configProvider && (
        <ProviderModal
          provider={configProvider}
          onClose={() => setConfigProvider(null)}
          onSaved={async () => {
            await reloadProviders();
            setConfigProvider(null);
          }}
        />
      )}

      {clearOpen && (
        <ConfirmDialog
          danger
          title={t("settings.clearCache")}
          message={t("settings.clearConfirm")}
          confirmLabel={t("settings.clearCache")}
          onConfirm={clearCache}
          onClose={() => setClearOpen(false)}
        />
      )}

      {removeTarget && (
        <ConfirmDialog
          danger
          title={t("providers.remove")}
          message={t("providers.removeConfirm")}
          confirmLabel={t("providers.remove")}
          onConfirm={async () => {
            await api.removeProvider(removeTarget);
            await reloadProviders();
            toast.success(t("toast.deleted"));
          }}
          onClose={() => setRemoveTarget(null)}
        />
      )}
    </div>
  );
}

function ProviderModal({ provider, onClose, onSaved }) {
  const { t } = useI18n();
  const toast = useToast();
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState(provider.models[0] || "");
  const [models, setModels] = useState([]);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState(null);

  const test = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      await api.testProvider(provider.id, apiKey || null, baseUrl || null);
      setTestResult("ok");
      toast.success(t("providers.testOk"));
    } catch (err) {
      setTestResult("error");
      toast.error(err.message);
    } finally {
      setTesting(false);
    }
  };

  const save = async () => {
    try {
      await api.configureProvider({
        provider: provider.id,
        api_key: apiKey || null,
        base_url: baseUrl || null,
        model: model || null,
      });
      toast.success(t("toast.saved"));
      onSaved();
    } catch (err) {
      toast.error(err.message);
    }
  };

  const fetchModels = async () => {
    try {
      const list = await api.providerModels(provider.id);
      setModels(list);
    } catch (err) {
      toast.error(err.message);
    }
  };

  return (
    <Modal
      title={`${provider.name}`}
      eyebrow={t("providers.title")}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button onClick={save}>{t("common.save")}</Button>
        </>
      }
    >
      {provider.needsKey && (
        <Field label={t("providers.apiKey")}>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="sk-…"
            autoFocus
          />
        </Field>
      )}
      <Field label={t("providers.baseUrl")} hint={provider.needsBase ? "required" : undefined}>
        <input
          type="text"
          value={baseUrl}
          onChange={(e) => setBaseUrl(e.target.value)}
          placeholder="https://…"
        />
      </Field>
      <Field label={t("providers.model")}>
        <div className="inputRow">
          {models.length > 0 ? (
            <select value={model} onChange={(e) => setModel(e.target.value)}>
              {models.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          ) : (
            <input
              type="text"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder={provider.models[0] || "model-id"}
            />
          )}
          <Button variant="ghost" size="sm" icon={RefreshCw} onClick={fetchModels}>
            {t("providers.refreshModels")}
          </Button>
        </div>
      </Field>

      <div className="modalActions" style={{ justifyContent: "flex-start" }}>
        <Button variant="ghost" icon={Check} onClick={test} disabled={testing}>
          {testing ? t("providers.testing") : t("providers.test")}
        </Button>
        {testResult === "ok" && <Badge tone="ok">{t("providers.testOk")}</Badge>}
        {testResult === "error" && <Badge tone="error">{t("providers.testFail")}</Badge>}
      </div>
    </Modal>
  );
}
