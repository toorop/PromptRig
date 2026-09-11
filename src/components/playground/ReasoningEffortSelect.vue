<script setup lang="ts">
import { computed } from "vue";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import LabelHint from "@/components/LabelHint.vue";

// Only rendered when the selected model actually reports reasoning-effort levels (see
// ModelInfo.reasoning_effort_levels) — hidden entirely rather than shown empty, same
// don't-show-what-isn't-supported rule GenerationParamsFields already follows for
// temperature/top_p. Many reasoning models (most of Gemini's, for instance) have no such control
// at all — they reason unconditionally — so this being absent is the common case, not an edge
// case to explain away.
const props = defineProps<{
  levels: string[] | null | undefined;
  modelValue: string | undefined;
  fieldId?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string | undefined];
}>();

const hasLevels = computed(() => !!props.levels && props.levels.length > 0);
const id = computed(() => props.fieldId ?? "reasoning-effort");
</script>

<template>
  <div v-if="hasLevels" class="flex flex-col gap-1.5">
    <LabelHint
      :field-id="id"
      hint="How hard the model 'thinks' before answering. Higher levels use more tokens (and time) but can improve quality on hard tasks. Leave unset to let the model decide."
    >
      Reasoning
    </LabelHint>
    <Select
      :model-value="modelValue"
      @update:model-value="(value) => emit('update:modelValue', value as string | undefined)"
    >
      <SelectTrigger :id="id" class="w-28 min-w-0 text-[15px]">
        <SelectValue placeholder="Default" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem v-for="level in levels" :key="level" :value="level">
          {{ level }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>
