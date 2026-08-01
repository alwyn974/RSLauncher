<script setup lang="ts">
/**
 * Main screen: account switcher + nav on top, modpack identity and the
 * big PLAY button in the middle, progress panel docked at the bottom.
 */
import { computed } from "vue";
import { launcher, MODPACK } from "../stores/launcher";
import AccountSwitcher from "../components/AccountSwitcher.vue";
import PixelButton from "../components/PixelButton.vue";
import PixelIcon from "../components/PixelIcon.vue";
import ProgressPanel from "../components/ProgressPanel.vue";

const stage = computed(() => launcher.state.progress.stage);

const playLabel = computed(() => {
  if (stage.value === "running") return "STOP";
  if (launcher.busy.value) return launcher.state.progress.step.toUpperCase();
  return "PLAY";
});

function onPlay() {
  if (stage.value === "running") launcher.stop();
  else launcher.play();
}
</script>

<template>
  <div class="flex h-full flex-col">
    <header class="flex items-center justify-between p-3">
      <AccountSwitcher />
      <nav class="flex items-center gap-2" aria-label="Launcher">
        <PixelButton
          class="px-2 py-1.5"
          aria-label="Game logs"
          title="Game logs"
          @click="launcher.setView('logs')"
        >
          <PixelIcon name="terminal" :size="16" />
        </PixelButton>
        <PixelButton
          class="px-2 py-1.5"
          aria-label="Settings"
          title="Settings"
          @click="launcher.setView('settings')"
        >
          <PixelIcon name="gear" :size="16" />
        </PixelButton>
      </nav>
    </header>

    <main class="flex min-h-0 flex-1 flex-col items-center justify-center gap-6">
      <div class="text-center">
        <h1 class="font-pixel pixel-shadow text-3xl tracking-wide text-white">
          {{ MODPACK.name }}
        </h1>
        <p class="mt-3 inline-block border-2 border-mc-border bg-mc-panel px-2 py-1 font-mono text-xs text-mc-gold">
          Minecraft {{ MODPACK.minecraft }} · {{ MODPACK.loader }} · {{ MODPACK.modCount }} mods
        </p>
      </div>

      <div class="flex items-center gap-3">
        <PixelButton
          :variant="stage === 'running' ? 'red' : 'green'"
          class="px-14 py-4 text-xl"
          :disabled="launcher.busy.value"
          @click="onPlay"
        >
          {{ playLabel }}
        </PixelButton>
        <PixelButton
          v-if="launcher.busy.value"
          variant="red"
          class="px-3 py-4"
          aria-label="Cancel launch"
          title="Cancel launch"
          @click="launcher.cancel()"
        >
          <PixelIcon name="x" :size="16" />
        </PixelButton>
      </div>
    </main>

    <footer class="p-3">
      <ProgressPanel />
    </footer>
  </div>
</template>
