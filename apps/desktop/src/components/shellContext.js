import { createContext, useContext } from "react";

export const ShellContext = createContext(null);

export function useShell() {
  const ctx = useContext(ShellContext);
  if (!ctx) throw new Error("useShell must be used within AppShell");
  return ctx;
}
