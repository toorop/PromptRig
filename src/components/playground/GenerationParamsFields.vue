<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import type { ModelCapabilities } from "@/lib/bindings";
import { usePromptDraftStore } from "@/stores/promptDraft";
import { Input } from "@/components/ui/input";
import LabelHint from "@/components/LabelHint.vue";

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

// Native number-input min/max attributes only affect the spinner buttons, not typed or pasted
// input — values still need clamping once the user leaves the field, so the underlying request
// never carries something out of range (e.g. a temperature of 20).
function clamp(value: number, min: number, max: number): number {
  if (typeof value !== "number" || Number.isNaN(value)) return min;
  return Math.min(max, Math.max(min, value));
}

function clampTemperature() {
  temperature.value = clamp(temperature.value, 0, 2);
}

function clampTopP() {
  topP.value = clamp(topP.value, 0, 1);
}

function clampMaxTokens() {
  maxTokens.value = Math.max(1, Math.round(clamp(maxTokens.value, 1, Infinity)));
}
</script>

<template>
  <div class="flex flex-wrap gap-4">
    <div v-if="showTemperature" class="flex flex-col gap-1.5">
      <LabelHint field-id="temperature" hint="0 = deterministic, 2 = very random">Temperature</LabelHint>
      <Input
        id="temperature"
        v-model.number="temperature"
        type="number"
        step="0.1"
        min="0"
        max="2"
        class="w-24 text-[15px]"
        @change="clampTemperature"
      />
    </div>
    <div v-if="showTopP" class="flex flex-col gap-1.5">
      <LabelHint field-id="top-p" hint="Alt. to temperature, usually tune one">Top P</LabelHint>
      <Input
        id="top-p"
        v-model.number="topP"
        type="number"
        step="0.05"
        min="0"
        max="1"
        class="w-24 text-[15px]"
        @change="clampTopP"
      />
    </div>
    <div v-if="showMaxTokens" class="flex flex-col gap-1.5">
      <LabelHint field-id="max-tokens" hint="Caps response length & cost">Max tokens</LabelHint>
      <Input
        id="max-tokens"
        v-model.number="maxTokens"
        type="number"
        step="1"
        min="1"
        class="w-28 text-[15px]"
        @change="clampMaxTokens"
      />
    </div>
  </div>
</template>
