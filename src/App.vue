<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterLink, RouterView } from "vue-router";
import { CircleHelp, Moon, Sun, X } from "@lucide/vue";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useThemeStore } from "@/stores/theme";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

const themeStore = useThemeStore();
const appVersion = ref<string>();

onMounted(async () => {
  appVersion.value = await getVersion();
});

function closeWindow() {
  getCurrentWindow().close();
}
</script>

<template>
  <TooltipProvider>
    <div class="flex h-screen flex-col bg-background text-foreground">
      <!-- The OS window is decoration-less (see tauri.conf.json) for a cleaner look, so this bar
           is the only way to move or close the window: `data-tauri-drag-region` makes its empty
           space act as a native title bar for dragging, and the "×" button below is the only
           remaining window control (no minimize/maximize, per the user's request). -->
      <nav data-tauri-drag-region class="flex items-center gap-1 border-b bg-card/60 px-4 py-2.5">
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
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon" class="ml-auto text-muted-foreground hover:text-foreground">
              <CircleHelp class="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>PromptRig{{ appVersion ? ` v${appVersion}` : "" }}</TooltipContent>
        </Tooltip>
        <Button
          variant="ghost"
          size="icon"
          class="text-muted-foreground hover:text-foreground"
          :title="themeStore.theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'"
          @click="themeStore.toggle"
        >
          <Sun v-if="themeStore.theme === 'dark'" class="size-4" />
          <Moon v-else class="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="text-muted-foreground hover:bg-destructive/20 hover:text-destructive"
          title="Close PromptRig"
          @click="closeWindow"
        >
          <X class="size-4" />
        </Button>
      </nav>
      <main class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden">
        <RouterView />
      </main>
    </div>
  </TooltipProvider>
</template>
