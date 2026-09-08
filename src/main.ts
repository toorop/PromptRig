// Self-hosted font files (no network fetch at runtime, no dependency on whatever happens to be
// installed on the user's system — see main.css's --font-sans/--font-mono for how they're used).
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import "./assets/main.css";
import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import { applyTheme, loadStoredTheme } from "./lib/theme";

// Applied before mount so there's no flash of the wrong theme — dark is the app's default
// identity (see main.css) unless the user has switched to light before on this device.
applyTheme(loadStoredTheme() ?? "dark");

createApp(App).use(createPinia()).use(router).mount("#app");
