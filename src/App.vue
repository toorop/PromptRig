<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
import { Moon, Sun } from "@lucide/vue";
import { useThemeStore } from "@/stores/theme";
import { Button } from "@/components/ui/button";

const themeStore = useThemeStore();
</script>

<template>
  <div class="flex h-screen flex-col bg-background text-foreground">
    <nav class="flex items-center gap-1 border-b bg-card/60 px-4 py-2.5">
      <span class="mr-3 flex items-center gap-1.5 text-sm font-semibold tracking-tight">
        <span class="inline-block size-2 rounded-full bg-primary" aria-hidden="true" />
        PromptRig
      </span>
      <RouterLink
        to="/"
        class="rounded-md px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        active-class="bg-accent font-medium text-foreground"
      >
        Playground
      </RouterLink>
      <RouterLink
        to="/compare"
        class="rounded-md px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        active-class="bg-accent font-medium text-foreground"
      >
        Compare
      </RouterLink>
      <RouterLink
        to="/settings"
        class="rounded-md px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        active-class="bg-accent font-medium text-foreground"
      >
        Settings
      </RouterLink>
      <Button
        variant="ghost"
        size="icon"
        class="ml-auto text-muted-foreground hover:text-foreground"
        :title="themeStore.theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'"
        @click="themeStore.toggle"
      >
        <Sun v-if="themeStore.theme === 'dark'" class="size-4" />
        <Moon v-else class="size-4" />
      </Button>
    </nav>
    <main class="flex-1 overflow-auto">
      <RouterView />
    </main>
  </div>
</template>
