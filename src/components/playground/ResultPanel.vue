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

async function copyResult() {
  const text = props.run?.result?.text;
  if (text) {
    await navigator.clipboard.writeText(text);
  }
}
</script>

<template>
  <Card class="flex flex-col overflow-hidden">
    <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
      <CardTitle class="text-sm font-medium">Result</CardTitle>
      <Button v-if="run?.result" variant="ghost" size="sm" @click="copyResult">Copy</Button>
    </CardHeader>

    <CardContent class="flex flex-1 flex-col gap-3 overflow-auto">
      <p v-if="running" class="text-sm text-muted-foreground">Running…</p>
      <p v-else-if="error" class="text-sm text-destructive">{{ error }}</p>
      <template v-else-if="run">
        <p v-if="run.error" class="text-sm text-destructive">{{ run.error }}</p>
        <pre v-else class="flex-1 whitespace-pre-wrap text-sm select-text">{{ run.result?.text }}</pre>

        <div class="mt-auto flex flex-wrap gap-2 border-t pt-3">
          <Badge variant="secondary">{{ run.result?.duration_ms ?? 0 }} ms</Badge>
          <Badge v-if="usage" variant="secondary">
            {{ usage.input_tokens }} in / {{ usage.output_tokens }} out
          </Badge>
          <Badge variant="secondary">{{ formatCost(run.estimated_cost_usd) }}</Badge>
        </div>
      </template>
      <p v-else class="text-sm text-muted-foreground">Run a prompt to see the result here.</p>
    </CardContent>
  </Card>
</template>
