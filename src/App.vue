<script setup lang="ts">
import { computed, onMounted } from "vue";
import { initLauncher, launcher } from "./stores/launcher";
import LoginView from "./views/LoginView.vue";
import PlayView from "./views/PlayView.vue";
import SettingsView from "./views/SettingsView.vue";
import LogsView from "./views/LogsView.vue";

const signedOut = computed(() => !launcher.activeAccount.value);

onMounted(() => {
  void initLauncher();
});
</script>

<template>
  <div class="h-full overflow-hidden">
    <LoginView v-if="signedOut" />
    <PlayView v-else-if="launcher.state.view === 'play'" />
    <SettingsView v-else-if="launcher.state.view === 'settings'" />
    <LogsView v-else />
  </div>
</template>
