<script setup lang="ts">
import { ref } from "vue";
import { commands, type ProviderStatus } from "@/lib/bindings";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";

const props = defineProps<{ status: ProviderStatus }>();
const emit = defineEmits<{ changed: [] }>();

const apiKeyInput = ref("");
const saving = ref(false);
const testing = ref(false);
const testResult = ref<{ ok: boolean; message: string } | null>(null);
const actionError = ref<string | null>(null);

async function save() {
  const key = apiKeyInput.value.trim();
  if (!key) return;

  saving.value = true;
  actionError.value = null;

  const result = await commands.saveApiKey(props.status.provider, key);
  if (result.status === "ok") {
    apiKeyInput.value = "";
    testResult.value = null;
    emit("changed");
  } else {
    actionError.value = result.error.message;
  }

  saving.value = false;
}

async function removeKey() {
  saving.value = true;
  actionError.value = null;

  const result = await commands.deleteApiKey(props.status.provider);
  if (result.status === "ok") {
    testResult.value = null;
    emit("changed");
  } else {
    actionError.value = result.error.message;
  }

  saving.value = false;
}

async function testConnection() {
  testing.value = true;
  testResult.value = null;

  const result = await commands.testProviderConnection(props.status.provider);
  testResult.value =
    result.status === "ok"
      ? { ok: true, message: "Connection OK" }
      : { ok: false, message: result.error.message };

  testing.value = false;
}
</script>

<template>
  <Card>
    <CardHeader class="flex flex-row items-center justify-between space-y-0">
      <div>
        <CardTitle>{{ status.display_name }}</CardTitle>
        <CardDescription v-if="!status.implemented">Not implemented yet</CardDescription>
      </div>
      <Badge :variant="status.configured ? 'default' : 'secondary'">
        {{ status.configured ? "Configured" : "Not configured" }}
      </Badge>
    </CardHeader>

    <CardContent v-if="status.implemented" class="flex flex-col gap-3">
      <div class="flex items-end gap-2">
        <div class="flex flex-1 flex-col gap-1">
          <Label :for="`${status.provider}-key`">API key</Label>
          <Input
            :id="`${status.provider}-key`"
            v-model="apiKeyInput"
            type="password"
            autocomplete="off"
            placeholder="sk-…"
          />
        </div>
        <Button :disabled="saving || !apiKeyInput.trim()" @click="save">Save</Button>
        <Button v-if="status.configured" variant="outline" :disabled="saving" @click="removeKey">
          Remove
        </Button>
      </div>

      <div class="flex items-center gap-2">
        <Button
          variant="secondary"
          size="sm"
          :disabled="!status.configured || testing"
          @click="testConnection"
        >
          {{ testing ? "Testing…" : "Test connection" }}
        </Button>
        <span
          v-if="testResult"
          :class="testResult.ok ? 'text-sm text-green-600' : 'text-sm text-destructive'"
        >
          {{ testResult.message }}
        </span>
      </div>

      <p v-if="actionError" class="text-sm text-destructive">{{ actionError }}</p>
    </CardContent>
  </Card>
</template>
