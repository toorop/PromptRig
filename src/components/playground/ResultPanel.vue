<script setup lang="ts">
import { computed } from "vue";
import type { Run } from "@/lib/bindings";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

const props = defineProps<{
  run: Run | null;
  error: string | null;
  running: boolean;
}>();

const usage = computed(() => props.run?.result?.usage ?? null);

function formatCost(cost: number | null | undefined): string {
  if (cost === null || cost === undefined) return "—";
  return `$${cost.toFixed(6)}`;
}

// Whatever text is actually on screen right now — a successful result, a Run's own error (e.g.
// a provider rejecting the request), or the top-level command error. The Copy button should
// work for all three, not just the success case.
const copyableText = computed(() => props.error ?? props.run?.error ?? props.run?.result?.text ?? null);

async function copyResult() {
  if (copyableText.value) {
    await navigator.clipboard.writeText(copyableText.value);
  }
}

// Selecting text in the result area with the mouse and copying it (Ctrl+C / right-click Copy)
// would otherwise copy the browser's default rich-HTML representation of the selection —
// including every inline style computed on the source elements (color, font, etc., see
// `main.css`'s theme variables) — which is useless when pasted anywhere that isn't itself a
// rich text editor. Intercepting the copy event and substituting the plain-text selection fixes
// that for any manual selection here, not just the dedicated "Copy" button above.
function forcePlainTextCopy(event: ClipboardEvent) {
  const selection = window.getSelection()?.toString();
  if (selection) {
    event.preventDefault();
    event.clipboardData?.setData("text/plain", selection);
  }
}
</script>

<template>
  <Card class="flex flex-col overflow-visible">
    <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
      <CardTitle class="text-sm font-medium">Result</CardTitle>
      <Button v-if="copyableText" variant="ghost" size="sm" @click="copyResult">Copy</Button>
    </CardHeader>

    <CardContent class="flex flex-1 flex-col gap-3" @copy="forcePlainTextCopy">
      <p v-if="running" class="font-mono text-sm text-muted-foreground">Running…</p>
      <p v-else-if="error" class="font-mono text-sm text-destructive">{{ error }}</p>
      <template v-else-if="run">
        <p v-if="run.error" class="font-mono text-sm text-destructive">{{ run.error }}</p>
        <pre v-else class="flex-1 whitespace-pre-wrap font-mono text-sm select-text">{{ run.result?.text }}</pre>

        <div class="mt-auto flex flex-wrap gap-2 border-t pt-3">
          <Badge variant="secondary">{{ run.result?.duration_ms ?? 0 }} ms</Badge>
          <Badge v-if="usage" variant="secondary">
            {{ usage.input_tokens }} in / {{ usage.output_tokens }} out
          </Badge>
          <Badge variant="secondary">{{ formatCost(run.estimated_cost_usd) }}</Badge>
        </div>
      </template>
      <p v-else class="font-mono text-sm text-muted-foreground">Run a prompt to see the result here.</p>
    </CardContent>
  </Card>
</template>
