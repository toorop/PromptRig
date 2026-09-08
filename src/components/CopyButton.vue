<script setup lang="ts">
import { ref } from "vue";
import { Check, Copy } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

// Small reusable "Copy" affordance for a plain text value (prompts, etc.) — copies straight
// from the JS string via the Clipboard API, so there's no risk of the rich-HTML-selection issue
// ResultPanel had to work around (see its `forcePlainTextCopy`).
const props = defineProps<{ text: string }>();

const copied = ref(false);

async function copy() {
  if (!props.text) return;
  await navigator.clipboard.writeText(props.text);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 1200);
}
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <Button
        variant="ghost"
        size="icon-xs"
        class="text-muted-foreground hover:text-foreground"
        :disabled="!text"
        @click="copy"
      >
        <Check v-if="copied" />
        <Copy v-else />
      </Button>
    </TooltipTrigger>
    <TooltipContent>{{ copied ? "Copied!" : "Copy" }}</TooltipContent>
  </Tooltip>
</template>
