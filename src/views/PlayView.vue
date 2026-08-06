<script setup lang="ts">
/**
 * Main screen: account switcher + nav on top, modpack identity and the
 * PLAY / QUICK PLAY buttons in the middle, progress panel docked at the bottom.
 */
import { computed } from "vue";
import { launcher } from "../stores/launcher";
import AccountSwitcher from "../components/AccountSwitcher.vue";
import ModpackSwitcher from "../components/ModpackSwitcher.vue";
import PixelButton from "../components/PixelButton.vue";
import PixelIcon from "../components/PixelIcon.vue";
import ProgressPanel from "../components/ProgressPanel.vue";
import SignInStatus from "../components/SignInStatus.vue";

const modpack = computed(() => launcher.state.catalog.modpack);
const stage = computed(() => launcher.state.progress.stage);
const running = computed(() => stage.value === "running");
const disabled = computed(() => launcher.busy.value || running.value);

const statusLabel = computed(() => {
  if (running.value) return "STOP";
  if (launcher.busy.value) return launcher.state.progress.step.toUpperCase();
  return null;
});

function onStopOrBusy() {
  if (running.value) launcher.stop();
}

function onPlay() {
  if (running.value) {
    launcher.stop();
    return;
  }
  launcher.play(false);
}

function onQuickPlay() {
  if (running.value || launcher.busy.value) return;
  launcher.play(true);
}
</script>

<template>
  <div class="relative flex h-full flex-col">
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
      <div class="flex flex-col items-center gap-3 text-center">
        <h1 class="sr-only">{{ modpack.name }}</h1>
        <ModpackSwitcher />
        <p class="inline-block border-2 border-mc-border bg-mc-panel px-2 py-1 font-mono text-xs text-mc-gold">
          <template v-if="modpack.version">{{ modpack.version }} · </template>
          Minecraft {{ modpack.minecraft }} · {{ modpack.loader }} {{ modpack.loaderVersion }}
          <template v-if="modpack.modCount > 0"> · {{ modpack.modCount }} mods</template>
        </p>
      </div>

      <div class="flex flex-col items-center gap-3">
        <div v-if="statusLabel" class="flex items-center gap-3">
          <PixelButton
            :variant="running ? 'red' : 'green'"
            class="px-14 py-4 text-xl"
            :disabled="launcher.busy.value && !running"
            @click="onStopOrBusy"
          >
            {{ statusLabel }}
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

        <div v-else class="flex items-center gap-3">
          <PixelButton
            variant="green"
            class="px-10 py-4 text-xl"
            :disabled="disabled"
            @click="onPlay"
          >
            PLAY
          </PixelButton>
          <PixelButton
            variant="gold"
            class="px-6 py-4 text-xl"
            :disabled="disabled || !launcher.state.settings.serverAddress.trim()"
            :title="`Connect to ${launcher.state.settings.serverAddress || '…'}`"
            @click="onQuickPlay"
          >
            QUICK PLAY
          </PixelButton>
        </div>

        <p
          v-if="running"
          class="font-mono text-xs text-mc-muted"
        >
          STOP closes the game - closing the launcher leaves it running
        </p>
        <p
          v-else-if="!statusLabel && launcher.state.settings.serverAddress"
          class="font-mono text-xs text-mc-muted"
        >
          Quick Play → {{ launcher.state.settings.serverAddress }}
        </p>
      </div>
    </main>

    <div
      v-if="launcher.state.loginPending"
      class="pointer-events-none absolute inset-x-0 bottom-20 z-40 flex justify-center px-4"
    >
      <div class="pointer-events-auto w-full max-w-md">
        <SignInStatus />
      </div>
    </div>

    <footer class="p-3">
      <ProgressPanel />
    </footer>
  </div>
</template>
