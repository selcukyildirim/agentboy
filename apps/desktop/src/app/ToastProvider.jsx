import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import { Check, Info, TriangleAlert, X } from "lucide-react";

const ToastContext = createContext(null);

let seq = 0;

export function ToastProvider({ children }) {
  const [toasts, setToasts] = useState([]);
  const timers = useRef(new Map());

  const dismiss = useCallback((id) => {
    setToasts((list) => list.filter((t) => t.id !== id));
    const timer = timers.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.current.delete(id);
    }
  }, []);

  const push = useCallback(
    (message, type = "ok", duration = 3200) => {
      const id = ++seq;
      setToasts((list) => [...list, { id, message, type }]);
      const timer = setTimeout(() => dismiss(id), duration);
      timers.current.set(id, timer);
      return id;
    },
    [dismiss]
  );

  const value = useMemo(
    () => ({
      push,
      success: (m) => push(m, "ok"),
      error: (m) => push(m, "error", 5000),
      warn: (m) => push(m, "warn"),
      info: (m) => push(m, "info"),
      dismiss,
    }),
    [push, dismiss]
  );

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div className="toastStack" role="status" aria-live="polite">
        {toasts.map((toast) => (
          <div key={toast.id} className={`toast ${toast.type}`}>
            <span className="toastIcon">
              {toast.type === "error" || toast.type === "warn" ? (
                <TriangleAlert size={15} />
              ) : toast.type === "info" ? (
                <Info size={15} />
              ) : (
                <Check size={15} />
              )}
            </span>
            <span className="toastText">{toast.message}</span>
            <button
              className="toastClose"
              aria-label="close"
              onClick={() => dismiss(toast.id)}
            >
              <X size={13} />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return ctx;
}
