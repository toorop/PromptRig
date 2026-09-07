<script setup lang="ts">
import { ref } from "vue";
import { Button } from "@/components/ui/button";

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
  <Button variant="ghost" size="sm" class="h-auto px-1.5 py-0 text-xs" :disabled="!text" @click="copy">
    {{ copied ? "Copied" : "Copy" }}
  </Button>
</template>
