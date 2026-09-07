import { defineStore } from "pinia";
import { ref } from "vue";

// The system prompt, user prompt, and base generation params are shared between Playground and
// Compare — editing either in one view carries over to the other, with no explicit "send to
// Compare" action and no requirement to visit one view before the other. Compare adds
// per-column Provider+Model selection on top of this; it doesn't duplicate the prompt/params
// editing UI concept.
export const usePromptDraftStore = defineStore("promptDraft", () => {
  const systemPrompt = ref("");
  const userPrompt = ref("");
  const temperature = ref<number>(0.7);
  const topP = ref<number>(1);
  const maxTokens = ref<number>(1024);

  return { systemPrompt, userPrompt, temperature, topP, maxTokens };
});
