import { defineStore } from "pinia";
import { ref, watch } from "vue";
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
  // Column-specific, unlike temperature/top_p/max_tokens (shared across all columns via
  // stores/promptDraft.ts): different models have different, non-overlapping sets of valid
  // reasoning-effort values (see ModelInfo.reasoning_effort_levels), so this can't be one global
  // control the way the other params are. Reset to undefined whenever this column's model
  // changes — see CompareView.vue's ModelCombobox handler — since a value valid for the old
  // model may not be valid for the new one.
  reasoningEffort?: string;
  run: Run | null;
  runError: string | null;
  running: boolean;
  // A pinned column is excluded from "Run all" (see CompareView.vue's runAll) so a model you've
  // already found good stays put while you try others against it — and, unlike every other
  // column, survives an app restart (see loadPinnedColumns/savePinnedColumns below). Everything
  // else about a pinned column (editing its provider/model, the per-column "Rerun" button) works
  // exactly as before; pinning only protects it from the *bulk* "Run all" sweep.
  pinned: boolean;
}

let nextKey = 0;
function makeColumn(): CompareColumn {
  nextKey += 1;
  return {
    key: `col-${nextKey}`,
    provider: undefined,
    modelId: undefined,
    reasoningEffort: undefined,
    run: null,
    runError: null,
    running: false,
    pinned: false,
  };
}

// Only pinned columns persist across a restart — deliberately simple (KISS, per the user):
// unpinned columns are working scratch space, gone on restart same as before this feature;
// only the "keep this one" case is worth the localStorage round-trip. A pinned column's `run`
// (the full result — text, usage, cost) is saved as-is; it's already a plain JSON-serializable
// value coming back from Tauri's IPC, no special handling needed.
const STORAGE_KEY = "promptrig.compare.pinnedColumns";

interface StoredPinnedColumn {
  provider?: ProviderId;
  modelId?: string;
  run: Run | null;
}

function loadPinnedColumns(): StoredPinnedColumn[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function savePinnedColumns(columns: CompareColumn[]) {
  try {
    const pinned: StoredPinnedColumn[] = columns
      .filter((c) => c.pinned)
      .map((c) => ({ provider: c.provider, modelId: c.modelId, run: c.run }));
    localStorage.setItem(STORAGE_KEY, JSON.stringify(pinned));
  } catch {
    // Best-effort only — losing pinned columns across a restart isn't worth surfacing an error.
  }
}

function restoredColumn(stored: StoredPinnedColumn): CompareColumn {
  return {
    ...makeColumn(),
    provider: stored.provider,
    modelId: stored.modelId,
    run: stored.run,
    pinned: true,
  };
}

export const useCompareStore = defineStore("compare", () => {
  const restored = loadPinnedColumns().map(restoredColumn);
  const columns = ref<CompareColumn[]>(restored.length > 0 ? [...restored, makeColumn()] : [makeColumn(), makeColumn()]);

  watch(columns, (value) => savePinnedColumns(value), { deep: true });

  function addColumn() {
    columns.value.push(makeColumn());
  }

  function removeColumn(column: CompareColumn) {
    columns.value = columns.value.filter((c) => c.key !== column.key);
  }

  function togglePin(column: CompareColumn) {
    column.pinned = !column.pinned;
  }

  return { columns, addColumn, removeColumn, togglePin };
});
