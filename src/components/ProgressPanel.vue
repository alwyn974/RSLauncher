<script setup lang="ts">
/**
 * Launch progress panel: current step, file, percent, speed, ETA,
 * and a blocky green fill bar. Reads straight from the launcher store.
 */
import { computed } from "vue";
import { launcher } from "../stores/launcher";

const p = computed(() => launcher.state.progress);

const speedText = computed(() =>
  p.value.bytesPerSec > 0 ? `${(p.value.bytesPerSec / 1024 / 1024).toFixed(1)} MB/s` : "",
);

const etaText = computed(() => {
  if (p.value.stage === "idle" || p.value.stage === "running") return "";
  if (p.value.etaSec <= 0) return "almost done";
  return p.value.etaSec === 1 ? "1s left" : `${p.value.etaSec}s left`;
});

const headline = computed(() => {
  switch (p.value.stage) {
    case "idle":
      return "Ready to play";
    case "running":
      return "Game running";
    case "error":
      return "Something went wrong";
    default:
      return p.value.step;
  }
});

const showBar = computed(
  () => p.value.stage === "downloading" || p.value.stage === "verifying",
);
</script>

<template>
  <section class="mc-panel p-3" aria-live="polite" aria-label="Launch progress">
    <div class="flex items-baseline justify-between gap-4">
      <div class="min-w-0">
        <p class="font-pixel pixel-shadow-sm text-xs text-mc-text">{{ headline }}</p>
        <p
          v-if="p.file"
          class="mt-1 truncate font-mono text-xs text-mc-muted"
          :title="p.file"
        >
          {{ p.file }}
        </p>
        <p v-else-if="p.stage === 'idle'" class="mt-1 text-xs text-mc-muted">
          Everything is up to date.
        </p>
      </div>
      <div class="shrink-0 text-right">
        <p
          v-if="showBar"
          class="font-pixel pixel-shadow-sm text-lg text-mc-gold"
        >
          {{ p.percent }}%
        </p>
        <p v-if="p.stage === 'downloading'" class="mt-1 font-mono text-xs text-mc-muted">
          {{ speedText }} · {{ etaText }}
        </p>
        <p v-if="showBar" class="font-mono text-xs text-mc-muted">
          {{ p.filesDone }}/{{ p.filesTotal }} files
        </p>
      </div>
    </div>

    <div
      v-if="showBar"
      class="mc-inset-well mt-2 h-4 w-full overflow-hidden"
      role="progressbar"
      :aria-valuenow="p.percent"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <div
        class="relative h-full bg-mc-green transition-[width] duration-150"
        :style="{ width: `${p.percent}%` }"
      >
        <div class="absolute inset-x-0 top-0 h-1 bg-mc-green-hi" />
      </div>
    </div>
  </section>
</template>
