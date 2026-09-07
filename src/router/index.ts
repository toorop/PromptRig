import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  // Tauri serves the frontend from a local dev server / packaged assets, so plain
  // history mode works fine here (no server-side rewrite rules needed).
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "playground",
      component: () => import("@/views/PlaygroundView.vue"),
    },
    {
      path: "/compare",
      name: "compare",
      component: () => import("@/views/CompareView.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/SettingsView.vue"),
    },
  ],
});

export default router;
