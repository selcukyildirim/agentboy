/** Formatting helpers for the UI. */

export function formatNumber(value) {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  return new Intl.NumberFormat("tr-TR").format(value);
}

export function formatPercent(value, digits = 0) {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  return `${(value * 100).toFixed(digits)}%`;
}

export function formatUsd(value, digits = 4) {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  if (value === 0) return "$0";
  if (value < 0.01) return `$${value.toFixed(digits)}`;
  return `$${value.toFixed(2)}`;
}

export function formatTokens(value) {
  if (!value) return "0";
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return String(value);
}

export function formatDuration(ms) {
  if (ms === null || ms === undefined || Number.isNaN(ms)) return "—";
  if (ms < 1000) return `${Math.round(ms)} ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)} sn`;
  const minutes = Math.floor(ms / 60_000);
  const seconds = Math.round((ms % 60_000) / 1000);
  return `${minutes} dk ${seconds} sn`;
}

export function formatBytes(bytes) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i += 1;
  }
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatDateTime(iso) {
  if (!iso) return "—";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat("tr-TR", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

export function relativeTime(iso) {
  if (!iso) return "—";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "—";
  const diff = Date.now() - date.getTime();
  const seconds = Math.round(diff / 1000);
  if (seconds < 60) return `${seconds} sn önce`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} dk önce`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} sa önce`;
  const days = Math.round(hours / 24);
  return `${days} gün önce`;
}

export function shortId(id, length = 8) {
  if (!id) return "—";
  return String(id).slice(0, length);
}
