<script setup lang="ts">
/**
 * 8x8 procedural Minecraft-style face, generated from the account's
 * avatarSeed — skin tone, hair, and eye color vary per account.
 */
import { computed } from "vue";

const props = withDefaults(defineProps<{ seed: number; size?: number }>(), {
  size: 32,
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

  // hair: top 2 rows + sideburns
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

  // eyes: white + iris (classic Steve layout, row 3)
  set(1, 3, "#ffffff");
  set(2, 3, eye);
  set(5, 3, eye);
  set(6, 3, "#ffffff");

  // nose shadow + mouth
  set(3, 4, skinDark);
  set(4, 4, skinDark);
  set(3, 6, "#00000055");
  set(4, 6, "#00000055");

  return grid;
});
</script>

<template>
  <div
    class="mc-bevel grid shrink-0 overflow-hidden bg-mc-inset"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      gridTemplateColumns: 'repeat(8, 1fr)',
      gridTemplateRows: 'repeat(8, 1fr)',
    }"
    role="img"
    aria-label="Account avatar"
  >
    <div v-for="(c, i) in cells" :key="i" :style="{ backgroundColor: c }" />
  </div>
</template>
