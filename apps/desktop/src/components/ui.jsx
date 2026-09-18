import { useEffect } from "react";
import { X } from "lucide-react";

export function cx(...parts) {
  return parts.filter(Boolean).join(" ");
}

export function Button({
  variant = "primary",
  size = "md",
  icon: Icon,
  children,
  className,
  ...rest
}) {
  return (
    <button className={cx("btn", variant, size, className)} {...rest}>
      {Icon ? <Icon size={size === "sm" ? 13 : 14} /> : null}
      {children}
    </button>
  );
}

export function IconButton({ label, active, children, className, ...rest }) {
  return (
    <button
      type="button"
      className={cx("iconBtn", active && "active", className)}
      title={label}
      aria-label={label}
      aria-pressed={active || undefined}
      {...rest}
    >
      {children}
    </button>
  );
}

export function Panel({ children, className, ...rest }) {
  return (
    <section className={cx("panel", className)} {...rest}>
      {children}
    </section>
  );
}

export function PanelHead({ title, subtitle, actions }) {
  return (
    <header className="panelHead">
      <div>
        <h2>{title}</h2>
        {subtitle ? <span>{subtitle}</span> : null}
      </div>
      {actions ? <div className="panelActions">{actions}</div> : null}
    </header>
  );
}

export function ScrollPanel({ children, className }) {
  return <div className={cx("scrollArea", className)}>{children}</div>;
}

export function Modal({ title, eyebrow, onClose, children, footer, size }) {
  useEffect(() => {
    const onKey = (e) => {
      if (e.key === "Escape") onClose?.();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="modalOverlay" onMouseDown={onClose} role="presentation">
      <div
        className={cx("modalCard", size === "lg" && "lg")}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="modalHead">
          <div>
            {eyebrow ? <span>{eyebrow}</span> : null}
            <h2>{title}</h2>
          </div>
          <button className="modalClose" aria-label="close" onClick={onClose}>
            <X size={17} />
          </button>
        </div>
        <div className="modalBody">{children}</div>
        {footer ? <div className="modalActions">{footer}</div> : null}
      </div>
    </div>
  );
}

export function ConfirmDialog({ title, message, confirmLabel, danger, onConfirm, onClose }) {
  return (
    <Modal
      title={title}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant={danger ? "danger" : "primary"}
            onClick={() => {
              onConfirm();
              onClose();
            }}
          >
            {confirmLabel}
          </Button>
        </>
      }
    >
      <p className="dialogText">{message}</p>
    </Modal>
  );
}

export function Switch({ checked, onChange, label }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={cx("switch", checked && "on")}
      onClick={() => onChange(!checked)}
    >
      <i />
    </button>
  );
}

export function Field({ label, hint, children }) {
  return (
    <label className="field">
      {label ? <span className="fieldLabel">{label}</span> : null}
      {children}
      {hint ? <span className="fieldHint">{hint}</span> : null}
    </label>
  );
}

export function Badge({ tone = "default", children }) {
  return <span className={cx("badge", tone)}>{children}</span>;
}

export function StatusPill({ status }) {
  const norm = String(status || "").toLowerCase();
  const tone =
    norm === "completed" || norm === "ready"
      ? "ok"
      : norm === "failed"
      ? "error"
      : norm === "running" || norm === "pending"
      ? "warn"
      : "muted";
  return (
    <span className={cx("statusPill", tone)}>
      <i />
      {status}
    </span>
  );
}

export function Spinner({ label }) {
  return (
    <div className="spinnerWrap" role="status">
      <span className="spinner" />
      {label ? <span className="spinnerLabel">{label}</span> : null}
    </div>
  );
}

export function EmptyState({ icon: Icon, title, hint, action }) {
  return (
    <div className="emptyState">
      {Icon ? <Icon size={26} /> : null}
      <strong>{title}</strong>
      {hint ? <span>{hint}</span> : null}
      {action}
    </div>
  );
}
