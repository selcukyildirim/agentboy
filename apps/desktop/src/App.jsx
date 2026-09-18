import { ThemeProvider } from "./app/ThemeProvider";
import { I18nProvider } from "./app/I18nProvider";
import { ToastProvider } from "./app/ToastProvider";
import { AppDataProvider } from "./app/AppDataProvider";
import { AppShell } from "./components/AppShell";

export default function App() {
  return (
    <ThemeProvider>
      <I18nProvider>
        <ToastProvider>
          <AppDataProvider>
            <AppShell />
          </AppDataProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}
