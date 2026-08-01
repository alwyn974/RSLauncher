<script setup lang="ts">
import { computed } from "vue";
import { launcher } from "./stores/launcher";
import LoginView from "./views/LoginView.vue";
import PlayView from "./views/PlayView.vue";
import SettingsView from "./views/SettingsView.vue";
import LogsView from "./views/LogsView.vue";

const signedOut = computed(() => !launcher.activeAccount.value);
</script>

<template>
  <div class="h-full overflow-hidden">
    <LoginView v-if="signedOut" />
    <PlayView v-else-if="launcher.state.view === 'play'" />
    <SettingsView v-else-if="launcher.state.view === 'settings'" />
    <LogsView v-else />
  </div>
</template>
