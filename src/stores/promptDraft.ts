import { defineStore } from "pinia";
import { ref, watch } from "vue";

// The system prompt, user prompt, and base generation params are shared between Playground and
// Compare — editing either in one view carries over to the other, with no explicit "send to
// Compare" action and no requirement to visit one view before the other. Compare adds
// per-column Provider+Model selection on top of this; it doesn't duplicate the prompt/params
// editing UI concept.

// Only the prompts themselves survive an app restart (not the params) — the same per-device
// localStorage pattern PlaygroundView.vue already uses for "last used provider/model"
// (`loadLastSelection`/`saveLastSelection`), just for these two fields instead.
const STORAGE_KEY = "promptrig.promptDraft";

interface StoredDraft {
  systemPrompt?: string;
  userPrompt?: string;
}

function loadStoredDraft(): StoredDraft {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveStoredDraft(draft: StoredDraft) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(draft));
  } catch {
    // Best-effort only — losing the remembered draft isn't worth surfacing an error for.
  }
}

export const usePromptDraftStore = defineStore("promptDraft", () => {
  const stored = loadStoredDraft();
  const systemPrompt = ref(stored.systemPrompt ?? "");
  const userPrompt = ref(stored.userPrompt ?? "");
  const temperature = ref<number>(0.7);
  const topP = ref<number>(1);
  const maxTokens = ref<number>(1024);

  watch([systemPrompt, userPrompt], ([systemPrompt, userPrompt]) => {
    saveStoredDraft({ systemPrompt, userPrompt });
  });

  return { systemPrompt, userPrompt, temperature, topP, maxTokens };
});
