<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { commands, type ModelInfo, type ProviderId, type Run } from "@/lib/bindings";
import { useProvidersStore } from "@/stores/providers";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import ResultPanel from "@/components/playground/ResultPanel.vue";

// Single-model playground: pick a provider/model, write prompts, run, inspect the result.
const providersStore = useProvidersStore();

// Remembers the last provider/model the user actually ran something with, so re-opening the
// Playground doesn't silently default to whichever model happens to sort first (which could be
// an expensive flagship model) — it's a per-device convenience, not shared/sensitive data, so
// localStorage is enough; wrapped defensively since a private-browsing-style webview could
// reject storage access.
const LAST_SELECTION_KEY = "promptrig.playground.lastSelection";

interface LastSelection {
  provider?: ProviderId;
  modelId?: string;
}

function loadLastSelection(): LastSelection {
  try {
    const raw = localStorage.getItem(LAST_SELECTION_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveLastSelection(selection: LastSelection) {
  try {
    localStorage.setItem(LAST_SELECTION_KEY, JSON.stringify(selection));
  } catch {
    // Best-effort only — losing the remembered selection isn't worth surfacing an error for.
  }
}

const selectedProvider = ref<ProviderId>();
const models = ref<ModelInfo[]>([]);
const modelsLoading = ref(false);
const modelsError = ref<string | null>(null);
const selectedModelId = ref<string>();

const systemPrompt = ref("");
const userPrompt = ref("");
// Kept as separate optional refs (rather than one GenerationParams object) so an empty input
// naturally means "unset" — GenerationParams itself is only assembled at Run time, converting
// `undefined` to `null` for the fields the backend expects. Pre-filled with commonly-used
// defaults (rather than left blank) so a first-time Run doesn't require understanding these
// before pressing the button.
const temperature = ref<number>(0.7);
const topP = ref<number>(1);
const maxTokens = ref<number>(1024);

const running = ref(false);
const run = ref<Run | null>(null);
const runError = ref<string | null>(null);

const configuredProviders = computed(() =>
  providersStore.providers.filter((p) => p.implemented && p.configured),
);

const selectedModel = computed(
  () => models.value.find((m) => m.model_id === selectedModelId.value) ?? null,
);

const canRun = computed(
  () =>
    !!selectedProvider.value &&
    !!selectedModelId.value &&
    userPrompt.value.trim().length > 0 &&
    !running.value,
);

onMounted(async () => {
  await providersStore.refresh();
  if (selectedProvider.value) return;

  const remembered = loadLastSelection();
  const rememberedIsConfigured = configuredProviders.value.some(
    (p) => p.provider === remembered.provider,
  );
  selectedProvider.value = rememberedIsConfigured
    ? remembered.provider
    : configuredProviders.value[0]?.provider;
});

watch(selectedProvider, async (provider) => {
  models.value = [];
  selectedModelId.value = undefined;
  modelsError.value = null;
  if (!provider) return;

  modelsLoading.value = true;
  const result = await commands.listModels(provider);
  if (result.status === "ok") {
    models.value = result.data;

    // Prefer the model used last time with this same provider. Otherwise, leave it unselected
    // rather than guessing — auto-picking an arbitrary (possibly expensive) model was the exact
    // surprise this is meant to avoid.
    const remembered = loadLastSelection();
    if (remembered.provider === provider && models.value.some((m) => m.model_id === remembered.modelId)) {
      selectedModelId.value = remembered.modelId;
    }
  } else {
    modelsError.value = result.error.message;
  }
  modelsLoading.value = false;
});

watch([selectedProvider, selectedModelId], ([provider, modelId]) => {
  if (provider && modelId) {
    saveLastSelection({ provider, modelId });
  }
});

async function runGeneration() {
  if (!selectedProvider.value || !selectedModelId.value) return;

  running.value = true;
  runError.value = null;
  run.value = null;

  const result = await commands.runGeneration({
    provider: selectedProvider.value,
    model_id: selectedModelId.value,
    system_prompt: systemPrompt.value,
    user_prompt: userPrompt.value,
    params: {
      temperature: temperature.value ?? null,
      top_p: topP.value ?? null,
      max_tokens: maxTokens.value ?? null,
    },
  });

  if (result.status === "ok") {
    run.value = result.data;
  } else {
    runError.value = result.error.message;
  }

  running.value = false;
}
</script>

<template>
  <div class="flex h-full flex-col gap-4 p-4">
    <div class="flex flex-wrap items-end gap-3 rounded-xl bg-card p-3 ring-1 ring-foreground/10">
      <div class="flex flex-col gap-1.5">
        <Label class="text-xs text-muted-foreground">Provider</Label>
        <Select v-model="selectedProvider">
          <SelectTrigger class="w-48">
            <SelectValue placeholder="Select a provider" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="p in configuredProviders" :key="p.provider" :value="p.provider">
              {{ p.display_name }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="flex flex-col gap-1.5">
        <Label class="text-xs text-muted-foreground">Model</Label>
        <Select v-model="selectedModelId" :disabled="!selectedProvider || modelsLoading">
          <SelectTrigger class="w-64">
            <SelectValue :placeholder="modelsLoading ? 'Loading models…' : 'Select a model'" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="m in models" :key="m.model_id" :value="m.model_id">
              {{ m.display_name }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <Button class="ml-auto" size="lg" :disabled="!canRun" @click="runGeneration">
        {{ running ? "Running…" : "Run" }}
      </Button>
    </div>

    <p v-if="configuredProviders.length === 0" class="text-sm text-muted-foreground">
      No provider is configured yet. Go to
      <RouterLink to="/settings" class="font-medium text-foreground underline underline-offset-2">
        Settings
      </RouterLink>
      to add an API key.
    </p>
    <p v-if="modelsError" class="text-sm text-destructive">{{ modelsError }}</p>

    <div class="grid flex-1 grid-cols-1 gap-4 overflow-auto md:grid-cols-2">
      <Card class="flex flex-col gap-4 overflow-auto p-4">
        <div class="flex flex-col gap-1.5">
          <Label for="system-prompt">System prompt</Label>
          <Textarea
            id="system-prompt"
            v-model="systemPrompt"
            rows="6"
            placeholder="You are a helpful assistant."
          />
        </div>

        <div class="flex flex-1 flex-col gap-1.5">
          <Label for="user-prompt">User prompt</Label>
          <Textarea
            id="user-prompt"
            v-model="userPrompt"
            rows="10"
            class="flex-1"
            placeholder="Ask something…"
          />
        </div>

        <div v-if="selectedModel" class="flex flex-wrap gap-4 border-t pt-4">
          <div v-if="selectedModel.capabilities.supports_temperature" class="flex flex-col gap-1.5">
            <Label for="temperature" class="text-xs text-muted-foreground">Temperature</Label>
            <Input
              id="temperature"
              v-model.number="temperature"
              type="number"
              step="0.1"
              min="0"
              max="2"
              class="w-24"
            />
            <p class="text-xs text-muted-foreground">0 = deterministic, 2 = very random</p>
          </div>
          <div v-if="selectedModel.capabilities.supports_top_p" class="flex flex-col gap-1.5">
            <Label for="top-p" class="text-xs text-muted-foreground">Top P</Label>
            <Input
              id="top-p"
              v-model.number="topP"
              type="number"
              step="0.05"
              min="0"
              max="1"
              class="w-24"
            />
            <p class="text-xs text-muted-foreground">Alt. to temperature, usually tune one</p>
          </div>
          <div v-if="selectedModel.capabilities.supports_max_tokens" class="flex flex-col gap-1.5">
            <Label for="max-tokens" class="text-xs text-muted-foreground">Max tokens</Label>
            <Input id="max-tokens" v-model.number="maxTokens" type="number" step="1" min="1" class="w-28" />
            <p class="text-xs text-muted-foreground">Caps response length &amp; cost</p>
          </div>
        </div>
      </Card>

      <ResultPanel :run="run" :error="runError" :running="running" />
    </div>
  </div>
</template>
