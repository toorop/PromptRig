<script setup lang="ts">
import { Eraser } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

// Clears a prompt field back to empty. The field's v-model is owned by the parent (a Pinia
// store ref in every current use), so this component just asks for the clear via an event
// rather than taking the value itself — same division of responsibility as CopyButton, which
// only reads.
defineProps<{ disabled?: boolean }>();

const emit = defineEmits<{ (e: "click"): void }>();
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <Button
        variant="ghost"
        size="icon-xs"
        class="text-muted-foreground hover:text-foreground"
        :disabled="disabled"
        @click="emit('click')"
      >
        <Eraser />
      </Button>
    </TooltipTrigger>
    <TooltipContent>Clear this field</TooltipContent>
  </Tooltip>
</template>
