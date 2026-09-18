import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

const ThemeContext = createContext(null);

function applyTheme(dark, compact) {
  const root = document.documentElement;
  root.classList.toggle("theme-dark", dark);
  root.classList.toggle("theme-light", !dark);
  document.body.classList.toggle("compact", compact);
}

export function ThemeProvider({ children }) {
  const [dark, setDark] = useState(
    () => localStorage.getItem("agentboy.theme") !== "light"
  );
  const [compact, setCompact] = useState(
    () => localStorage.getItem("agentboy.compact") === "true"
  );

  useEffect(() => {
    applyTheme(dark, compact);
    localStorage.setItem("agentboy.theme", dark ? "dark" : "light");
    localStorage.setItem("agentboy.compact", String(compact));
  }, [dark, compact]);

  const toggleDark = useCallback(() => setDark((v) => !v), []);
  const toggleCompact = useCallback(() => setCompact((v) => !v), []);

  const value = useMemo(
    () => ({ dark, compact, toggleDark, toggleCompact, setDark, setCompact }),
    [dark, compact, toggleDark, toggleCompact]
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within ThemeProvider");
  return ctx;
}
