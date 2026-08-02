<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { initLauncher, launcher } from "./stores/launcher";
import { enforceLauncherUpdate, updateState } from "./updater";
import LoginView from "./views/LoginView.vue";
import PlayView from "./views/PlayView.vue";
import SettingsView from "./views/SettingsView.vue";
import LogsView from "./views/LogsView.vue";
import UpdateGate from "./components/UpdateGate.vue";

const bootstrapped = ref(false);
const signedOut = computed(() => !launcher.activeAccount.value);
// Only after an update is actually found - never during the silent check.
const showUpdateGate = computed(() => updateState.blocking);

onMounted(() => {
  void (async () => {
    await enforceLauncherUpdate();
    if (!updateState.blocking) {
      await initLauncher();
      bootstrapped.value = true;
    }
  })();
});
</script>

<template>
  <div class="h-full overflow-hidden">
    <UpdateGate v-if="showUpdateGate" />
    <template v-else-if="bootstrapped">
      <LoginView v-if="signedOut" />
      <PlayView v-else-if="launcher.state.view === 'play'" />
      <SettingsView v-else-if="launcher.state.view === 'settings'" />
      <LogsView v-else />
    </template>
  </div>
</template>
