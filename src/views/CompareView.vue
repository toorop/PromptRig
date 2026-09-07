<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { commands, type ProviderId } from "@/lib/bindings";
import { useProvidersStore } from "@/stores/providers";
import { usePromptDraftStore } from "@/stores/promptDraft";
import { useCompareStore, type CompareColumn } from "@/stores/compare";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import GenerationParamsFields from "@/components/playground/GenerationParamsFields.vue";
import ResultPanel from "@/components/playground/ResultPanel.vue";
import CopyButton from "@/components/CopyButton.vue";

// Side-by-side comparison: one Experiment, one Run per column. The shared system/user prompt
// and params come from the same store Playground uses (see stores/promptDraft.ts) — Compare
// only adds per-column Provider+Model selection on top of that. Column state itself (see
// stores/compare.ts) is also a store, not local component state — otherwise it would reset
// every time this view is left and come back to (Vue destroys a view's local state on
// navigation away).
const providersStore = useProvidersStore();
const draft = usePromptDraftStore();
const { systemPrompt, userPrompt, temperature, topP, maxTokens } = storeToRefs(draft);
const compareStore = useCompareStore();
const { columns } = storeToRefs(compareStore);
const { addColumn, removeColumn } = compareStore;

// Unlike Playground/Settings, Compare doesn't otherwise trigger a fetch of the provider list —
// without this, landing here first (a fresh navigation, or a full page reload) shows "no
// provider configured" even when one is, until some other view happens to call refresh().
onMounted(() => {
  providersStore.refresh();
});

const configuredProviders = computed(() =>
  providersStore.providers.filter((p) => p.implemented && p.configured),
);

async function onProviderChange(column: CompareColumn, provider: ProviderId | undefined) {
  column.provider = provider;
  column.modelId = undefined;
  if (!provider) return;

  // Cached in the providers store — picking the same provider on another column reuses this
  // instead of firing a duplicate request.
  await providersStore.loadModels(provider);
}

function modelsForColumn(column: CompareColumn) {
  return column.provider ? (providersStore.modelsByProvider[column.provider] ?? []) : [];
}

function modelsLoadingForColumn(column: CompareColumn) {
  return column.provider ? !!providersStore.modelsLoading[column.provider] : false;
}

function modelsErrorForColumn(column: CompareColumn) {
  return column.provider ? (providersStore.modelsError[column.provider] ?? null) : null;
}

async function refreshModelsForColumn(column: CompareColumn) {
  if (column.provider) {
    // Clear the current selection first so the Select shows the "Loading models…" placeholder
    // instead of just greying out over whatever was already selected.
    column.modelId = undefined;
    await providersStore.loadModels(column.provider, { force: true });
  }
}

const running = ref(false);
const readyColumns = computed(() => columns.value.filter((c) => c.provider && c.modelId));
const canRunAll = computed(
  () =>
    !running.value &&
    userPrompt.value.trim().length > 0 &&
    columns.value.length > 0 &&
    readyColumns.value.length === columns.value.length,
);

async function runAll() {
  if (!canRunAll.value) return;

  running.value = true;
  for (const column of columns.value) {
    column.running = true;
    column.run = null;
    column.runError = null;
  }

  const result = await commands.runExperiment({
    system_prompt: systemPrompt.value,
    user_prompt: userPrompt.value,
    params: {
      temperature: temperature.value ?? null,
      top_p: topP.value ?? null,
      max_tokens: maxTokens.value ?? null,
    },
    // The backend runs these concurrently but returns results in this same order, so they can
    // be zipped back to columns by index.
    columns: columns.value.map((c) => ({ provider: c.provider as ProviderId, model_id: c.modelId as string })),
  });

  if (result.status === "ok") {
    result.data.runs.forEach((run, index) => {
      const column = columns.value[index];
      if (column) {
        column.run = run;
        column.running = false;
      }
    });
  } else {
    for (const column of columns.value) {
      column.runError = result.error.message;
      column.running = false;
    }
  }

  running.value = false;
}

