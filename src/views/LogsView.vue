<script setup lang="ts">
/**
 * Game log viewer.
 */
import { launcher } from "../stores/launcher";
import LogConsole from "../components/LogConsole.vue";
import PixelButton from "../components/PixelButton.vue";
import PixelIcon from "../components/PixelIcon.vue";
</script>

<template>
  <div class="flex h-full flex-col gap-3 p-3">
    <header class="flex items-center gap-3">
      <PixelButton class="px-2 py-1.5" aria-label="Back" @click="launcher.setView('play')">
        <PixelIcon name="back" :size="16" />
      </PixelButton>
      <h1 class="font-pixel pixel-shadow text-sm text-white">Game logs</h1>
    </header>
    <LogConsole />
  </div>
</template>
