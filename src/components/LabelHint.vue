<script setup lang="ts">
import { CircleHelp } from "@lucide/vue";
import { Label } from "@/components/ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

// A form label with a small "?" icon next to it that reveals an explanatory tooltip on
// hover/focus — used for every prompt/param field so first-time users can learn what each one
// does without permanently-visible caption text cluttering the layout (see
// GenerationParamsFields.vue for where that pattern started).
defineProps<{
  fieldId: string;
  hint: string;
}>();
</script>

<template>
  <div class="flex items-center gap-1">
    <Label :for="fieldId" class="text-xs text-muted-foreground">
      <slot />
    </Label>
    <Tooltip>
      <TooltipTrigger as-child>
        <button type="button" class="text-muted-foreground hover:text-foreground">
          <CircleHelp class="size-3.5" />
        </button>
      </TooltipTrigger>
      <TooltipContent>{{ hint }}</TooltipContent>
    </Tooltip>
  </div>
</template>
