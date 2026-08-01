<script setup lang="ts">
/**
 * Minecraft player head from the account UUID (mc-heads), with a small
 * procedural face as fallback when the skin can't be fetched.
 */
import { computed, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    uuid?: string | null;
    seed?: number;
    size?: number;
    username?: string | null;
  }>(),
  {
    uuid: null,
    seed: 0,
    size: 32,
    username: null,
  },
);

const failed = ref(false);

watch(
  () => [props.uuid, props.username],
  () => {
    failed.value = false;
  },
);

const src = computed(() => {
  const id = props.uuid?.replace(/-/g, "").trim();
  if (id) {
    // Helm = head + hat overlay, pixel-perfect at requested size.
    return `https://mc-heads.net/avatar/${id}/${props.size}`;
  }
  const name = props.username?.trim();
  if (name) {
    return `https://mc-heads.net/avatar/${encodeURIComponent(name)}/${props.size}`;
  }
  return null;
});

function mulberry32(seed: number) {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const SKINS = ["#c89a6b", "#a87b4f", "#8a5f3a", "#e0b58a", "#6e4a2c"];
const HAIRS = ["#3b2a1a", "#1c1c1c", "#7a4a1e", "#b06a2a", "#5a5a5a", "#2a1f4a"];
const EYES = ["#4a3aff", "#3aa655", "#8a4ad0", "#3a8ad0", "#5a3a20"];

const cells = computed(() => {
  const rnd = mulberry32(props.seed);
  const skin = SKINS[Math.floor(rnd() * SKINS.length)];
  const skinDark = "#00000033";
  const hair = HAIRS[Math.floor(rnd() * HAIRS.length)];
  const eye = EYES[Math.floor(rnd() * EYES.length)];
  const grid: string[] = new Array(64).fill(skin);

  const set = (x: number, y: number, c: string) => {
    grid[y * 8 + x] = c;
  };

  for (let x = 0; x < 8; x++) {
    set(x, 0, hair);
    set(x, 1, hair);
  }
  set(0, 2, hair);
  set(7, 2, hair);
  if (rnd() > 0.5) {
    set(0, 3, hair);
    set(7, 3, hair);
  }

  set(1, 3, "#ffffff");
  set(2, 3, eye);
  set(5, 3, eye);
  set(6, 3, "#ffffff");
  set(3, 4, skinDark);
  set(4, 4, skinDark);
  set(3, 6, "#00000055");
  set(4, 6, "#00000055");

  return grid;
});

const showImage = computed(() => !!src.value && !failed.value);
</script>

<template>
  <div
    class="mc-bevel relative shrink-0 overflow-hidden bg-mc-inset"
    :style="{ width: `${size}px`, height: `${size}px` }"
    role="img"
    :aria-label="username ? `${username} avatar` : 'Account avatar'"
  >
    <img
      v-if="showImage"
      :src="src!"
      :width="size"
      :height="size"
      alt=""
      draggable="false"
      class="size-full pixelated"
      @error="failed = true"
    />
    <div
      v-else
      class="grid size-full"
      :style="{
        gridTemplateColumns: 'repeat(8, 1fr)',
        gridTemplateRows: 'repeat(8, 1fr)',
      }"
    >
      <div v-for="(c, i) in cells" :key="i" :style="{ backgroundColor: c }" />
    </div>
  </div>
</template>

<style scoped>
.pixelated {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}
</style>
