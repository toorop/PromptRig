import { defineStore } from "pinia";
import { ref } from "vue";
import type { ModelInfo, ProviderId, Run } from "@/lib/bindings";

// Column state lives here (not as local component state in CompareView) so it survives
// navigating away and back — Vue destroys a view's local state when its route is left, but a
// Pinia store persists for the app's lifetime.
export interface CompareColumn {
  key: string;
  provider?: ProviderId;
  modelId?: string;
  models: ModelInfo[];
  modelsLoading: boolean;
  modelsError: string | null;
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
    models: [],
    modelsLoading: false,
    modelsError: null,
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
