// Shared between main.ts (applies the theme before the app mounts, avoiding a flash of the
// wrong theme) and stores/theme.ts (the reactive store backing the toggle button) — both need
// the exact same storage key and default so they can never disagree.
export const THEME_STORAGE_KEY = "promptrig.theme";
export type Theme = "light" | "dark";

export function loadStoredTheme(): Theme | null {
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    return raw === "light" || raw === "dark" ? raw : null;
  } catch {
    return null;
  }
}

export function applyTheme(theme: Theme) {
  document.documentElement.classList.toggle("dark", theme === "dark");
}
