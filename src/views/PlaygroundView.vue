<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { commands, type ProviderId, type Run } from "@/lib/bindings";
import { useProvidersStore } from "@/stores/providers";
import { usePromptDraftStore } from "@/stores/promptDraft";
import { RefreshCw } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import GenerationParamsFields from "@/components/playground/GenerationParamsFields.vue";
import ResultPanel from "@/components/playground/ResultPanel.vue";
import CopyButton from "@/components/CopyButton.vue";
import ResetButton from "@/components/ResetButton.vue";
import LabelHint from "@/components/LabelHint.vue";

// Single-model playground: pick a provider/model, write prompts, run, inspect the result. The
// prompt/params themselves live in a shared store (see stores/promptDraft.ts) so they carry
// over to/from the Compare view without an explicit hand-off step.
const providersStore = useProvidersStore();
const draft = usePromptDraftStore();
const { systemPrompt, userPrompt, temperature, topP, maxTokens } = storeToRefs(draft);

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
const selectedModelId = ref<string>();

const running = ref(false);
const run = ref<Run | null>(null);
const runError = ref<string | null>(null);

const configuredProviders = computed(() =>
  providersStore.providers.filter((p) => p.implemented && p.configured),
);

// Model lists live in the providers store (cached for the app's session — see
// stores/providers.ts) rather than as local state here, so picking the same provider again
// (or on the Compare view) doesn't re-fetch.
const models = computed(() => (selectedProvider.value ? providersStore.modelsByProvider[selectedProvider.value] ?? [] : []));
const modelsLoading = computed(() => (selectedProvider.value ? !!providersStore.modelsLoading[selectedProvider.value] : false));
const modelsError = computed(() => (selectedProvider.value ? providersStore.modelsError[selectedProvider.value] ?? null : null));

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
  selectedModelId.value = undefined;
  if (!provider) return;

  const fetchedModels = await providersStore.loadModels(provider);

  // Prefer the model used last time with this same provider. Otherwise, leave it unselected
  // rather than guessing — auto-picking an arbitrary (possibly expensive) model was the exact
  // surprise this is meant to avoid.
  const remembered = loadLastSelection();
  if (remembered.provider === provider && fetchedModels.some((m) => m.model_id === remembered.modelId)) {
    selectedModelId.value = remembered.modelId;
  }
});

async function refreshModels() {
  if (!selectedProvider.value) return;
  // Clear the current selection first so the Select shows the "Loading models…" placeholder
  // instead of just greying out over whatever was already selected — otherwise there's no
  // visible sign anything is happening.
  selectedModelId.value = undefined;
  await providersStore.loadModels(selectedProvider.value, { force: true });
}

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
        <LabelHint
          field-id="provider-select"
          hint="The API/company that will run the model: OpenAI, Anthropic, Google Gemini, Mistral, or OpenRouter."
        >
          Provider
        </LabelHint>
        <Select v-model="selectedProvider">
          <SelectTrigger id="provider-select" class="w-48 text-[15px]">
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
        <LabelHint
          field-id="model-select"
          hint="The specific model to run. Newer/larger models are usually more capable but cost more per token."
        >
          Model
        </LabelHint>
        <div class="flex gap-1.5">
          <Select v-model="selectedModelId" :disabled="!selectedProvider || modelsLoading">
            <SelectTrigger id="model-select" class="w-64 text-[15px]">
              <SelectValue :placeholder="modelsLoading ? 'Loading models…' : 'Select a model'" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="m in models" :key="m.model_id" :value="m.model_id">
                {{ m.display_name }}
              </SelectItem>
            </SelectContent>
          </Select>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                variant="outline"
                size="icon"
                :disabled="!selectedProvider || modelsLoading"
                @click="refreshModels"
              >
                <RefreshCw class="size-4" :class="{ 'animate-spin': modelsLoading }" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Re-fetch the model list from the provider</TooltipContent>
          </Tooltip>
        </div>
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
          <div class="flex items-center justify-between">
            <LabelHint
              field-id="system-prompt"
              hint="Instructions that set the assistant's behavior, tone, and constraints for this run."
            >
              System prompt
            </LabelHint>
            <div class="flex items-center gap-0.5">
              <CopyButton :text="systemPrompt" />
              <ResetButton :disabled="!systemPrompt" @click="systemPrompt = ''" />
            </div>
          </div>
          <Textarea
            id="system-prompt"
            v-model="systemPrompt"
            rows="6"
            class="font-mono text-sm"
            placeholder="You are a helpful assistant."
          />
        </div>

        <div class="flex flex-1 flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <LabelHint field-id="user-prompt" hint="The actual message or question sent to the model.">
              User prompt
            </LabelHint>
            <div class="flex items-center gap-0.5">
              <CopyButton :text="userPrompt" />
              <ResetButton :disabled="!userPrompt" @click="userPrompt = ''" />
            </div>
          </div>
          <Textarea
            id="user-prompt"
            v-model="userPrompt"
            rows="10"
            class="flex-1 font-mono text-sm"
            placeholder="Ask something…"
          />
        </div>

        <div v-if="selectedModel" class="border-t pt-4">
          <GenerationParamsFields :capabilities="selectedModel.capabilities" />
        </div>
      </Card>

      <ResultPanel :run="run" :error="runError" :running="running" />
    </div>
  </div>
</template>
