import { defineStore } from "pinia";
import { ref } from "vue";
import type { ProviderId, Run } from "@/lib/bindings";

// Column state lives here (not as local component state in CompareView) so it survives
// navigating away and back — Vue destroys a view's local state when its route is left, but a
// Pinia store persists for the app's lifetime.
//
// Model lists are deliberately *not* stored per column: they live in the providers store's
// session cache (stores/providers.ts) keyed by provider, so two columns on the same provider
// share one fetch instead of each keeping their own copy.
export interface CompareColumn {
  key: string;
  provider?: ProviderId;
  modelId?: string;
  run: Run | null;
  runError: string | null;
  running: boolean;
}

let nextKey = 0;
function makeColumn(): CompareColumn {
  nextKey += 1;
  return {
    key: `col-${nextKey}`,
    provider: undefined,
    modelId: undefined,
    run: null,
    runError: null,
    running: false,
  };
}

export const useCompareStore = defineStore("compare", () => {
  const columns = ref<CompareColumn[]>([makeColumn(), makeColumn()]);

  function addColumn() {
    columns.value.push(makeColumn());
  }

  function removeColumn(column: CompareColumn) {
    columns.value = columns.value.filter((c) => c.key !== column.key);
  }

  return { columns, addColumn, removeColumn };
});
