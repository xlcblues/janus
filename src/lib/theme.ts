export type Theme = "light" | "dark";

export const THEME_STORAGE_KEY = "janus-theme";

function canUseStorage(): boolean {
  return typeof window !== "undefined" && Boolean(window.localStorage);
}

export function getInitialTheme(): Theme {
  if (!canUseStorage()) return "light";

  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark") return stored;
  } catch {
    // A locked-down WebView can reject localStorage access; system preference remains usable.
  }

  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function applyTheme(theme: Theme, persist = true): void {
  if (typeof document === "undefined") return;

  document.documentElement.dataset.theme = theme;

  if (!persist || !canUseStorage()) return;

  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // Theme still applies for this session when persistence is unavailable.
  }
}

export function nextTheme(theme: Theme): Theme {
  return theme === "dark" ? "light" : "dark";
}
