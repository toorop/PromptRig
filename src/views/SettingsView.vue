<script setup lang="ts">
import { onMounted } from "vue";
import { useProvidersStore } from "@/stores/providers";
import ProviderCard from "@/components/settings/ProviderCard.vue";

// Provider configuration: enable/disable a provider, set its API key (stored in the OS
// keyring, never in the frontend), test the connection.
const providersStore = useProvidersStore();

onMounted(() => {
  providersStore.refresh();
});
</script>

<template>
  <div class="mx-auto flex max-w-4xl flex-col gap-6 p-6">
    <div>
      <h1 class="text-xl font-semibold tracking-tight">Providers</h1>
      <p class="text-sm text-muted-foreground">
        Add an API key to enable a provider in the Playground and comparisons.
      </p>
    </div>
    <p v-if="providersStore.error" class="text-sm text-destructive">{{ providersStore.error }}</p>

    <div class="grid gap-4 md:grid-cols-2">
      <ProviderCard
        v-for="status in providersStore.providers"
        :key="status.provider"
        :status="status"
        @changed="providersStore.refresh"
      />
    </div>
  </div>
</template>