// Reruns just this one column via the same single-run command Playground uses. Note: unlike
// `runAll`, this creates a standalone Run rather than one attached to the original Experiment —
// fine for now since there's no "browse past experiments" view yet that would care.
async function rerunColumn(column: CompareColumn) {
  if (!column.provider || !column.modelId) return;

  column.running = true;
  column.run = null;
  column.runError = null;

  const result = await commands.runGeneration({
    provider: column.provider,
    model_id: column.modelId,
    system_prompt: systemPrompt.value,
    user_prompt: userPrompt.value,
    params: {
      temperature: temperature.value ?? null,
      top_p: topP.value ?? null,
      max_tokens: maxTokens.value ?? null,
    },
  });

  if (result.status === "ok") {
    column.run = result.data;
  } else {
    column.runError = result.error.message;
  }
  column.running = false;
}
</script>

<template>
  <div class="flex h-full flex-col gap-4 p-4">
    <Card class="flex flex-col gap-4 p-4">
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <Label for="compare-system-prompt">System prompt</Label>
            <CopyButton :text="systemPrompt" />
          </div>
          <Textarea
            id="compare-system-prompt"
            v-model="systemPrompt"
            rows="4"
            placeholder="You are a helpful assistant."
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <Label for="compare-user-prompt">User prompt</Label>
            <CopyButton :text="userPrompt" />
          </div>
          <Textarea id="compare-user-prompt" v-model="userPrompt" rows="4" placeholder="Ask something…" />
        </div>
      </div>

      <div class="flex flex-wrap items-end justify-between gap-4 border-t pt-4">
        <GenerationParamsFields />
        <div class="flex gap-2">
          <Button variant="outline" @click="addColumn">Add column</Button>
          <Button size="lg" :disabled="!canRunAll" @click="runAll">
            {{ running ? "Running…" : "Run all" }}
          </Button>
        </div>
      </div>
    </Card>

    <p v-if="configuredProviders.length === 0" class="text-sm text-muted-foreground">
      No provider is configured yet. Go to
      <RouterLink to="/settings" class="font-medium text-foreground underline underline-offset-2">
        Settings
      </RouterLink>
      to add an API key.
    </p>

    <div class="flex flex-1 gap-4 overflow-x-auto pb-2">
      <Card v-for="column in columns" :key="column.key" class="flex w-80 shrink-0 flex-col gap-3 p-3">
        <div class="flex items-start justify-between gap-2">
          <div class="flex flex-1 flex-col gap-2">
            <Select
              :model-value="column.provider"
              @update:model-value="(value) => onProviderChange(column, value as ProviderId)"
            >
              <SelectTrigger>
                <SelectValue placeholder="Provider" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="p in configuredProviders" :key="p.provider" :value="p.provider">
                  {{ p.display_name }}
                </SelectItem>
              </SelectContent>
            </Select>

            <div class="flex gap-1.5">
              <Select
                :model-value="column.modelId"
                :disabled="!column.provider || modelsLoadingForColumn(column)"
                @update:model-value="(value) => (column.modelId = value as string)"
              >
                <SelectTrigger>
                  <SelectValue :placeholder="modelsLoadingForColumn(column) ? 'Loading models…' : 'Model'" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="m in modelsForColumn(column)" :key="m.model_id" :value="m.model_id">
                    {{ m.display_name }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button
                variant="outline"
                size="sm"
                :disabled="!column.provider || modelsLoadingForColumn(column)"
                title="Re-fetch the model list from the provider"
                @click="refreshModelsForColumn(column)"
              >
                ↻
              </Button>
            </div>
            <p v-if="modelsErrorForColumn(column)" class="text-xs text-destructive">
              {{ modelsErrorForColumn(column) }}
            </p>
          </div>

          <Button
            variant="ghost"
            size="sm"
            :disabled="columns.length <= 1"
            title="Remove this column"
            @click="removeColumn(column)"
          >
            ✕
          </Button>
        </div>

        <Button
          v-if="column.run || column.runError"
          variant="outline"
          size="sm"
          :disabled="!column.provider || !column.modelId || column.running"
          @click="rerunColumn(column)"
        >
          {{ column.running ? "Running…" : "Rerun this column" }}
        </Button>

        <ResultPanel :run="column.run" :error="column.runError" :running="column.running" class="flex-1" />
      </Card>
    </div>
  </div>
</template>
