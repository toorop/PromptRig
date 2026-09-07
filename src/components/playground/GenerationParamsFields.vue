<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import type { ModelCapabilities } from "@/lib/bindings";
import { usePromptDraftStore } from "@/stores/promptDraft";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

// Shared by Playground (one model, so only shows the fields that model's capabilities support)
// and Compare (potentially several different models at once, so — with no `capabilities` prop —
// it shows every field; the backend already silently drops whatever a given column's model
// doesn't support, see providers::openai::build_request).
const props = defineProps<{
  capabilities?: ModelCapabilities | null;
}>();

const draft = usePromptDraftStore();
const { temperature, topP, maxTokens } = storeToRefs(draft);

const showTemperature = computed(() => props.capabilities?.supports_temperature ?? true);
const showTopP = computed(() => props.capabilities?.supports_top_p ?? true);
const showMaxTokens = computed(() => props.capabilities?.supports_max_tokens ?? true);
</script>

<template>
  <div class="flex flex-wrap gap-4">
    <div v-if="showTemperature" class="flex flex-col gap-1.5">
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
    <div v-if="showTopP" class="flex flex-col gap-1.5">
      <Label for="top-p" class="text-xs text-muted-foreground">Top P</Label>
      <Input id="top-p" v-model.number="topP" type="number" step="0.05" min="0" max="1" class="w-24" />
      <p class="text-xs text-muted-foreground">Alt. to temperature, usually tune one</p>
    </div>
    <div v-if="showMaxTokens" class="flex flex-col gap-1.5">
      <Label for="max-tokens" class="text-xs text-muted-foreground">Max tokens</Label>
      <Input id="max-tokens" v-model.number="maxTokens" type="number" step="1" min="1" class="w-28" />
      <p class="text-xs text-muted-foreground">Caps response length &amp; cost</p>
    </div>
  </div>
</template>
