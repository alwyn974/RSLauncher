<script setup lang="ts">
/**
 * Game log console: level-colored mono lines, auto-scroll, copy, clear.
 */
import { nextTick, ref, watch } from "vue";
import { launcher, type LogLine } from "../stores/launcher";
import PixelButton from "./PixelButton.vue";
import PixelIcon from "./PixelIcon.vue";

const autoScroll = ref(true);
const copied = ref(false);
const scroller = ref<HTMLElement | null>(null);

const LEVEL_CLASS: Record<LogLine["level"], string> = {
  INFO: "text-mc-text",
  WARN: "text-mc-gold",
  ERROR: "text-mc-red",
};

watch(
  () => launcher.state.logs.length,
  async () => {
    if (!autoScroll.value) return;
    await scrollToEnd();
  },
);

async function scrollToEnd() {
  await nextTick();
  const el = scroller.value;
  if (el) el.scrollTop = el.scrollHeight;
}

function goToEnd() {
  autoScroll.value = true;
  void scrollToEnd();
}

async function copy() {
  const text = launcher.state.logs
    .map((l) => `[${l.time}] [${l.source}/${l.level}]: ${l.message}`)
    .join("\n");
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch {
    /* clipboard unavailable */
  }
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col gap-2">
    <div class="flex items-center gap-2">
      <PixelButton
        :variant="autoScroll ? 'gold' : 'stone'"
        class="px-2 py-1 text-[10px]"
        :aria-pressed="autoScroll"
        @click="autoScroll = !autoScroll"
      >
        Auto-scroll {{ autoScroll ? "on" : "off" }}
      </PixelButton>
      <PixelButton class="px-2 py-1 text-[10px]" aria-label="Scroll to end" @click="goToEnd">
        End
      </PixelButton>
      <PixelButton class="px-2 py-1 text-[10px]" @click="copy">
        <PixelIcon name="copy" :size="12" />
        {{ copied ? "Copied" : "Copy" }}
      </PixelButton>
      <PixelButton variant="red" class="px-2 py-1 text-[10px]" @click="launcher.clearLogs()">
        <PixelIcon name="trash" :size="12" />
        Clear
      </PixelButton>
    </div>

    <div
      ref="scroller"
      class="mc-inset-well min-h-0 flex-1 overflow-y-auto p-2 font-mono text-xs leading-5 select-text"
      aria-label="Game logs"
    >
      <p v-if="launcher.state.logs.length === 0" class="text-mc-faint">
        No logs yet. Launch the game to see output here.
      </p>
      <p
        v-for="(line, i) in launcher.state.logs"
        :key="i"
        class="break-all whitespace-pre-wrap"
      >
        <span class="text-mc-faint">[{{ line.time }}]</span>
        <span :class="LEVEL_CLASS[line.level]">
          [{{ line.source }}/{{ line.level }}]:
        </span>
        <span class="text-mc-text"> {{ line.message }}</span>
      </p>
    </div>
  </div>
</template>
