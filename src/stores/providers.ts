import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import { commands, type ModelInfo, type ProviderId, type ProviderStatus } from "@/lib/bindings";

// Shared between SettingsView (which changes provider configuration) and Playground/Compare
// (which only read it) — a Pinia store avoids one view needing to know when another changes
// something, or re-fetching on every navigation.
//
// The model list per provider is cached here too, for the app's lifetime (in memory — nothing
// is persisted to disk, so a fresh launch always re-fetches). This was a deliberate choice over
// a longer-lived on-disk cache: picking the same provider on several Compare columns no longer
// fires duplicate API calls, while every app launch still shows whatever the provider currently
// makes available — no "why is this a day stale" surprise, and no cache-invalidation logic to
// get wrong. `loadModels(provider, { force: true })` bypasses the cache for a manual refresh.
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

  const modelsByProvider = reactive<Partial<Record<ProviderId, ModelInfo[]>>>({});
  const modelsLoading = reactive<Partial<Record<ProviderId, boolean>>>({});
  const modelsError = reactive<Partial<Record<ProviderId, string | null>>>({});

  async function loadModels(provider: ProviderId, opts?: { force?: boolean }): Promise<ModelInfo[]> {
    if (!opts?.force && modelsByProvider[provider]) {
      return modelsByProvider[provider]!;
    }

    modelsLoading[provider] = true;
    modelsError[provider] = null;

    const result = await commands.listModels(provider);
    if (result.status === "ok") {
      modelsByProvider[provider] = result.data;
    } else {
      modelsError[provider] = result.error.message;
    }

    modelsLoading[provider] = false;
    return modelsByProvider[provider] ?? [];
  }

  return {
    providers,
    loading,
    error,
    refresh,
    modelsByProvider,
    modelsLoading,
    modelsError,
    loadModels,
  };
});
