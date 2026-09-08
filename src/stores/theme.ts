import { ref, watch } from "vue";
import { defineStore } from "pinia";
import { applyTheme, loadStoredTheme, THEME_STORAGE_KEY, type Theme } from "@/lib/theme";

// Dark is the app's default identity (see main.css's Nord palette) — light mode is opt-in and
// remembered per device. main.ts already applies the persisted/default theme before this store
// is ever created (so there's no flash of the wrong theme); the `immediate` watch below just
// keeps things in sync from here on, including writing back to storage.
export const useThemeStore = defineStore("theme", () => {
  const theme = ref<Theme>(loadStoredTheme() ?? "dark");

  watch(
    theme,
    (value) => {
      applyTheme(value);
      try {
        localStorage.setItem(THEME_STORAGE_KEY, value);
      } catch {
        // Best-effort only — losing the remembered theme isn't worth surfacing an error for.
      }
    },
    { immediate: true },
  );

  function toggle() {
    theme.value = theme.value === "dark" ? "light" : "dark";
  }

  return { theme, toggle };
});
