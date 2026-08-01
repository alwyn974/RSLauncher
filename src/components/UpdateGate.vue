<script setup lang="ts">
/**
 * Full-window gate for mandatory launcher updates — no dismiss.
 */
import { computed } from "vue";
import {
  retryForcedUpdate,
  updateFromToLabel,
  updatePercent,
  updateState,
  updateVersionLabel,
} from "../updater";
import PixelButton from "./PixelButton.vue";

const title = computed(() => {
  switch (updateState.phase) {
    case "available":
    case "downloading":
      return `Updating to ${updateVersionLabel()}`;
    case "installing":
      return `Installing ${updateVersionLabel()}…`;
    case "relaunching":
      return `Restarting ${updateVersionLabel()}…`;
    case "error":
      return `Update ${updateVersionLabel()} failed`;
    default:
      return `Updating to ${updateVersionLabel()}`;
  }
});

const detail = computed(() => {
  if (updateState.phase === "error") return updateState.error ?? "Unknown error";
  if (updateState.phase === "downloading") {
    const pct = updatePercent();
    if (updateState.contentLength > 0) {
      const mb = (n: number) => (n / (1024 * 1024)).toFixed(1);
      return `${pct}% · ${mb(updateState.downloaded)} / ${mb(updateState.contentLength)} MB`;
    }
    return "Downloading…";
  }
  if (updateState.phase === "installing" || updateState.phase === "relaunching") {
    return "Almost done — the launcher will restart.";
  }
  if (updateState.notes) return updateState.notes;
  return "Downloading the new launcher…";
});

const showBar = computed(
  () =>
    updateState.phase === "downloading" ||
    updateState.phase === "installing" ||
    updateState.phase === "relaunching",
);
</script>

<template>
  <div
    class="fixed inset-0 z-[100] flex items-center justify-center bg-mc-bg/95 p-6"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="update-gate-title"
  >
    <div class="mc-panel w-full max-w-md p-5">
      <p class="font-mono text-xs tracking-wide text-mc-gold">
        {{ updateFromToLabel() }}
      </p>
      <h2
        id="update-gate-title"
        class="font-pixel pixel-shadow-sm mt-2 text-lg tracking-wide text-white"
      >
        {{ title }}
      </h2>
      <p class="mt-3 max-h-32 overflow-y-auto font-mono text-sm leading-relaxed whitespace-pre-wrap text-mc-text">
        {{ detail }}
      </p>

      <div
        v-if="showBar"
        class="mc-bevel mt-4 h-4 overflow-hidden bg-mc-inset"
        role="progressbar"
        :aria-valuenow="updatePercent()"
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <div
          class="h-full bg-mc-green transition-[width] duration-150"
          :style="{
            width:
              updateState.phase === 'downloading'
                ? `${updatePercent()}%`
                : '100%',
          }"
        />
      </div>

      <p
        v-if="updateState.phase !== 'error'"
        class="mt-4 font-mono text-xs text-mc-muted"
      >
        This update is required. The launcher will restart when done.
      </p>

      <PixelButton
        v-if="updateState.phase === 'error'"
        variant="gold"
        class="mt-4 w-full py-3"
        @click="retryForcedUpdate()"
      >
        RETRY UPDATE
      </PixelButton>
    </div>
  </div>
</template>
