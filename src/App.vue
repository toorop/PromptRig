<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
</script>

<template>
  <div class="flex h-screen flex-col bg-background text-foreground">
    <nav class="flex items-center gap-4 border-b px-4 py-2 text-sm">
      <span class="font-semibold">PromptRig</span>
      <RouterLink to="/" class="text-muted-foreground hover:text-foreground" active-class="text-foreground font-medium">
        Playground
      </RouterLink>
      <RouterLink to="/compare" class="text-muted-foreground hover:text-foreground" active-class="text-foreground font-medium">
        Compare
      </RouterLink>
      <RouterLink to="/settings" class="ml-auto text-muted-foreground hover:text-foreground" active-class="text-foreground font-medium">
        Settings
      </RouterLink>
    </nav>
    <main class="flex-1 overflow-auto">
      <RouterView />
    </main>
  </div>
</template>
