<script setup lang="ts">
import { computed, ref } from "vue";
import type { HTMLAttributes } from "vue";
import { Check, ChevronsUpDown } from "@lucide/vue";
import type { ModelInfo } from "@/lib/bindings";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";

// A searchable replacement for a plain <Select> specifically for the model picker — OpenRouter
// alone can return 400+ models, where a plain dropdown with no filtering is unusable. Provider
// pickers stay a plain <Select> (5-6 items, no need for search there).
const props = defineProps<{
  modelId?: string;
  models: ModelInfo[];
  disabled?: boolean;
  loading?: boolean;
  placeholder?: string;
  class?: HTMLAttributes["class"];
}>();

const emit = defineEmits<{
  (e: "update:modelId", value: string): void;
}>();

const open = ref(false);

const selected = computed(() => props.models.find((m) => m.model_id === props.modelId));

// A per-million-token rate, not a computed cost (no usage exists yet) — see the Rust side's
// ModelInfo.pricing doc comment. "≈" mirrors ResultPanel's cost badge for the same reason: this
// number came from the OpenRouter cross-provider approximation, not an exact hand-curated price.
function formatRate(model: ModelInfo): string | null {
  if (!model.pricing) return null;
  const { input_per_million_usd, output_per_million_usd, is_estimate } = model.pricing;
  // specta exports every f64 as `number | null` (NaN/Infinity have no JSON representation) even
  // though these two are never actually optional on the Rust side — the `?? 0` is just to
  // satisfy that type, not a real fallback case.
  return `${is_estimate ? "≈" : ""}$${(input_per_million_usd ?? 0).toFixed(2)}/$${(output_per_million_usd ?? 0).toFixed(2)}`;
}

function select(modelId: string) {
  emit("update:modelId", modelId);
  open.value = false;
}
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        role="combobox"
        :aria-expanded="open"
        :disabled="disabled"
        :class="cn('min-w-0 justify-between font-normal', props.class)"
      >
        <span class="truncate">
          {{ selected?.display_name ?? (loading ? "Loading models…" : (placeholder ?? "Select a model")) }}
        </span>
        <ChevronsUpDown class="ml-2 size-4 shrink-0 opacity-50" />
      </Button>
    </PopoverTrigger>
    <PopoverContent class="w-96 p-0" align="start">
      <Command>
        <CommandInput placeholder="Search models…" />
        <CommandList>
          <CommandEmpty>No model found.</CommandEmpty>
          <CommandGroup>
            <CommandItem
              v-for="model in models"
              :key="model.model_id"
              :value="model.model_id"
              @select="select(model.model_id)"
            >
              <Check :class="['mt-0.5 size-4 shrink-0 self-start', model.model_id === modelId ? 'opacity-100' : 'opacity-0']" />
              <div class="flex min-w-0 flex-1 flex-col">
                <span class="whitespace-normal break-words">{{ model.display_name }}</span>
                <span v-if="formatRate(model)" class="text-xs text-muted-foreground">
                  {{ formatRate(model) }}
                </span>
              </div>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
