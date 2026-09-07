import { defineStore } from "pinia";
import { ref } from "vue";
import { commands, type ProviderStatus } from "@/lib/bindings";

// Shared between SettingsView (which changes provider configuration) and PlaygroundView (which
// only reads it) — a Pinia store avoids Playground needing to know when Settings changes
// something, or re-fetching on every navigation.
export const useProvidersStore = defineStore("providers", () => {
  const providers = ref<ProviderStatus[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function refresh() {
    loading.value = true;
    error.value = null;

    const result = await commands.listProviders();
    if (result.status === "ok") {
      providers.value = result.data;
    } else {
      error.value = result.error.message;
    }

    loading.value = false;
  }

  return { providers, loading, error, refresh };
});
