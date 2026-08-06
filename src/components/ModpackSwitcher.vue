<script setup lang="ts">
/**
 * Active-modpack title with a dropdown to switch curated packs.
 */
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { launcher } from "../stores/launcher";
import PixelIcon from "./PixelIcon.vue";

const open = ref(false);
const root = ref<HTMLElement | null>(null);

const active = computed(
  () =>
    launcher.state.modpacks.find((p) => p.id === launcher.state.activePackId) ??
    null,
);

const canSwitch = computed(() => launcher.state.modpacks.length > 1);

function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener("mousedown", onDocClick));
onBeforeUnmount(() => document.removeEventListener("mousedown", onDocClick));

async function choose(id: string) {
  if (id === launcher.state.activePackId) {
    open.value = false;
    return;
  }
  await launcher.setActiveModpack(id);
  open.value = false;
}
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      class="flex cursor-pointer items-center gap-2 px-1 py-1 transition-colors duration-150 hover:bg-mc-panel/40 disabled:cursor-not-allowed disabled:opacity-60"
      :class="{ 'cursor-default hover:bg-transparent': !canSwitch }"
      :aria-expanded="open"
      aria-haspopup="listbox"
      :disabled="launcher.busy.value || !canSwitch"
      @click="canSwitch && (open = !open)"
    >
      <span class="font-pixel pixel-shadow max-w-80 truncate text-3xl tracking-wide text-white">
        {{ active?.name ?? launcher.state.catalog.modpack.name }}
      </span>
      <PixelIcon
        v-if="canSwitch"
        name="chevron"
        :size="14"
        class="text-mc-muted transition-transform duration-150"
        :class="{ 'rotate-180': open }"
      />
    </button>

    <div
      v-if="open && canSwitch"
      class="mc-panel absolute left-1/2 z-50 mt-1 w-80 min-w-0 -translate-x-1/2 overflow-hidden p-1"
      role="listbox"
      aria-label="Modpacks"
    >
      <div
        v-for="pack in launcher.state.modpacks"
        :key="pack.id"
        role="option"
        :aria-selected="pack.id === launcher.state.activePackId"
        class="flex cursor-pointer flex-col gap-0.5 p-2 transition-colors duration-150 hover:bg-mc-panel-2"
        :class="{ 'bg-mc-panel-2': pack.id === launcher.state.activePackId }"
        @click="choose(pack.id)"
      >
        <div class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate text-sm text-mc-text">
            {{ pack.name }}
          </span>
          <PixelIcon
            v-if="pack.id === launcher.state.activePackId"
            name="check"
            :size="14"
            class="text-mc-gold"
          />
        </div>
        <span class="truncate text-xs text-mc-muted">
          <template v-if="pack.version">{{ pack.version }} · </template>
          MC {{ pack.minecraft }} · {{ pack.loader }}
          <template v-if="pack.installed"> · installed</template>
        </span>
      </div>
    </div>
  </div>
</template>
