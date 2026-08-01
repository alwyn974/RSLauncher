<script setup lang="ts">
/**
 * Launch progress panel: pipeline steps, current detail, percent bar.
 */
import { computed } from "vue";
import { LAUNCH_STEPS, launcher, type LaunchStage } from "../stores/launcher";

const p = computed(() => launcher.state.progress);
const installed = computed(() => launcher.state.installed);

const active = computed(
  () =>
    p.value.stage !== "idle" &&
    p.value.stage !== "error" &&
    p.value.stage !== "running",
);

const speedText = computed(() =>
  p.value.bytesPerSec > 0 ? `${(p.value.bytesPerSec / 1024 / 1024).toFixed(1)} MB/s` : "",
);

const etaText = computed(() => {
  if (p.value.stage !== "downloading") return "";
  if (p.value.etaSec <= 0) return "almost done";
  return p.value.etaSec === 1 ? "1s left" : `${p.value.etaSec}s left`;
});

const headline = computed(() => {
  switch (p.value.stage) {
    case "idle":
      return installed.value ? "Ready to play" : "Not installed";
    case "running":
      return "Game running";
    case "error":
      return cancelled.value ? "Cancelled" : p.value.step || "Launch failed";
    default:
      return p.value.step || "Working…";
  }
});

const subtitle = computed(() => {
  if (p.value.stage === "error" && p.value.file) return p.value.file;
  if (p.value.file) return p.value.file;
  if (p.value.stage === "idle") {
    return installed.value
      ? "Press PLAY to launch the game."
      : "Press PLAY to download and install the modpack.";
  }
  if (p.value.stage === "running") return p.value.file || "";
  return "";
});

const showBar = computed(
  () => active.value || p.value.stage === "running",
);

const showPipeline = computed(
  () => active.value || p.value.stage === "running" || p.value.stage === "error",
);

function stepIndex(stage: LaunchStage): number {
  const i = LAUNCH_STEPS.findIndex((s) => s.id === stage);
  return i >= 0 ? i : -1;
}

const currentIndex = computed(() => {
  if (p.value.stage === "error") {
    return stepIndex(launcher.state.progressPhase);
  }
  return stepIndex(p.value.stage as LaunchStage);
});

const cancelled = computed(
  () => p.value.stage === "error" && /cancel/i.test(p.value.step + p.value.file),
);

type StepState = "done" | "current" | "pending" | "error";

function stepState(index: number): StepState {
  const cur = currentIndex.value;
  if (p.value.stage === "error") {
    if (cur < 0) return index === 0 ? "error" : "pending";
    if (index < cur) return "done";
    if (index === cur) return "error";
    return "pending";
  }
  if (cur < 0) return "pending";
  if (index < cur) return "done";
  if (index === cur) return "current";
  return "pending";
}
</script>

<template>
  <section class="mc-panel p-3" aria-live="polite" aria-label="Launch progress">
    <div class="flex items-baseline justify-between gap-4">
      <div class="min-w-0 flex-1">
        <p
          class="font-pixel pixel-shadow-sm text-sm"
          :class="p.stage === 'error' ? 'text-mc-red' : 'text-mc-text'"
        >
          {{ headline }}
        </p>
        <p
          v-if="subtitle"
          class="mt-1 text-sm leading-relaxed break-all whitespace-pre-wrap"
          :class="p.stage === 'error' ? 'text-mc-red' : 'text-mc-muted'"
          :title="subtitle"
        >
          {{ subtitle }}
        </p>
      </div>
      <div class="shrink-0 text-right">
        <p
          v-if="showBar && p.stage !== 'running'"
          class="font-pixel pixel-shadow-sm text-lg text-mc-gold"
        >
          {{ p.percent }}%
        </p>
        <p
          v-if="p.stage === 'downloading' && (speedText || etaText)"
          class="mt-1 font-mono text-xs text-mc-muted"
        >
          <template v-if="speedText">{{ speedText }}</template>
          <template v-if="speedText && etaText"> · </template>
          <template v-if="etaText">{{ etaText }}</template>
        </p>
        <p
          v-if="p.filesTotal > 0"
          class="font-mono text-xs text-mc-muted"
        >
          {{ p.filesDone }}/{{ p.filesTotal }} files
        </p>
      </div>
    </div>

    <ol
      v-if="showPipeline"
      class="mt-3 flex flex-wrap items-center gap-x-1 gap-y-1.5"
      aria-label="Launch steps"
    >
      <li
        v-for="(step, index) in LAUNCH_STEPS"
        :key="step.id"
        class="flex items-center gap-1"
      >
        <span
          class="inline-flex items-center gap-1 font-pixel pixel-shadow-sm text-[10px]"
          :class="{
            'text-mc-green-hi': stepState(index) === 'done',
            'text-mc-gold': stepState(index) === 'current',
            'text-mc-faint': stepState(index) === 'pending',
            'text-mc-red': stepState(index) === 'error',
          }"
        >
          <span
            class="inline-block size-2 border border-mc-border"
            :class="{
              'bg-mc-green': stepState(index) === 'done',
              'bg-mc-gold mc-step-pulse': stepState(index) === 'current',
              'bg-mc-inset': stepState(index) === 'pending',
              'bg-mc-red': stepState(index) === 'error',
            }"
            aria-hidden="true"
          />
          {{ step.label }}
        </span>
        <span
          v-if="index < LAUNCH_STEPS.length - 1"
          class="mx-0.5 text-[10px] text-mc-faint"
          aria-hidden="true"
        >›</span>
      </li>
    </ol>

    <div
      v-if="showBar"
      class="mc-inset-well mt-2 h-4 w-full overflow-hidden"
      role="progressbar"
      :aria-valuenow="p.percent"
      aria-valuemin="0"
      aria-valuemax="100"
      :aria-label="headline"
    >
      <div
        class="relative h-full bg-mc-green transition-[width] duration-150"
        :class="{ 'bg-mc-red': p.stage === 'error' }"
        :style="{ width: `${p.stage === 'error' ? 100 : p.percent}%` }"
      >
        <div
          v-if="p.stage !== 'error'"
          class="absolute inset-x-0 top-0 h-1 bg-mc-green-hi"
        />
      </div>
    </div>
  </section>
</template>
