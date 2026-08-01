<script setup lang="ts">
/**
 * Live sign-in feedback: pulsing status, device code, and auth log stream.
 */
import { nextTick, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { launcher, type LogLine } from "../stores/launcher";
import PixelButton from "./PixelButton.vue";

const scroller = ref<HTMLElement | null>(null);

const LEVEL_CLASS: Record<LogLine["level"], string> = {
  INFO: "text-mc-text",
  WARN: "text-mc-gold",
  ERROR: "text-mc-red",
};

watch(
  () => launcher.loginLogs.value.length,
  async () => {
    await nextTick();
    const el = scroller.value;
    if (el) el.scrollTop = el.scrollHeight;
  },
);

async function openDeviceUrl() {
  const url = launcher.state.deviceCode?.url;
  if (!url) return;
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank");
  }
}

async function copyCode() {
  const code = launcher.state.deviceCode?.code;
  if (!code) return;
  try {
    await navigator.clipboard.writeText(code);
  } catch {
    /* clipboard unavailable */
  }
}
</script>

<template>
  <section
    class="mc-panel sign-in-status w-full min-w-0 max-w-md overflow-hidden p-4"
    aria-live="polite"
    :aria-busy="launcher.state.loginPending"
    aria-label="Sign-in progress"
  >
    <div class="flex items-start gap-3">
      <div
        v-if="launcher.state.loginPending"
        class="mc-spinner mt-0.5 shrink-0"
        aria-hidden="true"
      >
        <span /><span /><span />
      </div>
      <div class="min-w-0 flex-1">
        <p class="font-pixel pixel-shadow-sm text-sm text-mc-gold">
          {{ launcher.state.loginPending ? "Signing in" : "Sign-in" }}
        </p>
        <p class="mt-1 text-sm leading-relaxed text-mc-text">
          {{
            launcher.loginStatus.value ||
            (launcher.state.loginPending
              ? "Starting Microsoft authentication…"
              : "Finished")
          }}
        </p>
      </div>
    </div>

    <div
      v-if="launcher.state.deviceCode"
      class="mc-inset-well mt-3 p-3 text-center"
    >
      <p class="font-pixel pixel-shadow-sm mb-2 text-[10px] text-mc-muted">
        Enter this code
      </p>
      <p class="font-pixel pixel-shadow text-2xl tracking-widest text-mc-gold select-all">
        {{ launcher.state.deviceCode.code }}
      </p>
      <p class="mt-2 text-xs text-mc-muted">
        Open the Microsoft link, paste the code, then come back here.
      </p>
      <div class="mt-3 flex justify-center gap-2">
        <PixelButton class="px-3 py-1.5 text-xs" @click="openDeviceUrl">
          Open link
        </PixelButton>
        <PixelButton class="px-3 py-1.5 text-xs" @click="copyCode">
          Copy code
        </PixelButton>
      </div>
    </div>

    <div
      v-if="launcher.loginLogs.value.length > 0"
      ref="scroller"
      class="mc-inset-well mt-3 max-h-36 overflow-y-auto p-2 font-mono text-[11px] leading-5 select-text"
      aria-label="Sign-in activity"
    >
      <p
        v-for="(line, i) in launcher.loginLogs.value"
        :key="i"
        class="break-all whitespace-pre-wrap"
      >
        <span class="text-mc-faint">[{{ line.time }}]</span>
        <span :class="LEVEL_CLASS[line.level]"> [{{ line.source }}]</span>
        <span class="text-mc-text"> {{ line.message }}</span>
      </p>
    </div>
    <p
      v-else-if="launcher.state.loginPending"
      class="mt-3 font-mono text-[11px] text-mc-faint"
    >
      Waiting for auth activity…
    </p>
  </section>
</template>
