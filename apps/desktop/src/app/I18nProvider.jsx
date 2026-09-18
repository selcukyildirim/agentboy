import { createContext, useCallback, useContext, useMemo, useState } from "react";
import tr from "../lib/i18n/tr.json";
import en from "../lib/i18n/en.json";

const DICTS = { tr, en };

const I18nContext = createContext(null);

function interpolate(template, vars) {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, key) =>
    vars[key] !== undefined ? String(vars[key]) : `{${key}}`
  );
}

export function I18nProvider({ children }) {
  const [lang, setLangState] = useState(
    () => localStorage.getItem("agentboy.lang") || "tr"
  );

  const setLanguage = useCallback((next) => {
    setLangState(next);
    localStorage.setItem("agentboy.lang", next);
  }, []);

  const t = useCallback(
    (key, vars) => {
      const dict = DICTS[lang] || DICTS.tr;
      const value = dict[key] ?? DICTS.tr[key] ?? key;
      return interpolate(value, vars);
    },
    [lang]
  );

  const value = useMemo(
    () => ({ lang, setLanguage, t, languages: Object.keys(DICTS) }),
    [lang, setLanguage, t]
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}
