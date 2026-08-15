<script setup lang="ts">
/**
 * Live sign-in feedback: pulsing status, device code, and auth log stream.
 */
import { computed, nextTick, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { QrcodeSvg } from "qrcode.vue";
import { launcher, type LogLine } from "../stores/launcher";
import PixelButton from "./PixelButton.vue";

const scroller = ref<HTMLElement | null>(null);
const copyState = ref<"idle" | "copied" | "failed">("idle");

const copyFeedback = computed(() => {
  if (copyState.value === "copied") return "Code copied — paste it on the Microsoft page.";
  if (copyState.value === "failed") {
    return "Could not copy automatically. Select the code above and copy it manually.";
  }
  return "";
});

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

watch(
  () => launcher.state.deviceCode?.code,
  () => {
    copyState.value = "idle";
  },
);

async function writeClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Older WebViews can deny the async Clipboard API while still allowing a
    // user-triggered copy command.
    const input = document.createElement("textarea");
    input.value = text;
    input.setAttribute("readonly", "");
    input.style.position = "fixed";
    input.style.opacity = "0";
    document.body.appendChild(input);
    input.select();
    const copied = document.execCommand("copy");
    input.remove();
    return copied;
  }
}

async function openDeviceUrl() {
  const deviceCode = launcher.state.deviceCode;
  if (!deviceCode) return;

  copyState.value = (await writeClipboard(deviceCode.code)) ? "copied" : "failed";
  try {
    await openUrl(deviceCode.url);
  } catch {
    window.open(deviceCode.url, "_blank");
  }
}

async function copyCode() {
  const code = launcher.state.deviceCode?.code;
  if (!code) return;
  copyState.value = (await writeClipboard(code)) ? "copied" : "failed";
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
      class="mc-inset-well mt-3 grid grid-cols-[132px_minmax(0,1fr)] gap-3 p-3"
    >
      <div
        class="mc-bevel flex size-[132px] items-center justify-center bg-white"
        role="img"
        aria-label="QR code to open the Microsoft sign-in page on a phone"
      >
        <QrcodeSvg
          :value="launcher.state.deviceCode.url"
          :size="124"
          level="M"
          :margin="4"
          aria-hidden="true"
        />
      </div>

      <div class="min-w-0 self-center text-left">
        <p class="font-pixel pixel-shadow-sm text-[10px] text-mc-muted">
          Enter this code
        </p>
        <p
          class="font-pixel pixel-shadow mt-2 break-all text-xl tracking-widest text-mc-gold select-all"
        >
          {{ launcher.state.deviceCode.code }}
        </p>
        <p class="mt-2 text-xs leading-relaxed text-mc-muted">
          Scan with your phone, or open Microsoft here. Opening the link copies the code.
        </p>
        <div class="mt-3 flex flex-wrap gap-2">
          <PixelButton class="px-3 py-1.5 text-xs" @click="openDeviceUrl">
            Open link
          </PixelButton>
          <PixelButton class="px-3 py-1.5 text-xs" @click="copyCode">
            {{ copyState === "copied" ? "Copied!" : "Copy code" }}
          </PixelButton>
        </div>
      </div>

      <p
        v-if="copyFeedback"
        class="col-span-2 text-xs leading-relaxed"
        :class="copyState === 'failed' ? 'text-mc-red' : 'text-mc-gold'"
        role="status"
        aria-live="polite"
      >
        {{ copyFeedback }}
      </p>
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
