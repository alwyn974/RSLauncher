<script setup lang="ts">
/**
 * Settings: RAM, resolution, server address, custom JVM arguments.
 */
import { computed, reactive } from "vue";
import { launcher, type Settings } from "../stores/launcher";
import PixelButton from "../components/PixelButton.vue";
import PixelIcon from "../components/PixelIcon.vue";

const draft = reactive<Settings>({ ...launcher.state.settings });

const ramMin = computed(() => launcher.state.memory.minGb);
const ramMax = computed(() => Math.max(launcher.state.memory.totalGb, ramMin.value));
const ramRecommended = computed(() =>
  Math.min(
    Math.max(launcher.state.memory.recommendedGb, ramMin.value),
    ramMax.value,
  ),
);

function clampDraft() {
  draft.ramGb = Math.min(
    ramMax.value,
    Math.max(ramMin.value, Math.round(Number(draft.ramGb) || ramRecommended.value)),
  );
  draft.width = Math.min(7680, Math.max(640, Number(draft.width) || 1024));
  draft.height = Math.min(4320, Math.max(480, Number(draft.height) || 768));
}

function save() {
  clampDraft();
  launcher.saveSettings({ ...draft });
  launcher.setView("play");
}

function reset() {
  Object.assign(draft, launcher.resetSettings());
}
</script>

<template>
  <div class="flex h-full flex-col p-3">
    <header class="flex items-center gap-3">
      <PixelButton class="px-2 py-1.5" aria-label="Back" @click="launcher.setView('play')">
        <PixelIcon name="back" :size="16" />
      </PixelButton>
      <h1 class="font-pixel pixel-shadow text-sm text-white">Settings</h1>
    </header>

    <main class="mx-auto mt-4 flex w-full max-w-lg min-h-0 flex-1 flex-col gap-4 overflow-y-auto pb-2">
      <!-- Memory -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Memory</h2>
        <div class="mt-3 flex items-center gap-4">
          <input
            v-model.number="draft.ramGb"
            type="range"
            :min="ramMin"
            :max="ramMax"
            step="1"
            class="mc-range flex-1"
            aria-label="RAM allocation in gigabytes"
          />
          <span class="font-pixel pixel-shadow-sm w-20 text-right text-sm text-mc-text">
            {{ draft.ramGb }} GB
          </span>
        </div>
        <p class="mt-2 text-xs text-mc-muted">
          Pack recommends {{ ramRecommended }} GB (CurseForge).
          Max {{ ramMax }} GB (this PC).
        </p>
      </section>

      <!-- Resolution -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Resolution</h2>
        <div class="mt-3 flex items-center gap-3">
          <label class="flex flex-1 flex-col gap-1 text-xs text-mc-muted">
            Width
            <input
              v-model.number="draft.width"
              type="number"
              min="640"
              max="7680"
              step="10"
              class="mc-input w-full font-mono"
              :disabled="draft.fullscreen"
            />
          </label>
          <span class="pt-5 text-mc-faint">×</span>
          <label class="flex flex-1 flex-col gap-1 text-xs text-mc-muted">
            Height
            <input
              v-model.number="draft.height"
              type="number"
              min="480"
              max="4320"
              step="10"
              class="mc-input w-full font-mono"
              :disabled="draft.fullscreen"
            />
          </label>
        </div>
        <label class="mt-3 flex cursor-pointer items-center gap-2 text-sm text-mc-text">
          <button
            type="button"
            role="checkbox"
            :aria-checked="draft.fullscreen"
            class="mc-inset-well flex size-5 cursor-pointer items-center justify-center"
            @click="draft.fullscreen = !draft.fullscreen"
          >
            <span v-if="draft.fullscreen" class="size-2.5 bg-mc-gold" />
          </button>
          Fullscreen
        </label>
      </section>

      <!-- Server -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Server</h2>
        <label class="mt-3 flex flex-col gap-1 text-xs text-mc-muted">
          Display name
          <input
            v-model="draft.serverName"
            type="text"
            spellcheck="false"
            placeholder="RS Server"
            class="mc-input w-full font-mono placeholder:text-mc-faint"
          />
        </label>
        <label class="mt-3 flex flex-col gap-1 text-xs text-mc-muted">
          Address
          <input
            v-model="draft.serverAddress"
            type="text"
            spellcheck="false"
            placeholder="play.example.com"
            class="mc-input w-full font-mono placeholder:text-mc-faint"
          />
        </label>
        <p class="mt-2 text-xs text-mc-muted">
          Shown in Multiplayer. Quick Play connects straight to this address.
        </p>
      </section>

      <!-- Java -->
      <section class="mc-panel p-4">
        <h2 class="font-pixel pixel-shadow-sm text-xs text-mc-gold">Java</h2>
        <label class="mt-3 flex flex-col gap-1 text-xs text-mc-muted">
          Custom JVM arguments
          <textarea
            v-model="draft.jvmArgs"
            rows="3"
            spellcheck="false"
            placeholder="-XX:+UseG1GC -XX:+ParallelRefProcEnabled"
            class="mc-input w-full resize-none font-mono placeholder:text-mc-faint"
          />
        </label>
        <p class="mt-2 text-xs text-mc-muted">
          Appended to the launch command. -Xmx is set by the memory slider.
        </p>
      </section>
    </main>

    <footer class="mx-auto flex w-full max-w-lg justify-end gap-2 pt-2">
      <PixelButton @click="reset">Reset</PixelButton>
      <PixelButton variant="gold" @click="save">Save</PixelButton>
    </footer>
  </div>
</template>
